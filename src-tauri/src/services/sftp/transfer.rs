use std::collections::HashMap;
use std::path::Path;
use std::sync::Arc;
use tokio::fs::File as TokioFile;
use tokio::io::AsyncReadExt;
use tokio::sync::RwLock;
use uuid::Uuid;

use crate::models::sftp::{
    error::SFTPError,
    transfer::{TransferDirection, TransferProgress, TransferStatus},
};
use crate::services::sftp::service::SFTPService;
use tokio::io::AsyncWriteExt;

use chrono::Utc;
use tauri::Emitter;

/// Transfer metadata for resuming
#[derive(Debug, Clone)]
struct TransferMetadata {
    session_id: String,
    local_path: String,
    remote_path: String,
    direction: TransferDirection,
}

/// Transfer Manager for handling file transfers with progress tracking
pub struct TransferManager {
    active_transfers: Arc<RwLock<HashMap<String, TransferProgress>>>,
    transfer_metadata: Arc<RwLock<HashMap<String, TransferMetadata>>>,
    sftp_service: std::sync::Weak<SFTPService>,
}

impl TransferManager {
    /// Create new transfer manager
    pub fn new(sftp_service: Arc<SFTPService>) -> Self {
        Self {
            active_transfers: Arc::new(RwLock::new(HashMap::new())),
            transfer_metadata: Arc::new(RwLock::new(HashMap::new())),
            sftp_service: Arc::downgrade(&sftp_service),
        }
    }

    /// Upload file from local to remote
    pub async fn upload_file(
        &self,
        session_id: String,
        local_path: String,
        remote_path: String,
        app_handle: tauri::AppHandle,
    ) -> Result<String, SFTPError> {
        let transfer_id = Uuid::new_v4().to_string();

        // Check if file exists
        if !Path::new(&local_path).exists() {
            return Err(SFTPError::FileNotFound {
                path: local_path.clone(),
            });
        }

        // Get file size
        let metadata = tokio::fs::metadata(&local_path)
            .await
            .map_err(|e| SFTPError::IoError {
                message: format!("Failed to get file metadata: {}", e),
            })?;

        let total_bytes = metadata.len();

        // Create transfer progress entry
        let progress = TransferProgress {
            transfer_id: transfer_id.clone(),
            status: TransferStatus::Queued,
            direction: TransferDirection::Upload,
            local_path: local_path.clone(),
            remote_path: remote_path.clone(),
            total_bytes,
            transferred_bytes: 0,
            speed_bytes_per_sec: None,
            eta_seconds: None,
            error: None,
            started_at: Utc::now(),
            completed_at: None,
        };

        {
            let mut transfers = self.active_transfers.write().await;
            transfers.insert(transfer_id.clone(), progress);
        }

        // Store metadata for resume capability
        let metadata_entry = TransferMetadata {
            session_id: session_id.clone(),
            local_path: local_path.clone(),
            remote_path: remote_path.clone(),
            direction: TransferDirection::Upload,
        };

        {
            let mut metadata_map = self.transfer_metadata.write().await;
            metadata_map.insert(transfer_id.clone(), metadata_entry);
        }

        // Start transfer in background
        let transfer_manager = self.clone();
        let transfer_id_clone = transfer_id.clone();
        let app_handle_clone = app_handle.clone();
        tokio::spawn(async move {
            if let Err(e) = transfer_manager
                .execute_upload(
                    session_id,
                    local_path,
                    remote_path,
                    transfer_id_clone.clone(),
                    app_handle_clone.clone(),
                )
                .await
            {
                let mut transfers = transfer_manager.active_transfers.write().await;
                if let Some(progress) = transfers.get_mut(&transfer_id_clone) {
                    progress.status = TransferStatus::Failed;
                    progress.error = Some(e.to_string());
                    progress.completed_at = Some(Utc::now());
                }

                let _ = app_handle_clone.emit(
                    "sftp_transfer_error",
                    &serde_json::json!({
                        "transferId": transfer_id_clone,
                        "error": e.to_string(),
                    }),
                );
            }
        });

        Ok(transfer_id)
    }

