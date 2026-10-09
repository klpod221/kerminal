// Copyright (C) 2026 Bùi Thanh Xuân (klpod221)
// SPDX-License-Identifier: GPL-3.0-or-later

use std::collections::HashMap;
use std::sync::Arc;
use chrono::Utc;
use tauri::Emitter;
use tokio::sync::RwLock;
use tokio_util::sync::CancellationToken;
use uuid::Uuid;

use crate::models::sftp::{
    error::SFTPError,
    transfer::{TransferDirection, TransferProgress, TransferStatus},
};
use crate::services::sftp::queue::checkpoint::CheckpointStore;
use crate::services::sftp::queue::task::TransferMetadata;
use crate::services::sftp::service::SFTPService;
use crate::services::sftp::transfer_io::{
    download_file_pipelined, upload_file_pipelined, DownloadOptions, UploadOptions,
};

/// Default maximum concurrent transfers
pub const DEFAULT_MAX_CONCURRENT: usize = 3;

/// Transfer Manager for handling file transfers with pipelined streaming and queue processing
#[derive(Clone)]
pub struct TransferManager {
    active_transfers: Arc<RwLock<HashMap<String, TransferProgress>>>,
    metadata: Arc<RwLock<HashMap<String, TransferMetadata>>>,
    cancellation_tokens: Arc<RwLock<HashMap<String, CancellationToken>>>,
    checkpoints: Arc<CheckpointStore>,
    sftp_service: std::sync::Weak<SFTPService>,
    max_concurrent: usize,
}

impl TransferManager {
    /// Create new transfer manager
    pub fn new(sftp_service: Arc<SFTPService>) -> Self {
        Self {
            active_transfers: Arc::new(RwLock::new(HashMap::new())),
            metadata: Arc::new(RwLock::new(HashMap::new())),
            cancellation_tokens: Arc::new(RwLock::new(HashMap::new())),
            checkpoints: Arc::new(CheckpointStore::new()),
            sftp_service: Arc::downgrade(&sftp_service),
            max_concurrent: DEFAULT_MAX_CONCURRENT,
        }
    }

    /// Start processing the transfer queue in a background loop
    pub fn start_queue_processor(&self, app_handle: tauri::AppHandle) {
        let manager = self.clone();
        tokio::spawn(async move {
            loop {
                manager.process_queue(app_handle.clone()).await;
                tokio::time::sleep(tokio::time::Duration::from_millis(500)).await;
            }
        });
    }

    /// Upload file to remote server (queues transfer)
    pub async fn upload_file(
        &self,
        session_id: String,
        local_path: String,
        remote_path: String,
        app_handle: tauri::AppHandle,
    ) -> Result<String, SFTPError> {
        let local_meta = tokio::fs::metadata(&local_path)
            .await
            .map_err(|e| SFTPError::IoError {
                message: format!("Failed to stat local file {}: {}", local_path, e),
            })?;

        let transfer_id = Uuid::new_v4().to_string();
        let progress = TransferProgress {
            transfer_id: transfer_id.clone(),
            status: TransferStatus::Queued,
            direction: TransferDirection::Upload,
            local_path: local_path.clone(),
            remote_path: remote_path.clone(),
            total_bytes: local_meta.len(),
            transferred_bytes: 0,
            speed_bytes_per_sec: None,
            eta_seconds: None,
            error: None,
            started_at: Utc::now(),
            completed_at: None,
            priority: 0,
            retry_count: 0,
            max_retries: 3,
            next_retry_at: None,
        };

        let meta = TransferMetadata::new(
            transfer_id.clone(),
            session_id,
            local_path,
            remote_path,
            TransferDirection::Upload,
        );

        self.active_transfers.write().await.insert(transfer_id.clone(), progress.clone());
        self.metadata.write().await.insert(transfer_id.clone(), meta);

        let _ = app_handle.emit("sftp_transfer_progress", &serde_json::json!({
            "transferId": transfer_id,
            "transferredBytes": 0,
            "totalBytes": progress.total_bytes,
        }));

        Ok(transfer_id)
    }

