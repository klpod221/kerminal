use serde::{Deserialize, Serialize};

/// Request for creating a new SSH terminal
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CreateSshTerminalRequest {
    #[serde(rename = "profileId")]
    pub profile_id: String,
}

/// Request for creating a local terminal
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CreateLocalTerminalRequest {
    pub shell: Option<String>,
    #[serde(rename = "workingDir")]
    pub working_dir: Option<String>,
    pub title: Option<String>,
}

/// Request for closing a terminal
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CloseTerminalRequest {
    #[serde(rename = "terminalId")]
    pub terminal_id: String,
}

/// Request for getting terminal info
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GetTerminalInfoRequest {
    #[serde(rename = "terminalId")]
    pub terminal_id: String,
}
