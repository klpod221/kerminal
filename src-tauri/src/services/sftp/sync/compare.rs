// Copyright (C) 2026 Bùi Thanh Xuân (klpod221)
// SPDX-License-Identifier: GPL-3.0-or-later

use std::collections::HashMap;
use std::path::Path;
use std::sync::Arc;
use anyhow::Result;
use tokio::fs;

use crate::models::sftp::{
    file_entry::{FileEntry, FileType},
    sync::{DiffEntry, DiffType},
};
use crate::services::sftp::service::SFTPService;

/// Helper to get relative path from base
pub fn relative_path(base: &str, full: &str) -> String {
    if let Ok(rel) = Path::new(full).strip_prefix(base) {
        rel.to_str().unwrap_or(full).to_string()
    } else {
        full.to_string()
    }
}

/// Build local file tree recursively without recursion stack overflow
pub async fn build_local_tree(base_path: &str) -> Result<HashMap<String, FileEntry>> {
    let mut tree = HashMap::new();
    let mut stack = vec![base_path.to_string()];

    while let Some(current_path) = stack.pop() {
        let mut entries = fs::read_dir(&current_path).await?;

        while let Some(entry) = entries.next_entry().await? {
            let path = entry.path();
            let path_str = path.to_str().unwrap_or("").to_string();

            let metadata = entry.metadata().await?;
            let is_dir = metadata.is_dir();
            let is_symlink = metadata.is_symlink();
            let file_type = if is_dir {
                FileType::Directory
            } else if is_symlink {
                FileType::Symlink
            } else {
                FileType::File
            };

            let name = path
                .file_name()
                .and_then(|n| n.to_str())
                .unwrap_or("")
                .to_string();

            let permissions = {
                #[cfg(unix)]
                {
                    use std::os::unix::fs::PermissionsExt;
                    metadata.permissions().mode() as u32 & 0o777
                }
                #[cfg(not(unix))]
                {
                    0o644
                }
            };

            let modified = chrono::DateTime::<chrono::Utc>::from(metadata.modified()?);
            let accessed = metadata
                .accessed()
                .ok()
                .map(chrono::DateTime::<chrono::Utc>::from);

            let size = if matches!(file_type, FileType::File) {
                Some(metadata.len())
            } else {
                None
            };

            let symlink_target = if is_symlink {
                std::fs::read_link(&path)
                    .ok()
                    .and_then(|p| p.to_str().map(|s| s.to_string()))
            } else {
                None
            };

            tree.insert(
                path_str.clone(),
                FileEntry {
                    name,
                    path: path_str.clone(),
                    file_type,
                    size,
                    permissions,
                    modified,
                    accessed,
                    symlink_target,
                    uid: None,
                    gid: None,
                },
            );

            if is_dir {
                stack.push(path_str);
            }
        }
    }

    Ok(tree)
}

/// Build remote file tree recursively using iterative stack
pub async fn build_remote_tree(
    sftp_service: &Arc<SFTPService>,
    session_id: &str,
    base_path: &str,
) -> Result<HashMap<String, FileEntry>> {
    let mut tree = HashMap::new();
    let mut stack = vec![base_path.to_string()];

    while let Some(path) = stack.pop() {
        let entries = sftp_service
            .list_directory(session_id.to_string(), path.clone())
            .await
            .map_err(|e| anyhow::anyhow!("Failed to list remote directory: {}", e))?;

        for entry in entries {
            let path_str = entry.path.clone();
            tree.insert(path_str.clone(), entry.clone());

            if entry.is_directory() {
                stack.push(path_str);
            }
        }
    }

    Ok(tree)
}

/// Compare local and remote file trees and generate difference entries
pub fn compare_trees(
    local_base: &str,
    remote_base: &str,
    local_files: &HashMap<String, FileEntry>,
    remote_files: &HashMap<String, FileEntry>,
    clock_skew_seconds: Option<i64>,
) -> Vec<DiffEntry> {
    let mut diffs = Vec::new();

    // Files only in local
    for (path, local_entry) in local_files {
        let rel_path = relative_path(local_base, path);
        if !remote_files.contains_key(path) {
            diffs.push(DiffEntry {
                path: rel_path,
                diff_type: DiffType::OnlyLocal,
                local_entry: Some(local_entry.clone()),
                remote_entry: None,
            });
        }
    }

    // Files only in remote
    for (path, remote_entry) in remote_files {
        let rel_path = relative_path(remote_base, path);
        if !local_files.contains_key(path) {
            diffs.push(DiffEntry {
                path: rel_path,
                diff_type: DiffType::OnlyRemote,
                local_entry: None,
                remote_entry: Some(remote_entry.clone()),
            });
        }
    }

    // Compare common files
    for (path, local_entry) in local_files {
        if let Some(remote_entry) = remote_files.get(path) {
            let rel_path = relative_path(local_base, path);

            if local_entry.size != remote_entry.size {
                diffs.push(DiffEntry {
                    path: rel_path,
                    diff_type: DiffType::SizeDiffers,
                    local_entry: Some(local_entry.clone()),
                    remote_entry: Some(remote_entry.clone()),
                });
                continue;
            }

            let skew_tolerance = clock_skew_seconds.unwrap_or(1);
            let time_diff = (local_entry.modified - remote_entry.modified)
                .num_seconds()
                .abs();

            if time_diff > skew_tolerance {
                diffs.push(DiffEntry {
                    path: rel_path,
                    diff_type: DiffType::TimeDiffers,
                    local_entry: Some(local_entry.clone()),
                    remote_entry: Some(remote_entry.clone()),
                });
                continue;
            }

            if local_entry.permissions != remote_entry.permissions {
                diffs.push(DiffEntry {
                    path: rel_path,
                    diff_type: DiffType::PermissionsDiffer,
                    local_entry: Some(local_entry.clone()),
                    remote_entry: Some(remote_entry.clone()),
                });
                continue;
            }

            diffs.push(DiffEntry {
                path: rel_path,
                diff_type: DiffType::Identical,
                local_entry: Some(local_entry.clone()),
                remote_entry: Some(remote_entry.clone()),
            });
        }
    }

    diffs
}
