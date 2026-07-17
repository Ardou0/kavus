use tauri::{AppHandle, State, Emitter, Manager};
use std::time::{SystemTime, UNIX_EPOCH, Instant};
use std::fs;
use std::path::{Path, PathBuf};
use reqwest::Client;
use crate::system::download::{
    DownloadManager, DownloadLogEntry, DownloadProgressPayload,
    perform_file_download, verify_file_sha256, extract_archive,
    make_executable, get_binary_path
};

// Embed models.json at compile time
const MODELS_JSON: &str = include_str!("../../models.json");

#[derive(serde::Deserialize)]
#[allow(dead_code)]
struct BinaryAsset {
    url: String,
    mirror_url: String,
    sha256: String,
}

#[derive(serde::Deserialize)]
#[allow(dead_code)]
struct LlamaBinaries {
    windows_x86_64_cpu: BinaryAsset,
    windows_x86_64_gpu: BinaryAsset,
    windows_aarch64: BinaryAsset,
    macos_x86_64: BinaryAsset,
    macos_aarch64: BinaryAsset,
    linux_x86_64_cpu: BinaryAsset,
    linux_x86_64_gpu_vulkan: BinaryAsset,
    linux_x86_64_gpu_cuda: BinaryAsset,
    linux_aarch64: BinaryAsset,
}

#[derive(serde::Deserialize, Clone)]
#[allow(dead_code)]
pub struct ModelRequirements {
    pub min_ram_gb: f32,
    pub requires_gpu: bool,
    pub recommended_gpu: bool,
}

#[derive(serde::Deserialize, Clone)]
#[allow(dead_code)]
pub struct ModelConfig {
    pub id: String,
    pub name: String,
    pub size_bytes: u64,
    pub url: String,
    pub mirror_url: String,
    pub sha256: String,
    pub requirements: ModelRequirements,
}

#[derive(serde::Deserialize)]
#[allow(dead_code)]
struct ModelsJson {
    llama_binaries: LlamaBinaries,
    models: Vec<ModelConfig>,
}

#[cfg(target_os = "windows")]
const LLAMA_BIN_NAME: &str = "llama-cli.exe";
#[cfg(not(target_os = "windows"))]
const LLAMA_BIN_NAME: &str = "llama-cli";


#[derive(serde::Serialize)]
pub struct ModelInfoPayload {
    pub id: String,
    pub name: String,
    pub size_bytes: u64,
}

#[tauri::command]
pub fn get_available_models() -> Result<Vec<ModelInfoPayload>, String> {
    let models_data: ModelsJson = serde_json::from_str(MODELS_JSON)
        .map_err(|e| format!("Failed to parse models.json: {}", e))?;
    
    let res = models_data.models.iter().map(|m| ModelInfoPayload {
        id: m.id.clone(),
        name: m.name.clone(),
        size_bytes: m.size_bytes,
    }).collect();

    Ok(res)
}

#[tauri::command]
pub fn get_download_history(
    download_manager: State<'_, DownloadManager>,
) -> Result<Vec<DownloadLogEntry>, String> {
    download_manager.load_history()
}

#[tauri::command]
pub fn delete_downloaded_model(
    id: String,
    download_manager: State<'_, DownloadManager>,
) -> Result<Vec<DownloadLogEntry>, String> {
    let mut history = download_manager.load_history()?;
    if let Some(pos) = history.iter().position(|e| e.id == id) {
        let mut entry = history[pos].clone();
        entry.status = "deleted".to_string();
        
        let path = Path::new(&entry.file_path);
        if path.exists() {
            let _ = fs::remove_file(path);
        }
        
        history[pos] = entry;
        download_manager.save_history(&history)?;
    }
    Ok(history)
}

