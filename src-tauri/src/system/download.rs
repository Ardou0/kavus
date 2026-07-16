use std::fs::{self, File};
use std::path::{Path, PathBuf};
use std::io::{Read, Write};
use std::time::Instant;
use tauri::{AppHandle, Manager, Emitter};
use reqwest::Client;
use sha2::{Sha256, Digest};
use futures_util::StreamExt;

#[derive(serde::Serialize, serde::Deserialize, Clone, Debug)]
pub struct DownloadLogEntry {
    pub id: String,
    pub model_name: String,
    pub file_path: String,
    pub status: String, // "downloading", "completed", "failed", "deleted"
    pub size_bytes: u64,
    pub timestamp: u64,
}

#[derive(serde::Serialize, Clone, Debug)]
pub struct DownloadProgressPayload {
    pub id: String,
    pub model_name: String,
    pub progress: f32, // 0.0 to 100.0
    pub speed_mbps: f32,
    pub downloaded_bytes: u64,
    pub total_bytes: u64,
    pub status: String,
}

pub struct DownloadManager {
    file_path: PathBuf,
}

impl DownloadManager {
    pub fn new(app_handle: &AppHandle) -> Result<Self, String> {
        let mut path = app_handle
            .path()
            .app_local_data_dir()
            .map_err(|e| e.to_string())?;
        
        // Ensure local data dir exists
        if !path.exists() {
            fs::create_dir_all(&path).map_err(|e| e.to_string())?;
        }
        
        path.push("downloads.json");
        Ok(Self { file_path: path })
    }

    pub fn load_history(&self) -> Result<Vec<DownloadLogEntry>, String> {
        if !self.file_path.exists() {
            return Ok(Vec::new());
        }
        let content = fs::read_to_string(&self.file_path).map_err(|e| e.to_string())?;
        let history: Vec<DownloadLogEntry> = serde_json::from_str(&content).map_err(|e| e.to_string())?;
        Ok(history)
    }

    pub fn save_history(&self, history: &[DownloadLogEntry]) -> Result<(), String> {
        let content = serde_json::to_string_pretty(history).map_err(|e| e.to_string())?;
        fs::write(&self.file_path, content).map_err(|e| e.to_string())?;
        Ok(())
    }

    pub fn add_or_update_entry(&self, entry: DownloadLogEntry) -> Result<(), String> {
        let mut history = self.load_history()?;
        if let Some(pos) = history.iter().position(|e| e.id == entry.id) {
            history[pos] = entry;
        } else {
            history.push(entry);
        }
        self.save_history(&history)
    }
}

// Extraction Utilities
fn extract_zip(zip_path: &Path, target_dir: &Path) -> Result<(), String> {
    let file = File::open(zip_path).map_err(|e| e.to_string())?;
    let mut archive = zip::ZipArchive::new(file).map_err(|e| e.to_string())?;
    
    for i in 0..archive.len() {
        let mut file = archive.by_index(i).map_err(|e| e.to_string())?;
        let outpath = match file.enclosed_name() {
            Some(path) => target_dir.join(path),
            None => continue,
        };
        
        if file.name().ends_with('/') {
            fs::create_dir_all(&outpath).map_err(|e| e.to_string())?;
        } else {
            if let Some(p) = outpath.parent() {
                if !p.exists() {
                    fs::create_dir_all(p).map_err(|e| e.to_string())?;
                }
            }
            let mut outfile = File::create(&outpath).map_err(|e| e.to_string())?;
            std::io::copy(&mut file, &mut outfile).map_err(|e| e.to_string())?;
        }
    }
    Ok(())
}

pub fn extract_archive(archive_path: &Path, target_dir: &Path) -> Result<(), String> {
    let path_str = archive_path.to_string_lossy().to_ascii_lowercase();
    if path_str.ends_with(".tar.gz") || path_str.ends_with(".tgz") {
        let output = std::process::Command::new("tar")
            .args(&["-xzf", &archive_path.to_string_lossy(), "-C", &target_dir.to_string_lossy()])
            .output()
            .map_err(|e| format!("Failed to execute tar command: {}", e))?;
        if !output.status.success() {
            let err_msg = String::from_utf8_lossy(&output.stderr);
            return Err(format!("Tar extraction failed: {}", err_msg));
        }
        Ok(())
    } else {
        extract_zip(archive_path, target_dir)
    }
}

