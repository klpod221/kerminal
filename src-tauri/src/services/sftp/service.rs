use std::collections::HashMap;
use std::path::Path;
use std::sync::Arc;
use tokio::sync::{Mutex, RwLock};

use crate::core::proxy::create_proxy_stream;
use crate::models::sftp::search::SearchResult;
use crate::models::sftp::{error::SFTPError, file_entry::FileEntry, FileType};
use crate::models::ssh::AuthData;
use crate::services::ssh::{SSHKeyService, SSHService};

use crate::services::sftp::channel_stream::ChannelStream;
use anyhow::Result;
use async_trait::async_trait;
use chrono::Utc;
use russh::client::Config;
use russh_keys::key::PublicKey;
use russh_sftp::client::SftpSession;

/// Simple handler for SFTP connections
#[derive(Clone)]
pub struct SFTPClientHandler;

#[async_trait]
impl russh::client::Handler for SFTPClientHandler {
    type Error = russh::Error;

    async fn check_server_key(
        &mut self,
        _server_public_key: &PublicKey,
    ) -> Result<bool, Self::Error> {
        Ok(true)
    }
}

/// Internal SFTP session data
pub struct SFTPSessionData {
    pub sftp: SftpSession,
    pub client: Arc<russh::client::Handle<SFTPClientHandler>>,
    last_used: chrono::DateTime<Utc>,
}

/// SFTP Service for managing SFTP connections and file operations
pub struct SFTPService {
    ssh_service: Arc<SSHService>,
    ssh_key_service: Arc<Mutex<SSHKeyService>>,
    sessions: Arc<RwLock<HashMap<String, Arc<Mutex<SFTPSessionData>>>>>,
}

impl SFTPService {
    /// Create new SFTP service
    pub fn new(ssh_service: Arc<SSHService>, ssh_key_service: Arc<Mutex<SSHKeyService>>) -> Self {
        Self {
            ssh_service,
            ssh_key_service,
            sessions: Arc::new(RwLock::new(HashMap::new())),
        }
    }

    /// Connect to SFTP server using SSH profile
    pub async fn connect(&self, profile_id: String) -> Result<String, SFTPError> {
        // Get profile from database
        let profile = self
            .ssh_service
            .get_ssh_profile(&profile_id)
            .await
            .map_err(|e| SFTPError::Other {
                message: format!("Failed to get SSH profile: {}", e),
            })?;

        // Check if session already exists
        let session_key = format!("sftp:{}", profile_id);
        {
            let sessions = self.sessions.read().await;
            if sessions.contains_key(&session_key) {
                return Ok(session_key.clone());
            }
        }

        // Create SSH session
        let keepalive_interval = if profile.keep_alive {
            Some(std::time::Duration::from_secs(15))
        } else {
            None
        };

        let inactivity_timeout = profile
            .timeout
            .map(|t| std::time::Duration::from_secs(t as u64));

        let mut config = Config {
            inactivity_timeout,
            keepalive_interval,
            keepalive_max: 10,
            ..Default::default()
        };

        config.window_size = 2097152;
        config.maximum_packet_size = 32768;

        let config = Arc::new(config);
        let handler = SFTPClientHandler;

        let mut session = if let Some(proxy_config) = &profile.proxy {
            let stream = create_proxy_stream(proxy_config, &profile.host, profile.port)
                .await
                .map_err(|e| SFTPError::SessionFailed {
                    message: format!("Failed to create proxy connection: {}", e),
                })?;

            russh::client::connect_stream(config, stream, handler)
                .await
                .map_err(|e| SFTPError::SessionFailed {
                    message: format!("Failed to connect via proxy: {}", e),
                })?
        } else {
            russh::client::connect(config, (&profile.host as &str, profile.port), handler)
                .await
                .map_err(|e| SFTPError::SessionFailed {
                    message: format!(
                        "Failed to connect to {}:{}: {}",
                        profile.host, profile.port, e
                    ),
                })?
        };

        // Authenticate
        let authenticated = match &profile.auth_data {
            AuthData::Password { password } => session
                .authenticate_password(&profile.username, password)
                .await
                .map_err(|e| SFTPError::SessionFailed {
                    message: format!("Password authentication failed: {}", e),
                })?,
            AuthData::KeyReference { key_id } => {
                let key_service = self.ssh_key_service.lock().await;
                let resolved_key = key_service
                    .resolve_key_for_auth(key_id)
                    .await
                    .map_err(|e| SFTPError::SessionFailed {
                        message: format!("Failed to resolve SSH key: {}", e),
                    })?;

                let key = if Path::new(&resolved_key.private_key).exists() {
                    russh_keys::load_secret_key(
                        &resolved_key.private_key,
                        resolved_key.passphrase.as_deref(),
                    )
                    .map_err(|e| SFTPError::SessionFailed {
                        message: format!("Failed to load SSH key: {}", e),
                    })?
                } else {
                    russh_keys::decode_secret_key(
                        &resolved_key.private_key,
                        resolved_key.passphrase.as_deref(),
                    )
                    .map_err(|e| SFTPError::SessionFailed {
                        message: format!("Failed to parse SSH key: {}", e),
                    })?
                };

                session
                    .authenticate_publickey(&profile.username, Arc::new(key))
                    .await
                    .map_err(|e| SFTPError::SessionFailed {
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
                        SFTPError::SessionFailed {
                            message: format!("Failed to load certificate key: {}", e),
                        }
                    })?
                } else {
                    russh_keys::decode_secret_key(private_key, None).map_err(|e| {
                        SFTPError::SessionFailed {
                            message: format!("Failed to parse certificate key: {}", e),
                        }
                    })?
                };

                session
                    .authenticate_publickey(&profile.username, Arc::new(key))
                    .await
                    .map_err(|e| SFTPError::SessionFailed {
                        message: format!("Certificate authentication failed: {}", e),
                    })?
            }
        };

