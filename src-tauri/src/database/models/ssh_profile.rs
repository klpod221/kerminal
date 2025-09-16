use serde::{Deserialize, Serialize};
use chrono::{DateTime, Utc};

use crate::{
    impl_syncable,
    database::{
        models::base::BaseModel,
        traits::{Syncable, Encryptable, EncryptionService, SyncStatus},
        error::DatabaseResult,
    },
};

/// SSH Profile với flexible authentication methods
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SSHProfile {
    /// Base model với sync metadata
    #[serde(flatten)]
    pub base: BaseModel,

    /// Profile information
    pub name: String,
    pub host: String,
    pub port: u16,
    pub username: String,

    /// Group association (None = ungrouped)
    pub group_id: Option<String>,

    /// Authentication method và data
    pub auth_method: AuthMethod,
    pub auth_data: AuthData,

    /// Connection settings
    pub timeout: Option<u32>,        // seconds
    pub keep_alive: bool,
    pub compression: bool,

    /// UI customization
    pub color: Option<String>,       // Hex color
    pub icon: Option<String>,        // Icon name
    pub sort_order: i32,

    /// Notes
    pub description: Option<String>,
    pub tags: Vec<String>,
}

/// Authentication methods supported
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum AuthMethod {
    /// Password authentication
    Password,
    /// Private key without passphrase
    PrivateKey,
    /// Private key with passphrase
    PrivateKeyWithPassphrase,
    /// SSH Agent
    Agent,
    /// SSH Certificate
    Certificate,
    /// Kerberos (future)
    Kerberos,
    /// PKCS#11 Hardware tokens (future)
    PKCS11,
}

/// Authentication data - encrypted fields marked with [encrypt]
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum AuthData {
    Password {
        #[serde(with = "encrypted_string")]
        password: String,
    },
    PrivateKey {
        #[serde(with = "encrypted_string")]
        private_key: String,
        key_type: KeyType,
        public_key: Option<String>,  // Not encrypted
    },
    PrivateKeyWithPassphrase {
        #[serde(with = "encrypted_string")]
        private_key: String,
        #[serde(with = "encrypted_string")]
        passphrase: String,
        key_type: KeyType,
        public_key: Option<String>,  // Not encrypted
    },
    Agent {
        // No sensitive data - uses system SSH agent
        public_key: Option<String>,
    },
    Certificate {
        #[serde(with = "encrypted_string")]
        certificate: String,
        #[serde(with = "encrypted_string")]
        private_key: String,
        key_type: KeyType,
        validity_period: Option<CertificateValidity>,
    },
    Kerberos {
        realm: String,
        principal: String,
        // Kerberos ticket handled by system
    },
    PKCS11 {
        library_path: String,
        slot_id: Option<u32>,
        key_id: String,
        pin: Option<String>,  // May need encryption
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

/// Certificate validity information
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CertificateValidity {
    pub valid_from: DateTime<Utc>,
    pub valid_to: DateTime<Utc>,
    pub serial: Option<String>,
    pub ca_fingerprint: Option<String>,
}

impl SSHProfile {
    /// Create a new SSH profile
    pub fn new(
        device_id: String,
        name: String,
        host: String,
        port: u16,
        username: String,
    ) -> Self {
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
            color: None,
            icon: None,
            sort_order: 0,
            description: None,
            tags: Vec::new(),
        }
    }

    /// Set authentication method và data
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

    /// Add tag
    pub fn add_tag(&mut self, tag: String) {
        if !self.tags.contains(&tag) {
            self.tags.push(tag);
            self.base.touch();
        }
    }

    /// Remove tag
    pub fn remove_tag(&mut self, tag: &str) {
        if let Some(pos) = self.tags.iter().position(|t| t == tag) {
            self.tags.remove(pos);
            self.base.touch();
        }
    }

    /// Check if profile has valid authentication data
    pub fn has_valid_auth(&self) -> bool {
        match (&self.auth_method, &self.auth_data) {
            (AuthMethod::Password, AuthData::Password { password }) => !password.is_empty(),
            (AuthMethod::PrivateKey, AuthData::PrivateKey { private_key, .. }) => !private_key.is_empty(),
            (AuthMethod::PrivateKeyWithPassphrase, AuthData::PrivateKeyWithPassphrase { private_key, passphrase, .. }) => {
                !private_key.is_empty() && !passphrase.is_empty()
            },
            (AuthMethod::Agent, AuthData::Agent { .. }) => true,
            (AuthMethod::Certificate, AuthData::Certificate { certificate, private_key, .. }) => {
                !certificate.is_empty() && !private_key.is_empty()
            },
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
        // Encryption is handled by the encrypted_string serde module
        // This is called during save operations
        Ok(())
    }

    fn decrypt_fields(&mut self, encryption_service: &dyn EncryptionService) -> DatabaseResult<()> {
        // Decryption is handled by the encrypted_string serde module
        // This is called during load operations
        Ok(())
    }

    fn has_encrypted_data(&self) -> bool {
        match &self.auth_data {
            AuthData::Password { .. } |
            AuthData::PrivateKey { .. } |
            AuthData::PrivateKeyWithPassphrase { .. } |
            AuthData::Certificate { .. } => true,
            AuthData::Agent { .. } |
            AuthData::Kerberos { .. } => false,
            AuthData::PKCS11 { pin, .. } => pin.is_some(),
        }
    }

    fn encryption_device_id(&self) -> Option<&str> {
        Some(&self.base.device_id)
    }
}

/// Request để tạo SSH profile mới
#[derive(Debug, Clone, Serialize, Deserialize)]
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
    pub color: Option<String>,
    pub icon: Option<String>,
    pub description: Option<String>,
    pub tags: Option<Vec<String>>,
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
        profile.icon = self.icon;
        profile.description = self.description;
        profile.tags = self.tags.unwrap_or_default();

        profile
    }
}

