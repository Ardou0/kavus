use std::path::PathBuf;
use tauri::State;
use crate::system::project::{FileSystemItem, ProjectConfig, ProjectIndex, ProjectManager};

/// Command to open a native OS folder dialog and return the selected path.
#[tauri::command]
pub async fn pick_project_folder() -> Result<Option<String>, String> {
    tokio::task::spawn_blocking(|| {
        let folder = rfd::FileDialog::new()
            .set_title("Select Project Folder")
            .pick_folder();
        Ok(folder.map(|p| p.to_string_lossy().to_string()))
    })
    .await
    .map_err(|e| format!("Failed to spawn file dialog task: {}", e))?
}

/// Command to retrieve the index of all registered projects.
#[tauri::command]
pub async fn list_projects(
    project_manager: State<'_, ProjectManager>,
) -> Result<ProjectIndex, String> {
    let manager = project_manager.inner().clone();
    tokio::task::spawn_blocking(move || manager.load_index())
        .await
        .map_err(|e| format!("Failed to execute list_projects task: {}", e))?
}

/// Command to add/register a new project.
#[tauri::command]
pub async fn add_project(
    name: String,
    path: String,
    project_manager: State<'_, ProjectManager>,
) -> Result<ProjectConfig, String> {
    let manager = project_manager.inner().clone();
    tokio::task::spawn_blocking(move || manager.add_project(&name, PathBuf::from(path)))
        .await
        .map_err(|e| format!("Failed to execute add_project task: {}", e))?
}

/// Command to remove/unregister a project by its unique ID.
#[tauri::command]
pub async fn remove_project(
    id: String,
    project_manager: State<'_, ProjectManager>,
) -> Result<(), String> {
    let manager = project_manager.inner().clone();
    tokio::task::spawn_blocking(move || manager.remove_project(&id))
        .await
        .map_err(|e| format!("Failed to execute remove_project task: {}", e))?
}

/// Command to touch a project (updating its last_opened timestamp).
#[tauri::command]
pub async fn touch_project(
    id: String,
    project_manager: State<'_, ProjectManager>,
) -> Result<ProjectConfig, String> {
    let manager = project_manager.inner().clone();
    tokio::task::spawn_blocking(move || manager.touch_project(&id))
        .await
        .map_err(|e| format!("Failed to execute touch_project task: {}", e))?
}

/// Command to rename a project.
#[tauri::command]
pub async fn rename_project(
    id: String,
    new_name: String,
    project_manager: State<'_, ProjectManager>,
) -> Result<ProjectConfig, String> {
    let manager = project_manager.inner().clone();
    tokio::task::spawn_blocking(move || manager.rename_project(&id, &new_name))
        .await
        .map_err(|e| format!("Failed to execute rename_project task: {}", e))?
}

/// Command to scan a directory path inside a project and return files/folders.
#[tauri::command]
pub async fn list_project_files(
    path: String,
    project_manager: State<'_, ProjectManager>,
) -> Result<Vec<FileSystemItem>, String> {
    let manager = project_manager.inner().clone();
    tokio::task::spawn_blocking(move || manager.list_directory(&PathBuf::from(path)))
        .await
        .map_err(|e| format!("Failed to execute list_project_files task: {}", e))?
}

/// Command to create a directory folder.
#[tauri::command]
pub async fn create_project_directory(
    path: String,
    project_manager: State<'_, ProjectManager>,
) -> Result<(), String> {
    let manager = project_manager.inner().clone();
    tokio::task::spawn_blocking(move || manager.create_dir(&PathBuf::from(path)))
        .await
        .map_err(|e| format!("Failed to execute create_project_directory task: {}", e))?
}

/// Command to create or write content to a project file.
#[tauri::command]
pub async fn create_project_file(
    path: String,
    content: String,
    project_manager: State<'_, ProjectManager>,
) -> Result<(), String> {
    let manager = project_manager.inner().clone();
    tokio::task::spawn_blocking(move || manager.write_file(&PathBuf::from(path), &content))
        .await
        .map_err(|e| format!("Failed to execute create_project_file task: {}", e))?
}

/// Command to delete a file or directory recursively.
#[tauri::command]
pub async fn delete_project_item(
    path: String,
    project_manager: State<'_, ProjectManager>,
) -> Result<(), String> {
    let manager = project_manager.inner().clone();
    tokio::task::spawn_blocking(move || manager.delete_path(&PathBuf::from(path)))
        .await
        .map_err(|e| format!("Failed to execute delete_project_item task: {}", e))?
}
