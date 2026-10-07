pub mod file;
pub mod model;
mod native;

use crate::{settings::MAX_RETENTION, text::TextResources};
use chrono::Local;
pub use model::{Category, FileStatus, LogEntry, LogPage, LogQuery, Severity};
use std::{
    collections::VecDeque,
    io::{BufWriter, Write},
    path::Path,
    sync::{Arc, Mutex, MutexGuard},
};

struct LogData {
    entries: VecDeque<LogEntry>,
    retention: usize,
    next_id: u64,
    writer: Option<Box<dyn Write + Send>>,
    file: FileStatus,
}

pub struct LogStore {
    data: Mutex<LogData>,
    text: Arc<TextResources>,
}

impl LogStore {
    pub fn new(directory: Option<&Path>, retention: usize, text: Arc<TextResources>) -> Self {
        let result = match directory {
            Some(directory) => {
                file::create_log_file(directory, &Local::now().format("%Y%m%d%H%M%S").to_string())
            }
            None => Err(std::io::Error::other(text.get("native.pathUnavailable"))),
        };
        let (writer, status) = match result {
            Ok((file, path)) => (
                Some(Box::new(BufWriter::new(file)) as Box<dyn Write + Send>),
                FileStatus {
                    path: Some(path.to_string_lossy().into_owned()),
                    active: true,
                    error: None,
                },
            ),
            Err(error) => (
                None,
                FileStatus {
                    path: None,
                    active: false,
                    error: Some(error.to_string()),
                },
            ),
        };
        Self::with_writer(retention, text, writer, status)
    }

    pub(crate) fn with_writer(
        retention: usize,
        text: Arc<TextResources>,
        writer: Option<Box<dyn Write + Send>>,
        file: FileStatus,
    ) -> Self {
        Self {
            data: Mutex::new(LogData {
                entries: VecDeque::new(),
                retention: retention.clamp(1, MAX_RETENTION),
                next_id: 1,
                writer,
                file,
            }),
            text,
        }
    }

    fn lock(&self) -> Result<MutexGuard<'_, LogData>, String> {
        self.data
            .lock()
            .map_err(|_| self.text.get("native.stateUnavailable").to_owned())
    }

    pub fn record(
        &self,
        level: Severity,
        category: Category,
        source: &str,
        message: String,
    ) -> Result<(), String> {
        let mut data = self.lock()?;
        let entry = Self::entry(&mut data, level, category, source, message);
        // One serialized line per record, including escaped multiline exception stacks.
        let encoded = serde_json::to_vec(&entry).map_err(|error| error.to_string())?;
        let write_result = match &mut data.writer {
            Some(writer) => writer
                .write_all(&encoded)
                .and_then(|()| writer.write_all(b"\n"))
                .and_then(|()| writer.flush()),
            None => Ok(()),
        };
        Self::push(&mut data, entry);
        if let Err(error) = write_result {
            data.writer = None;
            data.file.active = false;
            data.file.error = Some(error.to_string());
            let entry = Self::entry(
                &mut data,
                Severity::Error,
                Category::Common,
                "logging",
                format!("{} {}", self.text.get("native.fileWriteFailed"), error),
            );
            Self::push(&mut data, entry);
        }
        Ok(())
    }

    fn entry(
        data: &mut LogData,
        level: Severity,
        category: Category,
        source: &str,
        message: String,
    ) -> LogEntry {
        let id = data.next_id;
        data.next_id += 1;
        LogEntry {
            id,
            timestamp: Local::now().format("%Y-%m-%d %H:%M:%S%.3f %:z").to_string(),
            level,
            category,
            source: source.into(),
            message,
        }
    }

    fn push(data: &mut LogData, entry: LogEntry) {
        data.entries.push_back(entry);
        while data.entries.len() > data.retention {
            data.entries.pop_front();
        }
    }

    pub fn set_retention(&self, retention: usize) -> Result<(), String> {
        if !(1..=MAX_RETENTION).contains(&retention) {
            return Err(self.text.get("native.invalidLimit").into());
        }
        let mut data = self.lock()?;
        data.retention = retention;
        while data.entries.len() > retention {
            data.entries.pop_front();
        }
        Ok(())
    }

    pub fn file_status(&self) -> Result<FileStatus, String> {
        Ok(self.lock()?.file.clone())
    }

    pub fn query(&self, query: &LogQuery) -> Result<LogPage, String> {
        let data = self.lock()?;
        let newest = data.next_id - 1;
        let anchor_id = query.anchor_id.map_or(newest, |id| id.min(newest));
        let mut total = 0;
        let mut entries = Vec::new();
        let limit = query.limit.clamp(1, 200);
        for entry in data.entries.iter().rev() {
            if entry.id > anchor_id
                || !query.levels.contains(&entry.level)
                || !query.categories.contains(&entry.category)
            {
                continue;
            }
            if total >= query.offset && entries.len() < limit {
                entries.push(entry.clone());
            }
            total += 1;
        }
        Ok(LogPage {
            entries,
            total,
            retained: data.entries.len(),
            retention_limit: data.retention,
            anchor_id,
            file: data.file.clone(),
        })
    }

    pub fn install(self: &Arc<Self>) -> Result<(), String> {
        native::install(self.clone())
    }
}
