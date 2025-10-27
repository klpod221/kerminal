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
    pub manual_conflicts: i32,
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

impl std::fmt::Display for SyncDirection {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            SyncDirection::Push => write!(f, "Push"),
            SyncDirection::Pull => write!(f, "Pull"),
            SyncDirection::Bidirectional => write!(f, "Bidirectional"),
        }
    }
}

impl std::str::FromStr for SyncDirection {
    type Err = String;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            "Push" => Ok(SyncDirection::Push),
            "Pull" => Ok(SyncDirection::Pull),
            "Bidirectional" | "Both" => Ok(SyncDirection::Bidirectional),
            _ => Err(format!("Unknown sync direction: {}", s)),
        }
    }
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
