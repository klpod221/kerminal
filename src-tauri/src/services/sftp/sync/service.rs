// Copyright (C) 2026 Bùi Thanh Xuân (klpod221)
// SPDX-License-Identifier: GPL-3.0-or-later

use std::path::Path;
use std::sync::Arc;
use anyhow::Result;
use log::{error, info, warn};
use tauri::Emitter;
use tokio::fs;
use tokio::sync::RwLock;

use crate::models::sftp::sync::{DiffEntry, DiffType, SyncDirection, SyncOperation};
use crate::models::sync::SyncProgressEvent;
use crate::services::sftp::service::SFTPService;
use crate::services::sftp::sync::compare::{build_local_tree, build_remote_tree, compare_trees};

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
        let local_files = build_local_tree(&local_path).await?;
        let remote_files = build_remote_tree(&self.sftp_service, &session_id, &remote_path).await?;

        Ok(compare_trees(
            &local_path,
            &remote_path,
            &local_files,
            &remote_files,
            clock_skew_seconds,
        ))
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
            SyncDirection::Bidirectional => self.sync_bidirectional(session_id, operation).await,
        }
    }

    /// Sync from local to remote
    async fn sync_local_to_remote(
        &self,
        session_id: String,
        operation: SyncOperation,
    ) -> Result<(), anyhow::Error> {
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

        for diff in upload_diffs {
            if let Some(max_size) = operation.max_file_size {
                if let Some(ref entry) = diff.local_entry {
                    if entry.size.unwrap_or(0) > max_size {
                        warn!("[SFTP Sync] Skipping large file: {}", diff.path);
                        continue;
                    }
                }
            }

            let local_path = Path::new(&operation.local_path).join(&diff.path);
            let remote_path = format!("{}/{}", operation.remote_path, diff.path);

            if local_path.is_dir() {
                continue;
            }
            if local_path.is_symlink() && !operation.preserve_symlinks {
                continue;
            }

            self.emit_progress(SyncProgressEvent::sftp_progress(
                "uploading",
                &diff.path,
                processed,
                total,
            ))
            .await;

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
                        info!("[SFTP Sync] Uploaded: {}", diff.path);
                        processed += 1;
                    }
                    Err(e) => {
                        error!("[SFTP Sync] Failed to upload {}: {}", diff.path, e);
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

        for diff in download_diffs {
            if let Some(max_size) = operation.max_file_size {
                if let Some(ref entry) = diff.remote_entry {
                    if entry.size.unwrap_or(0) > max_size {
                        warn!("[SFTP Sync] Skipping large file: {}", diff.path);
                        continue;
                    }
                }
            }

            let remote_path = format!("{}/{}", operation.remote_path, diff.path);
            let local_path = Path::new(&operation.local_path).join(&diff.path);

            if let Some(ref entry) = diff.remote_entry {
                if entry.is_directory() {
                    let _ = fs::create_dir_all(&local_path).await;
                    continue;
                }
            }

            self.emit_progress(SyncProgressEvent::sftp_progress(
                "downloading",
                &diff.path,
                processed,
                total,
            ))
            .await;

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
                    info!("[SFTP Sync] Downloaded: {}", diff.path);
                    processed += 1;
                }
                Err(e) => {
                    error!("[SFTP Sync] Failed to download {}: {}", diff.path, e);
                    self.emit_progress(SyncProgressEvent::sftp_error(&e.to_string()))
                        .await;
                }
            }
        }

        self.emit_progress(SyncProgressEvent::sftp_completed(processed))
            .await;
        Ok(())
    }

    /// Bidirectional sync with conflict resolution
    async fn sync_bidirectional(
        &self,
        session_id: String,
        operation: SyncOperation,
    ) -> Result<(), anyhow::Error> {
        let diffs = self
            .compare_directories(
                session_id.clone(),
                operation.local_path.clone(),
                operation.remote_path.clone(),
                operation.clock_skew_seconds,
            )
            .await?;

        for diff in diffs {
            if self.should_exclude(&diff.path, &operation.exclude_patterns) {
                continue;
            }

            match diff.diff_type {
                DiffType::OnlyLocal => {
                    let local_path = Path::new(&operation.local_path).join(&diff.path);
                    let remote_path = format!("{}/{}", operation.remote_path, diff.path);

                    if local_path.exists() && local_path.is_file() {
                        let _ = self
                            .sftp_service
                            .upload_file_bytes(
                                session_id.clone(),
                                local_path.to_string_lossy().to_string(),
                                remote_path,
                            )
                            .await;
                    }
                }
                DiffType::OnlyRemote => {
                    let remote_path = format!("{}/{}", operation.remote_path, diff.path);
                    let local_path = Path::new(&operation.local_path).join(&diff.path);

                    if let Some(ref entry) = diff.remote_entry {
                        if entry.is_directory() {
                            let _ = fs::create_dir_all(&local_path).await;
                            continue;
                        }
                    }

                    let _ = self
                        .sftp_service
                        .download_file_bytes(
                            session_id.clone(),
                            remote_path,
                            local_path.to_string_lossy().to_string(),
                        )
                        .await;
                }
                DiffType::SizeDiffers | DiffType::TimeDiffers => {
                    let local_path = Path::new(&operation.local_path).join(&diff.path);
                    let remote_path = format!("{}/{}", operation.remote_path, diff.path);

                    let local_time = diff.local_entry.as_ref().map(|e| e.modified);
                    let remote_time = diff.remote_entry.as_ref().map(|e| e.modified);

                    match (local_time, remote_time) {
                        (Some(local_modified), Some(remote_modified)) => {
                            if local_modified > remote_modified {
                                if local_path.exists() && local_path.is_file() {
                                    let _ = self
                                        .sftp_service
                                        .upload_file_bytes(
                                            session_id.clone(),
                                            local_path.to_string_lossy().to_string(),
                                            remote_path,
                                        )
                                        .await;
                                }
                            } else {
                                let _ = self
                                    .sftp_service
                                    .download_file_bytes(
                                        session_id.clone(),
                                        remote_path,
                                        local_path.to_string_lossy().to_string(),
                                    )
                                    .await;
                            }
                        }
                        _ => {
                            error!(
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
            if pattern.contains("**") {
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
                let parts: Vec<&str> = pattern.split('*').collect();
                if parts.len() == 2 {
                    let (prefix, suffix) = (parts[0], parts[1]);
                    if path.starts_with(prefix) && path.ends_with(suffix) {
                        return true;
                    }
                }
            } else if path.contains(pattern) {
                return true;
            }
        }
        false
    }
}
