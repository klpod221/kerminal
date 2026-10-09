// Copyright (C) 2026 Bùi Thanh Xuân (klpod221)
// SPDX-License-Identifier: GPL-3.0-or-later

use std::collections::HashMap;
use std::sync::atomic::{AtomicI64, Ordering};
use std::sync::Arc;

use async_trait::async_trait;
use chrono::{DateTime, Utc};
use russh::client;
use russh_sftp::client::RawSftpSession;
use tokio::sync::RwLock;

use std::path::{Path, PathBuf};

use crate::models::ssh::profile::{AuthData, SSHProfile};
use crate::services::sftp::channel_stream::ChannelStream;
use crate::services::sftp::errors::SftpError;
use crate::services::ssh::key::SSHKeyService;

/// Client handler with strict host key verification
pub struct SFTPClientHandler {
    host: String,
    port: u16,
    known_hosts_path: PathBuf,
}

impl SFTPClientHandler {
    pub fn new(host: String, port: u16, known_hosts_path: PathBuf) -> Self {
        Self {
            host,
            port,
            known_hosts_path,
        }
    }
}

#[async_trait]
impl client::Handler for SFTPClientHandler {
    type Error = russh::Error;

    async fn check_server_key(
        &mut self,
        server_public_key: &russh_keys::key::PublicKey,
    ) -> Result<bool, Self::Error> {
        if self.known_hosts_path.as_os_str().is_empty() {
            return Ok(false);
        }

        match russh_keys::check_known_hosts_path(
            &self.host,
            self.port,
            server_public_key,
            &self.known_hosts_path,
        ) {
            Ok(true) => Ok(true),
            _ => Ok(false),
        }
    }
}

/// A cloneable, lock-free SFTP connection handle
#[allow(dead_code)]
#[derive(Clone)]
pub struct SftpConnection {
    pub session_id: String,
    pub profile_id: String,
    pub profile_name: String,
    pub sftp: Arc<RawSftpSession>,
    pub home_dir: String,
    pub client: Arc<client::Handle<SFTPClientHandler>>,
    last_used: Arc<AtomicI64>,
    pub connected_at: DateTime<Utc>,
}

impl std::fmt::Debug for SftpConnection {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("SftpConnection")
            .field("session_id", &self.session_id)
            .field("profile_id", &self.profile_id)
            .field("profile_name", &self.profile_name)
            .field("home_dir", &self.home_dir)
            .field("connected_at", &self.connected_at)
            .finish()
    }
}

impl SftpConnection {
    /// Create new connection handle
    pub fn new(
        session_id: String,
        profile_id: String,
        profile_name: String,
        sftp: Arc<RawSftpSession>,
        home_dir: String,
        client: Arc<client::Handle<SFTPClientHandler>>,
    ) -> Self {
        let now = Utc::now();
        Self {
            session_id,
            profile_id,
            profile_name,
            sftp,
            home_dir,
            client,
            last_used: Arc::new(AtomicI64::new(now.timestamp())),
            connected_at: now,
        }
    }

    /// Update last used timestamp
    pub fn touch(&self) {
        self.last_used.store(Utc::now().timestamp(), Ordering::Relaxed);
    }

    /// Get last used timestamp as DateTime<Utc>
    pub fn last_used_at(&self) -> DateTime<Utc> {
        let ts = self.last_used.load(Ordering::Relaxed);
        DateTime::<Utc>::from_timestamp(ts, 0).unwrap_or_else(Utc::now)
    }

    /// Check if the underlying client channel is closed
    pub fn is_alive(&self) -> bool {
        !self.client.is_closed()
    }

    /// Close the SFTP session
    pub async fn close(&self) -> Result<(), SftpError> {
        let _ = self.sftp.close_session();
        Ok(())
    }
}

/// Connection pool / manager for SFTP sessions
pub struct SftpConnectionManager {
    connections: Arc<RwLock<HashMap<String, SftpConnection>>>,
}

impl SftpConnectionManager {
    pub fn new() -> Self {
        Self {
            connections: Arc::new(RwLock::new(HashMap::new())),
        }
    }

    /// Retrieve an existing active connection
    pub async fn get(&self, session_id: &str) -> Result<SftpConnection, SftpError> {
        let conns = self.connections.read().await;
        let conn = conns.get(session_id).cloned().ok_or_else(|| SftpError::SessionNotFound {
            session_id: session_id.to_string(),
        })?;

        if !conn.is_alive() {
            drop(conns);
            self.remove(session_id).await;
            return Err(SftpError::ConnectionLost {
                message: format!("Session {} connection has been lost", session_id),
            });
        }

        conn.touch();
        Ok(conn)
    }

    /// Check if connection ID already exists
    pub async fn contains(&self, session_id: &str) -> bool {
        let conns = self.connections.read().await;
        conns.contains_key(session_id)
    }

    /// Register a new connection
    pub async fn insert(&self, conn: SftpConnection) {
        let mut conns = self.connections.write().await;
        conns.insert(conn.session_id.clone(), conn);
    }

    /// Remove a connection
    pub async fn remove(&self, session_id: &str) -> Option<SftpConnection> {
        let mut conns = self.connections.write().await;
        conns.remove(session_id)
    }

