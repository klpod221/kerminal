// Copyright (C) 2026 Bùi Thanh Xuân (klpod221)
// SPDX-License-Identifier: GPL-3.0-or-later

use std::path::Path;
use std::sync::Arc;
use futures::stream::{FuturesUnordered, StreamExt};
use russh_sftp::client::RawSftpSession;
use russh_sftp::protocol::{FileAttributes, OpenFlags};
use tokio::io::{AsyncSeekExt, AsyncWriteExt, SeekFrom};
use tokio_util::sync::CancellationToken;
use uuid::Uuid;

use crate::services::sftp::errors::SftpError;
use crate::services::sftp::transfer_io::progress::{ProgressSnapshot, ProgressTracker};

/// Chunk size for pipelined reads (64KB)
pub const DOWNLOAD_CHUNK_SIZE: usize = 64 * 1024;

/// Concurrency pipeline depth (number of in-flight read requests)
pub const DOWNLOAD_PIPELINE_DEPTH: usize = 16;

/// Configuration options for downloading a file
pub struct DownloadOptions {
    pub remote_path: String,
    pub local_path: String,
    pub start_offset: u64,
    pub custom_part_path: Option<String>,
}

/// Result of a download operation
#[allow(dead_code)]
pub struct DownloadResult {
    pub final_path: String,
    pub partial_path: String,
    pub bytes_transferred: u64,
    pub total_bytes: u64,
    pub completed: bool,
}

/// Perform pipelined streaming download from remote SFTP server
pub async fn download_file_pipelined<F>(
    raw: &Arc<RawSftpSession>,
    options: DownloadOptions,
    cancellation_token: CancellationToken,
    on_progress: F,
) -> Result<DownloadResult, SftpError>
where
    F: Fn(ProgressSnapshot) + Send + Sync + 'static,
{
    // Stat remote file to determine total size
    let remote_meta = raw
        .stat(&options.remote_path)
        .await
        .map_err(|e| SftpError::from_russh_sftp(e, Some(&options.remote_path)))?
        .attrs;

    let total_bytes = remote_meta.size.unwrap_or(0);
    let start_offset = options.start_offset.min(total_bytes);

    // Ensure local parent directory exists
    let local_dest = Path::new(&options.local_path);
    if let Some(parent) = local_dest.parent() {
        tokio::fs::create_dir_all(parent)
            .await
            .map_err(|e| SftpError::IoError {
                message: format!("Failed to create local directory: {}", e),
            })?;
    }

    // Determine local partial file path
    let partial_path = options
        .custom_part_path
        .unwrap_or_else(|| format!("{}.kerminal-part-{}", options.local_path, Uuid::new_v4()));

    let mut local_file = tokio::fs::OpenOptions::new()
        .create(true)
        .write(true)
        .truncate(start_offset == 0)
        .open(&partial_path)
        .await
        .map_err(|e| SftpError::IoError {
            message: format!("Failed to open local partial file {}: {}", partial_path, e),
        })?;

    if start_offset > 0 {
        local_file
            .seek(SeekFrom::Start(start_offset))
            .await
            .map_err(|e| SftpError::IoError {
                message: format!("Failed to seek local partial file: {}", e),
            })?;
    }

    // Open remote file for reading
    let remote_handle = raw
        .open(&options.remote_path, OpenFlags::READ, FileAttributes::empty())
        .await
        .map_err(|e| SftpError::from_russh_sftp(e, Some(&options.remote_path)))?
        .handle;

    let tracker = ProgressTracker::new(total_bytes, start_offset);
    if let Some(snap) = tracker.check_emit(true) {
        on_progress(snap);
    }

    let mut request_offset = start_offset;
    let mut inflight = FuturesUnordered::new();
    let mut cancelled = false;

    // Buffer read and in-flight pipeline loop
    while request_offset < total_bytes || !inflight.is_empty() {
        if cancellation_token.is_cancelled() {
            cancelled = true;
            break;
        }

        // Fill pipeline up to DOWNLOAD_PIPELINE_DEPTH
        while request_offset < total_bytes && inflight.len() < DOWNLOAD_PIPELINE_DEPTH {
            if cancellation_token.is_cancelled() {
                cancelled = true;
                break;
            }

            let to_read = ((total_bytes - request_offset) as usize).min(DOWNLOAD_CHUNK_SIZE) as u32;
            let raw_clone = raw.clone();
            let handle_clone = remote_handle.clone();
            let offset = request_offset;

            inflight.push(async move {
                let res = raw_clone.read(&handle_clone, offset, to_read).await;
                (res, offset)
            });

            request_offset += to_read as u64;
        }

        // Await next read response from pipeline
        if let Some((read_res, offset)) = inflight.next().await {
            match read_res {
                Ok(data_packet) => {
                    let bytes = data_packet.data;
                    let chunk_len = bytes.len() as u64;

                    // Write to local file at exact offset
                    local_file
                        .seek(SeekFrom::Start(offset))
                        .await
                        .map_err(|e| SftpError::IoError {
                            message: format!("Failed to seek in local file: {}", e),
                        })?;

                    local_file
                        .write_all(&bytes)
                        .await
                        .map_err(|e| SftpError::IoError {
                            message: format!("Failed to write to local file: {}", e),
                        })?;

                    tracker.add_bytes(chunk_len);
                    if let Some(snap) = tracker.check_emit(false) {
                        on_progress(snap);
                    }
                }
                Err(e) => {
                    let _ = raw.close(&remote_handle).await;
                    return Err(SftpError::from_russh_sftp(e, Some(&options.remote_path)));
                }
            }
        }
    }

    // Close remote handle
    let _ = raw.close(remote_handle).await;
    let _ = local_file.flush().await;

    if cancelled {
        return Ok(DownloadResult {
            final_path: options.local_path,
            partial_path,
            bytes_transferred: tracker.current_bytes(),
            total_bytes,
            completed: false,
        });
    }

    // Final progress emit
    if let Some(snap) = tracker.check_emit(true) {
        on_progress(snap);
    }

    // Atomic rename local partial -> destination
    tokio::fs::rename(&partial_path, &options.local_path)
        .await
        .map_err(|e| SftpError::IoError {
            message: format!(
                "Failed to rename local partial file {} to {}: {}",
                partial_path, options.local_path, e
            ),
        })?;

    Ok(DownloadResult {
        final_path: options.local_path,
        partial_path,
        bytes_transferred: total_bytes,
        total_bytes,
        completed: true,
    })
}
