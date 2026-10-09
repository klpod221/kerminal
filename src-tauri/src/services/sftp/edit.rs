// Copyright (C) 2026 Bùi Thanh Xuân (klpod221)
// SPDX-License-Identifier: GPL-3.0-or-later

use std::sync::Arc;

use russh_sftp::client::error::Error as RusshSftpError;
use russh_sftp::client::RawSftpSession;
use russh_sftp::protocol::{FileAttributes, OpenFlags, StatusCode};
use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};
use uuid::Uuid;

use crate::services::sftp::errors::SftpError;

/// Maximum size allowed for opening/editing text files (default: 10MB)
pub const DEFAULT_MAX_TEXT_BYTES: usize = 10 * 1024 * 1024;

/// Threshold below which sha256 hash is computed for revisions
pub const HASH_THRESHOLD_BYTES: u64 = 2 * 1024 * 1024; // 2MB

/// File revision representing a snapshot of remote file state
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub struct FileRevision {
    /// File size in bytes
    pub size: u64,
    /// Last modified time in Unix seconds
    pub mtime: u64,
    /// Unix permission bits (e.g. 0o644)
    pub permissions_mode: Option<u32>,
    /// Optional SHA-256 hash of file content
    pub hash: Option<String>,
}

impl FileRevision {
    /// Compare whether this revision matches another revision for optimistic concurrency
    pub fn matches(&self, other: &FileRevision) -> bool {
        if self.size != other.size || self.mtime != other.mtime {
            return false;
        }

        match (&self.hash, &other.hash) {
            (Some(h1), Some(h2)) => h1 == h2,
            _ => true,
        }
    }
}

/// Request to read a remote file for editing
#[allow(dead_code)]
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ReadTextFileRequest {
    pub session_id: String,
    pub path: String,
    pub max_bytes: Option<usize>,
}

/// Response after reading a remote file
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ReadTextFileResponse {
    pub path: String,
    pub content: String,
    pub bytes_read: usize,
    pub truncated: bool,
    pub is_binary: bool,
    pub revision: FileRevision,
}

/// Request to write/save an edited remote text file
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct WriteTextFileRequest {
    pub session_id: String,
    pub path: String,
    pub content: String,
    pub expected_revision: Option<FileRevision>,
    pub overwrite: bool,
}

/// Response after writing a remote text file
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct WriteTextFileResponse {
    pub path: String,
    pub bytes_written: usize,
    pub new_revision: FileRevision,
}

/// Determine if raw buffer represents binary content (presence of NUL bytes)
pub fn is_binary_content(buffer: &[u8]) -> bool {
    let check_len = buffer.len().min(8192);
    buffer[..check_len].contains(&0)
}

/// Compute SHA-256 hex string of bytes
pub fn sha256_hex(bytes: &[u8]) -> String {
    let mut hasher = Sha256::new();
    hasher.update(bytes);
    let result = hasher.finalize();
    result.iter().map(|b| format!("{:02x}", b)).collect()
}

/// Get the current revision of a remote file
pub async fn get_file_revision(
    sftp: &Arc<RawSftpSession>,
    path: &str,
) -> Result<FileRevision, SftpError> {
    let meta = sftp
        .stat(path)
        .await
        .map_err(|e| SftpError::from_russh_sftp(e, Some(path)))?
        .attrs;

    let size = meta.size.unwrap_or(0);
    let mtime = meta.mtime.unwrap_or(0) as u64;
    let permissions_mode = meta.permissions;

    let hash = if size <= HASH_THRESHOLD_BYTES {
        let handle = sftp
            .open(path, OpenFlags::READ, FileAttributes::empty())
            .await
            .map_err(|e| SftpError::from_russh_sftp(e, Some(path)))?
            .handle;

        let read_res = sftp.read(&handle, 0, size as u32).await;
        let _ = sftp.close(&handle).await;

        match read_res {
            Ok(data) => Some(sha256_hex(&data.data)),
            Err(_) => None,
        }
    } else {
        None
    };

    Ok(FileRevision {
        size,
        mtime,
        permissions_mode,
        hash,
    })
}

/// Read text file with size limit, binary check, and revision computation
pub async fn read_text_file(
    sftp: &Arc<RawSftpSession>,
    path: &str,
    max_bytes: usize,
) -> Result<ReadTextFileResponse, SftpError> {
    let meta = sftp
        .stat(path)
        .await
        .map_err(|e| SftpError::from_russh_sftp(e, Some(path)))?
        .attrs;

    if !meta.is_regular() {
        return Err(SftpError::Other {
            message: format!("Path is not a regular file: {}", path),
        });
    }

    let file_size = meta.size.unwrap_or(0);
    let mtime = meta.mtime.unwrap_or(0) as u64;
    let permissions_mode = meta.permissions;

    let limit = max_bytes.min(DEFAULT_MAX_TEXT_BYTES);
    let handle = sftp
        .open(path, OpenFlags::READ, FileAttributes::empty())
        .await
        .map_err(|e| SftpError::from_russh_sftp(e, Some(path)))?
        .handle;

    // Read up to limit + 1 to detect truncation
    let read_target = (limit + 1).min(file_size as usize + 1);
    let read_res = sftp.read(&handle, 0, read_target as u32).await;
    let _ = sftp.close(&handle).await;

    let mut buffer = match read_res {
        Ok(data) => data.data.to_vec(),
        Err(RusshSftpError::Status(status)) if status.status_code == StatusCode::Eof => Vec::new(),
        Err(e) => return Err(SftpError::from_russh_sftp(e, Some(path))),
    };

    let truncated = buffer.len() > limit;
    if truncated {
        buffer.truncate(limit);
    }

    let is_binary = is_binary_content(&buffer);
    let content = if is_binary {
        String::new()
    } else {
        String::from_utf8_lossy(&buffer).into_owned()
    };

    let hash = if !truncated && file_size <= HASH_THRESHOLD_BYTES {
        Some(sha256_hex(&buffer))
    } else {
        None
    };

    let revision = FileRevision {
        size: file_size,
        mtime,
        permissions_mode,
        hash,
    };

    Ok(ReadTextFileResponse {
        path: path.to_string(),
        content,
        bytes_read: buffer.len(),
        truncated,
        is_binary,
        revision,
    })
}

