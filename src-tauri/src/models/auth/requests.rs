use serde::{Deserialize, Serialize};

/// Request for verifying master password
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct VerifyMasterPasswordRequest {
    pub password: String,
}

/// Request for changing master password
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ChangeMasterPasswordRequest {
    #[serde(rename = "oldPassword")]
    pub old_password: String,
    #[serde(rename = "newPassword")]
    pub new_password: String,
}

/// Request for updating master password configuration
#[allow(dead_code)]
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UpdateMasterPasswordConfigRequest {
    #[serde(rename = "autoUnlock")]
    pub auto_unlock: Option<bool>,
    #[serde(rename = "autoLockTimeout")]
    pub auto_lock_timeout: Option<u32>,
}
