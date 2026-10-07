use crate::{
    logging::{Category, LogPage, LogQuery, LogStore, Severity},
    settings::{Settings, SettingsStore},
    text::TextResources,
};
use serde::Serialize;
use std::sync::{Arc, Mutex};
use tauri::State;

pub struct AppState {
    pub logs: Arc<LogStore>,
    pub settings: Mutex<SettingsStore>,
    pub text: Arc<TextResources>,
}

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
pub struct SettingsView {
    log_retention: usize,
    path: Option<String>,
    load_warning: Option<String>,
}

impl SettingsView {
    fn from_store(store: &SettingsStore) -> Self {
        Self {
            log_retention: store.value.log_retention,
            path: store
                .path
                .as_ref()
                .map(|path| path.to_string_lossy().into_owned()),
            load_warning: store.load_warning.clone(),
        }
    }
}

#[tauri::command]
pub fn get_settings(state: State<'_, AppState>) -> Result<SettingsView, String> {
    let settings = state
        .settings
        .lock()
        .map_err(|_| state.text.get("native.stateUnavailable"))?;
    Ok(SettingsView::from_store(&settings))
}

#[tauri::command]
pub fn save_settings(
    log_retention: usize,
    state: State<'_, AppState>,
) -> Result<SettingsView, String> {
    let value = Settings { log_retention };
    value
        .validate()
        .map_err(|key| state.text.get(&key).to_owned())?;
    let mut settings = state
        .settings
        .lock()
        .map_err(|_| state.text.get("native.stateUnavailable"))?;
    if let Err(error) = settings.save(value) {
        let message = format!("{} {error}", state.text.get("native.settingsSaveFailed"));
        state.logs.record(
            Severity::Error,
            Category::Setting,
            "settings",
            message.clone(),
        )?;
        return Err(message);
    }
    state.logs.set_retention(log_retention)?;
    state.logs.record(
        Severity::Info,
        Category::Setting,
        "settings",
        state
            .text
            .get("native.settingsSaved")
            .replace("{limit}", &log_retention.to_string()),
    )?;
    Ok(SettingsView::from_store(&settings))
}

#[tauri::command]
pub fn get_logs(query: LogQuery, state: State<'_, AppState>) -> Result<LogPage, String> {
    state.logs.query(&query)
}

#[tauri::command]
pub fn write_frontend_log(
    level: Severity,
    source: String,
    message: String,
    state: State<'_, AppState>,
) -> Result<(), String> {
    state
        .logs
        .record(level, Category::Frontend, &source, message)
}
