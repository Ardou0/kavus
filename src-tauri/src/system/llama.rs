use std::path::PathBuf;
use std::process::Child;
use std::sync::Mutex;
use tauri::Manager;
use crate::system::settings::SettingsManager;
use crate::system::download::get_binary_path;

pub struct LlamaServerManager {
    child: Mutex<Option<Child>>,
}

impl LlamaServerManager {
    /// Forcefully terminates any running llama-server or llama-cli instances.
    /// This prevents port binding conflicts (port 18080 or custom ports) when the application
    /// restarts after a crash or unclean exit.
    pub fn clean_zombie_processes() {
        println!("Cleaning up any existing zombie llama-server or llama-cli processes...");
        #[cfg(target_os = "windows")]
        {
            let _ = std::process::Command::new("taskkill")
                .args(&["/F", "/IM", "llama-server.exe"])
                .stdout(std::process::Stdio::null())
                .stderr(std::process::Stdio::null())
                .status();
            let _ = std::process::Command::new("taskkill")
                .args(&["/F", "/IM", "llama-cli.exe"])
                .stdout(std::process::Stdio::null())
                .stderr(std::process::Stdio::null())
                .status();
        }

        #[cfg(not(target_os = "windows"))]
        {
            let _ = std::process::Command::new("pkill")
                .args(&["-9", "-f", "llama-server"])
                .stdout(std::process::Stdio::null())
                .stderr(std::process::Stdio::null())
                .status();
            let _ = std::process::Command::new("pkill")
                .args(&["-9", "-f", "llama-cli"])
                .stdout(std::process::Stdio::null())
                .stderr(std::process::Stdio::null())
                .status();
        }
        std::thread::sleep(std::time::Duration::from_millis(300));
    }

    pub fn new() -> Self {
        #[cfg(not(test))]
        Self::clean_zombie_processes();
        Self {
            child: Mutex::new(None),
        }
    }

    /// Stops the running server process by killing the child and ensuring port release.
    pub fn stop(&self) {
        let maybe_child = {
            let mut guard = self.child.lock().unwrap();
            guard.take()
        };
        
        if let Some(mut child) = maybe_child {
            println!("Stopping persistent llama-server (sync)...");
            let _ = child.kill();
            let _ = child.wait();
            // Perform active name cleanup as fail-safe
            Self::clean_zombie_processes();
            // Sleep a small duration to allow the OS to release the socket
            std::thread::sleep(std::time::Duration::from_millis(500));
        }
    }

    /// Spawns llama-server as a background process with the given model, thread count, GPU layers and port.
    pub fn start(
        &self,
        llama_bin_path: PathBuf,
        model_path: PathBuf,
        backend: &str,
        threads: u32,
        ngl: u32,
        ctx_size: u32,
        device_name: &str,
        port: u16,
    ) -> Result<(), String> {
        self.stop();

        let threads_str = threads.to_string();
        let ngl_str = ngl.to_string();
        let ctx_str = ctx_size.to_string();
        let port_str = port.to_string();

        let args = vec![
            "-m".to_string(), model_path.to_string_lossy().to_string(),
            "-c".to_string(), ctx_str,
            "--threads".to_string(), threads_str.clone(),
            "--n-gpu-layers".to_string(), ngl_str.clone(),
            "--port".to_string(), port_str.clone(),
            "--host".to_string(), "127.0.0.1".to_string(),
            "--parallel".to_string(), "1".to_string(),
        ];

        println!(
            "Spawning persistent llama-server (Backend: {}, GPU layers: {}, threads: {}, port: {}, bin: {:?})...",
            backend, ngl_str, threads_str, port_str, llama_bin_path
        );

        let mut cmd = std::process::Command::new(llama_bin_path);
        cmd.args(&args);
        
        // Suppress console output to prevent cluttering
        cmd.stdout(std::process::Stdio::null());
        cmd.stderr(std::process::Stdio::null());

        // Configure GPU device routing if layers are offloaded and specific hardware is targeted
        if !device_name.trim().is_empty() && ngl > 0 {
            cmd.env("GGML_VULKAN_DEVICE", device_name);
            cmd.env("CUDA_VISIBLE_DEVICES", device_name);
        }

        let child_proc = cmd.spawn().map_err(|e| format!("Failed to spawn llama-server: {}", e))?;
        
        {
            let mut guard = self.child.lock().unwrap();
            *guard = Some(child_proc);
        }
        
        // Allow the OS enough time to bind the server socket to the port before returning
        std::thread::sleep(std::time::Duration::from_millis(1500));

        Ok(())
    }
}

impl Drop for LlamaServerManager {
    fn drop(&mut self) {
        self.stop();
    }
}

/// Resolves directories and configuration, then starts the llama-server with appropriate GPU offload mapping.
pub async fn start_server_internal(app_handle: &tauri::AppHandle) -> Result<(), String> {
    let settings_manager = SettingsManager::new(app_handle)?;
    let settings = settings_manager.load()?;

    let app_dir = app_handle.path().app_local_data_dir()
        .map_err(|e| e.to_string())?;
    
    let mut bin_dir = app_dir.clone();
    bin_dir.push("bin");
    
    let mut model_dir = app_dir.clone();
    model_dir.push("models");

    #[cfg(target_os = "windows")]
    const SERVER_BIN_NAME: &str = "llama-server.exe";
    #[cfg(not(target_os = "windows"))]
    const SERVER_BIN_NAME: &str = "llama-server";

    let server_bin_path = get_binary_path(&bin_dir, SERVER_BIN_NAME)
        .ok_or_else(|| "Llama server binary missing. Please download it first in Settings > Autocorrection.".to_string())?;

    let mut model_path = model_dir.clone();
    model_path.push(format!("{}.gguf", settings.autocorrection_model));
    
    if !model_path.exists() {
        return Err(format!("Model file '{}.gguf' not found. Please download it first.", settings.autocorrection_model));
    }

    let (ngl, threads) = match settings.execution_backend.as_str() {
        "gpu" => (999, 1),
        "hybrid" => {
            let t = if settings.cpu_threads == 0 { 1 } else { settings.cpu_threads };
            (settings.gpu_layers, t)
        }
        _ => {
            let t = if settings.cpu_threads == 0 { 1 } else { settings.cpu_threads };
            (0, t)
        }
    };

    let server_manager = app_handle.state::<LlamaServerManager>();
    server_manager.start(
        server_bin_path,
        model_path,
        &settings.execution_backend,
        threads,
        ngl,
        settings.context_size,
        &settings.gpu_device_name,
        settings.autocorrection_port,
    )?;

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    /// Asserts that stopping a manager with no active subprocess is a clean, non-crashing operations.
    #[test]
    fn test_llama_server_manager_stop_inactive() {
        let manager = LlamaServerManager::new();
        manager.stop();
        assert!(manager.child.lock().unwrap().is_none());
    }

    /// Asserts that start fails with an error result when pointing to an invalid binary path.
    #[test]
    fn test_llama_server_manager_spawn_invalid_bin() {
        let manager = LlamaServerManager::new();
        let bin_path = PathBuf::from("/invalid/path/to/binary");
        let res = manager.start(
            bin_path,
            PathBuf::from("/dummy/model.gguf"),
            "cpu",
            1,
            0,
            2048,
            "",
            18080,
        );
        assert!(res.is_err());
    }
}
