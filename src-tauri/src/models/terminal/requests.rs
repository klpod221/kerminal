use serde::{Deserialize, Serialize};

/// Request for creating a new SSH terminal
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct CreateSshTerminalRequest {
    pub profile_id: String,
}

/// Request for creating a local terminal
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct CreateLocalTerminalRequest {
    pub shell: Option<String>,
    pub working_dir: Option<String>,
    pub title: Option<String>,
}

/// Request for closing a terminal
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct CloseTerminalRequest {
    pub terminal_id: String,
}

/// Request for getting terminal info
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct GetTerminalInfoRequest {
    pub terminal_id: String,
}
