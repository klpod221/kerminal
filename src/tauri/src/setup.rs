// Copyright (C) 2026 Bùi Thanh Xuân (klpod221)
// SPDX-License-Identifier: GPL-3.0-or-later

use crate::state::AppState;
use log::error;
use tauri::{App, Manager};

pub fn init(app: &mut App) -> std::result::Result<(), Box<dyn std::error::Error>> {
    #[cfg(desktop)]
    {
        if let Some(window) = app.get_webview_window("main") {
            let _ = window.set_title("Kerminal");
            if let Some(icon) = app.default_window_icon() {
                let _ = window.set_icon(icon.clone());
            } else if let Ok(icon) = tauri::image::Image::from_bytes(include_bytes!("../icons/128x128.png")) {
                let _ = window.set_icon(icon);
            }
        }
    }

    let app_handle = app.handle().clone();

    // Initialize updater service
    let updater_service = crate::updater::UpdaterService::new(app_handle.clone());
    updater_service.start_update_check_loop();

    let app_state = tauri::async_runtime::block_on(AppState::new(app_handle.clone()))
        .map_err(|e| {
            error!("Failed to initialize AppState: {}", e);
            e
        })?;

    let auth_session_manager = app_state.auth_session_manager.clone();
    let sftp_transfer_manager = app_state.sftp_transfer_manager.clone();
    let emitter = app_state.emitter.clone();

    app.manage(app_state);

    let auth_manager_clone = auth_session_manager.clone();

    tauri::async_runtime::spawn(async move {
        let mut manager = auth_manager_clone.lock().await;
        let _ = manager.initialize().await;
    });

    sftp_transfer_manager.start_queue_processor(Some(emitter));

    Ok(())
}
