use std::collections::HashMap;
use std::path::Path;
use std::sync::Arc;
use tokio::fs::File as TokioFile;
use tokio::io::AsyncReadExt;
use tokio::sync::RwLock;
use tokio_util::sync::CancellationToken;
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
    cancellation_tokens: Arc<RwLock<HashMap<String, CancellationToken>>>,
    sftp_service: std::sync::Weak<SFTPService>,
}

impl TransferManager {
    /// Create new transfer manager
    pub fn new(sftp_service: Arc<SFTPService>) -> Self {
        Self {
            active_transfers: Arc::new(RwLock::new(HashMap::new())),
            transfer_metadata: Arc::new(RwLock::new(HashMap::new())),
            cancellation_tokens: Arc::new(RwLock::new(HashMap::new())),
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
            priority: 0,
            retry_count: 0,
            max_retries: 3,
            next_retry_at: None,
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

        // Create cancellation token for this transfer
        let cancel_token = CancellationToken::new();
        {
            let mut tokens = self.cancellation_tokens.write().await;
            tokens.insert(transfer_id.clone(), cancel_token.clone());
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
                    cancel_token,
                )
                .await
            {
                let mut transfers = transfer_manager.active_transfers.write().await;
                if let Some(progress) = transfers.get_mut(&transfer_id_clone) {
                    // Check current status - if already paused/cancelled, don't change
                    match progress.status {
                        TransferStatus::Paused | TransferStatus::Cancelled => {
                            // Status already set, don't change
                        }
                        TransferStatus::InProgress => {
                            // Check error message to see if it was paused or cancelled
                            let error_msg = e.to_string();
                            if error_msg.contains("paused") {
                                progress.status = TransferStatus::Paused;
                            } else if error_msg.contains("cancelled") {
                                progress.status = TransferStatus::Cancelled;
                                progress.completed_at = Some(Utc::now());
                            } else {
                                progress.status = TransferStatus::Failed;
                                progress.error = Some(error_msg.clone());
                                progress.completed_at = Some(Utc::now());

                                let _ = app_handle_clone.emit(
                                    "sftp_transfer_error",
                                    &serde_json::json!({
                                        "transferId": transfer_id_clone,
                                        "error": error_msg,
                                    }),
                                );
                            }
                        }
                        _ => {
                            // Other status, don't change
                        }
                    }
                }
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
        cancel_token: CancellationToken,
    ) -> Result<(), SFTPError> {
        // Update status to in progress
        let resume_from: u64 = {
            let mut transfers = self.active_transfers.write().await;
            if let Some(progress) = transfers.get_mut(&transfer_id) {
                let resume_pos = progress.transferred_bytes;
                progress.status = TransferStatus::InProgress;
                resume_pos
            } else {
                return Err(SFTPError::TransferNotFound { transfer_id });
            }
        };

        // Get SFTP session
        let sftp_service = self
            .sftp_service
            .upgrade()
            .ok_or_else(|| SFTPError::Other {
                message: "SFTP service is no longer available".to_string(),
            })?;

        let session_data = sftp_service.get_session(&session_id).await?;
        let data = session_data.lock().await;

        // Open local file
        let mut local_file =
            TokioFile::open(&local_path)
                .await
                .map_err(|e| SFTPError::IoError {
                    message: format!("Failed to open local file: {}", e),
                })?;

        // Get file size
        let metadata = local_file
            .metadata()
            .await
            .map_err(|e| SFTPError::IoError {
                message: format!("Failed to get file metadata: {}", e),
            })?;
        let total = metadata.len();

        // Open remote file for writing and determine actual resume position
        use russh_sftp::protocol::OpenFlags;
        let (mut remote_file, actual_resume_from, actual_local_seek) = if resume_from > 0 {
            // Resume: check if remote file exists and has correct size
            let remote_meta = data.sftp.metadata(&remote_path).await.ok();
            if let Some(meta) = remote_meta {
                let remote_size = meta.size.unwrap_or(0);
                if remote_size == resume_from {
                    // File size matches resume position, append from here
                    let file = data
                        .sftp
                        .open_with_flags(&remote_path, OpenFlags::WRITE | OpenFlags::APPEND)
                        .await
                        .map_err(|e| SFTPError::Other {
                            message: format!("Failed to open remote file for append: {}", e),
                        })?;
                    (file, resume_from, resume_from)
                } else if remote_size < resume_from {
                    // File is smaller than expected, append from current size
                    // Adjust resume position to match actual remote file size
                    let file = data
                        .sftp
                        .open_with_flags(&remote_path, OpenFlags::WRITE | OpenFlags::APPEND)
                        .await
                        .map_err(|e| SFTPError::Other {
                            message: format!("Failed to open remote file for append: {}", e),
                        })?;

                    // Update progress to reflect actual position
                    let mut transfers = self.active_transfers.write().await;
                    if let Some(progress) = transfers.get_mut(&transfer_id) {
                        progress.transferred_bytes = remote_size;
                    }
                    drop(transfers);

                    (file, remote_size, remote_size)
                } else {
                    // File is larger than expected - truncate and restart from beginning
                    let file = data
                        .sftp
                        .open_with_flags(
                            &remote_path,
                            OpenFlags::CREATE | OpenFlags::TRUNCATE | OpenFlags::WRITE,
                        )
                        .await
                        .map_err(|e| SFTPError::Other {
                            message: format!("Failed to open remote file: {}", e),
                        })?;

                    // Reset progress since we're starting over
                    let mut transfers = self.active_transfers.write().await;
                    if let Some(progress) = transfers.get_mut(&transfer_id) {
                        progress.transferred_bytes = 0;
                    }
                    drop(transfers);

                    (file, 0, 0)
                }
            } else {
                // File doesn't exist, create new
                let file = data
                    .sftp
                    .open_with_flags(
                        &remote_path,
                        OpenFlags::CREATE | OpenFlags::TRUNCATE | OpenFlags::WRITE,
                    )
                    .await
                    .map_err(|e| SFTPError::Other {
                        message: format!("Failed to open remote file: {}", e),
                    })?;
                (file, 0, 0)
            }
        } else {
            // New transfer: create/truncate
            let file = data
                .sftp
                .open_with_flags(
                    &remote_path,
                    OpenFlags::CREATE | OpenFlags::TRUNCATE | OpenFlags::WRITE,
                )
                .await
                .map_err(|e| SFTPError::Other {
                    message: format!("Failed to open remote file: {}", e),
                })?;
            (file, 0, 0)
        };

        // Seek local file to actual resume position
        if actual_local_seek > 0 {
            use tokio::io::AsyncSeekExt;
            local_file
                .seek(std::io::SeekFrom::Start(actual_local_seek))
                .await
                .map_err(|e| SFTPError::IoError {
                    message: format!("Failed to seek local file: {}", e),
                })?;
        }

        // Upload file in chunks with progress updates
        let chunk_size = 64 * 1024; // 64KB chunks
        let mut transferred = actual_resume_from; // Start from actual resume position
        let mut buffer = vec![0u8; chunk_size];

        loop {
            tokio::select! {
                _ = cancel_token.cancelled() => {
                    // Transfer was paused or cancelled
                    {
                        let transfers = self.active_transfers.read().await;
                        if let Some(progress) = transfers.get(&transfer_id) {
                            if progress.status == TransferStatus::Paused {
                                return Err(SFTPError::Other {
                                    message: "Transfer paused".to_string(),
                                });
                            }
                        }
                    }
                    return Err(SFTPError::Other {
                        message: "Transfer cancelled".to_string(),
                    });
                }
                result = local_file.read(&mut buffer) => {
                    let bytes_read = result.map_err(|e| SFTPError::IoError {
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
                            // Check if status was changed to paused/cancelled
                            if progress.status == TransferStatus::Paused || progress.status == TransferStatus::Cancelled {
                                return Err(SFTPError::Other {
                                    message: format!("Transfer {:?}", progress.status),
                                });
                            }
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
            }
        }

        // Flush and close remote file
        remote_file.flush().await.map_err(|e| SFTPError::Other {
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
        let sftp_service = self
            .sftp_service
            .upgrade()
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
            priority: 0,
            retry_count: 0,
            max_retries: 3,
            next_retry_at: None,
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

        // Create cancellation token for this transfer
        let cancel_token = CancellationToken::new();
        {
            let mut tokens = self.cancellation_tokens.write().await;
            tokens.insert(transfer_id.clone(), cancel_token.clone());
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
                    cancel_token,
                )
                .await
            {
                let mut transfers = transfer_manager.active_transfers.write().await;
                if let Some(progress) = transfers.get_mut(&transfer_id_clone) {
                    // Check current status - if already paused/cancelled, don't change
                    match progress.status {
                        TransferStatus::Paused | TransferStatus::Cancelled => {
                            // Status already set, don't change
                        }
                        TransferStatus::InProgress => {
                            // Check error message to see if it was paused or cancelled
                            let error_msg = e.to_string();
                            if error_msg.contains("paused") {
                                progress.status = TransferStatus::Paused;
                            } else if error_msg.contains("cancelled") {
                                progress.status = TransferStatus::Cancelled;
                                progress.completed_at = Some(Utc::now());
                            } else {
                                progress.status = TransferStatus::Failed;
                                progress.error = Some(error_msg.clone());
                                progress.completed_at = Some(Utc::now());

                                let _ = app_handle_clone.emit(
                                    "sftp_transfer_error",
                                    &serde_json::json!({
                                        "transferId": transfer_id_clone,
                                        "error": error_msg,
                                    }),
                                );
                            }
                        }
                        _ => {
                            // Other status, don't change
                        }
                    }
                }
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
        cancel_token: CancellationToken,
    ) -> Result<(), SFTPError> {
        // Update status to in progress and get resume position
        let resume_from: u64 = {
            let mut transfers = self.active_transfers.write().await;
            if let Some(progress) = transfers.get_mut(&transfer_id) {
                let resume_pos = progress.transferred_bytes;
                progress.status = TransferStatus::InProgress;
                resume_pos
            } else {
                return Err(SFTPError::TransferNotFound { transfer_id });
            }
        };

        // Get SFTP session
        let sftp_service = self
            .sftp_service
            .upgrade()
            .ok_or_else(|| SFTPError::Other {
                message: "SFTP service is no longer available".to_string(),
            })?;

        let session_data = sftp_service.get_session(&session_id).await?;
        let data = session_data.lock().await;

        // Open remote file for reading
        let mut remote_file = data
            .sftp
            .open(&remote_path)
            .await
            .map_err(|e| SFTPError::Other {
                message: format!("Failed to open remote file: {}", e),
            })?;

        // Skip bytes to resume position if resuming
        // For SFTP, we need to read and discard bytes until we reach the position
        let actual_resume_from = if resume_from > 0 {
            use tokio::io::AsyncReadExt;
            let mut skip_buffer = vec![0u8; 64 * 1024]; // 64KB buffer for skipping
            let mut skipped = 0u64;

            while skipped < resume_from {
                let remaining = resume_from - skipped;
                let to_skip = std::cmp::min(skip_buffer.len() as u64, remaining) as usize;
                let bytes_read = remote_file
                    .read(&mut skip_buffer[..to_skip])
                    .await
                    .map_err(|e| SFTPError::Other {
                        message: format!("Failed to read from remote file during seek: {}", e),
                    })?;

                if bytes_read == 0 {
                    // EOF reached before resume position - file might be smaller than expected
                    // In this case, we'll start from where we are
                    break;
                }

                skipped += bytes_read as u64;
            }

            // Update resume position if file was shorter than expected
            if skipped < resume_from {
                let mut transfers = self.active_transfers.write().await;
                if let Some(progress) = transfers.get_mut(&transfer_id) {
                    progress.transferred_bytes = skipped;
                }
            }

            skipped
        } else {
            0
        };

        // Open local file for writing (append if resuming)
        let mut local_file = if actual_resume_from > 0 {
            // Check if local file exists and has correct size
            if Path::new(&local_path).exists() {
                use tokio::io::AsyncSeekExt;
                let mut file =
                    TokioFile::open(&local_path)
                        .await
                        .map_err(|e| SFTPError::IoError {
                            message: format!("Failed to open local file: {}", e),
                        })?;

                let meta = file.metadata().await.map_err(|e| SFTPError::IoError {
                    message: format!("Failed to get local file metadata: {}", e),
                })?;

                if meta.len() == actual_resume_from {
                    // File size matches, seek to end for append
                    file.seek(std::io::SeekFrom::End(0))
                        .await
                        .map_err(|e| SFTPError::IoError {
                            message: format!("Failed to seek local file: {}", e),
                        })?;
                    file
                } else {
                    // File size doesn't match, truncate and restart
                    drop(file);
                    TokioFile::create(&local_path)
                        .await
                        .map_err(|e| SFTPError::IoError {
                            message: format!("Failed to create local file: {}", e),
                        })?
                }
            } else {
                // File doesn't exist, create new
                TokioFile::create(&local_path)
                    .await
                    .map_err(|e| SFTPError::IoError {
                        message: format!("Failed to create local file: {}", e),
                    })?
            }
        } else {
            // New transfer: create/truncate
            TokioFile::create(&local_path)
                .await
                .map_err(|e| SFTPError::IoError {
                    message: format!("Failed to create local file: {}", e),
                })?
        };

        // Download file in chunks with progress updates
        let chunk_size = 64 * 1024; // 64KB chunks
        let mut transferred = actual_resume_from; // Start from actual resume position
        let mut buffer = vec![0u8; chunk_size];

        loop {
            tokio::select! {
                _ = cancel_token.cancelled() => {
                    // Transfer was paused or cancelled
                    {
                        let transfers = self.active_transfers.read().await;
                        if let Some(progress) = transfers.get(&transfer_id) {
                            if progress.status == TransferStatus::Paused {
                                return Err(SFTPError::Other {
                                    message: "Transfer paused".to_string(),
                                });
                            }
                        }
                    }
                    return Err(SFTPError::Other {
                        message: "Transfer cancelled".to_string(),
                    });
                }
                result = remote_file.read(&mut buffer) => {
                    let bytes_read = result.map_err(|e| SFTPError::Other {
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
                            // Check if status was changed to paused/cancelled
                            if progress.status == TransferStatus::Paused || progress.status == TransferStatus::Cancelled {
                                return Err(SFTPError::Other {
                                    message: format!("Transfer {:?}", progress.status),
                                });
                            }
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
            }
        }

        // Sync local file
        local_file
            .sync_all()
            .await
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
    pub async fn cancel_transfer(
        &self,
        transfer_id: String,
        app_handle: tauri::AppHandle,
    ) -> Result<(), SFTPError> {
        // Cancel the transfer token to stop the loop
        {
            let tokens = self.cancellation_tokens.read().await;
            if let Some(token) = tokens.get(&transfer_id) {
                token.cancel();
            }
        }

        let mut transfers = self.active_transfers.write().await;
        if let Some(progress) = transfers.get_mut(&transfer_id) {
            if progress.is_active() {
                progress.status = TransferStatus::Cancelled;
                progress.completed_at = Some(Utc::now());

                // Remove cancellation token
                drop(transfers);
                let mut tokens = self.cancellation_tokens.write().await;
                tokens.remove(&transfer_id);

                // Emit cancel event for realtime updates
                let _ = app_handle.emit(
                    "sftp_transfer_complete",
                    &serde_json::json!({
                        "transferId": transfer_id,
                    }),
                );

                Ok(())
            } else {
                Err(SFTPError::TransferNotResumable { transfer_id })
            }
        } else {
            Err(SFTPError::TransferNotFound { transfer_id })
        }
    }

    /// Pause transfer
    pub async fn pause_transfer(
        &self,
        transfer_id: String,
        app_handle: tauri::AppHandle,
    ) -> Result<(), SFTPError> {
        // Cancel the transfer token to stop the loop
        {
            let tokens = self.cancellation_tokens.read().await;
            if let Some(token) = tokens.get(&transfer_id) {
                token.cancel();
            }
        }

        let mut transfers = self.active_transfers.write().await;
        if let Some(progress) = transfers.get_mut(&transfer_id) {
            if progress.status == TransferStatus::InProgress {
                progress.status = TransferStatus::Paused;

                // Keep the cancellation token so we can track it's paused
                // Remove it when resuming

                // Emit pause event for realtime updates
                let _ = app_handle.emit(
                    "sftp_transfer_complete",
                    &serde_json::json!({
                        "transferId": transfer_id,
                    }),
                );

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
        if !matches!(
            progress.status,
            TransferStatus::Failed | TransferStatus::Paused
        ) {
            return Err(SFTPError::TransferNotResumable { transfer_id });
        }

        // Remove old cancellation token
        {
            let mut tokens = self.cancellation_tokens.write().await;
            tokens.remove(&transfer_id);
        }

        // Create new cancellation token for resumed transfer
        let cancel_token = CancellationToken::new();
        {
            let mut tokens = self.cancellation_tokens.write().await;
            tokens.insert(transfer_id.clone(), cancel_token.clone());
        }

        // Update status back to in progress
        {
            let mut transfers = self.active_transfers.write().await;
            if let Some(progress) = transfers.get_mut(&transfer_id) {
                progress.status = TransferStatus::InProgress;
            }
        }

        // Restart transfer from where it left off
        // Resume functionality is implemented in execute_upload and execute_download
        match metadata.direction {
            TransferDirection::Upload => {
                self.execute_upload(
                    metadata.session_id,
                    metadata.local_path,
                    metadata.remote_path,
                    transfer_id,
                    app_handle,
                    cancel_token,
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
                    cancel_token,
                )
                .await
            }
        }
    }

    /// Set transfer priority
    pub async fn set_priority(&self, transfer_id: String, priority: u8) -> Result<(), SFTPError> {
        let mut transfers = self.active_transfers.write().await;
        if let Some(progress) = transfers.get_mut(&transfer_id) {
            progress.priority = priority;
            Ok(())
        } else {
            Err(SFTPError::TransferNotFound { transfer_id })
        }
    }

    /// Get all transfers with optional status filter
    pub async fn get_all_transfers(
        &self,
        status_filter: Option<TransferStatus>,
    ) -> Vec<TransferProgress> {
        let transfers = self.active_transfers.read().await;
        let mut result: Vec<TransferProgress> = transfers
            .values()
            .filter(|t| {
                if let Some(ref filter) = status_filter {
                    &t.status == filter
                } else {
                    true
                }
            })
            .cloned()
            .collect();

        // Sort by priority (descending) then by started_at (ascending)
        result.sort_by(|a, b| {
            b.priority
                .cmp(&a.priority)
                .then_with(|| a.started_at.cmp(&b.started_at))
        });

        result
    }

    /// Reorder transfer queue manually
    /// Note: This is mainly for UI purposes. Actual execution order depends on priority and status.
    /// This method updates priorities based on the order provided.
    pub async fn reorder_queue(&self, transfer_ids: Vec<String>) -> Result<(), SFTPError> {
        let mut transfers = self.active_transfers.write().await;

        // Assign priorities in reverse order (first in list = highest priority)
        let max_priority = 255u8;
        let priority_step = if transfer_ids.len() > 1 {
            max_priority / (transfer_ids.len() as u8)
        } else {
            0
        };

        for (index, transfer_id) in transfer_ids.iter().enumerate() {
            if let Some(progress) = transfers.get_mut(transfer_id) {
                // Only reorder if transfer is queued or paused
                if matches!(
                    progress.status,
                    TransferStatus::Queued | TransferStatus::Paused
                ) {
                    progress.priority = max_priority.saturating_sub((index as u8) * priority_step);
                }
            }
        }

        Ok(())
    }

    /// Retry a failed transfer with exponential backoff
    pub async fn retry_transfer(
        &self,
        transfer_id: String,
        app_handle: tauri::AppHandle,
    ) -> Result<(), SFTPError> {
        // Check if transfer exists and is retry-able
        let should_retry = {
            let mut transfers = self.active_transfers.write().await;
            if let Some(progress) = transfers.get_mut(&transfer_id) {
                // Only retry failed transfers that haven't exceeded max retries
                if progress.status != TransferStatus::Failed {
                    return Err(SFTPError::Other {
                        message: "Transfer is not in failed state".to_string(),
                    });
                }

                if progress.retry_count >= progress.max_retries {
                    return Err(SFTPError::Other {
                        message: format!(
                            "Transfer has exceeded maximum retry attempts ({}/{})",
                            progress.retry_count, progress.max_retries
                        ),
                    });
                }

                // Increment retry count
                progress.retry_count += 1;

                // Calculate exponential backoff delay (base 2 seconds, max 60 seconds)
                let base_delay = 2;
                let delay_secs = std::cmp::min(base_delay * 2u64.pow(progress.retry_count - 1), 60);

                progress.next_retry_at =
                    Some(Utc::now() + chrono::Duration::seconds(delay_secs as i64));
                progress.status = TransferStatus::Queued;
                progress.error = None;

                true
            } else {
                return Err(SFTPError::TransferNotFound {
                    transfer_id: transfer_id.clone(),
                });
            }
        };

        if should_retry {
            // Wait for the backoff delay
            let wait_until = {
                let transfers = self.active_transfers.read().await;
                transfers.get(&transfer_id).and_then(|p| p.next_retry_at)
            };

            if let Some(retry_time) = wait_until {
                let now = Utc::now();
                if retry_time > now {
                    let wait_duration = (retry_time - now)
                        .to_std()
                        .unwrap_or(std::time::Duration::from_secs(0));
                    tokio::time::sleep(wait_duration).await;
                }
            }

            // Resume the transfer
            self.resume_transfer(transfer_id, app_handle).await
        } else {
            Ok(())
        }
    }
}

impl Clone for TransferManager {
    fn clone(&self) -> Self {
        Self {
            active_transfers: self.active_transfers.clone(),
            transfer_metadata: self.transfer_metadata.clone(),
            cancellation_tokens: self.cancellation_tokens.clone(),
            sftp_service: std::sync::Weak::clone(&self.sftp_service),
        }
    }
}
