use serde::{Deserialize, Serialize};

use super::profile::{AuthData, AuthMethod, KeyType, SSHProfile};
use crate::database::traits::SyncStatus;

/**
 * Represents a host entry from SSH config file (~/.ssh/config)
 * This is read-only data that reflects the user's SSH configuration
 */
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct SSHConfigHost {
    /// Host name/alias from config
    pub name: String,

    /// Hostname or IP address
    pub hostname: String,

    /// Port number (default: 22)
    pub port: u16,

    /// Username for connection
    pub user: Option<String>,

    /// Path to identity file (private key)
    pub identity_file: Option<String>,

    /// ProxyJump configuration
    pub proxy_jump: Option<String>,

    /// ProxyCommand configuration
    pub proxy_command: Option<String>,

    /// Whether to use agent forwarding
    pub forward_agent: Option<bool>,

    /// Additional options from config
    pub other_options: Option<String>,
}

impl SSHConfigHost {
    /**
     * Convert SSHConfigHost to a temporary SSHProfile for connection
     * This creates a profile that's not stored in the database
     * @param password - Optional password for authentication (used when no identity file)
     */
    pub fn to_temporary_profile(&self, password: Option<String>) -> Result<SSHProfile, String> {
        use chrono::Utc;
        use uuid::Uuid;

        use crate::models::base::BaseModel;

        let username = self
            .user
            .clone()
            .ok_or_else(|| "Username is required".to_string())?;

        let (auth_method, auth_data) = if let Some(identity_file) = &self.identity_file {
            // Has key file - use certificate auth
            let expanded_path = if identity_file.starts_with("~/") {
                let home = std::env::var("HOME")
                    .map_err(|_| "Cannot determine HOME directory".to_string())?;
                identity_file.replacen("~", &home, 1)
            } else {
                identity_file.clone()
            };

            let private_key = std::fs::read_to_string(&expanded_path)
                .map_err(|e| format!("Failed to read identity file '{}': {}", expanded_path, e))?;

            let key_type = if private_key.contains("BEGIN OPENSSH PRIVATE KEY")
                || private_key.contains("ssh-ed25519")
            {
                KeyType::Ed25519
            } else if private_key.contains("BEGIN EC PRIVATE KEY") {
                KeyType::ECDSA
            } else {
                KeyType::RSA
            };

            (
                AuthMethod::Certificate,
                AuthData::Certificate {
                    certificate: String::new(),
                    private_key,
                    key_type,
                    validity_period: None,
                },
            )
        } else {
            // No key file - use password auth
            let pwd =
                password.ok_or_else(|| "Password is required for authentication".to_string())?;

            (AuthMethod::Password, AuthData::Password { password: pwd })
        };

        Ok(SSHProfile {
            base: BaseModel {
                id: Uuid::new_v4().to_string(),
                created_at: Utc::now(),
                updated_at: Utc::now(),
                device_id: "temp".to_string(),
                version: 0,
                sync_status: SyncStatus::Synced,
            },
            name: self.name.clone(),
            host: self.hostname.clone(),
            port: self.port,
            username,
            group_id: None,
            auth_method,
            auth_data,
            timeout: Some(30),
            keep_alive: true,
            compression: false,
            proxy: None,
            jump_hosts: None,
            color: None,
            description: Some(format!("Temporary profile from SSH config: {}", self.name)),
            command: None,
            working_dir: None,
            env: None,
        })
    }

    /**
     * Check if this host requires password authentication
     * Returns true if no identity file is configured
     */
    pub fn requires_password(&self) -> bool {
        self.identity_file.is_none()
    }
}
