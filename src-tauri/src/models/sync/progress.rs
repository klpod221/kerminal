use serde::{Deserialize, Serialize};

/// Sync progress event for real-time updates
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct SyncProgressEvent {
    /// Type of sync: "sftp" or "database"
    pub sync_type: String,
    /// Current operation: "comparing", "uploading", "downloading", "syncing"
    pub operation: String,
    /// Current file/table being processed
    pub current_item: String,
    /// Number of items processed
    pub processed: u32,
    /// Total number of items
    pub total: u32,
    /// Status: "in_progress", "completed", "error"
    pub status: String,
    /// Error message if any
    pub error: Option<String>,
}

impl SyncProgressEvent {
    pub fn sftp_progress(operation: &str, item: &str, processed: u32, total: u32) -> Self {
        Self {
            sync_type: "sftp".to_string(),
            operation: operation.to_string(),
            current_item: item.to_string(),
            processed,
            total,
            status: "in_progress".to_string(),
            error: None,
        }
    }

    pub fn sftp_completed(total: u32) -> Self {
        Self {
            sync_type: "sftp".to_string(),
            operation: "completed".to_string(),
            current_item: String::new(),
            processed: total,
            total,
            status: "completed".to_string(),
            error: None,
        }
    }

    pub fn sftp_error(error: &str) -> Self {
        Self {
            sync_type: "sftp".to_string(),
            operation: "error".to_string(),
            current_item: String::new(),
            processed: 0,
            total: 0,
            status: "error".to_string(),
            error: Some(error.to_string()),
        }
    }

    #[allow(dead_code)]
    pub fn database_progress(table: &str, processed: u32, total: u32) -> Self {
        Self {
            sync_type: "database".to_string(),
            operation: "syncing".to_string(),
            current_item: table.to_string(),
            processed,
            total,
            status: "in_progress".to_string(),
            error: None,
        }
    }

    #[allow(dead_code)]
    pub fn database_completed(total: u32) -> Self {
        Self {
            sync_type: "database".to_string(),
            operation: "completed".to_string(),
            current_item: String::new(),
            processed: total,
            total,
            status: "completed".to_string(),
            error: None,
        }
    }
}
