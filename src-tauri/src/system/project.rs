use std::fs;
use std::path::{Path, PathBuf};
use std::time::{SystemTime, UNIX_EPOCH};
use serde::{Deserialize, Serialize};
use tauri::{AppHandle, Manager};

/// Configuration details for a single project.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProjectConfig {
    pub id: String,
    pub name: String,
    pub path: PathBuf,
    pub created_at: u64,
    pub last_opened: u64,
}

/// Structure representing the list of registered projects.
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct ProjectIndex {
    pub projects: Vec<ProjectConfig>,
}

/// Representation of a file or folder in the project file explorer.
#[derive(Debug, Clone, Serialize)]
pub struct FileSystemItem {
    pub name: String,
    pub path: String,
    pub is_dir: bool,
    pub size_bytes: Option<u64>,
}

/// ProjectManager handles the project index and filesystem CRUD operations.
pub struct ProjectManager {
    config_dir: PathBuf,
    index_file: PathBuf,
}

impl ProjectManager {
    /// Creates a new ProjectManager resolving the user's AppConfig directory.
    pub fn new(app_handle: &AppHandle) -> Result<Self, String> {
        let config_dir = app_handle
            .path()
            .app_config_dir()
            .map_err(|e| format!("Failed to resolve AppConfig directory: {}", e))?;
        
        let index_file = config_dir.join("projects.json");
        
        Ok(Self {
            config_dir,
            index_file,
        })
    }

    /// Loads the projects list from projects.json.
    /// Creates an empty list if the index file is missing.
    pub fn load_index(&self) -> Result<ProjectIndex, String> {
        if !self.config_dir.exists() {
            fs::create_dir_all(&self.config_dir)
                .map_err(|e| format!("Failed to create config folder: {}", e))?;
        }

        if !self.index_file.exists() {
            let default_index = ProjectIndex::default();
            self.save_index(&default_index)?;
            return Ok(default_index);
        }

        let content = fs::read_to_string(&self.index_file)
            .map_err(|e| format!("Failed to read projects index file: {}", e))?;
        
        let index: ProjectIndex = serde_json::from_str(&content)
            .map_err(|e| format!("Failed to parse projects.json: {}", e))?;
        
        Ok(index)
    }

    /// Saves the projects list to projects.json.
    pub fn save_index(&self, index: &ProjectIndex) -> Result<(), String> {
        let content = serde_json::to_string_pretty(index)
            .map_err(|e| format!("Failed to serialize projects index: {}", e))?;
        
        fs::write(&self.index_file, content)
            .map_err(|e| format!("Failed to write projects index file: {}", e))?;
        
        Ok(())
    }

    /// Registers a new project in the user index.
    pub fn add_project(&self, name: &str, path: PathBuf) -> Result<ProjectConfig, String> {
        let mut index = self.load_index()?;

        // Ensure the project folder actually exists
        if !path.exists() {
            fs::create_dir_all(&path)
                .map_err(|e| format!("Failed to create new project directory on disk: {}", e))?;
        }

        let now = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .map(|d| d.as_secs())
            .unwrap_or(0);

        let id = uuid_v4_placeholder();

        let new_project = ProjectConfig {
            id,
            name: name.to_string(),
            path,
            created_at: now,
            last_opened: now,
        };

        index.projects.push(new_project.clone());
        self.save_index(&index)?;

        Ok(new_project)
    }

    /// Unregisters a project from the user index.
    pub fn remove_project(&self, id: &str) -> Result<(), String> {
        let mut index = self.load_index()?;
        let initial_len = index.projects.len();
        
        index.projects.retain(|p| p.id != id);
        
        if index.projects.len() == initial_len {
            return Err("Project ID not found in registry".to_string());
        }

        self.save_index(&index)?;
        Ok(())
    }

    /// Updates the last_opened timestamp of a project in the registry.
    pub fn touch_project(&self, id: &str) -> Result<ProjectConfig, String> {
        let mut index = self.load_index()?;
        let now = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .map(|d| d.as_secs())
            .unwrap_or(0);
        
        if let Some(project) = index.projects.iter_mut().find(|p| p.id == id) {
            project.last_opened = now;
            let updated_project = project.clone();
            self.save_index(&index)?;
            Ok(updated_project)
        } else {
            Err("Project ID not found in registry".to_string())
        }
    }

    /// Renames an existing project in the registry.
    pub fn rename_project(&self, id: &str, new_name: &str) -> Result<ProjectConfig, String> {
        let mut index = self.load_index()?;
        
        if let Some(project) = index.projects.iter_mut().find(|p| p.id == id) {
            project.name = new_name.to_string();
            let updated_project = project.clone();
            self.save_index(&index)?;
            Ok(updated_project)
        } else {
            Err("Project ID not found in registry".to_string())
        }
    }

    /// Scans a directory path and returns its immediate files and folders.
    pub fn list_directory(&self, path: &Path) -> Result<Vec<FileSystemItem>, String> {
        if !path.exists() {
            return Err("Target directory path does not exist".to_string());
        }

        let mut items = Vec::new();
        let entries = fs::read_dir(path)
            .map_err(|e| format!("Failed to read directory entries: {}", e))?;

        for entry in entries {
            if let Ok(entry) = entry {
                let entry_path = entry.path();
                let name = entry_path
                    .file_name()
                    .and_then(|s| s.to_str())
                    .unwrap_or("unknown")
                    .to_string();
                
                let is_dir = entry_path.is_dir();
                let size_bytes = if is_dir {
                    None
                } else {
                    entry.metadata().ok().map(|m| m.len())
                };

                items.push(FileSystemItem {
                    name,
                    path: entry_path.to_string_lossy().to_string(),
                    is_dir,
                    size_bytes,
                });
            }
        }

        // Sort: directories first, then alphabetically
        items.sort_by(|a, b| {
            if a.is_dir != b.is_dir {
                b.is_dir.cmp(&a.is_dir)
            } else {
                a.name.to_lowercase().cmp(&b.name.to_lowercase())
            }
        });

        Ok(items)
    }

    /// Creates a directory folder.
    pub fn create_dir(&self, path: &Path) -> Result<(), String> {
        fs::create_dir_all(path)
            .map_err(|e| format!("Failed to create folder: {}", e))
    }

    /// Writes content to a file, creating it if missing.
    pub fn write_file(&self, path: &Path, content: &str) -> Result<(), String> {
        if let Some(parent) = path.parent() {
            fs::create_dir_all(parent)
                .map_err(|e| format!("Failed to create parent folder structure: {}", e))?;
        }
        
        fs::write(path, content)
            .map_err(|e| format!("Failed to write file content: {}", e))
    }

    /// Deletes a file or directory recursively.
    pub fn delete_path(&self, path: &Path) -> Result<(), String> {
        if !path.exists() {
            return Err("Target path does not exist".to_string());
        }

        if path.is_dir() {
            fs::remove_dir_all(path)
                .map_err(|e| format!("Failed to delete directory recursively: {}", e))
        } else {
            fs::remove_file(path)
                .map_err(|e| format!("Failed to delete file: {}", e))
        }
    }
}

/// Minimal helper to generate a unique pseudorandom ID.
fn uuid_v4_placeholder() -> String {
    let now = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map(|d| d.as_nanos())
        .unwrap_or(0);
    format!("{:x}", now)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_uuid_generation() {
        let id1 = uuid_v4_placeholder();
        let id2 = uuid_v4_placeholder();
        assert_ne!(id1, id2);
        assert!(!id1.is_empty());
    }
}
