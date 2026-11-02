// Copyright (C) 2026 Bùi Thanh Xuân (klpod221)
// SPDX-License-Identifier: GPL-3.0-or-later

use std::path::Path;
use std::sync::Arc;

use chrono::{DateTime, Utc};
use russh_sftp::client::error::Error as RusshSftpError;
use russh_sftp::client::RawSftpSession;
use russh_sftp::protocol::{FileAttributes, StatusCode};

use crate::models::sftp::file_entry::{FileEntry, FileType};
use crate::services::sftp::errors::SftpError;

/// Helper to convert Unix timestamp to chrono DateTime<Utc>
pub fn timestamp_to_datetime(ts: Option<u32>) -> DateTime<Utc> {
    ts.and_then(|t| DateTime::<Utc>::from_timestamp(t as i64, 0))
        .unwrap_or_else(Utc::now)
}

/// Helper to join remote paths with forward slash
pub fn join_remote_path(dir: &str, name: &str) -> String {
    if dir.is_empty() || dir == "/" {
        format!("/{}", name.trim_start_matches('/'))
    } else if dir.ends_with('/') {
        format!("{}{}", dir, name.trim_start_matches('/'))
    } else {
        format!("{}/{}", dir, name.trim_start_matches('/'))
    }
}

/// List directory contents without holding application locks
pub async fn list_directory(
    sftp: &Arc<RawSftpSession>,
    path: &str,
) -> Result<Vec<FileEntry>, SftpError> {
    let handle = sftp
        .opendir(path)
        .await
        .map_err(|e| SftpError::from_russh_sftp(e, Some(path)))?
        .handle;

    let mut entries = Vec::new();

    loop {
        match sftp.readdir(&handle).await {
            Ok(name) => {
                for file in name.files {
                    if file.filename == "." || file.filename == ".." {
                        continue;
                    }

                    let full_path = join_remote_path(path, &file.filename);
                    let attrs = file.attrs;

                    let file_type = if attrs.is_dir() {
                        FileType::Directory
                    } else if attrs.is_symlink() {
                        FileType::Symlink
                    } else if attrs.is_regular() {
                        FileType::File
                    } else {
                        FileType::Unknown
                    };

                    let symlink_target = if matches!(file_type, FileType::Symlink) {
                        match sftp.readlink(&full_path).await {
                            Ok(n) => n.files.first().map(|f| f.filename.clone()),
                            Err(_) => None,
                        }
                    } else {
                        None
                    };

                    entries.push(FileEntry {
                        name: file.filename,
                        path: full_path,
                        file_type,
                        size: attrs.size,
                        permissions: attrs.permissions.unwrap_or(0o644),
                        modified: timestamp_to_datetime(attrs.mtime),
                        accessed: attrs.atime.and_then(|t| DateTime::<Utc>::from_timestamp(t as i64, 0)),
                        symlink_target,
                        uid: attrs.uid,
                        gid: attrs.gid,
                    });
                }
            }
            Err(RusshSftpError::Status(status)) if status.status_code == StatusCode::Eof => break,
            Err(e) => {
                let _ = sftp.close(&handle).await;
                return Err(SftpError::from_russh_sftp(e, Some(path)));
            }
        }
    }

    let _ = sftp.close(handle).await;
    Ok(entries)
}

/// Stat a file or directory path
pub async fn stat_path(sftp: &Arc<RawSftpSession>, path: &str) -> Result<FileEntry, SftpError> {
    let attrs = sftp
        .stat(path)
        .await
        .map_err(|e| SftpError::from_russh_sftp(e, Some(path)))?
        .attrs;

    let file_type = if attrs.is_dir() {
        FileType::Directory
    } else if attrs.is_symlink() {
        FileType::Symlink
    } else if attrs.is_regular() {
        FileType::File
    } else {
        FileType::Unknown
    };

    let name = Path::new(path)
        .file_name()
        .and_then(|n| n.to_str())
        .unwrap_or(path)
        .to_string();

    let symlink_target = if file_type == FileType::Symlink {
        match sftp.readlink(path).await {
            Ok(n) => n.files.first().map(|f| f.filename.clone()),
            Err(_) => None,
        }
    } else {
        None
    };

    Ok(FileEntry {
        name,
        path: path.to_string(),
        file_type,
        size: attrs.size,
        permissions: attrs.permissions.unwrap_or(0o644),
        modified: timestamp_to_datetime(attrs.mtime),
        accessed: attrs.atime.and_then(|t| DateTime::<Utc>::from_timestamp(t as i64, 0)),
        symlink_target,
        uid: attrs.uid,
        gid: attrs.gid,
    })
}

/// Create a directory on remote server
pub async fn create_directory(sftp: &Arc<RawSftpSession>, path: &str) -> Result<(), SftpError> {
    sftp.mkdir(path, FileAttributes::empty())
        .await
        .map_err(|e| SftpError::from_russh_sftp(e, Some(path)))?;
    Ok(())
}

