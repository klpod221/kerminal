// Copyright (C) 2026 Bùi Thanh Xuân (klpod221)
// SPDX-License-Identifier: GPL-3.0-or-later

use crate::error::AppError;
use russh_keys::{
    check_known_hosts_path, learn_known_hosts_path,
    key::PublicKey,
};
use serde::{Deserialize, Serialize};
use std::path::{Path, PathBuf};
use std::sync::{Arc, Mutex};
use std::time::Duration;
use tauri::AppHandle;
use tauri::Manager;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Deserialize, Serialize)]
#[serde(rename_all = "lowercase")]
pub enum SshHostKeyStatus {
    Known,
    Unknown,
    Changed,
}

#[derive(Debug, Clone, PartialEq, Eq, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct SshHostKeyInspection {
    pub algorithm: String,
    pub fingerprint: String,
    pub host: String,
    pub port: u16,
    pub status: SshHostKeyStatus,
}

pub fn get_known_hosts_path(app_handle: &AppHandle) -> PathBuf {
    let app_dir = app_handle
        .path()
        .app_config_dir()
        .unwrap_or_else(|_| PathBuf::from("."));
    std::fs::create_dir_all(&app_dir).ok();
    app_dir.join("known_hosts")
}

pub fn classify_key_status(
    host: &str,
    port: u16,
    key: &PublicKey,
    known_hosts_path: &Path,
) -> SshHostKeyStatus {
    if !known_hosts_path.exists() {
        return SshHostKeyStatus::Unknown;
    }
    match check_known_hosts_path(host, port, key, known_hosts_path) {
        Ok(true) => SshHostKeyStatus::Known,
        Ok(false) => {
            SshHostKeyStatus::Changed
        }
        Err(_) => SshHostKeyStatus::Unknown,
    }
}

pub async fn probe_server_key(host: &str, port: u16) -> Result<PublicKey, AppError> {
    let captured = Arc::new(Mutex::new(None));
    let handler = HostKeyProbeHandler {
        captured: Arc::clone(&captured),
    };
    let config = Arc::new(russh::client::Config {
        inactivity_timeout: Some(Duration::from_secs(8)),
        ..Default::default()
    });
    let connect = russh::client::connect(config, (host, port), handler);
    let _ = tokio::time::timeout(Duration::from_secs(8), connect).await;
    
    let captured_key = captured
        .lock()
        .map_err(|_| AppError::terminal_error("Lock poisoned"))?
        .clone();
        
    captured_key.ok_or_else(|| AppError::terminal_error("Failed to probe SSH host key"))
}

#[tauri::command]
pub async fn inspect_ssh_host_key(
    app: AppHandle,
    host: String,
    port: u16,
) -> Result<SshHostKeyInspection, String> {
    let key = probe_server_key(&host, port).await.map_err(|e| e.to_string())?;
    let known_hosts_path = get_known_hosts_path(&app);
    let status = classify_key_status(&host, port, &key, &known_hosts_path);
    
    Ok(SshHostKeyInspection {
        algorithm: key.name().to_string(),
        fingerprint: key.fingerprint(),
        host,
        port,
        status,
    })
}

#[tauri::command]
pub async fn trust_ssh_host_key(
    app: AppHandle,
    host: String,
    port: u16,
    expected_fingerprint: String,
) -> Result<SshHostKeyInspection, String> {
    let key = probe_server_key(&host, port).await.map_err(|e| e.to_string())?;
    
    if key.fingerprint() != expected_fingerprint {
        return Err("Host key changed during verification".to_string());
    }
    
    let known_hosts_path = get_known_hosts_path(&app);
    learn_known_hosts_path(&host, port, &key, &known_hosts_path)
        .map_err(|e| format!("Failed to write known_hosts: {}", e))?;
        
    Ok(SshHostKeyInspection {
        algorithm: key.name().to_string(),
        fingerprint: key.fingerprint(),
        host,
        port,
        status: SshHostKeyStatus::Known,
    })
}

struct HostKeyProbeHandler {
    captured: Arc<Mutex<Option<PublicKey>>>,
}

#[async_trait::async_trait]
impl russh::client::Handler for HostKeyProbeHandler {
    type Error = russh::Error;

    async fn check_server_key(&mut self, server_public_key: &PublicKey) -> Result<bool, Self::Error> {
        if let Ok(mut captured) = self.captured.lock() {
            *captured = Some(server_public_key.clone());
        }
        Ok(false) // Abort connection
    }
}
