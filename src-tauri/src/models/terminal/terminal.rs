use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// Represents the type of terminal connection
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum TerminalType {
    Local,
    SSH,
}

/// Configuration for local terminal
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LocalConfig {
    pub shell: Option<String>,
    pub working_dir: Option<String>,
    pub env_vars: Option<HashMap<String, String>>,
}

impl Default for LocalConfig {
    fn default() -> Self {
        Self {
            shell: None,
            working_dir: None,
            env_vars: None,
        }
    }
}

/// Terminal configuration that can be either Local or SSH with profile ID
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TerminalConfig {
    pub terminal_type: TerminalType,
    pub local_config: Option<LocalConfig>,
    pub ssh_profile_id: Option<String>, // ID of SSH profile instead of direct config
}

/// Represents the current state of a terminal
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum TerminalState {
    Connecting,
    Connected,
    Disconnected,
    Error(String),
}

/// Information about a terminal instance
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TerminalInfo {
    pub id: String,
    pub config: TerminalConfig,
    pub state: TerminalState,
    pub created_at: chrono::DateTime<chrono::Utc>,
    pub title: Option<String>,
}

/// Data structure for terminal input/output
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TerminalData {
    pub terminal_id: String,
    pub data: Vec<u8>,
}

/// Request to create a new terminal
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CreateTerminalRequest {
    pub config: TerminalConfig,
    pub title: Option<String>,
}

/// Response when creating a terminal
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CreateTerminalResponse {
    pub terminal_id: String,
    pub info: TerminalInfo,
}

/// Request to write data to terminal
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WriteTerminalRequest {
    pub terminal_id: String,
    pub data: String,
}

/// Request to resize terminal
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ResizeTerminalRequest {
    pub terminal_id: String,
    pub cols: u16,
    pub rows: u16,
}

/// Event when terminal title changes
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TerminalTitleChanged {
    pub terminal_id: String,
    pub title: String,
}

/// Event when terminal exits
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TerminalExited {
    pub terminal_id: String,
    pub exit_code: Option<i32>,
}
