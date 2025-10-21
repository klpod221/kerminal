use serde::{Deserialize, Serialize};

use crate::{
    database::{
        error::DatabaseResult,
        traits::{Encryptable, EncryptionService},
    },
    impl_syncable,
    models::base::BaseModel,
};

/// SSH Profile with flexible authentication methods
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct SSHProfile {
    /// Base model with sync metadata
    #[serde(flatten)]
    pub base: BaseModel,

    /// Profile information
    pub name: String,
    pub host: String,
    pub port: u16,
    pub username: String,

    /// Group association (None = ungrouped)
    pub group_id: Option<String>,

    /// Authentication method and data
    pub auth_method: AuthMethod,
    pub auth_data: AuthData,

    /// Connection settings
    pub timeout: Option<u32>, // seconds
    pub keep_alive: bool,
    pub compression: bool,

    /// Proxy settings
    pub proxy: Option<ProxyConfig>,

    /// UI customization
    pub color: Option<String>, // Hex color

    /// Notes
    pub description: Option<String>,
}

/// Authentication methods supported
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "PascalCase")]
pub enum AuthMethod {
    /// Password authentication
    Password,
    /// Reference to stored SSH key (centralized key management)
    KeyReference,
}

/// Proxy configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ProxyConfig {
    pub proxy_type: ProxyType,
    pub host: String,
    pub port: u16,
    #[serde(default)]
    pub username: Option<String>,
    #[serde(default, with = "encrypted_option_string")]
    pub password: Option<String>,
}

/// Proxy types supported
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum ProxyType {
    Http,
    Socks5,
    Socks4,
}

impl std::fmt::Display for ProxyType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            ProxyType::Http => write!(f, "HTTP"),
            ProxyType::Socks4 => write!(f, "SOCKS4"),
            ProxyType::Socks5 => write!(f, "SOCKS5"),
        }
    }
}

/// Authentication data - encrypted fields marked with [encrypt]
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum AuthData {
    Password {
        #[serde(with = "encrypted_string")]
        password: String,
    },
    /// Reference to a stored SSH key (centralized key management)
    KeyReference {
        #[serde(rename = "keyId")]
        key_id: String, // Reference to ssh_keys table
    },
}

/// SSH key types
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum KeyType {
    RSA,
    Ed25519,
    ECDSA,
    DSA,
}

#[allow(dead_code)]
impl SSHProfile {
    /// Create a new SSH profile
    pub fn new(device_id: String, name: String, host: String, port: u16, username: String) -> Self {
        Self {
            base: BaseModel::new(device_id),
            name,
            host,
            port,
            username,
            group_id: None,
            auth_method: AuthMethod::Password,
            auth_data: AuthData::Password {
                password: String::new(),
            },
            timeout: Some(30),
            keep_alive: true,
            compression: false,
            proxy: None,
            color: None,
            description: None,
        }
    }

    /// Get profile by ID
    pub fn get_id(&self) -> &str {
        &self.base.id
    }

    /// Set proxy configuration
    pub fn set_proxy(&mut self, proxy: Option<ProxyConfig>) {
        self.proxy = proxy;
        self.base.touch();
    }

    /// Set authentication method and data
    pub fn set_authentication(&mut self, method: AuthMethod, data: AuthData) {
        self.auth_method = method;
        self.auth_data = data;
        self.base.touch();
    }

    /// Move to group
    pub fn set_group(&mut self, group_id: Option<String>) {
        self.group_id = group_id;
        self.base.touch();
    }

    /// Update connection settings
    pub fn set_connection_settings(
        &mut self,
        timeout: Option<u32>,
        keep_alive: bool,
        compression: bool,
    ) {
        self.timeout = timeout;
        self.keep_alive = keep_alive;
        self.compression = compression;
        self.base.touch();
    }

    /// Check if profile has valid authentication data
    pub fn has_valid_auth(&self) -> bool {
        match (&self.auth_method, &self.auth_data) {
            (AuthMethod::Password, AuthData::Password { password }) => !password.is_empty(),
            (AuthMethod::KeyReference, AuthData::KeyReference { key_id }) => !key_id.is_empty(),
            _ => false,
        }
    }

