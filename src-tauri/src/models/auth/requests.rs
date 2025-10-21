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
