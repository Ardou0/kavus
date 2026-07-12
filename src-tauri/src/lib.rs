use tauri::menu::{Menu, MenuItem};
use tauri::tray::{MouseButton, MouseButtonState, TrayIconBuilder, TrayIconEvent};
use tauri::webview::PageLoadEvent;
use tauri::Manager;

mod i18n;
mod mcp;
mod commands;
mod system;

pub fn run() {
    tauri::Builder::default()
        .plugin(tauri_plugin_opener::init())
        .plugin(tauri_plugin_store::Builder::default().build())
        .invoke_handler(commands::all_handlers())
        .on_page_load(|webview, payload| {
            if webview.label() == "main" && matches!(payload.event(), PageLoadEvent::Finished) {
                let _ = webview.window().show();
                let _ = webview.window().unminimize();
                let _ = webview.window().set_focus();
                println!("{}", i18n::strings().log_vue_ready);
            }
        })
        .setup(|app| {
            // Initialize Settings
            let settings_manager = system::settings::SettingsManager::new(app.app_handle())
                .map_err(|e| Box::<dyn std::error::Error>::from(e))?;
            let _settings = settings_manager.load()
                .map_err(|e| Box::<dyn std::error::Error>::from(e))?;
            app.manage(settings_manager);

            // Initialize Project Manager
            let project_manager = system::project::ProjectManager::new(app.app_handle())
                .map_err(|e| Box::<dyn std::error::Error>::from(e))?;
            let _projects = project_manager.load_index()
                .map_err(|e| Box::<dyn std::error::Error>::from(e))?;
            app.manage(project_manager);

            // System Tray
            const SHOW_APP: &str = "Show";
            const EXIT_APP: &str = "Exit";
            let show_item = MenuItem::with_id(
                app,
                SHOW_APP,
                i18n::strings().tray_show_label.as_str(),
                true,
                None::<&str>,
            )?;
            let quit_item = MenuItem::with_id(
                app,
                EXIT_APP,
                i18n::strings().tray_exit_label.as_str(),
                true,
                None::<&str>,
            )?;
            let tray_menu = Menu::with_items(app, &[&show_item, &quit_item])?;
            let _tray = TrayIconBuilder::new()
                .icon(app.default_window_icon().unwrap().clone())
                .menu(&tray_menu)
                .show_menu_on_left_click(false)
                .on_tray_icon_event(|tray, event| {
                    if let TrayIconEvent::Click { button, button_state, .. } = event {
                        if button == MouseButton::Left && button_state == MouseButtonState::Up {
                            let app_handle = tray.app_handle();
                            if let Some(window) = app_handle.get_webview_window("main") {
                                let _ = window.show();
                                let _ = window.unminimize();
                                let _ = window.set_focus();
                            }
                        }
                    }
                })
                .on_menu_event(|app_handle, event| {
                    if event.id.as_ref() == SHOW_APP {
                        if let Some(window) = app_handle.get_webview_window("main") {
                            let _ = window.show();
                            let _ = window.unminimize();
                            let _ = window.set_focus();
                        }
                    } else if event.id.as_ref() == EXIT_APP {
                        app_handle.exit(0);
                    }
                })
                .build(app)?;

            // MCP server
            tauri::async_runtime::spawn(async move {
                println!("{}", i18n::strings().log_mcp_start);

                if let Err(e) = mcp::server::start_server(3000).await {
                    eprintln!("{}: {:?}", i18n::strings().log_mcp_error, e);
                }
            });

            Ok(())
        })
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
