// Copyright (C) 2026 Bùi Thanh Xuân (klpod221)
// SPDX-License-Identifier: GPL-3.0-or-later

use serde_json::Value;
use std::sync::Arc;

/// Core interface for event emission from services to UI layer (Tauri or Native GUI)
pub trait CoreEventEmitter: Send + Sync + 'static {
    /// Emit a structured JSON event to the UI
    fn emit_event(&self, event_name: &str, payload: Value);

    /// Send raw binary output from terminal PTY or SSH stream
    fn send_terminal_output(&self, terminal_id: &str, data: &[u8]);
}

/// No-op implementation for headless testing or default state
#[derive(Default, Clone)]
pub struct NoopEventEmitter;

impl CoreEventEmitter for NoopEventEmitter {
    fn emit_event(&self, _event_name: &str, _payload: Value) {}
    fn send_terminal_output(&self, _terminal_id: &str, _data: &[u8]) {}
}

impl NoopEventEmitter {
    pub fn new() -> Arc<Self> {
        Arc::new(Self)
    }
}