    /// Execute upload transfer
    async fn execute_upload(
        &self,
        session_id: String,
        local_path: String,
        remote_path: String,
        transfer_id: String,
        app_handle_clone: tauri::AppHandle,
    ) -> Result<(), SFTPError> {
        // Update status to in progress
        {
            let mut transfers = self.active_transfers.write().await;
            if let Some(progress) = transfers.get_mut(&transfer_id) {
                progress.status = TransferStatus::InProgress;
            }
        }

        // Get SFTP session
        let sftp_service = self.sftp_service.upgrade()
            .ok_or_else(|| SFTPError::Other {
                message: "SFTP service is no longer available".to_string(),
            })?;

        let session_data = sftp_service.get_session(&session_id).await?;
        let data = session_data.lock().await;

        // Open local file
        let mut local_file = TokioFile::open(&local_path)
            .await
            .map_err(|e| SFTPError::IoError {
                message: format!("Failed to open local file: {}", e),
            })?;

        // Get file size
        let metadata = local_file.metadata().await
            .map_err(|e| SFTPError::IoError {
                message: format!("Failed to get file metadata: {}", e),
            })?;
        let total = metadata.len();

        // Open remote file for writing
        use russh_sftp::protocol::OpenFlags;
        let mut remote_file = data.sftp
            .open_with_flags(&remote_path, OpenFlags::CREATE | OpenFlags::TRUNCATE | OpenFlags::WRITE)
            .await
            .map_err(|e| SFTPError::Other {
                message: format!("Failed to open remote file: {}", e),
            })?;

        // Upload file in chunks with progress updates
        let chunk_size = 64 * 1024; // 64KB chunks
        let mut transferred = 0u64;
        let mut buffer = vec![0u8; chunk_size];

        loop {
            // Read chunk from local file
            let bytes_read = local_file.read(&mut buffer).await
                .map_err(|e| SFTPError::IoError {
                    message: format!("Failed to read local file: {}", e),
                })?;

            if bytes_read == 0 {
                break; // EOF
            }

            // Write chunk to remote file
            remote_file.write_all(&buffer[..bytes_read]).await
                .map_err(|e| SFTPError::Other {
                    message: format!("Failed to write to remote file: {}", e),
                })?;

            transferred += bytes_read as u64;

            // Update progress
            {
                let mut transfers = self.active_transfers.write().await;
                if let Some(progress) = transfers.get_mut(&transfer_id) {
                    progress.transferred_bytes = transferred;
                }
            }

            // Emit progress update
            let _ = app_handle_clone.emit(
                "sftp_transfer_progress",
                &serde_json::json!({
                    "transferId": transfer_id,
                    "transferredBytes": transferred,
                    "totalBytes": total,
                }),
            );
        }

        // Flush and close remote file
        remote_file.flush().await
            .map_err(|e| SFTPError::Other {
                message: format!("Failed to flush remote file: {}", e),
            })?;

        {
            let mut transfers = self.active_transfers.write().await;
            if let Some(progress) = transfers.get_mut(&transfer_id) {
                progress.transferred_bytes = total;
                progress.status = TransferStatus::Completed;
                progress.completed_at = Some(Utc::now());
            }
        }

                let _ = app_handle_clone.emit(
                    "sftp_transfer_complete",
                    &serde_json::json!({
                        "transferId": transfer_id,
                    }),
                );

        Ok(())
    }