#[tauri::command]
#[allow(unused_assignments)]
pub async fn start_model_download(
    model_name: String,
    app_handle: AppHandle,
    download_manager: State<'_, DownloadManager>,
) -> Result<(), String> {
    let models_data: ModelsJson = serde_json::from_str(MODELS_JSON)
        .map_err(|e| format!("Failed to parse models.json: {}", e))?;

    let model_info = models_data.models.iter().find(|m| m.id == model_name)
        .ok_or_else(|| format!("Model '{}' not found in config", model_name))?;

    let primary_url = model_info.url.clone();
    let mirror_url = model_info.mirror_url.clone();
    let expected_sha256 = model_info.sha256.clone();
    let total_bytes = model_info.size_bytes;

    let app_dir = app_handle.path().app_local_data_dir()
        .map_err(|e| e.to_string())?;

    let mut model_dir = app_dir.clone();
    model_dir.push("models");
    fs::create_dir_all(&model_dir).map_err(|e| e.to_string())?;

    let timestamp = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap()
        .as_secs();

    let model_filename = format!("{}.gguf", model_info.id);
    let model_path = model_dir.join(&model_filename);

    let log_entry = DownloadLogEntry {
        id: model_info.id.clone(),
        model_name: model_info.name.clone(),
        file_path: model_path.to_string_lossy().to_string(),
        status: "downloading".to_string(),
        size_bytes: total_bytes,
        timestamp,
    };
    download_manager.add_or_update_entry(log_entry)?;

    let app_handle_clone = app_handle.clone();
    let model_id = model_info.id.clone();
    let model_name = model_info.name.clone();

    tauri::async_runtime::spawn(async move {
        let client = Client::new();
        println!("Starting model download: {} -> {}", model_id, model_path.display());
        
        let download_res = perform_file_download(
            &client,
            &primary_url,
            &mirror_url,
            &model_path,
            &app_handle_clone,
            &model_id,
            &model_name,
            total_bytes,
            Instant::now(),
        ).await;

        let mut success = false;
        if download_res.is_ok() {
            if verify_file_sha256(&model_path, &expected_sha256).is_ok() {
                success = true;
            }
        }

        if success {
            if let Ok(dl_manager) = DownloadManager::new(&app_handle_clone) {
                if let Ok(mut history) = dl_manager.load_history() {
                    if let Some(pos) = history.iter().position(|e| e.id == model_id) {
                        history[pos].status = "completed".to_string();
                        let _ = dl_manager.save_history(&history);
                    }
                }
            }
            
            let _ = app_handle_clone.emit("download-progress", DownloadProgressPayload {
                id: model_id.clone(),
                model_name,
                progress: 100.0,
                speed_mbps: 0.0,
                downloaded_bytes: total_bytes,
                total_bytes,
                status: "completed".to_string(),
            });

            // Auto-start the server if this is the active model and auto-correction is enabled
            let settings_manager = crate::system::settings::SettingsManager::new(&app_handle_clone);
            if let Ok(sm) = settings_manager {
                if let Ok(settings) = sm.load() {
                    if settings.enable_autocorrection && settings.autocorrection_model == model_id {
                        println!("Model download completed and matches active model. Auto-starting llama-server...");
                        let _ = crate::system::llama::start_server_internal(&app_handle_clone).await;
                    }
                }
            }
        } else {
            if model_path.exists() {
                let _ = fs::remove_file(&model_path);
            }
            
            if let Ok(dl_manager) = DownloadManager::new(&app_handle_clone) {
                if let Ok(mut history) = dl_manager.load_history() {
                    if let Some(pos) = history.iter().position(|e| e.id == model_id) {
                        history[pos].status = "failed".to_string();
                        let _ = dl_manager.save_history(&history);
                    }
                }
            }
            
            let _ = app_handle_clone.emit("download-progress", DownloadProgressPayload {
                id: model_id,
                model_name,
                progress: 0.0,
                speed_mbps: 0.0,
                downloaded_bytes: 0,
                total_bytes,
                status: "failed".to_string(),
            });
        }
    });

    Ok(())
}

pub async fn get_required_arch_key(app_handle: &tauri::AppHandle) -> Result<String, String> {
    let settings_manager = crate::system::settings::SettingsManager::new(app_handle)?;
    let settings = settings_manager.load()?;
    
    let gpu_names = crate::commands::app::get_gpu_names().await;
    let has_gpu = !gpu_names.is_empty();
    
    let mut is_nvidia = false;
    for gpu in &gpu_names {
        let name_lower = gpu.to_ascii_lowercase();
        if name_lower.contains("nvidia") 
            || name_lower.contains("geforce") 
            || name_lower.contains("rtx") 
            || name_lower.contains("gtx") 
        {
            is_nvidia = true;
            break;
        }
    }
    if !is_nvidia {
        is_nvidia = std::path::Path::new("/dev/nvidia0").exists();
    }

    let mut arch_key = String::new();
    
    #[cfg(target_os = "windows")]
    {
        arch_key.push_str("windows");
    }
    #[cfg(target_os = "macos")]
    {
        arch_key.push_str("macos");
    }
    #[cfg(target_os = "linux")]
    {
        arch_key.push_str("linux");
    }
    
    arch_key.push_str("_");
    
    #[cfg(target_arch = "x86_64")]
    {
        arch_key.push_str("x86_64");
        let use_gpu_bin = (settings.execution_backend == "gpu" || settings.execution_backend == "hybrid") && has_gpu;
        
        if use_gpu_bin {
            #[cfg(target_os = "linux")]
            {
                if is_nvidia {
                    arch_key.push_str("_gpu_cuda");
                } else {
                    arch_key.push_str("_gpu_vulkan");
                }
            }
            #[cfg(not(target_os = "linux"))]
            {
                arch_key.push_str("_gpu");
            }
        } else {
            arch_key.push_str("_cpu");
        }
    }
    #[cfg(target_arch = "aarch64")]
    {
        arch_key.push_str("aarch64");
    }
    
    Ok(arch_key)
}

