#[tauri::command]
pub fn greet(name: &str) -> String {
    crate::i18n::strings().greet_template.replace("{}", name)
}

#[tauri::command]
pub fn close_application(window: tauri::WebviewWindow) {
    let _ = window.hide();
}

#[tauri::command]
pub fn start_drag(window: tauri::WebviewWindow) {
    let _ = window.start_dragging();
}

#[tauri::command]
pub fn minimize_application(window: tauri::WebviewWindow) {
    let _ = window.minimize();
}

#[tauri::command]
pub fn toggle_maximize_application(window: tauri::WebviewWindow) {
    if let Ok(maximized) = window.is_maximized() {
        if maximized {
            let _ = window.unmaximize();
        } else {
            let _ = window.maximize();
        }
    }
}

#[tauri::command]
pub fn hide_to_tray(window: tauri::WebviewWindow) {
    let _ = window.hide();
}

#[tauri::command]
pub fn exit_application(app_handle: tauri::AppHandle) {
    app_handle.exit(0);
}

#[derive(serde::Serialize)]
pub struct HardwareProfile {
    pub total_ram_gb: f32,
    pub has_gpu: bool,
    pub recommended_model: String,
    pub cpu_cores: usize,
}

/// Evaluates host RAM, CPU cores, and GPU capacity to determine the best local LLM size.
/// For Apple Silicon, unified memory sizes allow heavier models; Windows/Linux systems with
/// dedicated GPUs target high-fidelity 8-bit quantized models that fit fully in VRAM.
#[tauri::command]
pub async fn check_hardware_performance() -> HardwareProfile {
    use sysinfo::System;
    let mut sys = System::new_all();
    sys.refresh_memory();
    let total_ram_gb = (sys.total_memory() as f32) / 1024.0 / 1024.0 / 1024.0;
    let cpu_cores = sys.cpus().len();

    let gpu_names = get_gpu_names().await;
    let has_gpu = !gpu_names.is_empty();

    let recommended_model = if has_gpu {
        #[cfg(target_os = "macos")]
        {
            if total_ram_gb >= 30.0 {
                "qwen-32b".to_string()
            } else if total_ram_gb >= 15.0 {
                "llama3.1-8b".to_string()
            } else {
                "qwen-3b".to_string()
            }
        }
        #[cfg(not(target_os = "macos"))]
        {
            // Dedicated GPU system (Nvidia/Vulkan)
            "qwen-7b-q8".to_string()
        }
    } else {
        // CPU-only system
        if total_ram_gb >= 30.0 {
            "llama3.1-8b".to_string()
        } else if total_ram_gb >= 15.0 {
            "qwen-3b".to_string()
        } else {
            "qwen-1.5b".to_string()
        }
    };

    HardwareProfile {
        total_ram_gb,
        has_gpu,
        recommended_model,
        cpu_cores,
    }
}

