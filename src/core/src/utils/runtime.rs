// Copyright (C) 2026 Bùi Thanh Xuân (klpod221)
// SPDX-License-Identifier: GPL-3.0-or-later

use std::sync::OnceLock;
use tokio::runtime::Runtime;

static FALLBACK_RUNTIME: OnceLock<Runtime> = OnceLock::new();

/// Safely spawn an asynchronous task.
///
/// If called from within an active Tokio runtime thread, this uses
/// `tokio::runtime::Handle::current().spawn(future)`.
/// If called from a non-Tokio thread (e.g. main OS thread during GUI/Tauri setup),
/// it automatically falls back to a shared multi-thread Tokio runtime instead of panicking.
pub fn spawn<F>(future: F) -> tokio::task::JoinHandle<F::Output>
where
    F: std::future::Future + Send + 'static,
    F::Output: Send + 'static,
{
    if let Ok(handle) = tokio::runtime::Handle::try_current() {
        handle.spawn(future)
    } else {
        let rt = FALLBACK_RUNTIME.get_or_init(|| {
            tokio::runtime::Builder::new_multi_thread()
                .enable_all()
                .build()
                .expect("Failed to create background Tokio runtime")
        });
        rt.spawn(future)
    }
}

