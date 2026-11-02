// Copyright (C) 2026 Bùi Thanh Xuân (klpod221)
// SPDX-License-Identifier: GPL-3.0-or-later

#[allow(unused_imports)]
pub use kerminal_core::core::ssh_host_key::{
    get_default_known_hosts_path, inspect_host_key, trust_host_key, SshHostKeyInspection,
    SshHostKeyStatus,
};

pub fn get_known_hosts_path(_app: &tauri::AppHandle) -> std::path::PathBuf {
    get_default_known_hosts_path()
}

#[tauri::command]
pub async fn inspect_ssh_host_key(
    host: String,
    port: u16,
) -> Result<SshHostKeyInspection, String> {
    inspect_host_key(&host, port, None)
        .await
        .map_err(|e| e.to_string())
}

#[tauri::command]
pub async fn trust_ssh_host_key(
    host: String,
    port: u16,
    expected_fingerprint: String,
) -> Result<SshHostKeyInspection, String> {
    trust_host_key(&host, port, &expected_fingerprint, None)
        .await
        .map_err(|e| e.to_string())
}