    /// Get display name for UI
    pub fn display_name(&self) -> String {
        if self.name.is_empty() {
            format!("{}@{}", self.username, self.host)
        } else {
            self.name.clone()
        }
    }

    /// Get connection endpoint
    pub fn endpoint(&self) -> String {
        if self.port == 22 {
            format!("{}@{}", self.username, self.host)
        } else {
            format!("{}@{}:{}", self.username, self.host, self.port)
        }
    }
}

// Implement Syncable trait using macro
impl_syncable!(SSHProfile, "ssh_profiles");

impl Encryptable for SSHProfile {
    fn encrypted_fields() -> Vec<&'static str> {
        vec!["auth_data"]
    }

    fn encrypt_fields(&mut self, encryption_service: &dyn EncryptionService) -> DatabaseResult<()> {
        // Encrypt auth_data fields based on auth method
        match &mut self.auth_data {
            AuthData::Password { password } => {
                // Encrypt password
                let encrypted = tokio::task::block_in_place(|| {
                    tokio::runtime::Handle::current().block_on(async {
                        encryption_service
                            .encrypt_string(password, Some(&self.base.device_id))
                            .await
                    })
                })?;
                *password = encrypted;
            }
            AuthData::KeyReference { .. } => {
                // No encryption needed for key references
            }
        }
        Ok(())
    }

    fn decrypt_fields(&mut self, encryption_service: &dyn EncryptionService) -> DatabaseResult<()> {
        // Decrypt auth_data fields based on auth method
        match &mut self.auth_data {
            AuthData::Password { password } => {
                // Decrypt password
                let decrypted = tokio::task::block_in_place(|| {
                    tokio::runtime::Handle::current().block_on(async {
                        encryption_service
                            .decrypt_string(password, Some(&self.base.device_id))
                            .await
                    })
                })?;
                *password = decrypted;
            }
            AuthData::KeyReference { .. } => {
                // No decryption needed for key references
            }
        }
        Ok(())
    }

    fn has_encrypted_data(&self) -> bool {
        match &self.auth_data {
            AuthData::Password { .. } => true,
            AuthData::KeyReference { .. } => false,
        }
    }

    fn encryption_device_id(&self) -> Option<&str> {
        Some(&self.base.device_id)
    }
}

/// Request to create new SSH profile
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct CreateSSHProfileRequest {
    pub name: String,
    pub host: String,
    pub port: Option<u16>,
    pub username: String,
    pub group_id: Option<String>,
    pub auth_method: AuthMethod,
    pub auth_data: AuthData,
    pub timeout: Option<u32>,
    pub keep_alive: Option<bool>,
    pub compression: Option<bool>,
    pub proxy: Option<ProxyConfig>,
    pub color: Option<String>,
    pub description: Option<String>,
}

impl CreateSSHProfileRequest {
    pub fn to_profile(self, device_id: String) -> SSHProfile {
        let mut profile = SSHProfile::new(
            device_id,
            self.name,
            self.host,
            self.port.unwrap_or(22),
            self.username,
        );

        profile.group_id = self.group_id;
        profile.auth_method = self.auth_method;
        profile.auth_data = self.auth_data;
        profile.timeout = self.timeout.or(Some(30));
        profile.keep_alive = self.keep_alive.unwrap_or(true);
        profile.compression = self.compression.unwrap_or(false);
        profile.color = self.color;
        profile.description = self.description;

        profile
    }
}

/// Request to update SSH profile
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct UpdateSSHProfileRequest {
    pub name: Option<String>,
    pub host: Option<String>,
    pub port: Option<u16>,
    pub username: Option<String>,
    pub group_id: Option<Option<String>>, // None = no change, Some(None) = remove from group
    pub auth_method: Option<AuthMethod>,
    pub auth_data: Option<AuthData>,
    pub timeout: Option<Option<u32>>,
    pub keep_alive: Option<bool>,
    pub compression: Option<bool>,
    pub color: Option<Option<String>>,
    pub description: Option<Option<String>>,
}