// Find binary in directory recursively
pub fn get_binary_path(dir: &Path, bin_name: &str) -> Option<PathBuf> {
    fn walk(dir: &Path, bin_name: &str) -> Option<PathBuf> {
        if let Ok(entries) = fs::read_dir(dir) {
            for entry in entries.flatten() {
                let path = entry.path();
                if path.is_dir() {
                    if let Some(p) = walk(&path, bin_name) {
                        return Some(p);
                    }
                } else if path.file_name().and_then(|n| n.to_str()) == Some(bin_name) {
                    return Some(path);
                }
            }
        }
        None
    }
    walk(dir, bin_name)
}

#[cfg(any(target_os = "linux", target_os = "macos"))]
pub fn make_executable(path: &Path) -> Result<(), String> {
    use std::os::unix::fs::PermissionsExt;
    let mut perms = fs::metadata(path).map_err(|e| e.to_string())?.permissions();
    perms.set_mode(0o755);
    fs::set_permissions(path, perms).map_err(|e| e.to_string())?;
    Ok(())
}

#[cfg(target_os = "windows")]
pub fn make_executable(_path: &Path) -> Result<(), String> {
    Ok(())
}

// Verification Utilities
/// Verifies the checksum of the given file against the expected SHA256 string.
/// Uses a flat 64KB buffer for streaming to keep memory usage low and constant.
pub fn verify_file_sha256(file_path: &Path, expected_hex: &str) -> Result<(), String> {
    if expected_hex.trim().is_empty() {
        return Ok(());
    }
    
    let mut file = File::open(file_path).map_err(|e| e.to_string())?;
    let mut hasher = Sha256::new();
    let mut buffer = [0; 65536];
    
    loop {
        let bytes_read = file.read(&mut buffer).map_err(|e| e.to_string())?;
        if bytes_read == 0 {
            break;
        }
        hasher.update(&buffer[..bytes_read]);
    }
    
    let hash_result = hasher.finalize();
    let calculated_hex = format!("{:x}", hash_result);
    
    if calculated_hex.eq_ignore_ascii_case(expected_hex.trim()) {
        Ok(())
    } else {
        Err(format!(
            "SHA256 checksum mismatch! Expected: {}, Calculated: {}",
            expected_hex, calculated_hex
        ))
    }
}

