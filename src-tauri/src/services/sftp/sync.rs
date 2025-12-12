use std::collections::HashMap;
use std::path::Path;
use std::sync::Arc;

use crate::models::sftp::{
    file_entry::FileEntry,
    sync::{DiffEntry, DiffType, SyncDirection, SyncOperation},
};
use crate::models::sync::SyncProgressEvent;
use crate::services::sftp::service::SFTPService;

use anyhow::Result;
use chrono;
use tauri::Emitter;
use tokio::fs;
use tokio::sync::RwLock;

/// Sync Service for comparing and synchronizing directories
pub struct SyncService {
    sftp_service: Arc<SFTPService>,
    app_handle: Arc<RwLock<Option<tauri::AppHandle>>>,
}

impl SyncService {
    /// Create new sync service
    pub fn new(sftp_service: Arc<SFTPService>) -> Self {
        Self {
            sftp_service,
            app_handle: Arc::new(RwLock::new(None)),
        }
    }

    /// Set app handle for emitting events
    #[allow(dead_code)]
    pub async fn set_app_handle(&self, app_handle: tauri::AppHandle) {
        let mut handle = self.app_handle.write().await;
        *handle = Some(app_handle);
    }

    /// Emit progress event
    async fn emit_progress(&self, event: SyncProgressEvent) {
        if let Some(ref app_handle) = *self.app_handle.read().await {
            let _ = app_handle.emit("sync_progress", event);
        }
    }

