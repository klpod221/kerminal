use serde::{Deserialize, Serialize};

/// Master password configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MasterPasswordConfig {
    pub auto_unlock: bool,
    pub session_timeout_minutes: Option<u32>,
    pub require_on_startup: bool,
    pub use_keychain: bool,
}

impl Default for MasterPasswordConfig {
    fn default() -> Self {
        Self {
            auto_unlock: false,
            session_timeout_minutes: Some(15),
            require_on_startup: true,
            use_keychain: true,
        }
    }
}
