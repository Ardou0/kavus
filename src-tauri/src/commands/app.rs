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
    pub executable_models: Vec<String>,
    pub cpu_cores: usize,
    pub is_server_running: bool,
}

/// Evaluates host RAM, CPU cores, and GPU capacity to determine the best local LLM size.
/// Evaluates dynamically against requirements loaded from models.json.
#[tauri::command]
pub async fn check_hardware_performance(app_handle: tauri::AppHandle) -> HardwareProfile {
    use sysinfo::System;
    use tauri::Manager;
    let mut sys = System::new_all();
    sys.refresh_memory();
    let total_ram_gb = (sys.total_memory() as f32) / 1024.0 / 1024.0 / 1024.0;
    let cpu_cores = sys.cpus().len();

    let gpu_names = get_gpu_names().await;
    let has_gpu = !gpu_names.is_empty();

    let is_server_running = if let Some(server_manager) = app_handle.try_state::<crate::system::llama::LlamaServerManager>() {
        server_manager.is_running()
    } else {
        false
    };

    // Read models.json
    let models_json_str = include_str!("../../models.json");
    let mut executable_models = Vec::new();
    let mut recommended_model = "qwen-1.5b".to_string(); // fallback default
    let mut best_score = -1.0;

    // Detect GPU VRAM to ensure large models fit in VRAM
    let mut total_vram_gb: f32 = 0.0;
    
    #[cfg(target_os = "windows")]
    {
        if let Ok(output) = tokio::process::Command::new("powershell")
            .args(&["-NoProfile", "-Command", "Get-ItemProperty -Path 'HKLM:\\SYSTEM\\ControlSet001\\Control\\Class\\{4d36e968-e325-11ce-bfc1-08002be10318}\\0*' -ErrorAction SilentlyContinue | Where-Object { $_.'HardwareInformation.qwMemorySize' } | Select-Object -ExpandProperty 'HardwareInformation.qwMemorySize'"])
            .output()
            .await
        {
            let s = String::from_utf8_lossy(&output.stdout);
            let mut max_bytes: u64 = 0;
            for line in s.lines() {
                if let Ok(bytes) = line.trim().parse::<u64>() {
                    if bytes > max_bytes {
                        max_bytes = bytes;
                    }
                }
            }
            if max_bytes > 0 {
                total_vram_gb = (max_bytes as f32) / 1024.0 / 1024.0 / 1024.0;
            }
        }
    }

    #[cfg(target_os = "macos")]
    {
        // Apple Silicon uses Unified Memory where VRAM = System RAM
        total_vram_gb = total_ram_gb;
    }

    #[cfg(target_os = "linux")]
    {
        // Try reading sysfs files (e.g. amdgpu)
        if let Ok(entries) = std::fs::read_dir("/sys/class/drm") {
            let mut max_bytes: u64 = 0;
            for entry in entries.flatten() {
                let path = entry.path().join("device").join("mem_info_vram_total");
                if path.exists() {
                    if let Ok(content) = std::fs::read_to_string(path) {
                        if let Ok(bytes) = content.trim().parse::<u64>() {
                            if bytes > max_bytes {
                                max_bytes = bytes;
                            }
                        }
                    }
                }
            }
            if max_bytes > 0 {
                total_vram_gb = (max_bytes as f32) / 1024.0 / 1024.0 / 1024.0;
            }
        }
        
        // Fallback or override for NVIDIA using nvidia-smi if installed
        if total_vram_gb < 1.0 {
            if let Ok(output) = tokio::process::Command::new("nvidia-smi")
                .args(&["--query-gpu=memory.total", "--format=csv,noheader,nounits"])
                .output()
                .await
            {
                let s = String::from_utf8_lossy(&output.stdout);
                if let Some(line) = s.lines().next() {
                    if let Ok(mb) = line.trim().parse::<f32>() {
                        total_vram_gb = mb / 1024.0;
                    }
                }
            }
        }
    }

    #[derive(serde::Deserialize)]
    struct LocalRequirements {
        min_ram_gb: f32,
        requires_gpu: bool,
        recommended_gpu: bool,
    }

    #[derive(serde::Deserialize)]
    struct LocalModelConfig {
        id: String,
        #[allow(dead_code)]
        name: String,
        size_bytes: u64,
        requirements: LocalRequirements,
    }

    #[derive(serde::Deserialize)]
    struct LocalModelsJson {
        models: Vec<LocalModelConfig>,
    }

    if let Ok(data) = serde_json::from_str::<LocalModelsJson>(models_json_str) {
        for model in data.models {
            // Check GPU requirements
            if model.requirements.requires_gpu && !has_gpu {
                continue;
            }

            // Check RAM limits
            if total_ram_gb < model.requirements.min_ram_gb {
                continue;
            }

            // Estimate model memory footprint (size_bytes is in GGUF weight format, we add 2GB margin for Context/Prompt)
            let estimated_memory_need_gb = (model.size_bytes as f32) / 1024.0 / 1024.0 / 1024.0 + 2.0;

            // If user has GPU, check if the model would overflow dedicated VRAM, forcing slow swap/system memory fallback.
            // On non-mac dedicated GPUs, if the model size + context overhead exceeds the VRAM, it's not a good fit.
            #[cfg(not(target_os = "macos"))]
            {
                if has_gpu && total_vram_gb > 0.1 && estimated_memory_need_gb > total_vram_gb {
                    // Reduce priority or skip if dedicated GPU offload is recommended but we don't have enough VRAM
                    if model.requirements.requires_gpu || model.requirements.recommended_gpu {
                        continue;
                    }
                }
            }

            // If we match, this model is executable
            executable_models.push(model.id.clone());

            // Score for recommendation
            // We want the largest/best model that fits comfortably.
            let mut score = model.requirements.min_ram_gb; // base score is min_ram_gb (larger models have higher scores)
            
            // Add a bonus if it matches GPU profile and fits completely in VRAM
            if has_gpu && model.requirements.recommended_gpu {
                if total_vram_gb >= estimated_memory_need_gb {
                    score += 15.0; // High bonus for fully fitting in VRAM
                } else {
                    score -= 10.0; // Heavy penalty if it overflows VRAM (forces hybrid/swapping on PC)
                }
            }

            // Do not recommend models that require more than 90% of our RAM if we are CPU-only
            if !has_gpu && model.requirements.min_ram_gb > (total_ram_gb * 0.9) {
                score -= 20.0;
            }

            if score > best_score {
                best_score = score;
                recommended_model = model.id;
            }
        }
    }

    HardwareProfile {
        total_ram_gb,
        has_gpu,
        recommended_model,
        executable_models,
        cpu_cores,
        is_server_running,
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
