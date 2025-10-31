use serde::{Deserialize, Serialize};

/// Transfer progress information
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct TransferProgress {
    /// Unique transfer ID
    pub transfer_id: String,
    /// Transfer status
    pub status: TransferStatus,
    /// Transfer direction (upload or download)
    pub direction: TransferDirection,
    /// Local file path
    pub local_path: String,
    /// Remote file path
    pub remote_path: String,
    /// Total size in bytes
    pub total_bytes: u64,
    /// Transferred bytes so far
    pub transferred_bytes: u64,
    /// Transfer speed in bytes per second
    pub speed_bytes_per_sec: Option<u64>,
    /// Estimated time remaining in seconds
    pub eta_seconds: Option<u64>,
    /// Error message if transfer failed
    pub error: Option<String>,
    /// Timestamp when transfer started
    pub started_at: chrono::DateTime<chrono::Utc>,
    /// Timestamp when transfer completed (or None if still in progress)
    pub completed_at: Option<chrono::DateTime<chrono::Utc>>,
}

/// Transfer status
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "lowercase")]
pub enum TransferStatus {
    /// Transfer is queued
    Queued,
    /// Transfer is in progress
    InProgress,
    /// Transfer is paused
    Paused,
    /// Transfer completed successfully
    Completed,
    /// Transfer failed
    Failed,
    /// Transfer was cancelled
    Cancelled,
}

/// Transfer direction
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "lowercase")]
pub enum TransferDirection {
    /// Uploading from local to remote
    Upload,
    /// Downloading from remote to local
    Download,
}

impl TransferProgress {
    /// Check if transfer is active (queued or in progress)
    pub fn is_active(&self) -> bool {
        matches!(
            self.status,
            TransferStatus::Queued | TransferStatus::InProgress
        )
    }
}