pub async fn get_gpu_names() -> Vec<String> {
    let mut gpus = Vec::new();
    
    #[cfg(target_os = "windows")]
    {
        if let Ok(output) = tokio::process::Command::new("powershell")
            .args(&["-NoProfile", "-Command", "Get-CimInstance Win32_VideoController | Select-Object -ExpandProperty Name"])
            .output()
            .await
        {
            let s = String::from_utf8_lossy(&output.stdout);
            for line in s.lines() {
                let name = line.trim();
                if !name.is_empty() {
                    gpus.push(name.to_string());
                }
            }
        }
    }
    
    #[cfg(target_os = "macos")]
    {
        gpus.push("Apple Silicon Unified GPU".to_string());
    }
    
    #[cfg(target_os = "linux")]
    {
        if let Ok(output) = tokio::process::Command::new("lspci").output().await {
            let s = String::from_utf8_lossy(&output.stdout);
            for line in s.lines() {
                let line_lower = line.to_ascii_lowercase();
                if line_lower.contains("vga") 
                    || line_lower.contains("3d") 
                    || line_lower.contains("display") 
                {
                    let mut clean_name = String::new();
                    if let (Some(start), Some(end)) = (line.rfind('['), line.rfind(']')) {
                        if start < end {
                            let in_brackets = &line[start + 1..end];
                            let first_part = in_brackets.split('/').next().unwrap_or(in_brackets).trim();
                            clean_name = first_part.to_string();
                        }
                    }
                    
                    if clean_name.is_empty() {
                        if let Some(pos) = line.find("controller:") {
                            clean_name = line[pos + 11..].trim().to_string();
                        } else if let Some(pos) = line.find("VGA compatible controller:") {
                            clean_name = line[pos + 26..].trim().to_string();
                        } else {
                            clean_name = line.to_string();
                        }
                    }
                    
                    if let Some(rev_pos) = clean_name.find("(rev") {
                        clean_name = clean_name[..rev_pos].trim().to_string();
                    }
                    
                    if !clean_name.is_empty() && !gpus.contains(&clean_name) {
                        gpus.push(clean_name);
                    }
                }
            }
        }
        if gpus.is_empty() && std::path::Path::new("/dev/nvidia0").exists() {
            gpus.push("NVIDIA Graphics Card".to_string());
        }
    }
    
    gpus
}

#[tauri::command]
pub async fn get_gpu_devices() -> Result<Vec<String>, String> {
    Ok(get_gpu_names().await)
}

#[derive(serde::Serialize)]
pub struct DependencyStatus {
    pub name: String,
    pub status: String, // "installed", "missing", "native"
    pub details: String,
    pub download_url: String,
}