#[tauri::command]
pub async fn download_llama_engine(
    app_handle: AppHandle,
    download_manager: State<'_, DownloadManager>,
) -> Result<(), String> {
    let models_data: ModelsJson = serde_json::from_str(MODELS_JSON)
        .map_err(|e| format!("Failed to parse models.json: {}", e))?;

    let arch_key = get_required_arch_key(&app_handle).await?;

    let binary_asset = match arch_key.as_str() {
        "windows_x86_64_cpu" => &models_data.llama_binaries.windows_x86_64_cpu,
        "windows_x86_64_gpu" => &models_data.llama_binaries.windows_x86_64_gpu,
        "windows_aarch64" => &models_data.llama_binaries.windows_aarch64,
        "macos_x86_64" => &models_data.llama_binaries.macos_x86_64,
        "macos_aarch64" => &models_data.llama_binaries.macos_aarch64,
        "linux_x86_64_cpu" => &models_data.llama_binaries.linux_x86_64_cpu,
        "linux_x86_64_gpu_vulkan" => &models_data.llama_binaries.linux_x86_64_gpu_vulkan,
        "linux_x86_64_gpu_cuda" => &models_data.llama_binaries.linux_x86_64_gpu_cuda,
        "linux_aarch64" => &models_data.llama_binaries.linux_aarch64,
        _ => return Err(format!("Unsupported platform/architecture profile: {}", arch_key)),
    };

    let bin_url = binary_asset.url.clone();
    let bin_mirror_url = binary_asset.mirror_url.clone();
    let bin_sha256 = binary_asset.sha256.clone();

    let app_dir = app_handle.path().app_local_data_dir()
        .map_err(|e| e.to_string())?;
    
    let mut bin_dir = app_dir.clone();
    bin_dir.push("bin");
    fs::create_dir_all(&bin_dir).map_err(|e| e.to_string())?;

    let timestamp = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap()
        .as_secs();

    let bin_entry = DownloadLogEntry {
        id: "dl_llama_bin".to_string(),
        model_name: "Llama Compiler Engine".to_string(),
        file_path: bin_dir.join(LLAMA_BIN_NAME).to_string_lossy().to_string(),
        status: "downloading".to_string(),
        size_bytes: 12_000_000,
        timestamp,
    };
    download_manager.add_or_update_entry(bin_entry)?;

    let app_handle_clone = app_handle.clone();
    
    tauri::async_runtime::spawn(async move {
        let client = Client::new();
        println!("Starting download of Llama binary for target: {}", arch_key);
        let ext = if bin_url.ends_with(".tar.gz") { "tar.gz" } else { "zip" };
        let mut zip_path = bin_dir.clone();
        zip_path.push(format!("llama.{}", ext));
        
        let download_res = perform_file_download(
            &client,
            &bin_url,
            &bin_mirror_url,
            &zip_path,
            &app_handle_clone,
            "dl_llama_bin",
            "Llama Compiler Engine",
            12_000_000,
            Instant::now(),
        ).await;

        let mut final_bin_path: Option<PathBuf> = None;
        
        if download_res.is_ok() {
            if verify_file_sha256(&zip_path, &bin_sha256).is_ok() {
                if extract_archive(&zip_path, &bin_dir).is_ok() {
                    let _ = std::fs::write(bin_dir.join("backend.txt"), &arch_key);
                    final_bin_path = get_binary_path(&bin_dir, LLAMA_BIN_NAME);
                }
            }
            let _ = fs::remove_file(&zip_path);
        }

        if let Some(path) = final_bin_path {
            let _ = make_executable(&path);
            if let Ok(dl_manager) = DownloadManager::new(&app_handle_clone) {
                if let Ok(mut history) = dl_manager.load_history() {
                    if let Some(pos) = history.iter().position(|e| e.id == "dl_llama_bin") {
                        history[pos].status = "completed".to_string();
                        history[pos].file_path = path.to_string_lossy().to_string();
                        let _ = dl_manager.save_history(&history);
                    }
                }
            }
            let _ = app_handle_clone.emit("download-progress", DownloadProgressPayload {
                id: "dl_llama_bin".to_string(),
                model_name: "Llama Compiler Engine".to_string(),
                progress: 100.0,
                speed_mbps: 0.0,
                downloaded_bytes: 12_000_000,
                total_bytes: 12_000_000,
                status: "completed".to_string(),
            });
        } else {
            if let Ok(dl_manager) = DownloadManager::new(&app_handle_clone) {
                if let Ok(mut history) = dl_manager.load_history() {
                    if let Some(pos) = history.iter().position(|e| e.id == "dl_llama_bin") {
                        history[pos].status = "failed".to_string();
                        let _ = dl_manager.save_history(&history);
                    }
                }
            }
            let _ = app_handle_clone.emit("download-progress", DownloadProgressPayload {
                id: "dl_llama_bin".to_string(),
                model_name: "Llama Compiler Engine".to_string(),
                progress: 0.0,
                speed_mbps: 0.0,
                downloaded_bytes: 0,
                total_bytes: 12_000_000,
                status: "failed".to_string(),
            });
        }
    });

    Ok(())
}