/// Safely replace target file by renaming to backup first, then renaming temp over target.
/// If replacement fails, restores backup so the file is never lost.
pub async fn safe_replace_remote_file(
    sftp: &Arc<RawSftpSession>,
    temp_path: &str,
    target_path: &str,
) -> Result<(), SftpError> {
    let target_exists = sftp.stat(target_path).await.is_ok();

    if !target_exists {
        // Direct rename
        if let Err(e) = sftp.rename(temp_path, target_path).await {
            let _ = sftp.remove(temp_path).await;
            return Err(SftpError::from_russh_sftp(e, Some(target_path)));
        }
        return Ok(());
    }

    // Target exists: create backup name
    let backup_path = format!("{}.kerminal-bak-{}", target_path, Uuid::new_v4());

    // Step 1: rename target -> backup
    sftp.rename(target_path, &backup_path)
        .await
        .map_err(|e| SftpError::from_russh_sftp(e, Some(target_path)))?;

    // Step 2: rename temp -> target
    if let Err(rename_err) = sftp.rename(temp_path, target_path).await {
        // Step 2 failed: restore backup -> target
        let _ = sftp.rename(&backup_path, target_path).await;
        let _ = sftp.remove(temp_path).await;
        return Err(SftpError::from_russh_sftp(rename_err, Some(target_path)));
    }

    // Step 3: remove backup after successful replacement
    let _ = sftp.remove(&backup_path).await;
    Ok(())
}

/// Write text file with revision conflict checking, temporary file write, and atomic replace
pub async fn write_text_file(
    sftp: &Arc<RawSftpSession>,
    req: WriteTextFileRequest,
) -> Result<WriteTextFileResponse, SftpError> {
    let path = req.path.as_str();

    // Check revision if expected_revision is set and overwrite is false
    if !req.overwrite {
        if let Some(expected) = req.expected_revision.as_ref() {
            if let Ok(current) = get_file_revision(sftp, path).await {
                if !expected.matches(&current) {
                    return Err(SftpError::Conflict {
                        message: format!(
                            "Remote file {} was modified concurrently (size: {}B vs {}B, mtime: {} vs {}).",
                            path, current.size, expected.size, current.mtime, expected.mtime
                        ),
                    });
                }
            }
        }
    }

    // Read existing file permissions to preserve them
    let existing_mode = sftp.stat(path).await.ok().and_then(|m| m.attrs.permissions);

    // Generate temp file name
    let temp_path = format!("{}.kerminal-edit-{}.tmp", path, Uuid::new_v4());
    let bytes = req.content.as_bytes();

    let handle = sftp
        .open(
            &temp_path,
            OpenFlags::CREATE | OpenFlags::TRUNCATE | OpenFlags::WRITE,
            FileAttributes::empty(),
        )
        .await
        .map_err(|e| SftpError::from_russh_sftp(e, Some(&temp_path)))?
        .handle;

    let write_res = sftp.write(&handle, 0, bytes.to_vec()).await;
    let _ = sftp.close(&handle).await;

    write_res.map_err(|e| SftpError::from_russh_sftp(e, Some(&temp_path)))?;

    // Apply preserved permissions to temp file
    if let Some(mode) = existing_mode {
        let mut attrs = FileAttributes::empty();
        attrs.permissions = Some(mode);
        let _ = sftp.setstat(&temp_path, attrs).await;
    }

    // Perform safe replacement
    safe_replace_remote_file(sftp, &temp_path, path).await?;

    // Calculate new revision
    let new_revision = get_file_revision(sftp, path).await.unwrap_or_else(|_| FileRevision {
        size: bytes.len() as u64,
        mtime: chrono::Utc::now().timestamp() as u64,
        permissions_mode: existing_mode,
        hash: Some(sha256_hex(bytes)),
    });

    Ok(WriteTextFileResponse {
        path: req.path,
        bytes_written: bytes.len(),
        new_revision,
    })
}

/// Delete a file if it exists, ignoring not found errors
#[allow(dead_code)]
pub async fn delete_if_exists(sftp: &Arc<RawSftpSession>, path: &str) -> Result<(), SftpError> {
    let _ = sftp.remove(path).await;
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_is_binary_content() {
        assert!(!is_binary_content(b"Hello world\nThis is text!"));
        assert!(is_binary_content(b"Hello\x00world"));
    }

    #[test]
    fn test_sha256_hex() {
        let hash = sha256_hex(b"hello");
        assert_eq!(
            hash,
            "2cf24dba5fb0a30e26e83b2ac5b9e29e1b161e5c1fa7425e73043362938b9824"
        );
    }

    #[test]
    fn test_revision_matches() {
        let r1 = FileRevision {
            size: 100,
            mtime: 200,
            permissions_mode: Some(0o644),
            hash: Some("abc".into()),
        };
        let r2 = FileRevision {
            size: 100,
            mtime: 200,
            permissions_mode: Some(0o644),
            hash: Some("abc".into()),
        };
        let r3 = FileRevision {
            size: 101,
            mtime: 200,
            permissions_mode: Some(0o644),
            hash: Some("abc".into()),
        };
        assert!(r1.matches(&r2));
        assert!(!r1.matches(&r3));
    }
}