#[tauri::command]
pub async fn check_system_dependencies(app_handle: tauri::AppHandle) -> Result<Vec<DependencyStatus>, String> {
    use tauri::Manager;
    let mut deps = Vec::new();

    // 1. Llama Compiler Engine
    let app_dir = app_handle.path().app_local_data_dir()
        .map_err(|e| e.to_string())?;
    let mut bin_dir = app_dir.clone();
    bin_dir.push("bin");
    
    #[cfg(target_os = "windows")]
    const BIN_NAME: &str = "llama-cli.exe";
    #[cfg(not(target_os = "windows"))]
    const BIN_NAME: &str = "llama-cli";

    let has_llama = crate::system::download::get_binary_path(&bin_dir, BIN_NAME).is_some();
    let mut backend_match = false;
    let mut current_backend = String::new();
    let mut required_backend = String::new();

    if has_llama {
        if let Ok(req) = crate::commands::download::get_required_arch_key(&app_handle).await {
            required_backend = req;
            if let Ok(existing) = std::fs::read_to_string(bin_dir.join("backend.txt")) {
                current_backend = existing.trim().to_string();
                if current_backend == required_backend {
                    backend_match = true;
                }
            }
        }
    }

    let status = if has_llama && backend_match {
        "installed".to_string()
    } else if has_llama {
        "outdated".to_string()
    } else {
        "missing".to_string()
    };

    let details = if status == "installed" {
        format!("Compiler engine is ready (profile: {}).", current_backend)
    } else if status == "outdated" {
        format!("Compiler engine profile mismatch: current is '{}' but settings require '{}'. Click Setup to update it.", current_backend, required_backend)
    } else {
        "Required to execute local LLM inference models. Click Setup to download and install.".to_string()
    };

    deps.push(DependencyStatus {
        name: "Llama Compiler Engine".to_string(),
        status,
        details,
        download_url: "trigger_llama_download".to_string(),
    });

    // 2. GPU Driver & SDK
    let gpu_names = get_gpu_names().await;
    
    let mut is_nvidia = false;
    for gpu in &gpu_names {
        let name_lower = gpu.to_ascii_lowercase();
        if name_lower.contains("nvidia") || name_lower.contains("geforce") || name_lower.contains("rtx") || name_lower.contains("gtx") {
            is_nvidia = true;
            break;
        }
    }

    #[cfg(target_os = "macos")]
    {
        deps.push(DependencyStatus {
            name: "Apple Metal API".to_string(),
            status: "native".to_string(),
            details: "Native hardware acceleration support via Apple Metal.".to_string(),
            download_url: "".to_string(),
        });
    }

    #[cfg(target_os = "windows")]
    {
        if is_nvidia {
            let cuda_path_exists = std::env::var("CUDA_PATH").is_ok() 
                || std::path::Path::new("C:\\Windows\\System32\\nvcuda.dll").exists();
            
            deps.push(DependencyStatus {
                name: "NVIDIA CUDA Toolkit 12.x".to_string(),
                status: if cuda_path_exists { "installed".to_string() } else { "missing".to_string() },
                details: if cuda_path_exists { 
                    "CUDA drivers detected. High performance GPU offloading is ready.".to_string() 
                } else { 
                    "CUDA Toolkit is missing. Install it to offload layers to your NVIDIA GPU.".to_string() 
                },
                download_url: "https://developer.nvidia.com/cuda-downloads".to_string(),
            });
        } else {
            let vulkan_exists = std::path::Path::new("C:\\Windows\\System32\\vulkan-1.dll").exists();
            deps.push(DependencyStatus {
                name: "Vulkan Runtime".to_string(),
                status: if vulkan_exists { "installed".to_string() } else { "missing".to_string() },
                details: if vulkan_exists { 
                    "Vulkan Runtime library detected.".to_string() 
                } else { 
                    "Vulkan Runtime (`vulkan-1.dll`) is missing. Install Vulkan drivers to run Hybrid mode.".to_string() 
                },
                download_url: "https://vulkan.lunarg.com/sdk/home".to_string(),
            });
        }
    }

    #[cfg(target_os = "linux")]
    {
        if is_nvidia {
            let nvidia_exists = std::path::Path::new("/proc/driver/nvidia/version").exists() 
                || std::path::Path::new("/dev/nvidia0").exists();
            deps.push(DependencyStatus {
                name: "NVIDIA CUDA Driver".to_string(),
                status: if nvidia_exists { "installed".to_string() } else { "missing".to_string() },
                details: if nvidia_exists { 
                    "NVIDIA driver detected. Ready for CUDA offloading.".to_string() 
                } else { 
                    "NVIDIA graphics drivers or CUDA libraries are missing. Click setup to install.".to_string() 
                },
                download_url: "https://developer.nvidia.com/cuda-downloads".to_string(),
            });
        } else {
            let libvulkan_exists = std::path::Path::new("/usr/lib/libvulkan.so.1").exists()
                || std::path::Path::new("/usr/lib/x86_64-linux-gnu/libvulkan.so.1").exists()
                || std::path::Path::new("/usr/lib64/libvulkan.so.1").exists();
            deps.push(DependencyStatus {
                name: "Vulkan Loader (libvulkan)".to_string(),
                status: if libvulkan_exists { "installed".to_string() } else { "missing".to_string() },
                details: if libvulkan_exists { 
                    "Vulkan library detected.".to_string() 
                } else { 
                    "Vulkan Loader library (`libvulkan.so.1`) is missing. Install Vulkan drivers/packages.".to_string() 
                },
                download_url: "https://vulkan.lunarg.com/sdk/home".to_string(),
            });
        }
    }

    Ok(deps)
}

#[tauri::command]
pub async fn install_system_dependency(download_url: String) -> Result<(), String> {
    if !download_url.is_empty() {
        #[cfg(target_os = "windows")]
        let _ = std::process::Command::new("cmd").args(&["/c", "start", "", &download_url]).spawn();
        #[cfg(target_os = "macos")]
        let _ = std::process::Command::new("open").arg(&download_url).spawn();
        #[cfg(target_os = "linux")]
        let _ = std::process::Command::new("xdg-open").arg(&download_url).spawn();
    }
    Ok(())
}