    /// Compare local and remote directories
    pub async fn compare_directories(
        &self,
        session_id: String,
        local_path: String,
        remote_path: String,
        clock_skew_seconds: Option<i64>,
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

                // Check modification time with configurable clock skew tolerance
                let skew_tolerance = clock_skew_seconds.unwrap_or(1);
                let time_diff = (local_entry.modified - remote_entry.modified)
                    .num_seconds()
                    .abs();
                if time_diff > skew_tolerance {
                    // Allow configured difference for clock skew
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
            SyncDirection::LocalToRemote => self.sync_local_to_remote(session_id, operation).await,
            SyncDirection::RemoteToLocal => self.sync_remote_to_local(session_id, operation).await,
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

                let accessed = metadata
                    .accessed()
                    .ok()
                    .map(|t| chrono::DateTime::<chrono::Utc>::from(t));

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
        self.emit_progress(SyncProgressEvent::sftp_progress("comparing", "", 0, 0))
            .await;

        let diffs = self
            .compare_directories(
                session_id.clone(),
                operation.local_path.clone(),
                operation.remote_path.clone(),
                operation.clock_skew_seconds,
            )
            .await?;

        // Filter diffs to get files that need upload
        let upload_diffs: Vec<_> = diffs
            .iter()
            .filter(|diff| {
                if self.should_exclude(&diff.path, &operation.exclude_patterns) {
                    return false;
                }
                matches!(
                    diff.diff_type,
                    DiffType::OnlyLocal | DiffType::SizeDiffers | DiffType::TimeDiffers
                )
            })
            .collect();

        let total = upload_diffs.len() as u32;
        let mut processed = 0u32;

        // For each diff that needs to be synced, upload files
        for diff in upload_diffs {
            // Skip files exceeding max size
            if let Some(max_size) = operation.max_file_size {
                if let Some(ref entry) = diff.local_entry {
                    if entry.size.unwrap_or(0) > max_size {
                        eprintln!("[SFTP Sync] Skipping large file: {}", diff.path);
                        continue;
                    }
                }
            }

            let local_path = Path::new(&operation.local_path).join(&diff.path);
            let remote_path = format!("{}/{}", operation.remote_path, diff.path);

            // Skip directories (handled separately) and symlinks if not preserved
            if local_path.is_dir() {
                continue;
            }
            if local_path.is_symlink() && !operation.preserve_symlinks {
                continue;
            }

            // Emit progress before upload
            self.emit_progress(SyncProgressEvent::sftp_progress(
                "uploading",
                &diff.path,
                processed,
                total,
            ))
            .await;

            // Upload file
            if local_path.exists() && local_path.is_file() {
                match self
                    .sftp_service
                    .upload_file_bytes(
                        session_id.clone(),
                        local_path.to_string_lossy().to_string(),
                        remote_path.clone(),
                    )
                    .await
                {
                    Ok(_) => {
                        eprintln!("[SFTP Sync] Uploaded: {}", diff.path);
                        processed += 1;
                    }
                    Err(e) => {
                        eprintln!("[SFTP Sync] Failed to upload {}: {}", diff.path, e);
                        self.emit_progress(SyncProgressEvent::sftp_error(&e.to_string()))
                            .await;
                    }
                }
            }
        }

        self.emit_progress(SyncProgressEvent::sftp_completed(processed))
            .await;
        Ok(())
    }

    /// Sync from remote to local
    async fn sync_remote_to_local(
        &self,
        session_id: String,
        operation: SyncOperation,
    ) -> Result<(), anyhow::Error> {
        // Compare directories first
        self.emit_progress(SyncProgressEvent::sftp_progress("comparing", "", 0, 0))
            .await;

        let diffs = self
            .compare_directories(
                session_id.clone(),
                operation.local_path.clone(),
                operation.remote_path.clone(),
                operation.clock_skew_seconds,
            )
            .await?;

        // Filter diffs to get files that need download
        let download_diffs: Vec<_> = diffs
            .iter()
            .filter(|diff| {
                if self.should_exclude(&diff.path, &operation.exclude_patterns) {
                    return false;
                }
                matches!(
                    diff.diff_type,
                    DiffType::OnlyRemote | DiffType::SizeDiffers | DiffType::TimeDiffers
                )
            })
            .collect();

        let total = download_diffs.len() as u32;
        let mut processed = 0u32;

        // For each diff that needs to be synced, download files
        for diff in download_diffs {
            // Skip files exceeding max size
            if let Some(max_size) = operation.max_file_size {
                if let Some(ref entry) = diff.remote_entry {
                    if entry.size.unwrap_or(0) > max_size {
                        eprintln!("[SFTP Sync] Skipping large file: {}", diff.path);
                        continue;
                    }
                }
            }

            let remote_path = format!("{}/{}", operation.remote_path, diff.path);
            let local_path = Path::new(&operation.local_path).join(&diff.path);

            // Skip directories
            if let Some(ref entry) = diff.remote_entry {
                if entry.is_directory() {
                    // Create local directory
                    let _ = fs::create_dir_all(&local_path).await;
                    continue;
                }
            }

            // Emit progress before download
            self.emit_progress(SyncProgressEvent::sftp_progress(
                "downloading",
                &diff.path,
                processed,
                total,
            ))
            .await;

            // Download file
            match self
                .sftp_service
                .download_file_bytes(
                    session_id.clone(),
                    remote_path.clone(),
                    local_path.to_string_lossy().to_string(),
                )
                .await
            {
                Ok(_) => {
                    eprintln!("[SFTP Sync] Downloaded: {}", diff.path);
                    processed += 1;
                }
                Err(e) => {
                    eprintln!("[SFTP Sync] Failed to download {}: {}", diff.path, e);
                    self.emit_progress(SyncProgressEvent::sftp_error(&e.to_string()))
                        .await;
                }
            }
        }

        self.emit_progress(SyncProgressEvent::sftp_completed(processed))
            .await;
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
            .compare_directories(
                session_id.clone(),
                operation.local_path.clone(),
                operation.remote_path.clone(),
                operation.clock_skew_seconds,
            )
            .await?;

        // Sync in both directions
        // Files only in local -> upload
        // Files only in remote -> download
        // Different files -> use newer version (last write wins)
        for diff in diffs {
            // Skip files matching exclude patterns
            if self.should_exclude(&diff.path, &operation.exclude_patterns) {
                continue;
            }

            match diff.diff_type {
                DiffType::OnlyLocal => {
                    // Upload
                    let local_path = Path::new(&operation.local_path).join(&diff.path);
                    let remote_path = format!("{}/{}", operation.remote_path, diff.path);

                    if local_path.exists() && local_path.is_file() {
                        match self
                            .sftp_service
                            .upload_file_bytes(
                                session_id.clone(),
                                local_path.to_string_lossy().to_string(),
                                remote_path.clone(),
                            )
                            .await
                        {
                            Ok(_) => {
                                eprintln!("[SFTP Sync] Uploaded: {}", diff.path);
                            }
                            Err(e) => {
                                eprintln!("[SFTP Sync] Failed to upload {}: {}", diff.path, e);
                            }
                        }
                    }
                }
                DiffType::OnlyRemote => {
                    // Download
                    let remote_path = format!("{}/{}", operation.remote_path, diff.path);
                    let local_path = Path::new(&operation.local_path).join(&diff.path);

                    // Skip directories
                    if let Some(ref entry) = diff.remote_entry {
                        if entry.is_directory() {
                            let _ = fs::create_dir_all(&local_path).await;
                            continue;
                        }
                    }

                    match self
                        .sftp_service
                        .download_file_bytes(
                            session_id.clone(),
                            remote_path.clone(),
                            local_path.to_string_lossy().to_string(),
                        )
                        .await
                    {
                        Ok(_) => {
                            eprintln!("[SFTP Sync] Downloaded: {}", diff.path);
                        }
                        Err(e) => {
                            eprintln!("[SFTP Sync] Failed to download {}: {}", diff.path, e);
                        }
                    }
                }
                DiffType::SizeDiffers | DiffType::TimeDiffers => {
                    // Conflict resolution: use newer version based on modification time
                    let local_path = Path::new(&operation.local_path).join(&diff.path);
                    let remote_path = format!("{}/{}", operation.remote_path, diff.path);

                    let local_time = diff.local_entry.as_ref().map(|e| e.modified);
                    let remote_time = diff.remote_entry.as_ref().map(|e| e.modified);

                    match (local_time, remote_time) {
                        (Some(local_modified), Some(remote_modified)) => {
                            if local_modified > remote_modified {
                                // Local is newer, upload
                                if local_path.exists() && local_path.is_file() {
                                    match self
                                        .sftp_service
                                        .upload_file_bytes(
                                            session_id.clone(),
                                            local_path.to_string_lossy().to_string(),
                                            remote_path.clone(),
                                        )
                                        .await
                                    {
                                        Ok(_) => {
                                            eprintln!(
                                                "[SFTP Sync] Conflict resolved (local newer): uploaded {}",
                                                diff.path
                                            );
                                        }
                                        Err(e) => {
                                            eprintln!(
                                                "[SFTP Sync] Failed to upload {}: {}",
                                                diff.path, e
                                            );
                                        }
                                    }
                                }
                            } else {
                                // Remote is newer, download
                                match self
                                    .sftp_service
                                    .download_file_bytes(
                                        session_id.clone(),
                                        remote_path.clone(),
                                        local_path.to_string_lossy().to_string(),
                                    )
                                    .await
                                {
                                    Ok(_) => {
                                        eprintln!(
                                            "[SFTP Sync] Conflict resolved (remote newer): downloaded {}",
                                            diff.path
                                        );
                                    }
                                    Err(e) => {
                                        eprintln!(
                                            "[SFTP Sync] Failed to download {}: {}",
                                            diff.path, e
                                        );
                                    }
                                }
                            }
                        }
                        _ => {
                            // Cannot determine which is newer, log conflict
                            eprintln!(
                                "[SFTP Sync] Conflict: cannot determine newer version for {}",
                                diff.path
                            );
                        }
                    }
                }
                _ => {}
            }
        }

        Ok(())
    }

    /// Check if path matches any exclude patterns
    fn should_exclude(&self, path: &str, patterns: &[String]) -> bool {
        for pattern in patterns {
            // Simple glob matching - support * and **
            if pattern.contains("**") {
                // Match any path containing the pattern part
                let parts: Vec<&str> = pattern.split("**").collect();
                if parts.len() == 2 {
                    let (prefix, suffix) = (parts[0], parts[1]);
                    if (prefix.is_empty() || path.starts_with(prefix))
                        && (suffix.is_empty() || path.ends_with(suffix))
                    {
                        return true;
                    }
                }
            } else if pattern.contains('*') {
                // Simple wildcard
                let parts: Vec<&str> = pattern.split('*').collect();
                if parts.len() == 2 {
                    let (prefix, suffix) = (parts[0], parts[1]);
                    if path.starts_with(prefix) && path.ends_with(suffix) {
                        return true;
                    }
                }
            } else if path.contains(pattern) {
                // Exact substring match
                return true;
            }
        }
        false
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
