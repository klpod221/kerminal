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
        // Initialize updater service
        let updater_service = crate::services::updater::UpdaterService::new(app_handle.clone());
        updater_service.start_update_check_loop();

        match AppState::new().await {
            Ok(app_state) => {
                let auth_session_manager = app_state.auth_session_manager.clone();
                let sftp_transfer_manager = app_state.sftp_transfer_manager.clone();

                app_handle.manage(app_state);

                let auth_manager_clone = auth_session_manager.clone();
                let app_handle_clone = app_handle.clone();

                tokio::spawn(async move {
                    let mut manager = auth_manager_clone.lock().await;
                    manager.set_app_handle(app_handle_clone);
                    let _ = manager.initialize().await;
                });

                sftp_transfer_manager.start_queue_processor(app_handle.clone());
            }
            Err(e) => {
                eprintln!("Failed to initialize AppState: {}", e);
            }
        }
    });

    Ok(())
}
