use serde::{Deserialize, Serialize};
use std::{
    fs::{self, OpenOptions},
    io::{self, Write},
    path::{Path, PathBuf},
    sync::atomic::{AtomicU64, Ordering},
};

pub const DEFAULT_RETENTION: usize = 1000;
pub const MAX_RETENTION: usize = 999_999;
static TEMP_ID: AtomicU64 = AtomicU64::new(0);

#[derive(Clone, Debug, Deserialize, Serialize)]
#[serde(default)]
pub struct Settings {
    pub log_retention: usize,
    pub fix_random_seed: bool,
}

impl Default for Settings {
    fn default() -> Self {
        Self {
            log_retention: DEFAULT_RETENTION,
            fix_random_seed: true,
        }
    }
}

impl Settings {
    pub fn validate(&self) -> Result<(), String> {
        if (1..=MAX_RETENTION).contains(&self.log_retention) {
            Ok(())
        } else {
            Err("native.invalidLimit".into())
        }
    }
}

pub struct SettingsStore {
    pub value: Settings,
    pub path: Option<PathBuf>,
    pub load_warning: Option<String>,
}

impl SettingsStore {
    pub fn load(directory: Option<&Path>) -> Self {
        let path = directory.map(|directory| directory.join("settiong.toml"));
        let loaded = match &path {
            Some(path) => load_or_create(path),
            None => Err(io::Error::other("Executable directory unavailable")),
        };
        match loaded {
            Ok(value) => Self {
                value,
                path,
                load_warning: None,
            },
            Err(error) => Self {
                value: Settings::default(),
                path,
                load_warning: Some(error.to_string()),
            },
        }
    }

    pub fn save(&mut self, value: Settings) -> Result<(), String> {
        value.validate()?;
        let path = self.path.as_ref().ok_or("native.pathUnavailable")?;
        save_atomic(path, &value).map_err(|error| error.to_string())?;
        self.value = value;
        self.load_warning = None;
        Ok(())
    }
}

fn load_or_create(path: &Path) -> io::Result<Settings> {
    match fs::read_to_string(path) {
        Ok(contents) => {
            let value: Settings = toml::from_str(&contents).map_err(io::Error::other)?;
            value.validate().map_err(io::Error::other)?;
            Ok(value)
        }
        Err(error) if error.kind() == io::ErrorKind::NotFound => {
            let value = Settings::default();
            // create_new prevents another process's existing settings from being overwritten.
            let mut file = match OpenOptions::new().write(true).create_new(true).open(path) {
                Ok(file) => file,
                Err(error) if error.kind() == io::ErrorKind::AlreadyExists => {
                    return load_or_create(path);
                }
                Err(error) => return Err(error),
            };
            file.write_all(
                toml::to_string(&value)
                    .map_err(io::Error::other)?
                    .as_bytes(),
            )?;
            file.sync_all()?;
            Ok(value)
        }
        Err(error) => Err(error),
    }
}

fn save_atomic(path: &Path, settings: &Settings) -> io::Result<()> {
    let contents = toml::to_string(settings).map_err(io::Error::other)?;
    let (temporary, mut file) = loop {
        let id = TEMP_ID.fetch_add(1, Ordering::Relaxed);
        let temporary = path.with_extension(format!("toml.{}.{id}.tmp", std::process::id()));
        match OpenOptions::new()
            .write(true)
            .create_new(true)
            .open(&temporary)
        {
            Ok(file) => break (temporary, file),
            Err(error) if error.kind() == io::ErrorKind::AlreadyExists => continue,
            Err(error) => return Err(error),
        }
    };
    let written = file
        .write_all(contents.as_bytes())
        .and_then(|()| file.sync_all());
    drop(file);
    let result = written.and_then(|()| fs::rename(&temporary, path));
    if result.is_err() {
        // Best-effort cleanup; the original write/rename error is returned to the caller.
        let _cleanup = fs::remove_file(&temporary);
    }
    result
}
