// Copyright (C) 2026 Bùi Thanh Xuân (klpod221)
// SPDX-License-Identifier: GPL-3.0-or-later

use std::path::PathBuf;
use std::sync::Arc;

use chrono::Utc;
use russh_sftp::client::RawSftpSession;
use russh_sftp::protocol::{FileAttributes, OpenFlags};
use tokio::sync::Mutex;

use crate::models::sftp::file_entry::FileEntry;
use crate::models::sftp::search::SearchResult;
use crate::services::sftp::connection::{
    establish_sftp_connection, SFTPClientHandler, SftpConnection, SftpConnectionManager,
};
use crate::services::sftp::edit;
use crate::services::sftp::errors::SftpError;
use crate::services::sftp::ops;
use crate::services::sftp::search;
use crate::services::ssh::SSHService;
use crate::services::ssh::key::SSHKeyService;

/// Compatibility session data for legacy transfer manager until Bước 3 & 4
#[allow(dead_code)]
pub struct SFTPSessionData {
    pub sftp: Arc<RawSftpSession>,
    pub client: Arc<russh::client::Handle<SFTPClientHandler>>,
    pub last_used: chrono::DateTime<Utc>,
}

/// SFTP Service facade for managing connections and operations
pub struct SFTPService {
    ssh_service: Arc<SSHService>,
    ssh_key_service: Arc<Mutex<SSHKeyService>>,
    connection_manager: SftpConnectionManager,
}

impl SFTPService {
    /// Create new SFTP service
    pub fn new(ssh_service: Arc<SSHService>, ssh_key_service: Arc<Mutex<SSHKeyService>>) -> Self {
        Self {
            ssh_service,
            ssh_key_service,
            connection_manager: SftpConnectionManager::new(),
        }
    }

    /// Connect to SFTP server using SSH profile
    pub async fn connect(
        &self,
        profile_id: String,
        known_hosts_path: PathBuf,
    ) -> Result<String, SftpError> {
        let session_key = format!("sftp:{}", profile_id);

        if self.connection_manager.contains(&session_key).await {
            return Ok(session_key);
        }

        let profile = self
            .ssh_service
            .get_ssh_profile(&profile_id)
            .await
            .map_err(|e| SftpError::Other {
                message: format!("Failed to get SSH profile: {}", e),
            })?;

        let (sftp, client, home_dir) = establish_sftp_connection(
            &profile,
            known_hosts_path,
            &self.ssh_key_service,
        )
        .await?;

        let conn = SftpConnection::new(
            session_key.clone(),
            profile_id,
            profile.name,
            sftp,
            home_dir,
            client,
        );

        self.connection_manager.insert(conn).await;
        Ok(session_key)
    }

    /// Disconnect an active SFTP session
    pub async fn disconnect(&self, session_id: String) -> Result<(), SftpError> {
        if let Some(conn) = self.connection_manager.remove(&session_id).await {
            conn.close().await?;
            Ok(())
        } else {
            Err(SftpError::SessionNotFound { session_id })
        }
    }

    /// Get connection handle
    pub async fn get_connection(&self, session_id: &str) -> Result<SftpConnection, SftpError> {
        self.connection_manager.get(session_id).await
    }

    /// Compatibility method for legacy transfer.rs
    pub async fn get_session(
        &self,
        session_id: &str,
    ) -> Result<Arc<Mutex<SFTPSessionData>>, SftpError> {
        let conn = self.get_connection(session_id).await?;
        Ok(Arc::new(Mutex::new(SFTPSessionData {
            sftp: conn.sftp.clone(),
            client: conn.client.clone(),
            last_used: conn.last_used_at(),
        })))
    }

    /// Resolve remote user's home directory
    pub async fn get_home_directory(&self, session_id: String) -> Result<String, SftpError> {
        let conn = self.get_connection(&session_id).await?;
        Ok(conn.home_dir.clone())
    }

    /// List directory contents (concurrent, lock-free)
    pub async fn list_directory(
        &self,
        session_id: String,
        path: String,
    ) -> Result<Vec<FileEntry>, SftpError> {
        let conn = self.get_connection(&session_id).await?;
        ops::list_directory(&conn.sftp, &path).await
    }

    /// Stat file or directory (concurrent, lock-free)
    pub async fn stat(&self, session_id: String, path: String) -> Result<FileEntry, SftpError> {
        let conn = self.get_connection(&session_id).await?;
        ops::stat_path(&conn.sftp, &path).await
    }

    /// Create directory
    pub async fn create_directory(
        &self,
        session_id: String,
        path: String,
    ) -> Result<(), SftpError> {
        let conn = self.get_connection(&session_id).await?;
        ops::create_directory(&conn.sftp, &path).await
    }

    /// Rename file or directory
    pub async fn rename(
        &self,
        session_id: String,
        old_path: String,
        new_path: String,
    ) -> Result<(), SftpError> {
        let conn = self.get_connection(&session_id).await?;
        ops::rename_path(&conn.sftp, &old_path, &new_path).await
    }

    /// Delete file or directory (iterative post-order when recursive)
    pub async fn delete(
        &self,
        session_id: String,
        path: String,
        recursive: bool,
    ) -> Result<(), SftpError> {
        let conn = self.get_connection(&session_id).await?;
        ops::delete_path(&conn.sftp, &path, recursive).await
    }

    /// Set permissions (chmod)
    pub async fn set_permissions(
        &self,
        session_id: String,
        path: String,
        permissions: u32,
    ) -> Result<(), SftpError> {
        let conn = self.get_connection(&session_id).await?;
        ops::set_permissions(&conn.sftp, &path, permissions).await
    }

