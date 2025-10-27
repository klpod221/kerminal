use crate::state::AppState;
use tauri::{App, Manager};

pub fn init(app: &mut App) -> std::result::Result<(), Box<dyn std::error::Error>> {
    #[cfg(desktop)]
    {
        let window = app.get_webview_window("main").unwrap();
        window.set_title("Kerminal").unwrap();
    }

    let app_handle = app.handle().clone();
    tauri::async_runtime::spawn(async move {
        match AppState::new().await {
            Ok(app_state) => {
                let auth_session_manager = app_state.auth_session_manager.clone();
                app_handle.manage(app_state);

                tokio::spawn(async move {
                    let mut manager = auth_session_manager.lock().await;
                    let _ = manager.initialize().await;
                });
            }
            Err(e) => {
                eprintln!("Failed to initialize AppState: {}", e);
            }
        }
    });

    Ok(())
}
