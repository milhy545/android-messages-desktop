// Prevents additional console window on Windows in release, DO NOT REMOVE!!
#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

mod commands;
mod tts;

use std::sync::Mutex;
use tauri::{Manager, WebviewUrl, WebviewWindowBuilder, Emitter};
use tauri_plugin_global_shortcut::{GlobalShortcutExt, Shortcut, ShortcutState};
use tauri::menu::{Menu, MenuItem};
use tauri::tray::TrayIconBuilder;

pub struct AppState {
    pub active_service: Mutex<String>,
    pub tts_muted: Mutex<bool>,
}

const PRELOAD_SCRIPT: &str = r#"
document.addEventListener('DOMContentLoaded', () => {
  console.log('Unifikátor Preload Script Injected');

  const OriginalNotification = window.Notification;
  if (OriginalNotification) {
    class CustomNotification {
      constructor(title, options) {
        this.title = title;
        this.options = options;
        window.__TAURI_INTERNALS__.invoke('trigger_os_notification', {
            title: title,
            body: options ? options.body : '',
            service: window.location.href.includes('chat.google.com') ? 'chat' : 'messages'
        });
      }
      static get permission() { return 'granted'; }
      static requestPermission(callback) { if (callback) callback('granted'); return Promise.resolve('granted'); }
    }
    window.Notification = CustomNotification;
  }

  const observeChatMutations = () => {
    const isMessages = window.location.href.includes('messages.google.com');
    const isChat = window.location.href.includes('chat.google.com');
    const observer = new MutationObserver((mutations) => {
      for (const mutation of mutations) {
        if (mutation.type === 'childList' && mutation.addedNodes.length > 0) {
          mutation.addedNodes.forEach(node => {
            if (node.nodeType === Node.ELEMENT_NODE) {
              let textToRead = null;
              if (isMessages && node.tagName && node.tagName.toLowerCase() === 'mws-message-part') {
                  const content = node.textContent;
                  if (content && content.trim().length > 0) textToRead = content;
              } else if (isChat && (node.getAttribute('role') === 'listitem' || node.classList.contains('c-P'))) {
                  const textContentDiv = node.querySelector('[data-text]');
                  textToRead = textContentDiv ? textContentDiv.textContent : node.textContent;
              }
              if (textToRead && textToRead.trim().length > 0) {
                window.__TAURI_INTERNALS__.invoke('play_edge_tts', { text: textToRead.trim() });
              }
            }
          });
        }
      }
    });
    observer.observe(document.body, { childList: true, subtree: true });
  };
  setTimeout(observeChatMutations, 3000);

  window.__TAURI_INTERNALS__.listen('start_stt', (event) => {
      const SpeechRecognition = window.SpeechRecognition || window.webkitSpeechRecognition;
      if (!SpeechRecognition) return;
      const recognition = new SpeechRecognition();
      recognition.lang = 'cs-CZ';
      recognition.interimResults = false;
      recognition.maxAlternatives = 1;
      recognition.onresult = (e) => {
        const transcript = e.results[0][0].transcript;
        let inputField = document.querySelector('textarea[placeholder="Text message"]') || document.querySelector('[contenteditable="true"]');
        if (inputField) {
          inputField.focus();
          if (inputField.tagName.toLowerCase() === 'textarea') {
            inputField.value = transcript;
            inputField.dispatchEvent(new Event('input', { bubbles: true }));
          } else {
            inputField.textContent = transcript;
            inputField.dispatchEvent(new Event('input', { bubbles: true }));
          }
          setTimeout(() => {
            const enterEvent = new KeyboardEvent('keydown', { bubbles: true, cancelable: true, keyCode: 13, key: 'Enter' });
            inputField.dispatchEvent(enterEvent);
          }, 100);
        }
      };
      recognition.start();
  });
});
"#;

fn main() {
    tauri::Builder::default()
        .plugin(tauri_plugin_notification::init())
        .plugin(tauri_plugin_global_shortcut::Builder::new().with_handler(move |app, _shortcut, event| {
            if event.state == ShortcutState::Pressed {
                let state = app.state::<AppState>();
                let active = state.active_service.lock().map(|s| s.clone()).unwrap_or_else(|_| "messages".to_string());
                if let Some(main_window) = app.get_webview_window("main") {
                    if let Some(webview) = main_window.get_webview_window(&format!("{}_webview", active)) {
                        let _ = webview.emit("start_stt", ());
                    }
                }
            }
        }).build())
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

            let pos = main_window.outer_position().unwrap_or(tauri::PhysicalPosition::new(0, 0));
            let size = main_window.inner_size().unwrap_or(tauri::PhysicalSize::new(1100, 800));

            let messages_window = WebviewWindowBuilder::new(
                app,
                "messages_webview",
                WebviewUrl::External("https://messages.google.com/web".parse().unwrap())
            )
            .title("Messages")
            .inner_size(size.width.saturating_sub(sidebar_width) as f64, size.height as f64)
            .position(pos.x as f64 + sidebar_width as f64, pos.y as f64)
            .decorations(false)
            .initialization_script(PRELOAD_SCRIPT)
            .build()
            .unwrap();

            let chat_window = WebviewWindowBuilder::new(
                app,
                "chat_webview",
                WebviewUrl::External("https://chat.google.com/".parse().unwrap())
            )
            .title("Chat")
            .inner_size(size.width.saturating_sub(sidebar_width) as f64, size.height as f64)
            .position(pos.x as f64 + sidebar_width as f64, pos.y as f64)
            .decorations(false)
            .initialization_script(PRELOAD_SCRIPT)
            .build()
            .unwrap();

            let _ = chat_window.hide();

            let messages_clone = messages_window.clone();
            let chat_clone = chat_window.clone();
            main_window.on_window_event(move |event| {
                match event {
                    tauri::WindowEvent::Resized(size) => {
                        let _ = messages_clone.set_size(tauri::PhysicalSize::new(size.width.saturating_sub(sidebar_width), size.height));
                        let _ = chat_clone.set_size(tauri::PhysicalSize::new(size.width.saturating_sub(sidebar_width), size.height));
                    },
                    tauri::WindowEvent::Moved(pos) => {
                        let _ = messages_clone.set_position(tauri::PhysicalPosition::new(pos.x + sidebar_width as i32, pos.y));
                        let _ = chat_clone.set_position(tauri::PhysicalPosition::new(pos.x + sidebar_width as i32, pos.y));
                    },
                    _ => {}
                }
            });

            let ctrl_space = "ctrl+space".parse::<Shortcut>().unwrap();
            app.global_shortcut().register(ctrl_space).unwrap();

            // Tray Icon setup
            let show_i = MenuItem::with_id(app, "show", "Zobrazit aplikaci", true, None::<&str>).unwrap();
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
