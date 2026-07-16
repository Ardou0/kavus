use tauri::{State, Manager};
use crate::system::settings::{AppSettings, SettingsManager};

/// Tauri command to retrieve the current application settings.
#[tauri::command]
pub async fn get_settings(
    settings_manager: State<'_, SettingsManager>,
) -> Result<AppSettings, String> {
    let manager = settings_manager.inner().clone();
    tokio::task::spawn_blocking(move || manager.load())
        .await
        .map_err(|e| format!("Failed to execute blocking task: {}", e))?
}

/// Tauri command to save updated application settings.
#[tauri::command]
pub async fn save_settings(
    settings: AppSettings,
    settings_manager: State<'_, SettingsManager>,
    app_handle: tauri::AppHandle,
) -> Result<(), String> {
    let manager = settings_manager.inner().clone();
    let settings_clone = settings.clone();
    tokio::task::spawn_blocking(move || manager.save(&settings_clone))
        .await
        .map_err(|e| format!("Failed to execute blocking task: {}", e))??;

    // Sync the server with the new settings
    if settings.enable_autocorrection {
        // Try starting the server (this is self-healing, if model is not downloaded it fails silently)
        let _ = crate::system::llama::start_server_internal(&app_handle).await;
    } else {
        if let Some(server_manager) = app_handle.try_state::<crate::system::llama::LlamaServerManager>() {
            server_manager.stop();
        }
    }

    Ok(())
}
