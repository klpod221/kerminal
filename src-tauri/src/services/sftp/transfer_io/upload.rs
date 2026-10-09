// Copyright (C) 2026 Bùi Thanh Xuân (klpod221)
// SPDX-License-Identifier: GPL-3.0-or-later

use std::path::Path;
use std::sync::Arc;
use futures::stream::{FuturesUnordered, StreamExt};
use russh_sftp::client::RawSftpSession;
use russh_sftp::protocol::{FileAttributes, OpenFlags};
use tokio::io::{AsyncReadExt, AsyncSeekExt, SeekFrom};
use tokio_util::sync::CancellationToken;
use uuid::Uuid;

use crate::services::sftp::errors::SftpError;
use crate::services::sftp::transfer_io::progress::{ProgressSnapshot, ProgressTracker};

/// Chunk size for pipelined writes (64KB)
pub const UPLOAD_CHUNK_SIZE: usize = 64 * 1024;

/// Concurrency pipeline depth (number of in-flight write requests)
pub const UPLOAD_PIPELINE_DEPTH: usize = 16;

/// Configuration options for uploading a file
pub struct UploadOptions {
    pub local_path: String,
    pub remote_path: String,
    pub start_offset: u64,
    pub custom_part_path: Option<String>,
}

/// Result of a upload operation
#[allow(dead_code)]
pub struct UploadResult {
    pub final_path: String,
    pub partial_path: String,
    pub bytes_transferred: u64,
    pub total_bytes: u64,
    pub completed: bool,
}

/// Perform pipelined streaming upload to remote SFTP server
pub async fn upload_file_pipelined<F>(
    raw: &Arc<RawSftpSession>,
    options: UploadOptions,
    cancellation_token: CancellationToken,
    on_progress: F,
) -> Result<UploadResult, SftpError>
where
    F: Fn(ProgressSnapshot) + Send + Sync + 'static,
{
    let local_path = Path::new(&options.local_path);
    let mut local_file = tokio::fs::File::open(local_path)
        .await
        .map_err(|e| SftpError::IoError {
            message: format!("Failed to open local file {}: {}", options.local_path, e),
        })?;

    let meta = local_file
        .metadata()
        .await
        .map_err(|e| SftpError::IoError {
            message: format!("Failed to stat local file {}: {}", options.local_path, e),
        })?;
    let total_bytes = meta.len();

    let start_offset = options.start_offset.min(total_bytes);
    if start_offset > 0 {
        local_file
            .seek(SeekFrom::Start(start_offset))
            .await
            .map_err(|e| SftpError::IoError {
                message: format!("Failed to seek local file {}: {}", options.local_path, e),
            })?;
    }

    // Determine partial file path for atomic write
    let partial_path = options
        .custom_part_path
        .unwrap_or_else(|| format!("{}.kerminal-part-{}", options.remote_path, Uuid::new_v4()));

    // Open remote partial file
    let open_flags = if start_offset > 0 {
        OpenFlags::CREATE | OpenFlags::WRITE
    } else {
        OpenFlags::CREATE | OpenFlags::TRUNCATE | OpenFlags::WRITE
    };

    let remote_handle = raw
        .open(&partial_path, open_flags, FileAttributes::empty())
        .await
        .map_err(|e| SftpError::from_russh_sftp(e, Some(&partial_path)))?
        .handle;

    let tracker = ProgressTracker::new(total_bytes, start_offset);
    if let Some(snap) = tracker.check_emit(true) {
        on_progress(snap);
    }

    let mut current_offset = start_offset;
    let mut inflight = FuturesUnordered::new();
    let mut cancelled = false;

    // Buffer read and in-flight pipeline loop
    while current_offset < total_bytes || !inflight.is_empty() {
        if cancellation_token.is_cancelled() {
            cancelled = true;
            break;
        }

        // Fill pipeline up to UPLOAD_PIPELINE_DEPTH
        while current_offset < total_bytes && inflight.len() < UPLOAD_PIPELINE_DEPTH {
            if cancellation_token.is_cancelled() {
                cancelled = true;
                break;
            }

            let to_read = ((total_bytes - current_offset) as usize).min(UPLOAD_CHUNK_SIZE);
            let mut buf = vec![0u8; to_read];

            match local_file.read_exact(&mut buf).await {
                Ok(_) => {
                    let raw_clone = raw.clone();
                    let handle_clone = remote_handle.clone();
                    let offset = current_offset;
                    let chunk_len = to_read as u64;

                    inflight.push(async move {
                        let res = raw_clone.write(&handle_clone, offset, buf).await;
                        (res, chunk_len)
                    });

                    current_offset += chunk_len;
                }
                Err(e) => {
                    let _ = raw.close(&remote_handle).await;
                    return Err(SftpError::IoError {
                        message: format!("Failed to read local file chunk: {}", e),
                    });
                }
            }
        }

        // Await next write completion from pipeline
        if let Some((write_res, bytes_done)) = inflight.next().await {
            match write_res {
                Ok(_) => {
                    tracker.add_bytes(bytes_done);
                    if let Some(snap) = tracker.check_emit(false) {
                        on_progress(snap);
                    }
                }
                Err(e) => {
                    let _ = raw.close(&remote_handle).await;
                    return Err(SftpError::from_russh_sftp(e, Some(&partial_path)));
                }
            }
        }
    }

    // Close remote handle
    let _ = raw.close(remote_handle).await;

    if cancelled {
        return Ok(UploadResult {
            final_path: options.remote_path,
            partial_path,
            bytes_transferred: tracker.current_bytes(),
            total_bytes,
            completed: false,
        });
    }

    // Emit final progress snapshot
    if let Some(snap) = tracker.check_emit(true) {
        on_progress(snap);
    }

    // Atomic rename partial -> target
    // Check if target exists to do safe replace
    let target_exists = raw.stat(&options.remote_path).await.is_ok();
    if target_exists {
        let bak_path = format!("{}.kerminal-bak-{}", options.remote_path, Uuid::new_v4());
        raw.rename(&options.remote_path, &bak_path)
            .await
            .map_err(|e| SftpError::from_russh_sftp(e, Some(&options.remote_path)))?;

        if let Err(e) = raw.rename(&partial_path, &options.remote_path).await {
            let _ = raw.rename(&bak_path, &options.remote_path).await;
            return Err(SftpError::from_russh_sftp(e, Some(&options.remote_path)));
        }
        let _ = raw.remove(&bak_path).await;
    } else {
        raw.rename(&partial_path, &options.remote_path)
            .await
            .map_err(|e| SftpError::from_russh_sftp(e, Some(&options.remote_path)))?;
    }

    Ok(UploadResult {
        final_path: options.remote_path,
        partial_path,
        bytes_transferred: total_bytes,
        total_bytes,
        completed: true,
    })
}