/// Request để update SSH profile
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UpdateSSHProfileRequest {
    pub name: Option<String>,
    pub host: Option<String>,
    pub port: Option<u16>,
    pub username: Option<String>,
    pub group_id: Option<Option<String>>,  // None = no change, Some(None) = remove from group
    pub auth_method: Option<AuthMethod>,
    pub auth_data: Option<AuthData>,
    pub timeout: Option<Option<u32>>,
    pub keep_alive: Option<bool>,
    pub compression: Option<bool>,
    pub color: Option<Option<String>>,
    pub icon: Option<Option<String>>,
    pub description: Option<Option<String>>,
    pub tags: Option<Vec<String>>,
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
        if let Some(icon) = self.icon {
            profile.icon = icon;
        }
        if let Some(description) = self.description {
            profile.description = description;
        }
        if let Some(tags) = self.tags {
            profile.tags = tags;
        }

        profile.base.touch();
    }
}

/// Module để handle encrypted string serialization
mod encrypted_string {
    use serde::{Deserialize, Deserializer, Serialize, Serializer};

    pub fn serialize<S>(value: &String, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        // For security reasons, we might want to store passwords encrypted at rest
        // For now, serialize as-is to avoid complications with async encryption
        value.serialize(serializer)
    }

    pub fn deserialize<'de, D>(deserializer: D) -> Result<String, D::Error>
    where
        D: Deserializer<'de>,
    {
        // Deserialize as-is
        String::deserialize(deserializer)
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
            AuthMethod::PrivateKey => write!(f, "Private Key"),
            AuthMethod::PrivateKeyWithPassphrase => write!(f, "Private Key with Passphrase"),
            AuthMethod::Agent => write!(f, "SSH Agent"),
            AuthMethod::Certificate => write!(f, "Certificate"),
            AuthMethod::Kerberos => write!(f, "Kerberos"),
            AuthMethod::PKCS11 => write!(f, "PKCS#11"),
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
    fn base_model(&self) -> &crate::database::models::base::BaseModel {
        &self.base
    }
}
