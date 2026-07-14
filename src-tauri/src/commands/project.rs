use std::path::PathBuf;
use tauri::State;
use crate::system::project::{FileSystemItem, ProjectConfig, ProjectIndex, ProjectManager};

/// Command to open a native OS folder dialog and return the selected path.
#[tauri::command]
pub fn pick_project_folder() -> Result<Option<String>, String> {
    let folder = rfd::FileDialog::new()
        .set_title("Select Project Folder")
        .pick_folder();
    
    Ok(folder.map(|p| p.to_string_lossy().to_string()))
}

/// Command to retrieve the index of all registered projects.
#[tauri::command]
pub fn list_projects(
    project_manager: State<'_, ProjectManager>,
) -> Result<ProjectIndex, String> {
    project_manager.load_index()
}

/// Command to add/register a new project.
#[tauri::command]
pub fn add_project(
    name: String,
    path: String,
    project_manager: State<'_, ProjectManager>,
) -> Result<ProjectConfig, String> {
    project_manager.add_project(&name, PathBuf::from(path))
}

/// Command to remove/unregister a project by its unique ID.
#[tauri::command]
pub fn remove_project(
    id: String,
    project_manager: State<'_, ProjectManager>,
) -> Result<(), String> {
    project_manager.remove_project(&id)
}

/// Command to touch a project (updating its last_opened timestamp).
#[tauri::command]
pub fn touch_project(
    id: String,
    project_manager: State<'_, ProjectManager>,
) -> Result<ProjectConfig, String> {
    project_manager.touch_project(&id)
}

/// Command to rename a project.
#[tauri::command]
pub fn rename_project(
    id: String,
    new_name: String,
    project_manager: State<'_, ProjectManager>,
) -> Result<ProjectConfig, String> {
    project_manager.rename_project(&id, &new_name)
}

/// Command to scan a directory path inside a project and return files/folders.
#[tauri::command]
pub fn list_project_files(
    path: String,
    project_manager: State<'_, ProjectManager>,
) -> Result<Vec<FileSystemItem>, String> {
    project_manager.list_directory(&PathBuf::from(path))
}

/// Command to create a directory folder.
#[tauri::command]
pub fn create_project_directory(
    path: String,
    project_manager: State<'_, ProjectManager>,
) -> Result<(), String> {
    project_manager.create_dir(&PathBuf::from(path))
}

/// Command to create or write content to a project file.
#[tauri::command]
pub fn create_project_file(
    path: String,
    content: String,
    project_manager: State<'_, ProjectManager>,
) -> Result<(), String> {
    project_manager.write_file(&PathBuf::from(path), &content)
}

/// Command to delete a file or directory recursively.
#[tauri::command]
pub fn delete_project_item(
    path: String,
    project_manager: State<'_, ProjectManager>,
) -> Result<(), String> {
    project_manager.delete_path(&PathBuf::from(path))
}
