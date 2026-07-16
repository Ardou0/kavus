use std::fs;
use std::path::PathBuf;
use serde::{Deserialize, Serialize};
use tauri::{AppHandle, Manager};

fn default_cpu_threads() -> u32 { 4 }
fn default_gpu_layers() -> u32 { 0 }
fn default_context_size() -> u32 { 2048 }
fn default_gpu_device_name() -> String { "".to_string() }
fn default_execution_backend() -> String { "cpu".to_string() }
fn default_autocorrection_port() -> u16 { 18080 }

/// Struct representing the application settings.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AppSettings {
    /// Whether the application should start minimized in the system tray.
    pub start_minimized: bool,
    /// Verbosity level of application logs (debug, info, warn, error).
    pub log_level: String,
    /// Color theme of the application (dark, light).
    pub theme: String,
    /// Default directory path pre-filled for new projects.
    pub default_project_path: String,
    /// Whether the warning modal/notice regarding sandbox directories is shown.
    pub show_sandbox_warning: bool,
    /// Whether LLM-based code auto-correction is enabled.
    pub enable_autocorrection: bool,
    /// The model selected for running code corrections.
    pub autocorrection_model: String,
    
    /// The number of CPU threads to allocate for inference.
    #[serde(default = "default_cpu_threads")]
    pub cpu_threads: u32,
    /// The number of layers to offload to GPU (0 = CPU only).
    #[serde(default = "default_gpu_layers")]
    pub gpu_layers: u32,
    /// The context size in tokens (e.g. 512, 2048).
    #[serde(default = "default_context_size")]
    pub context_size: u32,
    /// The GPU device name to target (empty for default).
    #[serde(default = "default_gpu_device_name")]
    pub gpu_device_name: String,
    /// The execution backend: cpu, gpu, or hybrid.
    #[serde(default = "default_execution_backend")]
    pub execution_backend: String,
    /// The port to run the autocorrection server on.
    #[serde(default = "default_autocorrection_port")]
    pub autocorrection_port: u16,
}

impl Default for AppSettings {
    fn default() -> Self {
        Self {
            start_minimized: true,
            log_level: "info".to_string(),
            theme: "dark".to_string(),
            default_project_path: "".to_string(),
            show_sandbox_warning: true,
            enable_autocorrection: false,
            autocorrection_model: "llama3-8b".to_string(),
            cpu_threads: 4,
            gpu_layers: 0,
            context_size: 2048,
            gpu_device_name: "".to_string(),
            execution_backend: "cpu".to_string(),
            autocorrection_port: 18080,
        }
    }
}

/// SettingsManager serves as the abstraction layer for loading and saving settings.
#[derive(Clone)]
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

    /// Asserts that AppSettings initializes with standard default traits (autostart, info logging, dark theme).
    #[test]
    fn test_default_settings() {
        let defaults = AppSettings::default();
        assert!(defaults.start_minimized);
        assert_eq!(defaults.log_level, "info");
        assert_eq!(defaults.theme, "dark");
    }
}
