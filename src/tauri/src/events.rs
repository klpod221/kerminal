// Copyright (C) 2026 Bùi Thanh Xuân (klpod221)
// SPDX-License-Identifier: GPL-3.0-or-later

use kerminal_core::events::CoreEventEmitter;
use serde_json::Value;
use std::sync::Arc;
use tauri::{AppHandle, Emitter};
use tokio::sync::RwLock;

/// Tauri adapter for core event emission with binary IPC channel support
pub struct TauriEventEmitter {
    app_handle: AppHandle,
    output_channel: Arc<RwLock<Option<tauri::ipc::Channel<tauri::ipc::InvokeResponseBody>>>>,
}

impl TauriEventEmitter {
    pub fn new(app_handle: AppHandle) -> Self {
        Self {
            app_handle,
            output_channel: Arc::new(RwLock::new(None)),
        }
    }

    /// Set binary channel for high-throughput zero-copy terminal streaming
    pub async fn set_output_channel(
        &self,
        channel: tauri::ipc::Channel<tauri::ipc::InvokeResponseBody>,
    ) {
        let mut ch = self.output_channel.write().await;
        *ch = Some(channel);
    }
}

impl CoreEventEmitter for TauriEventEmitter {
    fn emit_event(&self, event_name: &str, payload: Value) {
        let _ = self.app_handle.emit(event_name, payload);
    }

    fn send_terminal_output(&self, terminal_id: &str, data: &[u8]) {
        // Try sending via high-performance binary channel first
        if let Ok(ch_guard) = self.output_channel.try_read() {
            if let Some(ref channel) = *ch_guard {
                let id_bytes = terminal_id.as_bytes();
                let id_len = id_bytes.len().min(255) as u8;

                let mut frame = Vec::with_capacity(1 + (id_len as usize) + data.len());
                frame.push(id_len);
                frame.extend_from_slice(&id_bytes[..id_len as usize]);
                frame.extend_from_slice(data);

                if channel.send(tauri::ipc::InvokeResponseBody::Raw(frame)).is_ok() {
                    return;
                }
            }
        }

        // Fallback: emit via JSON event
        let text = String::from_utf8_lossy(data).into_owned();
        let payload = serde_json::json!({
            "terminalId": terminal_id,
            "data": text
        });
        let _ = self.app_handle.emit("terminal-output", payload);
    }
}