    /// Download file from remote to local
    pub async fn download_file(
        &self,
        session_id: String,
        remote_path: String,
        local_path: String,
        app_handle: tauri::AppHandle,
    ) -> Result<String, SFTPError> {
        let transfer_id = Uuid::new_v4().to_string();

        // Create transfer progress entry
        // We'll need to get file size from remote first
        let sftp_service = self.sftp_service.upgrade()
            .ok_or_else(|| SFTPError::Other {
                message: "SFTP service is no longer available".to_string(),
            })?;
        let entry = sftp_service
            .stat(session_id.clone(), remote_path.clone())
            .await?;

        let total_bytes = entry.size.unwrap_or(0);

        let progress = TransferProgress {
            transfer_id: transfer_id.clone(),
            status: TransferStatus::Queued,
            direction: TransferDirection::Download,
            local_path: local_path.clone(),
            remote_path: remote_path.clone(),
            total_bytes,
            transferred_bytes: 0,
            speed_bytes_per_sec: None,
            eta_seconds: None,
            error: None,
            started_at: Utc::now(),
            completed_at: None,
        };

        {
            let mut transfers = self.active_transfers.write().await;
            transfers.insert(transfer_id.clone(), progress);
        }

        // Store metadata
        let metadata_entry = TransferMetadata {
            session_id: session_id.clone(),
            local_path: local_path.clone(),
            remote_path: remote_path.clone(),
            direction: TransferDirection::Download,
        };

        {
            let mut metadata_map = self.transfer_metadata.write().await;
            metadata_map.insert(transfer_id.clone(), metadata_entry);
        }

        // Start transfer in background
        let transfer_manager = self.clone();
        let transfer_id_clone = transfer_id.clone();
        let app_handle_clone = app_handle.clone();
        tokio::spawn(async move {
            if let Err(e) = transfer_manager
                .execute_download(
                    session_id,
                    remote_path,
                    local_path,
                    transfer_id_clone.clone(),
                    app_handle_clone.clone(),
                )
                .await
            {
                let mut transfers = transfer_manager.active_transfers.write().await;
                if let Some(progress) = transfers.get_mut(&transfer_id_clone) {
                    progress.status = TransferStatus::Failed;
                    progress.error = Some(e.to_string());
                    progress.completed_at = Some(Utc::now());
                }

                let _ = app_handle_clone.emit(
                    "sftp_transfer_error",
                    &serde_json::json!({
                        "transferId": transfer_id_clone,
                        "error": e.to_string(),
                    }),
                );
            }
        });

        Ok(transfer_id)
    }

    /// Execute download transfer
    async fn execute_download(
        &self,
        session_id: String,
        remote_path: String,
        local_path: String,
        transfer_id: String,
        app_handle_clone: tauri::AppHandle,
    ) -> Result<(), SFTPError> {
        // Update status to in progress
        {
            let mut transfers = self.active_transfers.write().await;
            if let Some(progress) = transfers.get_mut(&transfer_id) {
                progress.status = TransferStatus::InProgress;
            }
        }

        // Get SFTP session
        let sftp_service = self.sftp_service.upgrade()
            .ok_or_else(|| SFTPError::Other {
                message: "SFTP service is no longer available".to_string(),
            })?;

        let session_data = sftp_service.get_session(&session_id).await?;
        let data = session_data.lock().await;

        // Open remote file for reading
        let mut remote_file = data.sftp
            .open(&remote_path)
            .await
            .map_err(|e| SFTPError::Other {
                message: format!("Failed to open remote file: {}", e),
            })?;

        // Create local file for writing
        let mut local_file = TokioFile::create(&local_path)
            .await
            .map_err(|e| SFTPError::IoError {
                message: format!("Failed to create local file: {}", e),
            })?;

        // Download file in chunks with progress updates
        let chunk_size = 64 * 1024; // 64KB chunks
        let mut transferred = 0u64;
        let mut buffer = vec![0u8; chunk_size];

        loop {
            // Read chunk from remote file
            use tokio::io::AsyncReadExt;
            let bytes_read = remote_file.read(&mut buffer).await
                .map_err(|e| SFTPError::Other {
                    message: format!("Failed to read from remote file: {}", e),
                })?;

            if bytes_read == 0 {
                break; // EOF
            }

            // Write chunk to local file
            local_file.write_all(&buffer[..bytes_read]).await
                .map_err(|e| SFTPError::IoError {
                    message: format!("Failed to write to local file: {}", e),
                })?;

            transferred += bytes_read as u64;

            // Update progress
            {
                let mut transfers = self.active_transfers.write().await;
                if let Some(progress) = transfers.get_mut(&transfer_id) {
                    progress.transferred_bytes = transferred;
                }
            }

            // Emit progress update
            {
                let transfers = self.active_transfers.read().await;
                if let Some(progress) = transfers.get(&transfer_id) {
                    let _ = app_handle_clone.emit(
                        "sftp_transfer_progress",
                        &serde_json::json!({
                            "transferId": transfer_id,
                            "transferredBytes": transferred,
                            "totalBytes": progress.total_bytes,
                        }),
                    );
                }
            }
        }

        // Sync local file
        local_file.sync_all().await
            .map_err(|e| SFTPError::IoError {
                message: format!("Failed to sync local file: {}", e),
            })?;

        {
            let mut transfers = self.active_transfers.write().await;
            if let Some(progress) = transfers.get_mut(&transfer_id) {
                let total = progress.total_bytes;
                progress.transferred_bytes = total;
                progress.status = TransferStatus::Completed;
                progress.completed_at = Some(Utc::now());
            }
        }

        let _ = app_handle_clone.emit(
            "sftp_transfer_complete",
            &serde_json::json!({
                "transferId": transfer_id,
            }),
        );

        Ok(())
    }

