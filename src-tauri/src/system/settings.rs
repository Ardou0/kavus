use std::fs;
use std::path::PathBuf;
use serde::{Deserialize, Serialize};
use tauri::{AppHandle, Manager};

/// Struct representing the application settings.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AppSettings {
    /// Whether the application should start minimized in the system tray.
    pub start_minimized: bool,
    /// Verbosity level of application logs (debug, info, warn, error).
    pub log_level: String,
    /// Color theme of the application (dark, light).
    pub theme: String,
}

impl Default for AppSettings {
    fn default() -> Self {
        Self {
            start_minimized: true,
            log_level: "info".to_string(),
            theme: "dark".to_string(),
        }
    }
}

/// SettingsManager serves as the abstraction layer for loading and saving settings.
pub struct SettingsManager {
    config_dir: PathBuf,
    settings_file: PathBuf,
}

impl SettingsManager {
    /// Creates a new SettingsManager by resolving the standard OS user config directory.
    pub fn new(app_handle: &AppHandle) -> Result<Self, String> {
        let config_dir = app_handle
            .path()
            .app_config_dir()
            .map_err(|e| format!("Failed to resolve standard user config directory: {}", e))?;
        
        let settings_file = config_dir.join("settings.json");
        
        Ok(Self {
            config_dir,
            settings_file,
        })
    }

    /// Loads settings from the local JSON config file.
    pub fn load(&self) -> Result<AppSettings, String> {
        if !self.config_dir.exists() {
            fs::create_dir_all(&self.config_dir)
                .map_err(|e| format!("Failed to create settings directory hierarchy: {}", e))?;
        }

        if !self.settings_file.exists() {
            let default_settings = AppSettings::default();
            self.save(&default_settings)?;
            return Ok(default_settings);
        }

        let content = fs::read_to_string(&self.settings_file)
            .map_err(|e| format!("Failed to read settings file: {}", e))?;
        
        let settings: AppSettings = serde_json::from_str(&content).map_err(|e| {
            let fallback = AppSettings::default();
            let _ = self.save(&fallback);
            format!("Invalid JSON format in settings, reset to default: {}", e)
        })?;

        Ok(settings)
    }

    /// Serializes and writes the settings to the local JSON config file.
    pub fn save(&self, settings: &AppSettings) -> Result<(), String> {
        if !self.config_dir.exists() {
            fs::create_dir_all(&self.config_dir)
                .map_err(|e| format!("Failed to create settings directory hierarchy: {}", e))?;
        }

        let content = serde_json::to_string_pretty(settings)
            .map_err(|e| format!("Failed to serialize application settings: {}", e))?;
        
        fs::write(&self.settings_file, content)
            .map_err(|e| format!("Failed to write to settings file: {}", e))?;
        
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_default_settings() {
        let defaults = AppSettings::default();
        assert!(defaults.start_minimized);
        assert_eq!(defaults.log_level, "info");
        assert_eq!(defaults.theme, "dark");
    }
}
