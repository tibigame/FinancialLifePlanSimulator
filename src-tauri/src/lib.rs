mod commands;
mod logging;
mod settings;
mod text;

use commands::AppState;
use logging::{Category, LogStore, Severity};
use std::{
    error::Error,
    sync::{Arc, Mutex},
};
use text::TextResources;

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() -> Result<(), Box<dyn Error>> {
    let text = Arc::new(TextResources::load()?);
    let directory = std::env::current_exe().and_then(|path| {
        path.parent()
            .map(std::path::Path::to_path_buf)
            .ok_or_else(|| std::io::Error::other(text.get("native.pathUnavailable")))
    });
    let settings = settings::SettingsStore::load(directory.as_deref().ok());
    let logs = Arc::new(LogStore::new(
        directory.as_deref().ok(),
        settings.value.log_retention,
        text.clone(),
    ));
    if let Err(error) = logs.install() {
        logs.record(
            Severity::Error,
            Category::Common,
            "logging",
            format!("{} {error}", text.get("native.loggerFailed")),
        )?;
    }
    logs.record(
        Severity::Info,
        Category::Common,
        "app",
        text.get("native.startup").into(),
    )?;
    if let Err(error) = directory {
        logs.record(
            Severity::Error,
            Category::Common,
            "app",
            format!("{} {error}", text.get("native.pathUnavailable")),
        )?;
    }
    let file = logs.file_status()?;
    if let Some(error) = file.error {
        logs.record(
            Severity::Warn,
            Category::Common,
            "logging",
            format!("{} {error}", text.get("native.fileDisabled")),
        )?;
    }
    if let Some(error) = &settings.load_warning {
        logs.record(
            Severity::Warn,
            Category::Setting,
            "settings",
            format!("{} {error}", text.get("native.settingsLoadFailed")),
        )?;
    } else {
        logs.record(
            Severity::Info,
            Category::Setting,
            "settings",
            text.get("native.settingsLoaded").into(),
        )?;
    }
    let exit_logs = logs.clone();
    let exit_text = text.clone();
    tauri::Builder::default()
        .plugin(tauri_plugin_opener::init())
        .manage(AppState {
            logs,
            settings: Mutex::new(settings),
            text,
        })
        .invoke_handler(tauri::generate_handler![
            commands::get_settings,
            commands::save_settings,
            commands::get_logs,
            commands::write_frontend_log,
        ])
        .build(tauri::generate_context!())?
        .run(move |_, event| {
            if let tauri::RunEvent::Exit = event
                && let Err(error) = exit_logs.record(
                    Severity::Info,
                    Category::Common,
                    "app",
                    exit_text.get("native.shutdown").into(),
                )
            {
                eprintln!("{error}");
            }
        });
    Ok(())
}

#[cfg(test)]
#[path = "../tests/unit/mod.rs"]
mod tests;
