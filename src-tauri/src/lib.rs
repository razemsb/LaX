mod commands;
mod config;
mod discover;
mod error;
mod hosts;
mod logs;
mod paths;
mod php;
mod portable;
mod process;
mod projects;
mod services;
mod state;
mod vhosts;

use tauri::{
    menu::{Menu, MenuItem},
    tray::TrayIconBuilder,
    Manager,
};

use crate::state::AppState;

pub fn run() {
    let _ = std::fs::create_dir_all(crash_log_dir());
    let hook_dir = crash_log_dir();
    std::panic::set_hook(Box::new(move |info| {
        let msg = format!("{info}\n");
        let path = hook_dir.join("lax-crash.log");
        let _ = std::fs::write(&path, msg.as_bytes());
        eprintln!("{info}");
    }));

    tracing_subscriber::fmt()
        .with_env_filter(
            tracing_subscriber::EnvFilter::try_from_default_env()
                .unwrap_or_else(|_| tracing_subscriber::EnvFilter::new("info")),
        )
        .init();

    let state = AppState::new().unwrap_or_else(|e| {
        let _ = std::fs::write(
            crash_log_dir().join("lax-crash.log"),
            format!("failed to load LaX state: {e}\n"),
        );
        AppState::fallback()
    });

    tauri::Builder::default()
        .plugin(tauri_plugin_opener::init())
        .plugin(tauri_plugin_shell::init())
        .manage(state)
        .setup(|app| {
            let show = MenuItem::with_id(app, "show", "Open LaX", true, None::<&str>)?;
            let quit = MenuItem::with_id(app, "quit", "Quit", true, None::<&str>)?;
            let menu = Menu::with_items(app, &[&show, &quit])?;
            let mut tray = TrayIconBuilder::with_id("tray")
                .menu(&menu)
                .show_menu_on_left_click(true)
                .on_menu_event(|app, event| match event.id.as_ref() {
                    "show" => {
                        if let Some(w) = app.get_webview_window("main") {
                            let _ = w.show();
                            let _ = w.set_focus();
                        }
                    }
                    "quit" => {
                        if let Some(state) = app.try_state::<AppState>() {
                            if let Ok(mut g) = state.inner.lock() {
                                g.stop_all();
                            }
                        }
                        app.exit(0);
                    }
                    _ => {}
                });
            if let Some(icon) = app.default_window_icon().cloned() {
                tray = tray.icon(icon);
            }
            let _ = tray.build(app);

            if let Some(win) = app.get_webview_window("main") {
                let win_h = win.clone();
                win.on_window_event(move |event| {
                    if let tauri::WindowEvent::CloseRequested { api, .. } = event {
                        api.prevent_close();
                        let _ = win_h.hide();
                    }
                });
            }
            Ok(())
        })
        .invoke_handler(tauri::generate_handler![
            commands::snapshot,
            commands::status,
            commands::start_all,
            commands::stop_all,
            commands::start_service,
            commands::stop_service,
            commands::switch_php,
            commands::create_project,
            commands::get_config,
            commands::save_config,
            commands::read_logs,
            commands::list_php_extensions,
            commands::set_php_extension,
            commands::open_ini,
            commands::open_url,
            commands::open_path,
            commands::open_terminal,
            commands::open_vscode,
        ])
        .run(tauri::generate_context!())
        .expect("error while running LaX");
}

fn crash_log_dir() -> std::path::PathBuf {
    crate::paths::detect_root().join("logs")
}
