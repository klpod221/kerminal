use serde::{Deserialize, Serialize};

/// Request for getting terminal buffer
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GetTerminalBufferRequest {
    #[serde(rename = "terminalId")]
    pub terminal_id: String,
}

/// Request for getting terminal buffer chunk
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GetTerminalBufferChunkRequest {
    #[serde(rename = "terminalId")]
    pub terminal_id: String,
    #[serde(rename = "startLine")]
    pub start_line: usize,
    #[serde(rename = "chunkSize")]
    pub chunk_size: usize,
}

/// Terminal buffer chunk response
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TerminalBufferChunk {
    #[serde(rename = "terminalId")]
    pub terminal_id: String,
    #[serde(rename = "startLine")]
    pub start_line: usize,
    #[serde(rename = "endLine")]
    pub end_line: usize,
    #[serde(rename = "totalLines")]
    pub total_lines: usize,
    #[serde(rename = "data")]
    pub data: String,
    #[serde(rename = "hasMore")]
    pub has_more: bool,
}

/// Request for checking if terminal has buffer
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HasTerminalBufferRequest {
    #[serde(rename = "terminalId")]
    pub terminal_id: String,
}

/// Request for cleaning up terminal buffers
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CleanupTerminalBuffersRequest {
    #[serde(rename = "activeTerminalIds")]
    pub active_terminal_ids: Vec<String>,
}
