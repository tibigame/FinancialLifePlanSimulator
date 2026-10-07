use super::{TestDirectory, TestResult};
use crate::settings::{DEFAULT_RETENTION, MAX_RETENTION, Settings, SettingsStore};
use std::fs;

#[test]
fn creates_portable_default_and_persists_changes() -> TestResult {
    let directory = TestDirectory::new()?;
    let mut store = SettingsStore::load(Some(&directory.0));
    assert_eq!(store.value.log_retention, DEFAULT_RETENTION);
    assert!(directory.0.join("settiong.toml").is_file());
    assert!(store.load_warning.is_none());
    store.save(Settings {
        log_retention: MAX_RETENTION,
    })?;
    let restored = SettingsStore::load(Some(&directory.0));
    assert_eq!(restored.value.log_retention, MAX_RETENTION);
    assert!(restored.load_warning.is_none());
    store.save(Settings { log_retention: 1 })?;
    assert_eq!(
        SettingsStore::load(Some(&directory.0)).value.log_retention,
        1
    );
    Ok(())
}

#[test]
fn validates_limits_without_changing_saved_or_active_value() -> TestResult {
    let directory = TestDirectory::new()?;
    let mut store = SettingsStore::load(Some(&directory.0));
    for invalid in [0, MAX_RETENTION + 1, usize::MAX] {
        assert!(
            store
                .save(Settings {
                    log_retention: invalid
                })
                .is_err()
        );
        assert_eq!(store.value.log_retention, DEFAULT_RETENTION);
        assert_eq!(
            SettingsStore::load(Some(&directory.0)).value.log_retention,
            DEFAULT_RETENTION
        );
    }
    Ok(())
}

#[test]
fn invalid_settings_fall_back_without_overwriting_original() -> TestResult {
    let directory = TestDirectory::new()?;
    let path = directory.0.join("settiong.toml");
    for contents in [
        "not valid toml!",
        "log_retention = 0",
        "log_retention = 1000000",
        "log_retention = -1",
    ] {
        fs::write(&path, contents)?;
        let store = SettingsStore::load(Some(&directory.0));
        assert_eq!(store.value.log_retention, DEFAULT_RETENTION);
        assert!(store.load_warning.is_some());
        assert_eq!(fs::read_to_string(&path)?, contents);
    }
    Ok(())
}

#[test]
fn failed_save_keeps_old_value_and_cleans_temporary_file() -> TestResult {
    let directory = TestDirectory::new()?;
    let mut store = SettingsStore::load(Some(&directory.0));
    let path = directory.0.join("settiong.toml");
    fs::remove_file(&path)?;
    fs::create_dir(&path)?;
    assert!(store.save(Settings { log_retention: 25 }).is_err());
    assert_eq!(store.value.log_retention, DEFAULT_RETENTION);
    assert_eq!(fs::read_dir(&directory.0)?.count(), 1);
    Ok(())
}

#[test]
fn missing_executable_directory_uses_defaults() {
    let mut store = SettingsStore::load(None);
    assert_eq!(store.value.log_retention, DEFAULT_RETENTION);
    assert!(store.load_warning.is_some());
    assert!(store.save(Settings { log_retention: 10 }).is_err());
}