        if !authenticated {
            return Err(SFTPError::SessionFailed {
                message: "Authentication failed".to_string(),
            });
        }

        // Open SFTP channel
        let channel =
            session
                .channel_open_session()
                .await
                .map_err(|e| SFTPError::SessionFailed {
                    message: format!("Failed to open SSH channel: {}", e),
                })?;

        // Request SFTP subsystem
        channel
            .request_subsystem(false, "sftp")
            .await
            .map_err(|e| SFTPError::SessionFailed {
                message: format!("Failed to request SFTP subsystem: {}", e),
            })?;

        // Create SFTP session from channel stream
        let stream = ChannelStream::new(channel);
        let sftp = SftpSession::new(stream)
            .await
            .map_err(|e| SFTPError::SessionFailed {
                message: format!("Failed to initialize SFTP session: {}", e),
            })?;

        let now = Utc::now();
        let session_data = SFTPSessionData {
            sftp,
            client: Arc::new(session),
            last_used: now,
        };

        let session_arc = Arc::new(Mutex::new(session_data));
        {
            let mut sessions = self.sessions.write().await;
            sessions.insert(session_key.clone(), session_arc);
        }

        Ok(session_key)
    }

    /// Disconnect SFTP session
    pub async fn disconnect(&self, session_id: String) -> Result<(), SFTPError> {
        let mut sessions = self.sessions.write().await;
        if sessions.remove(&session_id).is_some() {
            Ok(())
        } else {
            Err(SFTPError::SessionNotFound { session_id })
        }
    }

    /// Get SFTP session (internal helper)
    pub async fn get_session(
        &self,
        session_id: &str,
    ) -> Result<Arc<Mutex<SFTPSessionData>>, SFTPError> {
        let sessions = self.sessions.read().await;
        sessions
            .get(session_id)
            .cloned()
            .ok_or_else(|| SFTPError::SessionNotFound {
                session_id: session_id.to_string(),
            })
    }

    /// List directory contents
    pub async fn list_directory(
        &self,
        session_id: String,
        path: String,
    ) -> Result<Vec<FileEntry>, SFTPError> {
        let session_data = self.get_session(&session_id).await?;
        let mut data = session_data.lock().await;
        data.last_used = Utc::now();

        let mut entries = Vec::new();
        let mut read_dir = data
            .sftp
            .read_dir(&path)
            .await
            .map_err(|e| SFTPError::Other {
                message: format!("Failed to read directory {}: {}", path, e),
            })?;

        while let Some(dir_entry) = read_dir.next() {
            let name = dir_entry.file_name();

            let full_path = if path.ends_with('/') {
                format!("{}{}", path, name)
            } else {
                format!("{}/{}", path, name)
            };

            let attrs = dir_entry.metadata();
            let file_type = match attrs.file_type() {
                russh_sftp::protocol::FileType::Dir => FileType::Directory,
                russh_sftp::protocol::FileType::File => FileType::File,
                russh_sftp::protocol::FileType::Symlink => FileType::Symlink,
                _ => FileType::Unknown,
            };

            let symlink_target = if matches!(file_type, FileType::Symlink) {
                data.sftp.read_link(&full_path).await.ok()
            } else {
                None
            };

            entries.push(FileEntry {
                name: name.clone(),
                path: full_path.clone(),
                file_type,
                size: attrs.size,
                permissions: attrs.permissions.unwrap_or(0o644),
                modified: attrs
                    .mtime
                    .map(|t| {
                        chrono::DateTime::<Utc>::from_timestamp(t as i64, 0)
                            .unwrap_or_else(|| Utc::now())
                    })
                    .unwrap_or_else(|| Utc::now()),
                accessed: attrs
                    .atime
                    .map(|t| chrono::DateTime::<Utc>::from_timestamp(t as i64, 0))
                    .flatten(),
                symlink_target,
                uid: attrs.uid,
                gid: attrs.gid,
            });
        }

        Ok(entries)
    }

    /// Get file attributes (stat)
    pub async fn stat(&self, session_id: String, path: String) -> Result<FileEntry, SFTPError> {
        let session_data = self.get_session(&session_id).await?;
        let mut data = session_data.lock().await;
        data.last_used = Utc::now();

        let attrs = data.sftp.metadata(&path).await.map_err(|e| {
            if e.to_string().contains("not found") || e.to_string().contains("No such file") {
                SFTPError::FileNotFound { path: path.clone() }
            } else {
                SFTPError::Other {
                    message: format!("Failed to get metadata for {}: {}", path, e),
                }
            }
        })?;

        let file_type = match attrs.file_type() {
            russh_sftp::protocol::FileType::Dir => FileType::Directory,
            russh_sftp::protocol::FileType::File => FileType::File,
            russh_sftp::protocol::FileType::Symlink => FileType::Symlink,
            _ => FileType::Unknown,
        };

        let name = Path::new(&path)
            .file_name()
            .and_then(|n| n.to_str())
            .unwrap_or(&path)
            .to_string();

        let size = attrs.size;

        let permissions = attrs.permissions.unwrap_or(0o644);

        let modified = attrs
            .mtime
            .map(|t| {
                chrono::DateTime::<Utc>::from_timestamp(t as i64, 0).unwrap_or_else(|| Utc::now())
            })
            .unwrap_or_else(|| Utc::now());

        let accessed = attrs
            .atime
            .map(|t| chrono::DateTime::<Utc>::from_timestamp(t as i64, 0))
            .flatten();

        let symlink_target = if file_type == FileType::Symlink {
            data.sftp.read_link(&path).await.ok()
        } else {
            None
        };

        Ok(FileEntry {
            name,
            path,
            file_type,
            size,
            permissions,
            modified,
            accessed,
            symlink_target,
            uid: attrs.uid,
            gid: attrs.gid,
        })
    }

    /// Create directory
    pub async fn create_directory(
        &self,
        session_id: String,
        path: String,
    ) -> Result<(), SFTPError> {
        let session_data = self.get_session(&session_id).await?;
        let mut data = session_data.lock().await;
        data.last_used = Utc::now();

        data.sftp.create_dir(&path).await.map_err(|e| {
            if e.to_string().contains("already exists") {
                SFTPError::FileExists { path }
            } else {
                SFTPError::Other {
                    message: format!("Failed to create directory: {}", e),
                }
            }
        })?;

        Ok(())
    }

    /// Rename/move file or directory
    pub async fn rename(
        &self,
        session_id: String,
        old_path: String,
        new_path: String,
    ) -> Result<(), SFTPError> {
        let session_data = self.get_session(&session_id).await?;
        let mut data = session_data.lock().await;
        data.last_used = Utc::now();

        data.sftp
            .rename(&old_path, &new_path)
            .await
            .map_err(|e| SFTPError::Other {
                message: format!("Failed to rename {} to {}: {}", old_path, new_path, e),
            })?;

        Ok(())
    }

    /// Delete file or directory
    pub async fn delete(
        &self,
        session_id: String,
        path: String,
        recursive: bool,
    ) -> Result<(), SFTPError> {
        let session_data = self.get_session(&session_id).await?;
        let mut data = session_data.lock().await;
        data.last_used = Utc::now();

        if recursive {
            // For directories, we need to delete recursively
            let attrs = data.sftp.metadata(&path).await.map_err(|e| {
                if e.to_string().contains("not found") {
                    SFTPError::FileNotFound { path: path.clone() }
                } else {
                    SFTPError::Other {
                        message: format!("Failed to get metadata for {}: {}", path, e),
                    }
                }
            })?;

            if attrs.file_type() == russh_sftp::protocol::FileType::Dir {
                // List and delete contents
                let mut read_dir =
                    data.sftp
                        .read_dir(&path)
                        .await
                        .map_err(|e| SFTPError::Other {
                            message: format!("Failed to read directory {}: {}", path, e),
                        })?;

                while let Some(dir_entry) = read_dir.next() {
                    let name = dir_entry.file_name();

                    if name == "." || name == ".." {
                        continue;
                    }

                    let full_path = if path.ends_with('/') {
                        format!("{}{}", path, name)
                    } else {
                        format!("{}/{}", path, name)
                    };

                    // Recursive delete
                    self.delete_internal(&mut data, &full_path, true).await?;
                }
            }
        }

        self.delete_internal(&mut data, &path, false).await
    }

    /// Internal delete helper
    async fn delete_internal(
        &self,
        data: &mut SFTPSessionData,
        path: &str,
        is_recursive_call: bool,
    ) -> Result<(), SFTPError> {
        let attrs = data.sftp.metadata(path).await.map_err(|e| {
            if e.to_string().contains("not found") && !is_recursive_call {
                SFTPError::FileNotFound {
                    path: path.to_string(),
                }
            } else {
                SFTPError::Other {
                    message: format!("Failed to get metadata for {}: {}", path, e),
                }
            }
        })?;

        match attrs.file_type() {
            russh_sftp::protocol::FileType::Dir => {
                data.sftp
                    .remove_dir(path)
                    .await
                    .map_err(|e| SFTPError::Other {
                        message: format!("Failed to remove directory {}: {}", path, e),
                    })?;
            }
            russh_sftp::protocol::FileType::File | russh_sftp::protocol::FileType::Symlink => {
                data.sftp
                    .remove_file(path)
                    .await
                    .map_err(|e| SFTPError::Other {
                        message: format!("Failed to remove file {}: {}", path, e),
                    })?;
            }
            _ => {
                return Err(SFTPError::Other {
                    message: format!("Unknown file type for {}", path),
                });
            }
        }

        Ok(())
    }

    /// Set file permissions (chmod)
    pub async fn set_permissions(
        &self,
        session_id: String,
        path: String,
        mode: u32,
    ) -> Result<(), SFTPError> {
        let session_data = self.get_session(&session_id).await?;
        let mut data = session_data.lock().await;
        data.last_used = Utc::now();

        let mut attrs = data.sftp.metadata(&path).await.map_err(|e| {
            if e.to_string().contains("not found") || e.to_string().contains("No such file") {
                SFTPError::FileNotFound { path: path.clone() }
            } else {
                SFTPError::Other {
                    message: format!("Failed to get metadata for {}: {}", path, e),
                }
            }
        })?;

        // Mask mode to only include permission bits (0o777)
        // This ensures we don't accidentally set file type bits
        let permission_mode = mode & 0o777;

        // Only update permissions, preserve all other attributes
        attrs.permissions = Some(permission_mode);

        data.sftp
            .set_metadata(&path, attrs)
            .await
            .map_err(|e| SFTPError::Other {
                message: format!("Failed to set permissions on {}: {}", path, e),
            })?;

        Ok(())
    }

    /// Create symlink
    pub async fn create_symlink(
        &self,
        session_id: String,
        target: String,
        link_path: String,
    ) -> Result<(), SFTPError> {
        let session_data = self.get_session(&session_id).await?;
        let mut data = session_data.lock().await;
        data.last_used = Utc::now();

        data.sftp
            .symlink(&link_path, &target)
            .await
            .map_err(|e| SFTPError::Other {
                message: format!(
                    "Failed to create symlink {} -> {}: {}",
                    link_path, target, e
                ),
            })?;

        Ok(())
    }

    /// Read symlink target
    pub async fn read_symlink(
        &self,
        session_id: String,
        path: String,
    ) -> Result<String, SFTPError> {
        let session_data = self.get_session(&session_id).await?;
        let mut data = session_data.lock().await;
        data.last_used = Utc::now();

        let target = data.sftp.read_link(&path).await.map_err(|e| {
            if e.to_string().contains("not found") {
                SFTPError::FileNotFound { path: path.clone() }
            } else {
                SFTPError::Other {
                    message: format!("Failed to read symlink {}: {}", path, e),
                }
            }
        })?;

        Ok(target)
    }

    /// Read file content as text
    pub async fn read_file(&self, session_id: String, path: String) -> Result<String, SFTPError> {
        let session_data = self.get_session(&session_id).await?;
        let mut data = session_data.lock().await;
        data.last_used = Utc::now();

        // Check if file exists and is a regular file
        let attrs = data.sftp.metadata(&path).await.map_err(|e| {
            if e.to_string().contains("not found") || e.to_string().contains("No such file") {
                SFTPError::FileNotFound { path: path.clone() }
            } else {
                SFTPError::Other {
                    message: format!("Failed to get metadata for {}: {}", path, e),
                }
            }
        })?;

        if attrs.file_type() != russh_sftp::protocol::FileType::File {
            return Err(SFTPError::Other {
                message: format!("Path is not a regular file: {}", path),
            });
        }

        // Check file size (limit to 10MB for text files)
        let file_size = attrs.size.unwrap_or(0);
        if file_size > 10 * 1024 * 1024 {
            return Err(SFTPError::Other {
                message: format!(
                    "File too large to edit ({} bytes). Maximum size is 10MB",
                    file_size
                ),
            });
        }

        // Open and read file
        let mut remote_file = data.sftp.open(&path).await.map_err(|e| SFTPError::Other {
            message: format!("Failed to open file {}: {}", path, e),
        })?;

        use tokio::io::AsyncReadExt;
        let mut buffer = Vec::with_capacity(file_size as usize);
        remote_file
            .read_to_end(&mut buffer)
            .await
            .map_err(|e| SFTPError::Other {
                message: format!("Failed to read file {}: {}", path, e),
            })?;

        // Try to decode as UTF-8
        String::from_utf8(buffer).map_err(|e| SFTPError::Other {
            message: format!("File {} is not valid UTF-8: {}", path, e),
        })
    }

    /// Search for files containing text using grep
    pub async fn search(
        &self,
        session_id: String,
        path: String,
        query: String,
    ) -> Result<Vec<SearchResult>, SFTPError> {
        let session_data = self.get_session(&session_id).await?;

        // Clone client handle to avoid holding lock during search
        let client = {
            let mut data = session_data.lock().await;
            data.last_used = Utc::now();
            data.client.clone()
        };

        // Open a new channel for the search command
        let mut channel = client
            .channel_open_session()
            .await
            .map_err(|e| SFTPError::Other {
                message: format!("Failed to open channel for search: {}", e),
            })?;

        // Escape query to prevent command injection
        // This is a basic escaping, ideally we'd use a robust shell escaping library
        let escaped_query = query.replace("\"", "\\\"");
        let command = format!("grep -rInH \"{}\" \"{}\"", escaped_query, path);

        channel
            .exec(true, command)
            .await
            .map_err(|e| SFTPError::Other {
                message: format!("Failed to execute search command: {}", e),
            })?;

        // Read stdout
        let output = {
            let mut stdout = channel.make_reader();
            use tokio::io::AsyncReadExt;
            let mut buffer = Vec::new();
            stdout
                .read_to_end(&mut buffer)
                .await
                .map_err(|e| SFTPError::Other {
                    message: format!("Failed to read search output: {}", e),
                })?;
            String::from_utf8_lossy(&buffer).to_string()
        };

        let mut results = Vec::new();
        for line in output.lines() {
            // Grep output format: filename:line:content
            // Note: filename might contain colons, so we should look for the first two colons carefully
            // But standard grep output puts filename first.
            // Split by colon, limit 3 parts? No, content might have colons.

            let parts: Vec<&str> = line.splitn(3, ':').collect();
            if parts.len() >= 3 {
                let file_path = parts[0].to_string();
                if let Ok(line_number) = parts[1].parse::<u64>() {
                    // content is rest
                    let content = parts[2].to_string();
                    results.push(SearchResult {
                        file_path,
                        line_number,
                        content,
                    });
                }
            }
        }

        Ok(results)
    }

    /// Write file content as text
    pub async fn write_file(
        &self,
        session_id: String,
        path: String,
        content: String,
    ) -> Result<(), SFTPError> {
        let session_data = self.get_session(&session_id).await?;
        let mut data = session_data.lock().await;
        data.last_used = Utc::now();

        use russh_sftp::protocol::OpenFlags;
        use tokio::io::AsyncWriteExt;

        // Open file for writing (create if not exists, truncate if exists)
        let mut remote_file = data
            .sftp
            .open_with_flags(
                &path,
                OpenFlags::CREATE | OpenFlags::TRUNCATE | OpenFlags::WRITE,
            )
            .await
            .map_err(|e| SFTPError::Other {
                message: format!("Failed to open file for writing {}: {}", path, e),
            })?;

        // Write content
        remote_file
            .write_all(content.as_bytes())
            .await
            .map_err(|e| SFTPError::Other {
                message: format!("Failed to write file {}: {}", path, e),
            })?;

        // Flush to ensure data is written
        remote_file.flush().await.map_err(|e| SFTPError::Other {
            message: format!("Failed to flush file {}: {}", path, e),
        })?;

        Ok(())
    }
}
