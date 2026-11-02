// Copyright (C) 2026 Bùi Thanh Xuân (klpod221)
// SPDX-License-Identifier: GPL-3.0-or-later

use crate::events::TauriEventEmitter;
use kerminal_core::state::AppState as CoreAppState;
use std::ops::Deref;
use std::sync::Arc;
use tauri::AppHandle;

pub struct AppState {
    pub core: CoreAppState,
    pub emitter: Arc<TauriEventEmitter>,
}

impl Deref for AppState {
    type Target = CoreAppState;

    fn deref(&self) -> &Self::Target {
        &self.core
    }
}

impl AppState {
    pub async fn new(app_handle: AppHandle) -> Result<Self, String> {
        let core = CoreAppState::new().await?;
        let emitter = Arc::new(TauriEventEmitter::new(app_handle));

        // Connect emitter to services
        core.auth_session_manager.lock().await.set_emitter(emitter.clone());
        core.sftp_transfer_manager.set_emitter(emitter.clone()).await;
        core.sftp_sync_service.set_emitter(emitter.clone()).await;

        Ok(Self { core, emitter })
    }
}
