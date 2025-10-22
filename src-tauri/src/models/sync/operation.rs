use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "camelCase")]
pub enum SyncOperationType {
    Create,
    Update,
    Delete,
    Sync,
}

impl std::fmt::Display for SyncOperationType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            SyncOperationType::Create => write!(f, "Create"),
            SyncOperationType::Update => write!(f, "Update"),
            SyncOperationType::Delete => write!(f, "Delete"),
            SyncOperationType::Sync => write!(f, "Sync"),
        }
    }
}

impl std::str::FromStr for SyncOperationType {
    type Err = String;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            "Create" => Ok(SyncOperationType::Create),
            "Update" => Ok(SyncOperationType::Update),
            "Delete" => Ok(SyncOperationType::Delete),
            "Sync" => Ok(SyncOperationType::Sync),
            _ => Err(format!("Unknown sync operation type: {}", s)),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "camelCase")]
pub enum SyncOperationStatus {
    Pending,
    InProgress,
    Completed,
    Failed,
}

impl std::fmt::Display for SyncOperationStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            SyncOperationStatus::Pending => write!(f, "Pending"),
            SyncOperationStatus::InProgress => write!(f, "InProgress"),
            SyncOperationStatus::Completed => write!(f, "Completed"),
            SyncOperationStatus::Failed => write!(f, "Failed"),
        }
    }
}

impl std::str::FromStr for SyncOperationStatus {
    type Err = String;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            "Pending" => Ok(SyncOperationStatus::Pending),
            "InProgress" => Ok(SyncOperationStatus::InProgress),
            "Completed" => Ok(SyncOperationStatus::Completed),
            "Failed" => Ok(SyncOperationStatus::Failed),
            _ => Err(format!("Unknown sync operation status: {}", s)),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct SyncOperation {
    pub id: String,
    pub operation_type: SyncOperationType,
    pub entity_type: String,
    pub entity_id: String,
    pub source_db: String,
    pub target_db: String,
    pub status: SyncOperationStatus,
    pub error_message: Option<String>,
    pub started_at: DateTime<Utc>,
    pub completed_at: Option<DateTime<Utc>>,
}
