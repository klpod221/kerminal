
use serde::{Deserialize, Serialize};

use crate::{
    database::{
        error::DatabaseResult,
        traits::{Encryptable, EncryptionService},
    },
    impl_syncable,
    models::base::BaseModel,
};

/// SSH Tunnel for port forwarding and SOCKS proxy
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct SSHTunnel {
    /// Base model with sync metadata
    #[serde(flatten)]
    pub base: BaseModel,

    /// Tunnel identification
    pub name: String,
    pub description: Option<String>,

    /// SSH profile reference
    pub profile_id: String,

    /// Tunnel configuration
    pub tunnel_type: TunnelType,
    pub local_host: String, // Usually "127.0.0.1" or "0.0.0.0"
    pub local_port: u16,

    /// Remote configuration (not used for Dynamic tunnels)
    pub remote_host: Option<String>,
    pub remote_port: Option<u16>,

    /// Auto-start configuration
    pub auto_start: bool,

    /// Runtime status (not persisted, used for UI)
    #[serde(skip)]
    pub status: TunnelStatus,

    /// Last error message (not persisted, used for UI)
    #[serde(skip)]
    pub error_message: Option<String>,
}

/// Types of SSH tunnels
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "PascalCase")]
pub enum TunnelType {
    /// Local port forwarding (-L)
    /// Local port -> Remote host:port via SSH server
    Local,
    /// Remote port forwarding (-R)
    /// SSH server port -> Local host:port
    Remote,
    /// Dynamic port forwarding (-D)
    /// SOCKS5 proxy via SSH server
    Dynamic,
}

/// Runtime status of tunnel
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "lowercase")]
#[derive(Default)]
pub enum TunnelStatus {
    #[default]
    Stopped,
    Starting,
    Running,
    Error,
}


impl SSHTunnel {
    /// Create a new SSH tunnel
    pub fn new(
        device_id: String,
        name: String,
        profile_id: String,
        tunnel_type: TunnelType,
        local_host: String,
        local_port: u16,
        remote_host: Option<String>,
        remote_port: Option<u16>,
    ) -> Self {
        Self {
            base: BaseModel::new(device_id),
            name,
            description: None,
            profile_id,
            tunnel_type,
            local_host,
            local_port,
            remote_host,
            remote_port,
            auto_start: false,
            status: TunnelStatus::default(),
            error_message: None,
        }
    }

    /// Validate tunnel configuration
    pub fn validate(&self) -> Result<(), String> {
        // Validate name
        if self.name.trim().is_empty() {
            return Err("Tunnel name cannot be empty".to_string());
        }

        // Validate port ranges
        if self.local_port == 0 {
            return Err("Local port cannot be 0".to_string());
        }

        // Validate remote configuration for Local and Remote tunnels
        match self.tunnel_type {
            TunnelType::Local | TunnelType::Remote => {
                if self.remote_host.is_none() || self.remote_port.is_none() {
                    return Err(
                        "Remote host and port are required for Local and Remote tunnels"
                            .to_string(),
                    );
                }
                if let Some(port) = self.remote_port {
                    if port == 0 {
                        return Err("Remote port cannot be 0".to_string());
                    }
                }
                if let Some(host) = &self.remote_host {
                    if host.trim().is_empty() {
                        return Err("Remote host cannot be empty".to_string());
                    }
                }
            }
            TunnelType::Dynamic => {
                // Dynamic tunnels don't need remote configuration
            }
        }

        // Validate profile_id
        if self.profile_id.trim().is_empty() {
            return Err("Profile ID cannot be empty".to_string());
        }

        Ok(())
    }

    /// Get display string for tunnel configuration
    pub fn get_tunnel_config_display(&self) -> String {
        match self.tunnel_type {
            TunnelType::Local => {
                format!(
                    "{}:{} -> {}:{}",
                    self.local_host,
                    self.local_port,
                    self.remote_host.as_deref().unwrap_or(""),
                    self.remote_port.unwrap_or(0)
                )
            }
            TunnelType::Remote => {
                format!(
                    "Remote:{} -> {}:{}",
                    self.local_port,
                    self.remote_host.as_deref().unwrap_or(""),
                    self.remote_port.unwrap_or(0)
                )
            }
            TunnelType::Dynamic => {
                format!("SOCKS5 {}:{}", self.local_host, self.local_port)
            }
        }
    }

