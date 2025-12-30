use serde::{Deserialize, Serialize};
use std::collections::HashMap;

use crate::models::ssh::config_host::SSHConfigHost;

/// Represents the type of terminal connection
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum TerminalType {
    Local,
    SSH,
    SSHConfig,
}

/// Configuration for local terminal
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
#[derive(Default)]
pub struct LocalConfig {
    pub shell: Option<String>,
    pub working_dir: Option<String>,
    pub env_vars: Option<HashMap<String, String>>,
    pub command: Option<String>,
}

/// Terminal configuration that can be either Local or SSH with profile ID
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct TerminalConfig {
    pub terminal_type: TerminalType,
    pub local_config: Option<LocalConfig>,
    pub ssh_profile_id: Option<String>,
    pub terminal_profile_id: Option<String>,
    pub ssh_config_host: Option<SSHConfigHost>,
    pub ssh_config_password: Option<String>,
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
#[serde(rename_all = "camelCase")]
pub struct TerminalInfo {
    pub id: String,
    pub config: TerminalConfig,
    pub state: TerminalState,
    pub created_at: chrono::DateTime<chrono::Utc>,
    pub title: Option<String>,
}

/// Data structure for terminal input/output
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
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
#[serde(rename_all = "camelCase")]
pub struct CreateTerminalResponse {
    pub terminal_id: String,
    pub info: TerminalInfo,
}

/// Request to write data to terminal
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct WriteTerminalRequest {
    pub terminal_id: String,
    pub data: String,
}

/// Request to write data to multiple terminals (batch operation)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WriteBatchTerminalRequest {
    pub requests: Vec<WriteTerminalRequest>,
}

/// Request to resize terminal
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ResizeTerminalRequest {
    pub terminal_id: String,
    pub cols: u16,
    pub rows: u16,
}

/// Event when terminal title changes
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct TerminalTitleChanged {
    pub terminal_id: String,
    pub title: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct TerminalExited {
    pub terminal_id: String,
    pub exit_code: Option<i32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub reason: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct TerminalLatency {
    pub terminal_id: String,
    pub latency_ms: u64,
}

/// Event for graphics protocol data (Sixel)
#[allow(dead_code)]
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct TerminalGraphicsData {
    pub terminal_id: String,
    pub protocol: String, // "sixel"
    pub width: Option<u32>,
    pub height: Option<u32>,
    pub data: Vec<u8>, // Sixel sequence data
}
