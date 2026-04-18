// MacroSpend Tracker - Tauri main process
// Tray-only app. Window is hidden by default and shown via tray menu.

#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

mod sampler;
mod ingest;
mod storage;
mod deeplink;

use std::sync::Arc;
use parking_lot::RwLock;
use tauri::{
    menu::{Menu, MenuItem, PredefinedMenuItem},
    tray::{TrayIconBuilder, TrayIconEvent},
    Manager, WindowEvent,
};
use tauri_plugin_autostart::MacosLauncher;

#[derive(Clone, Default)]
pub struct AppState {
    pub paused: Arc<RwLock<bool>>,
    pub connected: Arc<RwLock<bool>>,
    pub device_key: Arc<RwLock<Option<String>>>,
}

#[derive(serde::Serialize)]
struct Status {
    paused: bool,
    connected: bool,
}

#[tauri::command]
fn get_status(state: tauri::State<AppState>) -> Status {
    Status {
        paused: *state.paused.read(),
        connected: *state.connected.read(),
    }
}

#[tauri::command]
fn pause_sampling(state: tauri::State<AppState>) {
    *state.paused.write() = true;
    log::info!("sampling paused");
}

#[tauri::command]
fn resume_sampling(state: tauri::State<AppState>) {
    *state.paused.write() = false;
    log::info!("sampling resumed");
}

fn main() {
    env_logger::init();

    tauri::Builder::default()
        .plugin(tauri_plugin_single_instance::init(|app, _argv, _cwd| {
            // Bring existing instance forward when user double-clicks the app again
            if let Some(w) = app.get_webview_window("main") {
                let _ = w.show();
                let _ = w.set_focus();
            }
        }))
        .plugin(tauri_plugin_autostart::init(
            MacosLauncher::LaunchAgent,
            Some(vec![]),
        ))
        .plugin(tauri_plugin_deep_link::init())
        .plugin(tauri_plugin_updater::Builder::new().build())
        .manage(AppState::default())
        .invoke_handler(tauri::generate_handler![
            get_status,
            pause_sampling,
            resume_sampling
        ])
        .setup(|app| {
            // Restore device key from OS keychain (if previously registered)
            let state = app.state::<AppState>();
            if let Ok(key) = storage::load_device_key() {
                *state.device_key.write() = Some(key);
                log::info!("device key restored from keychain");
            }

            // Tray menu
            let open_i = MenuItem::with_id(app, "open", "Open dashboard", true, None::<&str>)?;
            let pause_i = MenuItem::with_id(app, "pause", "Pause sampling", true, None::<&str>)?;
            let resume_i = MenuItem::with_id(app, "resume", "Resume sampling", true, None::<&str>)?;
            let update_i = MenuItem::with_id(app, "update", "Check for updates", true, None::<&str>)?;
            let sep = PredefinedMenuItem::separator(app)?;
            let quit_i = MenuItem::with_id(app, "quit", "Quit", true, None::<&str>)?;
            let menu = Menu::with_items(app, &[&open_i, &pause_i, &resume_i, &sep, &update_i, &sep, &quit_i])?;

            let _tray = TrayIconBuilder::with_id("main")
                .icon(app.default_window_icon().unwrap().clone())
                .icon_as_template(true)
                .menu(&menu)
                .show_menu_on_left_click(false)
                .on_menu_event(|app, event| match event.id.as_ref() {
                    "open" => {
                        if let Some(w) = app.get_webview_window("main") {
                            let _ = w.show();
                            let _ = w.set_focus();
                        }
                    }
                    "pause" => {
                        let s = app.state::<AppState>();
                        *s.paused.write() = true;
                    }
                    "resume" => {
                        let s = app.state::<AppState>();
                        *s.paused.write() = false;
                    }
                    "update" => {
                        // Updater dialog is wired via plugin config (dialog: true).
                        // Triggering programmatically: see PLAN.md phase 4.
                        log::info!("manual update check requested");
                    }
                    "quit" => app.exit(0),
                    _ => {}
                })
                .on_tray_icon_event(|tray, event| {
                    if let TrayIconEvent::Click { .. } = event {
                        if let Some(w) = tray.app_handle().get_webview_window("main") {
                            let _ = w.show();
                            let _ = w.set_focus();
                        }
                    }
                })
                .build(app)?;

            // Deep-link handler: macrospend://register?key=...
            deeplink::register_handler(app.handle().clone());

            // Spawn sampler + ingest loop
            let app_handle = app.handle().clone();
            tauri::async_runtime::spawn(async move {
                sampler::run_loop(app_handle).await;
            });

            Ok(())
        })
        .on_window_event(|window, event| {
            // Hide window on close instead of quitting (tray app pattern)
            if let WindowEvent::CloseRequested { api, .. } = event {
                let _ = window.hide();
                api.prevent_close();
            }
        })
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
