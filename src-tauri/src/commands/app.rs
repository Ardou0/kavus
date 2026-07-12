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