/// Rename / move a remote path
pub async fn rename_path(
    sftp: &Arc<RawSftpSession>,
    old_path: &str,
    new_path: &str,
) -> Result<(), SftpError> {
    sftp.rename(old_path, new_path)
        .await
        .map_err(|e| SftpError::from_russh_sftp(e, Some(old_path)))?;
    Ok(())
}

/// Delete a file or directory. If `recursive` is true, deletes all descendants using
/// an iterative post-order traversal to prevent stack overflow.
pub async fn delete_path(
    sftp: &Arc<RawSftpSession>,
    path: &str,
    recursive: bool,
) -> Result<(), SftpError> {
    let attrs = sftp
        .stat(path)
        .await
        .map_err(|e| SftpError::from_russh_sftp(e, Some(path)))?
        .attrs;

    let is_dir = attrs.is_dir();

    if !is_dir {
        sftp.remove(path)
            .await
            .map_err(|e| SftpError::from_russh_sftp(e, Some(path)))?;
        return Ok(());
    }

    if !recursive {
        sftp.rmdir(path)
            .await
            .map_err(|e| SftpError::from_russh_sftp(e, Some(path)))?;
        return Ok(());
    }

    // Iterative post-order deletion of directory tree
    let mut stack = vec![(path.to_string(), false)];

    while let Some((curr_path, visited)) = stack.pop() {
        if visited {
            sftp.rmdir(&curr_path)
                .await
                .map_err(|e| SftpError::from_russh_sftp(e, Some(&curr_path)))?;
            continue;
        }

        let handle = sftp
            .opendir(&curr_path)
            .await
            .map_err(|e| SftpError::from_russh_sftp(e, Some(&curr_path)))?
            .handle;

        stack.push((curr_path.clone(), true));

        loop {
            match sftp.readdir(&handle).await {
                Ok(name) => {
                    for file in name.files {
                        if file.filename == "." || file.filename == ".." {
                            continue;
                        }

                        let child_path = join_remote_path(&curr_path, &file.filename);
                        if file.attrs.is_dir() {
                            stack.push((child_path, false));
                        } else {
                            sftp.remove(&child_path)
                                .await
                                .map_err(|e| SftpError::from_russh_sftp(e, Some(&child_path)))?;
                        }
                    }
                }
                Err(RusshSftpError::Status(status)) if status.status_code == StatusCode::Eof => break,
                Err(e) => {
                    let _ = sftp.close(&handle).await;
                    return Err(SftpError::from_russh_sftp(e, Some(&curr_path)));
                }
            }
        }

        let _ = sftp.close(&handle).await;
    }

    Ok(())
}

/// Set file permissions (chmod)
pub async fn set_permissions(
    sftp: &Arc<RawSftpSession>,
    path: &str,
    mode: u32,
) -> Result<(), SftpError> {
    let mut attrs = FileAttributes::empty();
    attrs.permissions = Some(mode);

    sftp.setstat(path, attrs)
        .await
        .map_err(|e| SftpError::from_russh_sftp(e, Some(path)))?;
    Ok(())
}

/// Create a symbolic link
pub async fn create_symlink(
    sftp: &Arc<RawSftpSession>,
    path: &str,
    target: &str,
) -> Result<(), SftpError> {
    sftp.symlink(path, target)
        .await
        .map_err(|e| SftpError::from_russh_sftp(e, Some(path)))?;
    Ok(())
}

/// Read symbolic link target
pub async fn read_symlink(sftp: &Arc<RawSftpSession>, path: &str) -> Result<String, SftpError> {
    let name = sftp
        .readlink(path)
        .await
        .map_err(|e| SftpError::from_russh_sftp(e, Some(path)))?;
    Ok(name.files.first().map(|f| f.filename.clone()).unwrap_or_default())
}

/// Resolve remote user's home directory via SSH_FXP_REALPATH (".")
#[allow(dead_code)]
pub async fn resolve_home_dir(sftp: &Arc<RawSftpSession>) -> Result<String, SftpError> {
    let name = sftp
        .realpath(".")
        .await
        .map_err(|e| SftpError::from_russh_sftp(e, Some(".")))?;
    Ok(name.files.first().map(|f| f.filename.clone()).unwrap_or_else(|| "/".to_string()))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_join_remote_path() {
        assert_eq!(join_remote_path("/", "foo"), "/foo");
        assert_eq!(join_remote_path("/home/user", "test.txt"), "/home/user/test.txt");
        assert_eq!(join_remote_path("/home/user/", "test.txt"), "/home/user/test.txt");
        assert_eq!(join_remote_path("/var/log", "/nginx/access.log"), "/var/log/nginx/access.log");
    }

    #[test]
    fn test_timestamp_to_datetime() {
        let dt = timestamp_to_datetime(Some(1700000000));
        assert_eq!(dt.timestamp(), 1700000000);
    }
}