    /// Download file from remote server (queues transfer)
    pub async fn download_file(
        &self,
        session_id: String,
        remote_path: String,
        local_path: String,
        app_handle: tauri::AppHandle,
    ) -> Result<String, SFTPError> {
        let sftp_service = self
            .sftp_service
            .upgrade()
            .ok_or_else(|| SFTPError::Other {
                message: "SFTP service unavailable".to_string(),
            })?;

        let remote_entry = sftp_service
            .stat(session_id.clone(), remote_path.clone())
            .await?;

        let transfer_id = Uuid::new_v4().to_string();
        let progress = TransferProgress {
            transfer_id: transfer_id.clone(),
            status: TransferStatus::Queued,
            direction: TransferDirection::Download,
            local_path: local_path.clone(),
            remote_path: remote_path.clone(),
            total_bytes: remote_entry.size.unwrap_or(0),
            transferred_bytes: 0,
            speed_bytes_per_sec: None,
            eta_seconds: None,
            error: None,
            started_at: Utc::now(),
            completed_at: None,
            priority: 0,
            retry_count: 0,
            max_retries: 3,
            next_retry_at: None,
        };

        let meta = TransferMetadata::new(
            transfer_id.clone(),
            session_id,
            local_path,
            remote_path,
            TransferDirection::Download,
        );

        self.active_transfers.write().await.insert(transfer_id.clone(), progress.clone());
        self.metadata.write().await.insert(transfer_id.clone(), meta);

        let _ = app_handle.emit("sftp_transfer_progress", &serde_json::json!({
            "transferId": transfer_id,
            "transferredBytes": 0,
            "totalBytes": progress.total_bytes,
        }));

        Ok(transfer_id)
    }

    /// Process queued transfers respecting max concurrency and priority
    pub async fn process_queue(&self, app_handle: tauri::AppHandle) {
        let active_count = {
            let transfers = self.active_transfers.read().await;
            transfers
                .values()
                .filter(|t| t.status == TransferStatus::InProgress)
                .count()
        };

        if active_count >= self.max_concurrent {
            return;
        }

        let slots = self.max_concurrent - active_count;
        let mut queued_ids = {
            let transfers = self.active_transfers.read().await;
            let mut list: Vec<_> = transfers
                .values()
                .filter(|t| t.status == TransferStatus::Queued)
                .collect();
            list.sort_by(|a, b| b.priority.cmp(&a.priority));
            list.into_iter().take(slots).map(|t| t.transfer_id.clone()).collect::<Vec<_>>()
        };

        for transfer_id in queued_ids.drain(..) {
            let token = CancellationToken::new();
            self.cancellation_tokens.write().await.insert(transfer_id.clone(), token.clone());

            {
                let mut transfers = self.active_transfers.write().await;
                if let Some(t) = transfers.get_mut(&transfer_id) {
                    t.status = TransferStatus::InProgress;
                }
            }

            let manager = self.clone();
            let app_h = app_handle.clone();
            tokio::spawn(async move {
                manager.execute_transfer(transfer_id, token, app_h).await;
            });
        }
    }

