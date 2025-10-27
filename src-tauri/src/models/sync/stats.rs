use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

/// Sync statistics for monitoring
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct SyncStats {
    pub total_records: u32,
    pub synced_records: u32,
    pub pending_records: u32,
    pub failed_records: u32,
    pub conflicts: u32,
    pub last_sync: Option<DateTime<Utc>>,
    pub sync_enabled: bool,
    pub databases: Vec<String>,
}