// HTTP Download Streaming logic
/// Streams a file from a primary URL with transparent automatic fallback to a mirror URL.
/// Emits regular throttled progress payloads to the Tauri front-end interface.
pub async fn perform_file_download(
    client: &Client,
    primary_url: &str,
    mirror_url: &str,
    dest_path: &Path,
    app_handle: &AppHandle,
    model_id: &str,
    model_name: &str,
    estimated_bytes: u64,
    start_time: Instant,
) -> Result<(), String> {
    // Attempt download via primary link first, fall back to mirror if HTTP call fails
    let mut response_opt = client.get(primary_url).send().await.ok();
    
    if response_opt.is_none() || !response_opt.as_ref().unwrap().status().is_success() {
        println!("Primary link failed. Falling back to mirror: {}", mirror_url);
        response_opt = client.get(mirror_url).send().await.ok();
    }
    
    let response = response_opt.ok_or_else(|| "Failed to reach both primary and mirror servers".to_string())?;
    if !response.status().is_success() {
        return Err(format!("Download failed with HTTP status: {}", response.status()));
    }
    
    let total_bytes = response.content_length().unwrap_or(estimated_bytes);
    
    let mut file = File::create(dest_path).map_err(|e| e.to_string())?;
    let mut downloaded: u64 = 0;
    let mut stream = response.bytes_stream();
    
    let mut last_emit = Instant::now();
    let mut last_progress = -1.0;
    
    while let Some(chunk_result) = stream.next().await {
        let chunk = chunk_result.map_err(|e| e.to_string())?;
        file.write_all(&chunk).map_err(|e| e.to_string())?;
        downloaded += chunk.len() as u64;
        
        let progress = (downloaded as f32 / total_bytes as f32) * 100.0;
        let now = Instant::now();
        
        // Emit events at most every 250ms or 0.5% progress step to avoid flooding the front-end IPC channel
        if progress - last_progress >= 0.5 || now.duration_since(last_emit).as_millis() >= 250 || downloaded == total_bytes {
            last_progress = progress;
            last_emit = now;
            
            let elapsed = start_time.elapsed().as_secs_f32();
            let speed_mbps = if elapsed > 0.0 {
                (downloaded as f32 / 1024.0 / 1024.0) / elapsed
            } else {
                0.0
            };
            
            let _ = app_handle.emit("download-progress", DownloadProgressPayload {
                id: model_id.to_string(),
                model_name: model_name.to_string(),
                progress,
                speed_mbps,
                downloaded_bytes: downloaded,
                total_bytes,
                status: "downloading".to_string(),
            });
        }
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    fn get_temp_test_dir(name: &str) -> PathBuf {
        let mut path = std::env::temp_dir();
        path.push(format!("kavus_test_{}_{}", name, std::time::SystemTime::now().duration_since(std::time::UNIX_EPOCH).unwrap().as_nanos()));
        std::fs::create_dir_all(&path).unwrap();
        path
    }

    /// Asserts that SHA256 verification validates matching files and rejects mismatched hashes.
    #[test]
    fn test_verify_file_sha256() {
        let dir = get_temp_test_dir("sha256");
        let file_path = dir.join("test.txt");
        
        let mut file = File::create(&file_path).unwrap();
        file.write_all(b"hello world").unwrap();
        
        let expected_hash = "b94d27b9934d3e08a52e52d7da7dabfac484efe37a5380ee9088f7ace2efcde9";
        
        assert!(verify_file_sha256(&file_path, expected_hash).is_ok());
        assert!(verify_file_sha256(&file_path, "wrong_hash").is_err());
        let _ = std::fs::remove_dir_all(&dir);
    }

    /// Asserts that DownloadManager correctly logs, updates, and saves download items to disk.
    #[test]
    fn test_download_manager_history() {
        let dir = get_temp_test_dir("history");
        let file_path = dir.join("downloads_test.json");
        let manager = DownloadManager { file_path: file_path.clone() };
        
        let history = manager.load_history().unwrap();
        assert!(history.is_empty());
        
        let entry = DownloadLogEntry {
            id: "test-model".to_string(),
            model_name: "Test Model".to_string(),
            file_path: "/dummy/path".to_string(),
            status: "completed".to_string(),
            size_bytes: 12345,
            timestamp: 1600000000,
        };
        
        manager.add_or_update_entry(entry.clone()).unwrap();
        
        let loaded = manager.load_history().unwrap();
        assert_eq!(loaded.len(), 1);
        assert_eq!(loaded[0].id, "test-model");
        assert_eq!(loaded[0].status, "completed");
        let _ = std::fs::remove_dir_all(&dir);
    }

    /// Asserts that zip archives are extracted correctly and nested binary filepaths are located recursively.
    #[test]
    fn test_extract_archive_zip_and_find_binary() {
        let dir = get_temp_test_dir("zip_extract");
        let zip_path = dir.join("archive.zip");
        let target_dir = dir.join("extracted");
        
        let file = File::create(&zip_path).unwrap();
        let mut zip = zip::ZipWriter::new(file);
        
        let options = zip::write::FileOptions::<()>::default();
        zip.start_file("bin/llama-cli", options).unwrap();
        zip.write_all(b"dummy binary contents").unwrap();
        zip.finish().unwrap();
        
        extract_archive(&zip_path, &target_dir).unwrap();
        
        let bin_path = get_binary_path(&target_dir, "llama-cli");
        assert!(bin_path.is_some());
        let path = bin_path.unwrap();
        assert!(path.exists());
        assert!(path.to_string_lossy().ends_with("llama-cli"));
        let _ = std::fs::remove_dir_all(&dir);
    }
}