    /// Execute single transfer using pipelined I/O
    async fn execute_transfer(
        &self,
        transfer_id: String,
        token: CancellationToken,
        app_handle: tauri::AppHandle,
    ) {
        let (meta, start_offset, part_path) = {
            let metas = self.metadata.read().await;
            let m = match metas.get(&transfer_id) {
                Some(m) => m.clone(),
                None => return,
            };
            let cp = self.checkpoints.get(&transfer_id).await;
            let offset = cp.as_ref().map(|c| c.confirmed_offset).unwrap_or(0);
            let part = cp.map(|c| c.partial_path);
            (m, offset, part)
        };

        let sftp_service = match self.sftp_service.upgrade() {
            Some(s) => s,
            None => return,
        };

        let conn = match sftp_service.get_connection(&meta.session_id).await {
            Ok(c) => c,
            Err(e) => {
                self.fail_transfer(&transfer_id, &e.to_string(), &app_handle).await;
                return;
            }
        };

        let app_handle_for_progress = app_handle.clone();
        let tid_progress = transfer_id.clone();
        let transfers_ref = self.active_transfers.clone();

        let on_progress = move |snap: crate::services::sftp::transfer_io::ProgressSnapshot| {
            if let Ok(mut tr) = transfers_ref.try_write() {
                if let Some(t) = tr.get_mut(&tid_progress) {
                    t.transferred_bytes = snap.bytes_transferred;
                    t.speed_bytes_per_sec = Some(snap.speed_bytes_per_sec);
                    t.eta_seconds = snap.eta_seconds;
                }
            }

            let _ = app_handle_for_progress.emit("sftp_transfer_progress", &serde_json::json!({
                "transferId": tid_progress,
                "transferredBytes": snap.bytes_transferred,
                "totalBytes": snap.total_bytes,
                "speed": snap.speed_bytes_per_sec,
                "eta": snap.eta_seconds,
            }));
        };

        let res = match meta.direction {
            TransferDirection::Upload => {
                upload_file_pipelined(
                    &conn.sftp,
                    UploadOptions {
                        local_path: meta.local_path.clone(),
                        remote_path: meta.remote_path.clone(),
                        start_offset,
                        custom_part_path: part_path,
                    },
                    token.clone(),
                    on_progress,
                )
                .await
                .map(|r| (r.completed, r.partial_path, r.bytes_transferred, r.total_bytes))
            }
            TransferDirection::Download => {
                download_file_pipelined(
                    &conn.sftp,
                    DownloadOptions {
                        remote_path: meta.remote_path.clone(),
                        local_path: meta.local_path.clone(),
                        start_offset,
                        custom_part_path: part_path,
                    },
                    token.clone(),
                    on_progress,
                )
                .await
                .map(|r| (r.completed, r.partial_path, r.bytes_transferred, r.total_bytes))
            }
        };

        match res {
            Ok((completed, partial_path, transferred, total)) => {
                if completed {
                    self.checkpoints.remove(&transfer_id).await;
                    let mut transfers = self.active_transfers.write().await;
                    if let Some(t) = transfers.get_mut(&transfer_id) {
                        t.status = TransferStatus::Completed;
                        t.transferred_bytes = total;
                        t.completed_at = Some(Utc::now());
                    }
                    let _ = app_handle.emit("sftp_transfer_complete", &serde_json::json!({
                        "transferId": transfer_id,
                    }));
                } else {
                    // Paused or interrupted: save checkpoint
                    self.checkpoints.update(&transfer_id, transferred, &partial_path, total).await;
                    let mut transfers = self.active_transfers.write().await;
                    if let Some(t) = transfers.get_mut(&transfer_id) {
                        if t.status == TransferStatus::InProgress {
                            t.status = TransferStatus::Paused;
                        }
                    }
                }
            }
            Err(e) => {
                self.fail_transfer(&transfer_id, &e.to_string(), &app_handle).await;
            }
        }

        self.cancellation_tokens.write().await.remove(&transfer_id);
    }

    async fn fail_transfer(&self, transfer_id: &str, error: &str, app_handle: &tauri::AppHandle) {
        let mut transfers = self.active_transfers.write().await;
        if let Some(t) = transfers.get_mut(transfer_id) {
            t.status = TransferStatus::Failed;
            t.error = Some(error.to_string());
        }
        let _ = app_handle.emit("sftp_transfer_error", &serde_json::json!({
            "transferId": transfer_id,
            "error": error,
        }));
    }

    /// Pause active transfer
    pub async fn pause_transfer(&self, transfer_id: String, app_handle: tauri::AppHandle) -> Result<(), SFTPError> {
        if let Some(token) = self.cancellation_tokens.read().await.get(&transfer_id) {
            token.cancel();
        }
        let mut transfers = self.active_transfers.write().await;
        if let Some(t) = transfers.get_mut(&transfer_id) {
            if t.status == TransferStatus::InProgress || t.status == TransferStatus::Queued {
                t.status = TransferStatus::Paused;
                let _ = app_handle.emit("sftp_transfer_complete", &serde_json::json!({ "transferId": transfer_id }));
                return Ok(());
            }
            return Err(SFTPError::TransferNotResumable { transfer_id });
        }
        Err(SFTPError::TransferNotFound { transfer_id })
    }

