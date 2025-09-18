use serde::{Deserialize, Serialize};

/// Request for getting terminal buffer
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GetTerminalBufferRequest {
    #[serde(rename = "terminalId")]
    pub terminal_id: String,
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
