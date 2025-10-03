use serde::{Deserialize, Serialize};

/// Request for getting terminal buffer
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct GetTerminalBufferRequest {
    pub terminal_id: String,
}

/// Request for getting terminal buffer chunk
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct GetTerminalBufferChunkRequest {
    pub terminal_id: String,
    pub start_line: usize,
    pub chunk_size: usize,
}

/// Terminal buffer chunk response
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct TerminalBufferChunk {
    pub terminal_id: String,
    pub start_line: usize,
    pub end_line: usize,
    pub total_lines: usize,
    pub data: String,
    pub has_more: bool,
}

/// Request for checking if terminal has buffer
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct HasTerminalBufferRequest {
    pub terminal_id: String,
}

/// Request for cleaning up terminal buffers
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct CleanupTerminalBuffersRequest {
    pub active_terminal_ids: Vec<String>,
}
