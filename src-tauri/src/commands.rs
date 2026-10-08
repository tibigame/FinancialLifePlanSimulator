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
    pub random: Mutex<crate::rand::SystemRandom>,
    pub text: Arc<TextResources>,
}

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
pub struct SettingsView {
    log_retention: usize,
    fix_random_seed: bool,
    path: Option<String>,
    load_warning: Option<String>,
}

impl SettingsView {
    fn from_store(store: &SettingsStore) -> Self {
        Self {
            log_retention: store.value.log_retention,
            fix_random_seed: store.value.fix_random_seed,
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
    fix_random_seed: bool,
    state: State<'_, AppState>,
) -> Result<SettingsView, String> {
    let value = Settings {
        log_retention,
        fix_random_seed,
    };
    value
        .validate()
        .map_err(|key| state.text.get(&key).to_owned())?;
    let mut settings = state
        .settings
        .lock()
        .map_err(|_| state.text.get("native.stateUnavailable"))?;
    // Prepare the replacement before persisting, so initialization failure keeps
    // both the saved setting and the active stream unchanged.
    let replacement = if settings.value.fix_random_seed != fix_random_seed {
        Some(crate::rand::SystemRandom::new(fix_random_seed).map_err(|error| error.to_string())?)
    } else {
        None
    };
    let mut random = state
        .random
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
    if let Some(replacement) = replacement {
        *random = replacement;
    }
    drop(random);
    state.logs.set_retention(log_retention)?;
    state.logs.record(
        Severity::Info,
        Category::Setting,
        "settings",
        state
            .text
            .get("native.settingsSaved")
            .replace("{limit}", &log_retention.to_string())
            .replace("{fixed}", if fix_random_seed { "ON" } else { "OFF" }),
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
