use std::collections::HashMap;
use std::path::Path;
use std::sync::Arc;

use crate::models::sftp::{
    sync::{DiffEntry, DiffType, SyncDirection, SyncOperation},
    file_entry::FileEntry,
};
use crate::services::sftp::service::SFTPService;

use anyhow::Result;
use chrono;
use tokio::fs;

/// Sync Service for comparing and synchronizing directories
pub struct SyncService {
    sftp_service: Arc<SFTPService>,
}

impl SyncService {
    /// Create new sync service
    pub fn new(sftp_service: Arc<SFTPService>) -> Self {
        Self { sftp_service }
    }

    /// Compare local and remote directories
    pub async fn compare_directories(
        &self,
        session_id: String,
        local_path: String,
        remote_path: String,
    ) -> Result<Vec<DiffEntry>, anyhow::Error> {
        // Get local file tree
        let local_files = Self::build_local_tree(&local_path).await?;

        // Get remote file tree
        let remote_files = self
            .build_remote_tree(session_id.clone(), &remote_path)
            .await?;

        // Compare and generate diffs
        let mut diffs = Vec::new();

        // Find files only in local
        for (path, local_entry) in &local_files {
            let relative_path = Self::relative_path(&local_path, path);
            if !remote_files.contains_key(path) {
                diffs.push(DiffEntry {
                    path: relative_path,
                    diff_type: DiffType::OnlyLocal,
                    local_entry: Some(local_entry.clone()),
                    remote_entry: None,
                });
            }
        }

        // Find files only in remote
        for (path, remote_entry) in &remote_files {
            let relative_path = Self::relative_path(&remote_path, path);
            if !local_files.contains_key(path) {
                diffs.push(DiffEntry {
                    path: relative_path,
                    diff_type: DiffType::OnlyRemote,
                    local_entry: None,
                    remote_entry: Some(remote_entry.clone()),
                });
            }
        }

        // Find differences in files that exist in both
        for (path, local_entry) in &local_files {
            if let Some(remote_entry) = remote_files.get(path) {
                let relative_path = Self::relative_path(&local_path, path);

                // Check size
                if local_entry.size != remote_entry.size {
                    diffs.push(DiffEntry {
                        path: relative_path.clone(),
                        diff_type: DiffType::SizeDiffers,
                        local_entry: Some(local_entry.clone()),
                        remote_entry: Some(remote_entry.clone()),
                    });
                    continue;
                }

                // Check modification time
                let time_diff = (local_entry.modified - remote_entry.modified)
                    .num_seconds()
                    .abs();
                if time_diff > 1 {
                    // Allow 1 second difference for clock skew
                    diffs.push(DiffEntry {
                        path: relative_path.clone(),
                        diff_type: DiffType::TimeDiffers,
                        local_entry: Some(local_entry.clone()),
                        remote_entry: Some(remote_entry.clone()),
                    });
                    continue;
                }

                // Check permissions
                if local_entry.permissions != remote_entry.permissions {
                    diffs.push(DiffEntry {
                        path: relative_path.clone(),
                        diff_type: DiffType::PermissionsDiffer,
                        local_entry: Some(local_entry.clone()),
                        remote_entry: Some(remote_entry.clone()),
                    });
                    continue;
                }

                // Files appear identical
                diffs.push(DiffEntry {
                    path: relative_path,
                    diff_type: DiffType::Identical,
                    local_entry: Some(local_entry.clone()),
                    remote_entry: Some(remote_entry.clone()),
                });
            }
        }

        Ok(diffs)
    }

    /// Synchronize directories according to sync operation
    pub async fn sync_directories(
        &self,
        session_id: String,
        operation: SyncOperation,
    ) -> Result<(), anyhow::Error> {
        match operation.direction {
            SyncDirection::LocalToRemote => {
                self.sync_local_to_remote(session_id, operation).await
            }
            SyncDirection::RemoteToLocal => {
                self.sync_remote_to_local(session_id, operation).await
            }
            SyncDirection::Bidirectional => {
                // For bidirectional, sync in both directions with conflict resolution
                self.sync_bidirectional(session_id, operation).await
            }
        }
    }

