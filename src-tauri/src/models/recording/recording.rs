use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct SessionRecording {
    pub id: String,
    pub terminal_id: String,
    pub session_name: String,
    pub terminal_type: String, // "Local" | "SSH"
    pub started_at: DateTime<Utc>,
    pub ended_at: Option<DateTime<Utc>>,
    pub duration_ms: Option<i64>,
    pub file_path: String,
    pub file_size: i64,
    pub width: u16,
    pub height: u16,
    pub metadata: Option<String>, // JSON
    pub created_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AsciicastHeader {
    pub version: u8,
    pub width: u16,
    pub height: u16,
    pub timestamp: Option<i64>,
    pub title: Option<String>,
    pub env: Option<serde_json::Value>,
}
