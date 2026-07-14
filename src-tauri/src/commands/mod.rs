mod app;
mod settings;
mod project;

pub fn all_handlers() -> impl Fn(tauri::ipc::Invoke) -> bool {
    tauri::generate_handler![
        app::greet,
        app::minimize_application,
        app::toggle_maximize_application,
        app::close_application,
        app::start_drag,
        app::hide_to_tray,
        app::exit_application,
        settings::get_settings,
        settings::save_settings,
        project::list_projects,
        project::add_project,
        project::remove_project,
        project::touch_project,
        project::rename_project,
        project::pick_project_folder,
        project::list_project_files,
        project::create_project_directory,
        project::create_project_file,
        project::delete_project_item
    ]
}
