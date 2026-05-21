// Prevents additional console window on Windows in release, DO NOT REMOVE!!
#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

mod commands;
mod tts;

use std::sync::Mutex;
use tauri::menu::{Menu, MenuItem};
use tauri::tray::TrayIconBuilder;
use tauri::{Emitter, Manager, WebviewUrl, WebviewWindowBuilder};
use tauri_plugin_global_shortcut::{GlobalShortcutExt, Shortcut, ShortcutState};

pub struct AppState {
    pub active_service: Mutex<String>,
    pub tts_muted: Mutex<bool>,
}

const PRELOAD_SCRIPT: &str = include_str!("inject.js");

fn main() {
    tauri::Builder::default()
        .plugin(tauri_plugin_notification::init())
        .plugin(
            tauri_plugin_global_shortcut::Builder::new()
                .with_handler(move |app, _shortcut, event| {
                    if event.state == ShortcutState::Pressed {
                        let state = app.state::<AppState>();
                        let active = state
                            .active_service
                            .lock()
                            .map(|s| s.clone())
                            .unwrap_or_else(|_| "messages".to_string());
                        if let Some(main_window) = app.get_webview_window("main") {
                            if let Some(webview) =
                                main_window.get_webview_window(&format!("{}_webview", active))
                            {
                                let _ = webview.emit("start_stt", ());
                            }
                        }
                    }
                })
                .build(),
        )
        .on_window_event(|window, event| {
            if let tauri::WindowEvent::CloseRequested { api, .. } = event {
                window.hide().unwrap();
                api.prevent_close();
            }
        })
        .manage(AppState {
            active_service: Mutex::new("messages".to_string()),
            tts_muted: Mutex::new(false),
        })
        .invoke_handler(tauri::generate_handler![
            commands::switch_webview,
            commands::toggle_tts_mute,
            commands::trigger_os_notification,
            commands::play_edge_tts,
        ])
        .setup(|app| {
            let main_window = app.get_webview_window("main").unwrap();
            let sidebar_width = 64;

            let pos = main_window
                .outer_position()
                .unwrap_or(tauri::PhysicalPosition::new(0, 0));
            let size = main_window
                .inner_size()
                .unwrap_or(tauri::PhysicalSize::new(1100, 800));

            let messages_window = WebviewWindowBuilder::new(
                app,
                "messages_webview",
                WebviewUrl::External("https://messages.google.com/web".parse().unwrap()),
            )
            .title("Messages")
            .inner_size(
                size.width.saturating_sub(sidebar_width) as f64,
                size.height as f64,
            )
            .position(pos.x as f64 + sidebar_width as f64, pos.y as f64)
            .decorations(false)
            .initialization_script(PRELOAD_SCRIPT)
            .build()
            .unwrap();

            let chat_window = WebviewWindowBuilder::new(
                app,
                "chat_webview",
                WebviewUrl::External("https://chat.google.com/".parse().unwrap()),
            )
            .title("Chat")
            .inner_size(
                size.width.saturating_sub(sidebar_width) as f64,
                size.height as f64,
            )
            .position(pos.x as f64 + sidebar_width as f64, pos.y as f64)
            .decorations(false)
            .initialization_script(PRELOAD_SCRIPT)
            .build()
            .unwrap();

            let _ = chat_window.hide();

            let messages_clone = messages_window.clone();
            let chat_clone = chat_window.clone();
            main_window.on_window_event(move |event| match event {
                tauri::WindowEvent::Resized(size) => {
                    let _ = messages_clone.set_size(tauri::PhysicalSize::new(
                        size.width.saturating_sub(sidebar_width),
                        size.height,
                    ));
                    let _ = chat_clone.set_size(tauri::PhysicalSize::new(
                        size.width.saturating_sub(sidebar_width),
                        size.height,
                    ));
                }
                tauri::WindowEvent::Moved(pos) => {
                    let _ = messages_clone.set_position(tauri::PhysicalPosition::new(
                        pos.x + sidebar_width as i32,
                        pos.y,
                    ));
                    let _ = chat_clone.set_position(tauri::PhysicalPosition::new(
                        pos.x + sidebar_width as i32,
                        pos.y,
                    ));
                }
                _ => {}
            });

            let ctrl_space = "ctrl+space".parse::<Shortcut>().unwrap();
            app.global_shortcut().register(ctrl_space).unwrap();

            // Tray Icon setup
            let show_i =
                MenuItem::with_id(app, "show", "Zobrazit aplikaci", true, None::<&str>).unwrap();
            let quit_i = MenuItem::with_id(app, "quit", "Ukončit", true, None::<&str>).unwrap();
            let menu = Menu::with_items(app, &[&show_i, &quit_i]).unwrap();

            let main_window_clone = main_window.clone();
            let _tray = TrayIconBuilder::new()
                .menu(&menu)
                .show_menu_on_left_click(true)
                .icon(app.default_window_icon().unwrap().clone())
                .on_menu_event(move |_app, event| match event.id.as_ref() {
                    "quit" => {
                        std::process::exit(0);
                    }
                    "show" => {
                        main_window_clone.show().unwrap();
                    }
                    _ => {}
                })
                .build(app)?;

            Ok(())
        })
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