    /// Resume paused transfer
    pub async fn resume_transfer(&self, transfer_id: String, app_handle: tauri::AppHandle) -> Result<(), SFTPError> {
        let mut transfers = self.active_transfers.write().await;
        if let Some(t) = transfers.get_mut(&transfer_id) {
            if t.status == TransferStatus::Paused || t.status == TransferStatus::Failed {
                t.status = TransferStatus::Queued;
                t.error = None;
                let _ = app_handle.emit("sftp_transfer_progress", &serde_json::json!({
                    "transferId": transfer_id,
                    "transferredBytes": t.transferred_bytes,
                    "totalBytes": t.total_bytes,
                }));
                return Ok(());
            }
            return Err(SFTPError::TransferNotResumable { transfer_id });
        }
        Err(SFTPError::TransferNotFound { transfer_id })
    }

    /// Cancel transfer
    pub async fn cancel_transfer(&self, transfer_id: String, app_handle: tauri::AppHandle) -> Result<(), SFTPError> {
        if let Some(token) = self.cancellation_tokens.read().await.get(&transfer_id) {
            token.cancel();
        }
        self.checkpoints.remove(&transfer_id).await;
        let mut transfers = self.active_transfers.write().await;
        if let Some(t) = transfers.get_mut(&transfer_id) {
            t.status = TransferStatus::Cancelled;
            let _ = app_handle.emit("sftp_transfer_complete", &serde_json::json!({ "transferId": transfer_id }));
            return Ok(());
        }
        Err(SFTPError::TransferNotFound { transfer_id })
    }

    /// Retry failed transfer
    pub async fn retry_transfer(&self, transfer_id: String, app_handle: tauri::AppHandle) -> Result<(), SFTPError> {
        let mut transfers = self.active_transfers.write().await;
        if let Some(t) = transfers.get_mut(&transfer_id) {
            if t.status == TransferStatus::Failed || t.status == TransferStatus::Cancelled {
                t.status = TransferStatus::Queued;
                t.retry_count += 1;
                t.error = None;
                let _ = app_handle.emit("sftp_transfer_progress", &serde_json::json!({
                    "transferId": transfer_id,
                    "transferredBytes": t.transferred_bytes,
                    "totalBytes": t.total_bytes,
                }));
                return Ok(());
            }
            return Err(SFTPError::TransferNotResumable { transfer_id });
        }
        Err(SFTPError::TransferNotFound { transfer_id })
    }

    /// Set transfer priority
    pub async fn set_priority(&self, transfer_id: String, priority: u8) -> Result<(), SFTPError> {
        let mut transfers = self.active_transfers.write().await;
        if let Some(t) = transfers.get_mut(&transfer_id) {
            t.priority = priority;
            return Ok(());
        }
        Err(SFTPError::TransferNotFound { transfer_id })
    }

    /// Reorder queue
    pub async fn reorder_queue(&self, transfer_ids: Vec<String>) -> Result<(), SFTPError> {
        let mut transfers = self.active_transfers.write().await;
        let total = transfer_ids.len();
        for (index, id) in transfer_ids.iter().enumerate() {
            if let Some(t) = transfers.get_mut(id) {
                t.priority = (total - index).min(255) as u8;
            }
        }
        Ok(())
    }

    /// Get transfer progress
    pub async fn get_progress(&self, transfer_id: String) -> Result<TransferProgress, SFTPError> {
        let transfers = self.active_transfers.read().await;
        transfers.get(&transfer_id).cloned().ok_or_else(|| SFTPError::TransferNotFound { transfer_id })
    }

    /// Get all transfers with optional status filter
    pub async fn get_all_transfers(
        &self,
        status_filter: Option<TransferStatus>,
    ) -> Vec<TransferProgress> {
        let transfers = self.active_transfers.read().await;
        transfers
            .values()
            .filter(|t| {
                if let Some(ref filter) = status_filter {
                    &t.status == filter
                } else {
                    true
                }
            })
            .cloned()
            .collect()
    }

    /// Clear completed and cancelled transfers
    pub async fn clear_completed(&self) {
        let mut transfers = self.active_transfers.write().await;
        transfers.retain(|_, t| t.status != TransferStatus::Completed && t.status != TransferStatus::Cancelled);
        let mut metadata = self.metadata.write().await;
        metadata.retain(|id, _| transfers.contains_key(id));
    }
}
