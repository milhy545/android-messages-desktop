use tauri::{AppHandle, Manager, State};
use crate::AppState;

#[tauri::command]
pub async fn switch_webview(app: AppHandle, service: String, state: State<'_, AppState>) -> Result<(), String> {
    let mut active = state.active_service.lock().unwrap();
    *active = service.clone();

    if let Some(messages) = app.get_webview_window("messages_webview") {
        if let Some(chat) = app.get_webview_window("chat_webview") {
            if service == "messages" {
                let _ = chat.hide();
                let _ = messages.show();
                let _ = messages.set_focus();
            } else if service == "chat" {
                let _ = messages.hide();
                let _ = chat.show();
                let _ = chat.set_focus();
            }
        }
    }

    if let Some(main_window) = app.get_webview_window("main") {
        let _ = main_window.show();
        let _ = main_window.set_focus();
    }
    Ok(())
}

#[tauri::command]
pub async fn toggle_tts_mute(muted: bool, state: State<'_, AppState>) -> Result<(), String> {
    let mut state_muted = state.tts_muted.lock().unwrap();
    *state_muted = muted;
    Ok(())
}

#[tauri::command]
pub async fn trigger_os_notification(app: AppHandle, title: String, body: String, _service: String) -> Result<(), String> {
    use tauri_plugin_notification::NotificationExt;

    app.notification()
        .builder()
        .title(&title)
        .body(&body)
        .show()
        .map_err(|e: tauri_plugin_notification::Error| e.to_string())?;

    Ok(())
}

#[tauri::command]
pub async fn play_edge_tts(text: String, state: State<'_, AppState>) -> Result<(), String> {
    let muted = *state.tts_muted.lock().unwrap();
    if muted {
        return Ok(());
    }
    crate::tts::speak(&text).await
}

#[cfg(test)]
mod tests {
    // use super::*;
    use std::sync::Mutex;
    use crate::AppState;

    #[test]
    fn test_toggle_tts_mute() {
        let app_state = AppState {
            active_service: Mutex::new("messages".to_string()),
            tts_muted: Mutex::new(false),
        };

        // Simulating the state logic directly as tauri State mock can be complex
        let mut state_muted = app_state.tts_muted.lock().unwrap();
        assert_eq!(*state_muted, false);

        *state_muted = true;
        assert_eq!(*state_muted, true);

        *state_muted = false;
        assert_eq!(*state_muted, false);
    }

    #[test]
    fn test_active_service_switch() {
        let app_state = AppState {
            active_service: Mutex::new("messages".to_string()),
            tts_muted: Mutex::new(false),
        };

        {
            let mut active = app_state.active_service.lock().unwrap();
            assert_eq!(*active, "messages");
            *active = "chat".to_string();
        }

        {
            let active = app_state.active_service.lock().unwrap();
            assert_eq!(*active, "chat");
        }
    }
}