    /// Check if tunnel is active (starting or running)
    pub fn is_active(&self) -> bool {
        matches!(self.status, TunnelStatus::Starting | TunnelStatus::Running)
    }
}

/// Request to create a new SSH tunnel
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct CreateSSHTunnelRequest {
    pub name: String,
    pub description: Option<String>,
    pub profile_id: String,
    pub tunnel_type: TunnelType,
    pub local_host: String,
    pub local_port: u16,
    pub remote_host: Option<String>,
    pub remote_port: Option<u16>,
    pub auto_start: Option<bool>,
}

/// Request to update an existing SSH tunnel
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct UpdateSSHTunnelRequest {
    pub name: Option<String>,
    pub description: Option<String>,
    pub profile_id: Option<String>,
    pub tunnel_type: Option<TunnelType>,
    pub local_host: Option<String>,
    pub local_port: Option<u16>,
    pub remote_host: Option<String>,
    pub remote_port: Option<u16>,
    pub auto_start: Option<bool>,
}

// Implement Syncable trait using the macro
impl_syncable!(SSHTunnel, "ssh_tunnels");

// Implement Encryptable trait (no encrypted fields for tunnels)
impl Encryptable for SSHTunnel {
    fn encrypted_fields() -> Vec<&'static str> {
        vec![] // No encrypted fields
    }

    fn encrypt_fields(
        &mut self,
        _encryption_service: &dyn EncryptionService,
    ) -> DatabaseResult<()> {
        // No encrypted fields in SSH tunnels
        Ok(())
    }

    fn decrypt_fields(
        &mut self,
        _encryption_service: &dyn EncryptionService,
    ) -> DatabaseResult<()> {
        // No encrypted fields in SSH tunnels
        Ok(())
    }

    fn has_encrypted_data(&self) -> bool {
        false
    }

    fn encryption_device_id(&self) -> Option<&str> {
        Some(&self.base.device_id)
    }
}

impl Encryptable for CreateSSHTunnelRequest {
    fn encrypted_fields() -> Vec<&'static str> {
        vec![] // No encrypted fields
    }

    fn encrypt_fields(
        &mut self,
        _encryption_service: &dyn EncryptionService,
    ) -> DatabaseResult<()> {
        Ok(())
    }

    fn decrypt_fields(
        &mut self,
        _encryption_service: &dyn EncryptionService,
    ) -> DatabaseResult<()> {
        Ok(())
    }

    fn has_encrypted_data(&self) -> bool {
        false
    }

    fn encryption_device_id(&self) -> Option<&str> {
        None
    }
}

impl Encryptable for UpdateSSHTunnelRequest {
    fn encrypted_fields() -> Vec<&'static str> {
        vec![] // No encrypted fields
    }

    fn encrypt_fields(
        &mut self,
        _encryption_service: &dyn EncryptionService,
    ) -> DatabaseResult<()> {
        Ok(())
    }

    fn decrypt_fields(
        &mut self,
        _encryption_service: &dyn EncryptionService,
    ) -> DatabaseResult<()> {
        Ok(())
    }

    fn has_encrypted_data(&self) -> bool {
        false
    }

    fn encryption_device_id(&self) -> Option<&str> {
        None
    }
}

/// Tunnel manager response with status
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct TunnelWithStatus {
    #[serde(flatten)]
    pub tunnel: SSHTunnel,
    pub status: TunnelStatus,
    pub error_message: Option<String>,
}

impl From<SSHTunnel> for TunnelWithStatus {
    fn from(tunnel: SSHTunnel) -> Self {
        Self {
            status: tunnel.status.clone(),
            error_message: tunnel.error_message.clone(),
            tunnel,
        }
    }
}