    /// List all active session IDs
    #[allow(dead_code)]
    pub async fn list_sessions(&self) -> Vec<SftpConnection> {
        let conns = self.connections.read().await;
        conns.values().cloned().collect()
    }
}

/// Helper function to establish SSH connection and SFTP subsystem
pub async fn establish_sftp_connection(
    profile: &SSHProfile,
    known_hosts_path: PathBuf,
    ssh_key_service: &Arc<tokio::sync::Mutex<SSHKeyService>>,
) -> Result<(Arc<RawSftpSession>, Arc<client::Handle<SFTPClientHandler>>, String), SftpError> {
    let mut config = client::Config::default();
    if profile.keep_alive {
        config.keepalive_interval = Some(std::time::Duration::from_secs(15));
        config.keepalive_max = 10;
    }
    if let Some(t) = profile.timeout {
        config.inactivity_timeout = Some(std::time::Duration::from_secs(t as u64));
    }
    let config = Arc::new(config);

    let handler = SFTPClientHandler::new(
        profile.host.clone(),
        profile.port,
        known_hosts_path,
    );

    let mut client = client::connect(
        config,
        (profile.host.as_str(), profile.port),
        handler,
    )
    .await
    .map_err(|e| SftpError::SessionFailed {
        message: format!("Failed to connect to {}:{}: {}", profile.host, profile.port, e),
    })?;

    // Authenticate using profile auth_data
    let authenticated = match &profile.auth_data {
        AuthData::Password { password } => client
            .authenticate_password(&profile.username, password)
            .await
            .map_err(|e| SftpError::SessionFailed {
                message: format!("Password authentication failed: {}", e),
            })?,
        AuthData::KeyReference { key_id } => {
            let key_service = ssh_key_service.lock().await;
            let resolved_key = key_service
                .resolve_key_for_auth(key_id)
                .await
                .map_err(|e| SftpError::SessionFailed {
                    message: format!("Failed to resolve SSH key: {}", e),
                })?;

            let key = if Path::new(&resolved_key.private_key).exists() {
                russh_keys::load_secret_key(
                    &resolved_key.private_key,
                    resolved_key.passphrase.as_deref(),
                )
                .map_err(|e| SftpError::SessionFailed {
                    message: format!("Failed to load SSH key: {}", e),
                })?
            } else {
                russh_keys::decode_secret_key(
                    &resolved_key.private_key,
                    resolved_key.passphrase.as_deref(),
                )
                .map_err(|e| SftpError::SessionFailed {
                    message: format!("Failed to parse SSH key: {}", e),
                })?
            };

            client
                .authenticate_publickey(&profile.username, Arc::new(key))
                .await
                .map_err(|e| SftpError::SessionFailed {
                    message: format!("SSH key authentication failed: {}", e),
                })?
        }
        AuthData::Certificate {
            certificate: _,
            private_key,
            ..
        } => {
            let key = if Path::new(private_key).exists() {
                russh_keys::load_secret_key(private_key, None).map_err(|e| {
                    SftpError::SessionFailed {
                        message: format!("Failed to load certificate key: {}", e),
                    }
                })?
            } else {
                russh_keys::decode_secret_key(private_key, None).map_err(|e| {
                    SftpError::SessionFailed {
                        message: format!("Failed to parse certificate key: {}", e),
                    }
                })?
            };

            client
                .authenticate_publickey(&profile.username, Arc::new(key))
                .await
                .map_err(|e| SftpError::SessionFailed {
                    message: format!("Certificate authentication failed: {}", e),
                })?
        }
    };

    if !authenticated {
        return Err(SftpError::SessionFailed {
            message: format!("Authentication failed for user {}", profile.username),
        });
    }

    // Open channel for SFTP subsystem
    let channel = client
        .channel_open_session()
        .await
        .map_err(|e| SftpError::SessionFailed {
            message: format!("Failed to open SSH channel: {}", e),
        })?;

    channel
        .request_subsystem(false, "sftp")
        .await
        .map_err(|e| SftpError::SessionFailed {
            message: format!("Failed to request SFTP subsystem: {}", e),
        })?;

    let stream = ChannelStream::new(channel);
    let raw = RawSftpSession::new(stream);
    raw.init()
        .await
        .map_err(|e| SftpError::SessionFailed {
            message: format!("Failed to initialize SFTP session: {}", e),
        })?;

    let sftp = Arc::new(raw);

    // Resolve home directory
    let home_dir = match sftp.realpath(".").await {
        Ok(name) => name
            .files
            .first()
            .map(|f| f.filename.clone())
            .unwrap_or_else(|| "/".to_string()),
        Err(_) => "/".to_string(),
    };

    Ok((sftp, Arc::new(client), home_dir))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_connection_manager_empty_and_get_missing() {
        let mgr = SftpConnectionManager::new();
        assert!(!mgr.contains("sftp:missing").await);
        let err = mgr.get("sftp:missing").await.unwrap_err();
        assert_eq!(
            err,
            SftpError::SessionNotFound {
                session_id: "sftp:missing".to_string()
            }
        );
    }
}
