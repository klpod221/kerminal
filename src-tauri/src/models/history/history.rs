use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

/// A single command history entry
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct CommandHistoryEntry {
    /// The command text
    pub command: String,
    /// Timestamp when the command was executed (if available)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub timestamp: Option<DateTime<Utc>>,
    /// Index in the history (for ordering)
    pub index: usize,
}

/// Request to get history for a terminal
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct GetTerminalHistoryRequest {
    pub terminal_id: String,
    /// Maximum number of entries to return (0 = all)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub limit: Option<usize>,
}

/// Request to search history
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct SearchHistoryRequest {
    pub terminal_id: String,
    /// Search query (case-insensitive)
    pub query: String,
    /// Maximum number of results to return (0 = all)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub limit: Option<usize>,
}

/// Response for search history
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct SearchHistoryResponse {
    pub entries: Vec<CommandHistoryEntry>,
    pub total_count: usize,
}

/// Request to export history
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ExportHistoryRequest {
    pub terminal_id: String,
    /// Export format: "json" or "txt"
    pub format: String,
    /// File path to export to
    pub file_path: String,
    /// Optional search query to filter before export
    #[serde(skip_serializing_if = "Option::is_none")]
    pub query: Option<String>,
}
