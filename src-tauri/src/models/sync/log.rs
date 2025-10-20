use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

/// Sync log entry
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct SyncLog {
    pub id: String,
    pub database_id: String,
    pub device_id: String,
    pub direction: SyncDirection,
    pub status: SyncStatus,
    pub started_at: DateTime<Utc>,
    pub completed_at: Option<DateTime<Utc>>,
    pub records_synced: i32,
    pub conflicts_resolved: i32,
    pub error_message: Option<String>,
}

/// Sync direction
#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub enum SyncDirection {
    Push,
    Pull,
    Bidirectional,
}

/// Sync status
#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub enum SyncStatus {
    InProgress,
    Completed,
    Failed,
    Cancelled,
}