    /// Get transfer progress
    pub async fn get_progress(&self, transfer_id: String) -> Result<TransferProgress, SFTPError> {
        let transfers = self.active_transfers.read().await;
        transfers
            .get(&transfer_id)
            .cloned()
            .ok_or_else(|| SFTPError::TransferNotFound { transfer_id })
    }

    /// Cancel transfer
    pub async fn cancel_transfer(&self, transfer_id: String) -> Result<(), SFTPError> {
        let mut transfers = self.active_transfers.write().await;
        if let Some(progress) = transfers.get_mut(&transfer_id) {
            if progress.is_active() {
                progress.status = TransferStatus::Cancelled;
                progress.completed_at = Some(Utc::now());
                Ok(())
            } else {
                Err(SFTPError::TransferNotResumable { transfer_id })
            }
        } else {
            Err(SFTPError::TransferNotFound { transfer_id })
        }
    }

    /// Resume interrupted transfer
    pub async fn resume_transfer(
        &self,
        transfer_id: String,
        app_handle: tauri::AppHandle,
    ) -> Result<(), SFTPError> {
        let metadata = {
            let metadata_map = self.transfer_metadata.read().await;
            metadata_map
                .get(&transfer_id)
                .cloned()
                .ok_or_else(|| SFTPError::TransferNotFound {
                    transfer_id: transfer_id.clone(),
                })?
        };

        // Check if transfer is resumable
        let progress = self.get_progress(transfer_id.clone()).await?;
        if !matches!(progress.status, TransferStatus::Failed | TransferStatus::Paused) {
            return Err(SFTPError::TransferNotResumable { transfer_id });
        }

        // Restart transfer from where it left off
        match metadata.direction {
            TransferDirection::Upload => {
                self.execute_upload(
                    metadata.session_id,
                    metadata.local_path,
                    metadata.remote_path,
                    transfer_id,
                    app_handle,
                )
                .await
            }
            TransferDirection::Download => {
                self.execute_download(
                    metadata.session_id,
                    metadata.remote_path,
                    metadata.local_path,
                    transfer_id,
                    app_handle,
                )
                .await
            }
        }
    }
}

impl Clone for TransferManager {
    fn clone(&self) -> Self {
        Self {
            active_transfers: self.active_transfers.clone(),
            transfer_metadata: self.transfer_metadata.clone(),
            sftp_service: std::sync::Weak::clone(&self.sftp_service),
        }
    }
}

