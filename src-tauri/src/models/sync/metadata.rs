use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

/// Sync operation status
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum SyncOperationStatus {
    InProgress,
    Completed,
    Failed,
    PartialSuccess,
}

/// Sync statistics to display in UI
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct SyncStats {
    pub total_records: u32,
    pub synced_records: u32,
    pub pending_records: u32,
    pub failed_records: u32,
    pub conflicts: u32,
    pub last_sync: Option<DateTime<Utc>>,
    pub sync_enabled: bool,
    pub databases: Vec<DatabaseSyncStats>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DatabaseSyncStats {
    pub database_id: String,
    pub database_name: String,
    pub last_sync: Option<DateTime<Utc>>,
    pub records_synced: u32,
    pub sync_status: SyncOperationStatus,
    pub error_message: Option<String>,
}

impl std::fmt::Display for SyncOperationStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            SyncOperationStatus::InProgress => write!(f, "In Progress"),
            SyncOperationStatus::Completed => write!(f, "Completed"),
            SyncOperationStatus::Failed => write!(f, "Failed"),
            SyncOperationStatus::PartialSuccess => write!(f, "Partial Success"),
        }
    }
}
