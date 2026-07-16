mod app;
mod settings;
mod project;
pub mod download;
pub mod inference;

pub fn all_handlers() -> impl Fn(tauri::ipc::Invoke) -> bool {
    tauri::generate_handler![
        app::greet,
        app::minimize_application,
        app::toggle_maximize_application,
        app::close_application,
        app::start_drag,
        app::hide_to_tray,
        app::exit_application,
        app::check_hardware_performance,
        app::get_gpu_devices,
        app::check_system_dependencies,
        app::install_system_dependency,
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
        project::delete_project_item,
        download::get_available_models,
        download::get_download_history,
        download::delete_downloaded_model,
        download::start_model_download,
        download::download_llama_engine,
        inference::run_model_inference,
        inference::abort_model_inference
    ]
}