/// Request to test SSH connection (minimal required fields)
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct TestSSHConnectionRequest {
    pub host: String,
    pub port: u16,
    pub username: String,
    pub auth_method: AuthMethod,
    pub auth_data: AuthData,
    pub timeout: Option<u32>,
    pub keep_alive: bool,
    pub compression: bool,
    pub proxy: Option<ProxyConfig>,
}

impl TestSSHConnectionRequest {
    /// Convert to a temporary SSHProfile for testing
    pub fn to_profile(self, device_id: String) -> SSHProfile {
        SSHProfile {
            base: BaseModel::new(device_id),
            name: format!("test-{}", self.host),
            host: self.host,
            port: self.port,
            username: self.username,
            group_id: None,
            auth_method: self.auth_method,
            auth_data: self.auth_data,
            timeout: self.timeout,
            keep_alive: self.keep_alive,
            compression: self.compression,
            proxy: self.proxy,
            color: None,
            description: None,
        }
    }
}

impl UpdateSSHProfileRequest {
    pub fn apply_to_profile(self, profile: &mut SSHProfile) {
        if let Some(name) = self.name {
            profile.name = name;
        }
        if let Some(host) = self.host {
            profile.host = host;
        }
        if let Some(port) = self.port {
            profile.port = port;
        }
        if let Some(username) = self.username {
            profile.username = username;
        }
        if let Some(group_id) = self.group_id {
            profile.group_id = group_id;
        }
        if let Some(auth_method) = self.auth_method {
            profile.auth_method = auth_method;
        }
        if let Some(auth_data) = self.auth_data {
            profile.auth_data = auth_data;
        }
        if let Some(timeout) = self.timeout {
            profile.timeout = timeout;
        }
        if let Some(keep_alive) = self.keep_alive {
            profile.keep_alive = keep_alive;
        }
        if let Some(compression) = self.compression {
            profile.compression = compression;
        }
        if let Some(color) = self.color {
            profile.color = color;
        }
        if let Some(description) = self.description {
            profile.description = description;
        }

        profile.base.touch();
    }
}

/// Module to handle encrypted string serialization
mod encrypted_string {
    use serde::{Deserialize, Deserializer, Serialize, Serializer};

    pub fn serialize<S>(value: &String, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        value.serialize(serializer)
    }

    pub fn deserialize<'de, D>(deserializer: D) -> Result<String, D::Error>
    where
        D: Deserializer<'de>,
    {
        String::deserialize(deserializer)
    }
}

mod encrypted_option_string {
    use serde::{Deserialize, Deserializer, Serialize, Serializer};

    pub fn serialize<S>(value: &Option<String>, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        // For security reasons, we might want to store passwords encrypted at rest
        // For now, serialize as-is to avoid complications with async encryption
        value.serialize(serializer)
    }

    pub fn deserialize<'de, D>(deserializer: D) -> Result<Option<String>, D::Error>
    where
        D: Deserializer<'de>,
    {
        // Deserialize as-is
        Option::<String>::deserialize(deserializer)
    }
}

impl Default for AuthMethod {
    fn default() -> Self {
        Self::Password
    }
}

impl Default for AuthData {
    fn default() -> Self {
        Self::Password {
            password: String::new(),
        }
    }
}

impl std::fmt::Display for AuthMethod {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            AuthMethod::Password => write!(f, "Password"),
            AuthMethod::KeyReference => write!(f, "SSH Key"),
        }
    }
}

impl std::fmt::Display for KeyType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            KeyType::RSA => write!(f, "RSA"),
            KeyType::Ed25519 => write!(f, "Ed25519"),
            KeyType::ECDSA => write!(f, "ECDSA"),
            KeyType::DSA => write!(f, "DSA"),
        }
    }
}

impl crate::database::sync::strategies::HasBaseModel for SSHProfile {
    fn base_model(&self) -> &crate::models::base::BaseModel {
        &self.base
    }
}
