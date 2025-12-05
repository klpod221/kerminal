use serde::{Deserialize, Serialize};
use std::collections::HashMap;

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct TerminalProfile {
    pub id: String,
    pub name: String,
    pub shell: String,
    pub working_dir: Option<String>,
    pub env: Option<HashMap<String, String>>,
    pub icon: Option<String>,
    pub color: Option<String>,
    pub command: Option<String>,
    pub created_at: i64,
    pub updated_at: i64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct CreateTerminalProfileRequest {
    pub name: String,
    pub shell: String,
    pub working_dir: Option<String>,
    pub env: Option<HashMap<String, String>>,
    pub icon: Option<String>,
    pub color: Option<String>,
    pub command: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct UpdateTerminalProfileRequest {
    pub name: Option<String>,
    pub shell: Option<String>,
    pub working_dir: Option<String>,
    pub env: Option<HashMap<String, String>>,
    pub icon: Option<String>,
    pub color: Option<String>,
    pub command: Option<String>,
}
