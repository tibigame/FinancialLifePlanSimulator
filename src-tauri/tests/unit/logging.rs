use super::{TestDirectory, TestResult};
use crate::{
    logging::{Category, FileStatus, LogQuery, LogStore, Severity, file::create_log_file},
    text::TextResources,
};
use std::{
    fs,
    io::{self, Write},
    sync::Arc,
};

fn all_logs() -> LogQuery {
    LogQuery {
        levels: vec![
            Severity::Trace,
            Severity::Debug,
            Severity::Info,
            Severity::Warn,
            Severity::Error,
        ],
        categories: vec![
            Category::Common,
            Category::Setting,
            Category::Frontend,
            Category::External,
        ],
        anchor_id: None,
        offset: 0,
        limit: 200,
    }
}

#[test]
fn retains_newest_but_file_keeps_every_record() -> TestResult {
    let directory = TestDirectory::new()?;
    let store = LogStore::new(Some(&directory.0), 3, Arc::new(TextResources::load()?));
    for number in 1..=5 {
        store.record(
            Severity::Info,
            Category::Common,
            "test",
            format!("message-{number}"),
        )?;
    }
    let page = store.query(&all_logs())?;
    assert_eq!(
        page.entries
            .iter()
            .map(|entry| entry.id)
            .collect::<Vec<_>>(),
        vec![5, 4, 3]
    );
    let path = page.file.path.ok_or("Log path missing")?;
    assert_eq!(fs::read_to_string(&path)?.lines().count(), 5);
    store.set_retention(1)?;
    assert_eq!(store.query(&all_logs())?.entries[0].id, 5);
    store.set_retention(1000)?;
    assert_eq!(store.query(&all_logs())?.total, 1);
    assert_eq!(fs::read_to_string(path)?.lines().count(), 5);
    assert!(store.set_retention(0).is_err());
    assert!(store.set_retention(1_000_000).is_err());
    Ok(())
}

#[test]
fn filters_intersect_and_empty_selection_matches_nothing() -> TestResult {
    let store = LogStore::new(None, 1000, Arc::new(TextResources::load()?));
    store.record(Severity::Error, Category::Setting, "settings", "a".into())?;
    store.record(Severity::Info, Category::Setting, "settings", "b".into())?;
    store.record(Severity::Error, Category::Common, "app", "c".into())?;
    let mut query = all_logs();
    query.levels = vec![Severity::Error];
    query.categories = vec![Category::Setting];
    let page = store.query(&query)?;
    assert_eq!(page.total, 1);
    assert_eq!(page.entries[0].message, "a");
    query.levels.clear();
    assert_eq!(store.query(&query)?.total, 0);
    query.levels = vec![Severity::Error];
    query.categories.clear();
    assert_eq!(store.query(&query)?.total, 0);
    Ok(())
}

#[test]
fn anchored_paging_stays_stable_when_new_logs_arrive() -> TestResult {
    let store = LogStore::new(None, 1000, Arc::new(TextResources::load()?));
    for number in 0..300 {
        store.record(
            Severity::Debug,
            Category::External,
            "library",
            number.to_string(),
        )?;
    }
    let first = store.query(&all_logs())?;
    assert_eq!(first.entries.len(), 200);
    let mut query = all_logs();
    query.anchor_id = Some(first.anchor_id);
    query.offset = 200;
    query.limit = 999_999;
    store.record(Severity::Error, Category::Frontend, "window", "new".into())?;
    let older = store.query(&query)?;
    assert_eq!(older.total, 300);
    assert_eq!(older.entries.len(), 100);
    assert_eq!(older.entries[0].id, 100);
    assert_eq!(older.entries[99].id, 1);
    assert_eq!(store.query(&all_logs())?.total, 301);
    Ok(())
}

#[test]
fn file_names_only_get_suffix_on_collision_and_never_overwrite() -> TestResult {
    let directory = TestDirectory::new()?;
    let stamp = "20261007123450";
    let (mut file, first) = create_log_file(&directory.0, stamp)?;
    file.write_all(b"original")?;
    drop(file);
    let (_, second) = create_log_file(&directory.0, stamp)?;
    let (_, third) = create_log_file(&directory.0, stamp)?;
    assert_eq!(
        first.file_name().and_then(|name| name.to_str()),
        Some("20261007123450.log")
    );
    assert_eq!(
        second.file_name().and_then(|name| name.to_str()),
        Some("20261007123450_001.log")
    );
    assert_eq!(
        third.file_name().and_then(|name| name.to_str()),
        Some("20261007123450_002.log")
    );
    assert_eq!(fs::read_to_string(first)?, "original");
    Ok(())
}