    /// Build local file tree recursively
    async fn build_local_tree(base_path: &str) -> Result<HashMap<String, FileEntry>> {
        let mut tree = HashMap::new();
        // Use iterative approach with a stack to avoid recursion
        let mut stack = vec![base_path.to_string()];

        while let Some(current_path) = stack.pop() {
            let mut entries = fs::read_dir(&current_path).await?;

            while let Some(entry) = entries.next_entry().await? {
                let path = entry.path();
                let path_str = path.to_str().unwrap().to_string();

                let metadata = entry.metadata().await?;
                let is_dir = metadata.is_dir();
                let is_symlink = metadata.is_symlink();
                let file_type = if is_dir {
                    crate::models::sftp::file_entry::FileType::Directory
                } else if is_symlink {
                    crate::models::sftp::file_entry::FileType::Symlink
                } else {
                    crate::models::sftp::file_entry::FileType::File
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

                let accessed = metadata.accessed().ok().map(|t| chrono::DateTime::<chrono::Utc>::from(t));

                let size = if matches!(file_type, crate::models::sftp::file_entry::FileType::File) {
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

                // Add directories to stack for processing
                if is_dir {
                    stack.push(path_str);
                }
            }
        }

        Ok(tree)
    }

    /// Build remote file tree recursively
    async fn build_remote_tree(
        &self,
        session_id: String,
        base_path: &str,
    ) -> Result<HashMap<String, FileEntry>> {
        let mut tree = HashMap::new();
        self.build_remote_tree_recursive(session_id, base_path, base_path, &mut tree)
            .await?;
        Ok(tree)
    }

    /// Recursive helper for building remote tree (using iterative approach to avoid boxing)
    async fn build_remote_tree_recursive(
        &self,
        session_id: String,
        _base_path: &str,
        current_path: &str,
        tree: &mut HashMap<String, FileEntry>,
    ) -> Result<()> {
        // Use iterative approach with a stack to avoid recursion
        let mut stack = vec![current_path.to_string()];

        while let Some(path) = stack.pop() {
            let entries = self
                .sftp_service
                .list_directory(session_id.clone(), path.clone())
                .await
                .map_err(|e| anyhow::anyhow!("Failed to list remote directory: {}", e))?;

            for entry in entries {
                let path_str = entry.path.clone();
                tree.insert(path_str.clone(), entry.clone());

                // Add directories to stack for processing
                if entry.is_directory() {
                    stack.push(path_str);
                }
            }
        }

        Ok(())
    }

    /// Sync from local to remote
    async fn sync_local_to_remote(
        &self,
        session_id: String,
        operation: SyncOperation,
    ) -> Result<(), anyhow::Error> {
        // Compare directories first
        let diffs = self
            .compare_directories(session_id.clone(), operation.local_path.clone(), operation.remote_path.clone())
            .await?;

        // For each diff that needs to be synced, upload files
        for diff in diffs {
            if matches!(diff.diff_type, DiffType::OnlyLocal | DiffType::SizeDiffers | DiffType::TimeDiffers) {
                let _local_path = Path::new(&operation.local_path).join(&diff.path);
                let _remote_path = format!("{}/{}", operation.remote_path, diff.path);
                
                // Upload file (would use TransferManager in production)
                // For now, just verify file exists
                if _local_path.exists() && _local_path.is_file() {
                    // File would be uploaded here
                }
            }
        }

        Ok(())
    }

    /// Sync from remote to local
    async fn sync_remote_to_local(
        &self,
        session_id: String,
        operation: SyncOperation,
    ) -> Result<(), anyhow::Error> {
        // Compare directories first
        let diffs = self
            .compare_directories(session_id.clone(), operation.local_path.clone(), operation.remote_path.clone())
            .await?;

        // For each diff that needs to be synced, download files
        for diff in diffs {
            if matches!(diff.diff_type, DiffType::OnlyRemote | DiffType::SizeDiffers | DiffType::TimeDiffers) {
                let _remote_path = format!("{}/{}", operation.remote_path, diff.path);
                let local_path = Path::new(&operation.local_path).join(&diff.path);
                
                // Download file (would use TransferManager in production)
                // For now, just ensure parent directory exists
                if let Some(parent) = local_path.parent() {
                    let _ = fs::create_dir_all(parent).await;
                }
            }
        }

        Ok(())
    }

    /// Bidirectional sync
    async fn sync_bidirectional(
        &self,
        session_id: String,
        operation: SyncOperation,
    ) -> Result<(), anyhow::Error> {
        // Compare directories first
        let diffs = self
            .compare_directories(session_id.clone(), operation.local_path.clone(), operation.remote_path.clone())
            .await?;

        // Sync in both directions
        // Files only in local -> upload
        // Files only in remote -> download
        // Different files -> use newer version
        for diff in diffs {
            match diff.diff_type {
                DiffType::OnlyLocal => {
                    // Upload
                    let local_path = Path::new(&operation.local_path).join(&diff.path);
                    if local_path.exists() && local_path.is_file() {
                        // Upload would happen here
                    }
                }
                DiffType::OnlyRemote => {
                    // Download
                    let local_path = Path::new(&operation.local_path).join(&diff.path);
                    if let Some(parent) = local_path.parent() {
                        let _ = fs::create_dir_all(parent).await;
                    }
                }
                DiffType::SizeDiffers | DiffType::TimeDiffers => {
                    // Conflict - for now, skip. In production would ask user or use newer
                }
                _ => {}
            }
        }

        Ok(())
    }

    /// Get relative path from base
    fn relative_path(base: &str, full: &str) -> String {
        if let Ok(rel) = Path::new(full).strip_prefix(base) {
            rel.to_str().unwrap_or(full).to_string()
        } else {
            full.to_string()
        }
    }
}

