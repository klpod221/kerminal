use serde::{Deserialize, Serialize};

/// Request for verifying master password
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct VerifyMasterPasswordRequest {
    pub password: String,
}

/// Request for changing master password
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ChangeMasterPasswordRequest {
    pub old_password: String,
    pub new_password: String,
}

/// Request for updating master password configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct UpdateMasterPasswordConfigRequest {
    pub auto_unlock: Option<bool>,
    pub auto_lock_timeout: Option<u32>,
}