#[test]
fn creation_failure_keeps_memory_logging_available() -> TestResult {
    let directory = TestDirectory::new()?;
    fs::write(directory.0.join("log"), "file blocks directory")?;
    let store = LogStore::new(Some(&directory.0), 1000, Arc::new(TextResources::load()?));
    store.record(
        Severity::Info,
        Category::Common,
        "app",
        "still running".into(),
    )?;
    let page = store.query(&all_logs())?;
    assert_eq!(page.total, 1);
    assert!(!page.file.active);
    assert!(page.file.error.is_some());
    Ok(())
}

struct FailingWriter;
impl Write for FailingWriter {
    fn write(&mut self, _: &[u8]) -> io::Result<usize> {
        Err(io::Error::other("disk full"))
    }
    fn flush(&mut self) -> io::Result<()> {
        Ok(())
    }
}

#[test]
fn write_failure_disables_file_once_and_keeps_logging_in_memory() -> TestResult {
    let store = LogStore::with_writer(
        1000,
        Arc::new(TextResources::load()?),
        Some(Box::new(FailingWriter)),
        FileStatus {
            path: Some("test.log".into()),
            error: None,
            active: true,
        },
    );
    store.record(Severity::Info, Category::Common, "app", "first".into())?;
    store.record(Severity::Trace, Category::Common, "app", "second".into())?;
    let page = store.query(&all_logs())?;
    assert_eq!(page.total, 3);
    assert_eq!(page.entries[1].level, Severity::Error);
    assert_eq!(page.entries[2].message, "first");
    assert!(!page.file.active);
    assert!(page.file.error.is_some());
    Ok(())
}

#[test]
fn multiline_exceptions_are_one_json_line_and_preserve_content() -> TestResult {
    let directory = TestDirectory::new()?;
    let store = LogStore::new(Some(&directory.0), 1000, Arc::new(TextResources::load()?));
    let message = "Error: 例外\n    at component (app.tsx:12:3)";
    store.record(
        Severity::Error,
        Category::Frontend,
        "window",
        message.into(),
    )?;
    let contents = fs::read_to_string(store.file_status()?.path.ok_or("Missing path")?)?;
    assert_eq!(contents.lines().count(), 1);
    let entry: serde_json::Value = serde_json::from_str(contents.trim())?;
    assert_eq!(entry["message"], message);
    assert_eq!(entry["category"], "Frontend");
    Ok(())
}

#[test]
fn captures_both_external_log_and_tracing_events() -> TestResult {
    let store = Arc::new(LogStore::new(None, 1000, Arc::new(TextResources::load()?)));
    store.install()?;
    log::trace!(target: "test_external_log", "trace from dependency");
    tracing::error!(target: "test_external_tracing", code = 42, "error from dependency");
    let page = store.query(&all_logs())?;
    assert!(
        page.entries
            .iter()
            .any(|entry| entry.source == "test_external_log"
                && entry.level == Severity::Trace
                && entry.category == Category::External)
    );
    assert!(
        page.entries
            .iter()
            .any(|entry| entry.source == "test_external_tracing"
                && entry.level == Severity::Error
                && entry.message.contains("42"))
    );
    Ok(())
}

#[test]
fn concurrent_records_have_unique_monotonic_ids() -> TestResult {
    let store = Arc::new(LogStore::new(None, 1000, Arc::new(TextResources::load()?)));
    let workers: Vec<_> = (0..4)
        .map(|_| {
            let store = store.clone();
            std::thread::spawn(move || -> Result<(), String> {
                for _ in 0..50 {
                    store.record(Severity::Info, Category::Common, "worker", "message".into())?;
                }
                Ok(())
            })
        })
        .collect();
    for worker in workers {
        worker.join().map_err(|_| "Worker panicked")??;
    }
    let page = store.query(&all_logs())?;
    assert_eq!(page.total, 200);
    assert!(
        page.entries
            .windows(2)
            .all(|pair| pair[0].id == pair[1].id + 1)
    );
    Ok(())
}