    /// Create symlink
    pub async fn create_symlink(
        &self,
        session_id: String,
        path: String,
        target: String,
    ) -> Result<(), SftpError> {
        let conn = self.get_connection(&session_id).await?;
        ops::create_symlink(&conn.sftp, &path, &target).await
    }

    /// Read symlink
    pub async fn read_symlink(
        &self,
        session_id: String,
        path: String,
    ) -> Result<String, SftpError> {
        let conn = self.get_connection(&session_id).await?;
        ops::read_symlink(&conn.sftp, &path).await
    }

    /// Search for text using bounded grep
    pub async fn search(
        &self,
        session_id: String,
        path: String,
        query: String,
    ) -> Result<Vec<SearchResult>, SftpError> {
        let conn = self.get_connection(&session_id).await?;
        search::execute_search(&conn.client, &path, &query, None).await
    }

    /// Read text file with default limit
    pub async fn read_file(&self, session_id: String, path: String) -> Result<String, SftpError> {
        let resp = self
            .read_text_file(session_id, path, edit::DEFAULT_MAX_TEXT_BYTES)
            .await?;
        if resp.is_binary {
            return Err(SftpError::Other {
                message: "File appears to be binary, not text".to_string(),
            });
        }
        Ok(resp.content)
    }

    /// Read text file with revision and binary detection
    pub async fn read_text_file(
        &self,
        session_id: String,
        path: String,
        max_bytes: usize,
    ) -> Result<edit::ReadTextFileResponse, SftpError> {
        let conn = self.get_connection(&session_id).await?;
        edit::read_text_file(&conn.sftp, &path, max_bytes).await
    }

    /// Write text file with safe temporary file and atomic replace
    pub async fn write_file(
        &self,
        session_id: String,
        path: String,
        content: String,
    ) -> Result<(), SftpError> {
        self.write_text_file(edit::WriteTextFileRequest {
            session_id,
            path,
            content,
            expected_revision: None,
            overwrite: true,
        })
        .await?;
        Ok(())
    }

    /// Write text file with optimistic revision concurrency check
    pub async fn write_text_file(
        &self,
        request: edit::WriteTextFileRequest,
    ) -> Result<edit::WriteTextFileResponse, SftpError> {
        let conn = self.get_connection(&request.session_id).await?;
        edit::write_text_file(&conn.sftp, request).await
    }

    /// Upload local file bytes to remote destination (used by sync service)
    pub async fn upload_file_bytes(
        &self,
        session_id: String,
        local_path: String,
        remote_path: String,
    ) -> Result<(), SftpError> {
        let conn = self.get_connection(&session_id).await?;
        let bytes = tokio::fs::read(&local_path)
            .await
            .map_err(|e| SftpError::IoError {
                message: format!("Failed to read local file {}: {}", local_path, e),
            })?;

        let handle = conn
            .sftp
            .open(
                &remote_path,
                OpenFlags::CREATE | OpenFlags::TRUNCATE | OpenFlags::WRITE,
                FileAttributes::empty(),
            )
            .await
            .map_err(|e| SftpError::from_russh_sftp(e, Some(&remote_path)))?
            .handle;

        let mut offset = 0u64;
        let mut write_res = Ok(());
        for chunk in bytes.chunks(32768) {
            if let Err(e) = conn.sftp.write(&handle, offset, chunk.to_vec()).await {
                write_res = Err(SftpError::from_russh_sftp(e, Some(&remote_path)));
                break;
            }
            offset += chunk.len() as u64;
        }
        let _ = conn.sftp.close(&handle).await;

        write_res?;
        Ok(())
    }

    /// Download remote file to local destination (used by sync service)
    pub async fn download_file_bytes(
        &self,
        session_id: String,
        remote_path: String,
        local_path: String,
    ) -> Result<(), SftpError> {
        let conn = self.get_connection(&session_id).await?;
        let handle = conn
            .sftp
            .open(
                &remote_path,
                OpenFlags::READ,
                FileAttributes::empty(),
            )
            .await
            .map_err(|e| SftpError::from_russh_sftp(e, Some(&remote_path)))?
            .handle;

        let meta = conn
            .sftp
            .fstat(&handle)
            .await
            .map_err(|e| SftpError::from_russh_sftp(e, Some(&remote_path)))?
            .attrs;
        let size = meta.size.unwrap_or(0);

        let mut file_data = Vec::with_capacity(size.min(10 * 1024 * 1024) as usize);
        let mut offset = 0u64;
        let mut read_err = None;

        while offset < size {
            let to_read = (size - offset).min(32768) as u32;
            match conn.sftp.read(&handle, offset, to_read).await {
                Ok(chunk) => {
                    if chunk.data.is_empty() {
                        break;
                    }
                    offset += chunk.data.len() as u64;
                    file_data.extend_from_slice(&chunk.data);
                }
                Err(e) => {
                    read_err = Some(SftpError::from_russh_sftp(e, Some(&remote_path)));
                    break;
                }
            }
        }
        let _ = conn.sftp.close(&handle).await;

        if let Some(err) = read_err {
            return Err(err);
        }

        // Ensure parent directory exists locally
        if let Some(parent) = std::path::Path::new(&local_path).parent() {
            tokio::fs::create_dir_all(parent)
                .await
                .map_err(|e| SftpError::IoError {
                    message: format!("Failed to create local parent directory: {}", e),
                })?;
        }

        tokio::fs::write(&local_path, &file_data)
            .await
            .map_err(|e| SftpError::IoError {
                message: format!("Failed to create local file {}: {}", local_path, e),
            })?;

        Ok(())
    }
}
