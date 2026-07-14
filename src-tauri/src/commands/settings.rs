use tauri::State;
use crate::system::settings::{AppSettings, SettingsManager};

/// Tauri command to retrieve the current application settings.
#[tauri::command]
pub fn get_settings(
    settings_manager: State<'_, SettingsManager>,
) -> Result<AppSettings, String> {
    settings_manager.load()
}

/// Tauri command to save updated application settings.
#[tauri::command]
pub fn save_settings(
    settings: AppSettings,
    settings_manager: State<'_, SettingsManager>,
) -> Result<(), String> {
    settings_manager.save(&settings)
}
