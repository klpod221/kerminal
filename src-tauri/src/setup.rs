use crate::state::AppState;
use tauri::{App, Manager};

pub fn init(app: &mut App) -> std::result::Result<(), Box<dyn std::error::Error>> {
    let window = app.get_webview_window("main").unwrap();
    window.set_title("Kerminal").unwrap();

    let app_state = app.state::<AppState>();
    let auth_session_manager = app_state.auth_session_manager.clone();

    tokio::spawn(async move {
        let mut manager = auth_session_manager.lock().await;
        let _ = manager.initialize().await;
    });

    Ok(())
}
