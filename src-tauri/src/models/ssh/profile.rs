use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

use crate::{
    database::{
        error::DatabaseResult,
        traits::{Encryptable, EncryptionService},
    },
    impl_syncable,
    models::base::BaseModel,
};

/// SSH Profile với flexible authentication methods
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
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
    pub timeout: Option<u32>, // seconds
    pub keep_alive: bool,
    pub compression: bool,

    /// Proxy settings
    pub proxy: Option<ProxyConfig>,

    /// UI customization
    pub color: Option<String>, // Hex color

    /// Notes
    pub description: Option<String>,
    pub tags: Vec<String>,
}

/// Authentication methods supported
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "PascalCase")]
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

/// Proxy configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ProxyConfig {
    pub proxy_type: ProxyType,
    pub host: String,
    pub port: u16,
    pub username: Option<String>,
    #[serde(with = "encrypted_option_string")]
    pub password: Option<String>,
}

/// Proxy types supported
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum ProxyType {
    Http,
    Socks5,
    Socks4,
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
        public_key: Option<String>, // Not encrypted
    },
    PrivateKeyWithPassphrase {
        #[serde(with = "encrypted_string")]
        private_key: String,
        #[serde(with = "encrypted_string")]
        passphrase: String,
        key_type: KeyType,
        public_key: Option<String>, // Not encrypted
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
        pin: Option<String>, // May need encryption
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
            tags: Vec::new(),
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

    /// Static method to get profile by ID (will be implemented in service layer)
    pub async fn get_by_id(
        _profile_id: &str,
    ) -> Result<Option<SSHProfile>, crate::database::error::DatabaseError> {
        // TODO: This should be implemented in the service layer
        // For now, return an error to indicate it needs implementation
        Err(crate::database::error::DatabaseError::NotImplemented(
            "get_by_id should be implemented in service layer".to_string(),
        ))
    }

    /// Check if profile has valid authentication data
    pub fn has_valid_auth(&self) -> bool {
        match (&self.auth_method, &self.auth_data) {
            (AuthMethod::Password, AuthData::Password { password }) => !password.is_empty(),
            (AuthMethod::PrivateKey, AuthData::PrivateKey { private_key, .. }) => {
                !private_key.is_empty()
            }
            (
                AuthMethod::PrivateKeyWithPassphrase,
                AuthData::PrivateKeyWithPassphrase {
                    private_key,
                    passphrase,
                    ..
                },
            ) => !private_key.is_empty() && !passphrase.is_empty(),
            (AuthMethod::Agent, AuthData::Agent { .. }) => true,
            (
                AuthMethod::Certificate,
                AuthData::Certificate {
                    certificate,
                    private_key,
                    ..
                },
            ) => !certificate.is_empty() && !private_key.is_empty(),
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

    fn encrypt_fields(
        &mut self,
        encryption_service: &dyn EncryptionService,
    ) -> DatabaseResult<()> {
        // Encrypt auth_data fields based on auth method
        match &mut self.auth_data {
            AuthData::Password { password } => {
                // Encrypt password
                let encrypted = tokio::task::block_in_place(|| {
                    tokio::runtime::Handle::current().block_on(async {
                        encryption_service.encrypt_string(password, Some(&self.base.device_id)).await
                    })
                })?;
                *password = encrypted;
            }
            AuthData::PrivateKey { private_key, key_type: _, public_key: _ } => {
                // Encrypt private key
                let encrypted = tokio::task::block_in_place(|| {
                    tokio::runtime::Handle::current().block_on(async {
                        encryption_service.encrypt_string(private_key, Some(&self.base.device_id)).await
                    })
                })?;
                *private_key = encrypted;
            }
            AuthData::PrivateKeyWithPassphrase { private_key, passphrase, key_type: _, public_key: _ } => {
                // Encrypt both private key and passphrase
                let encrypted_key = tokio::task::block_in_place(|| {
                    tokio::runtime::Handle::current().block_on(async {
                        encryption_service.encrypt_string(private_key, Some(&self.base.device_id)).await
                    })
                })?;
                let encrypted_passphrase = tokio::task::block_in_place(|| {
                    tokio::runtime::Handle::current().block_on(async {
                        encryption_service.encrypt_string(passphrase, Some(&self.base.device_id)).await
                    })
                })?;
                *private_key = encrypted_key;
                *passphrase = encrypted_passphrase;
            }
            AuthData::Certificate { certificate, private_key, key_type: _, validity_period: _ } => {
                // Encrypt certificate and private key
                let encrypted_cert = tokio::task::block_in_place(|| {
                    tokio::runtime::Handle::current().block_on(async {
                        encryption_service.encrypt_string(certificate, Some(&self.base.device_id)).await
                    })
                })?;
                let encrypted_key = tokio::task::block_in_place(|| {
                    tokio::runtime::Handle::current().block_on(async {
                        encryption_service.encrypt_string(private_key, Some(&self.base.device_id)).await
                    })
                })?;
                *certificate = encrypted_cert;
                *private_key = encrypted_key;
            }
            _ => {
                // No encryption needed for other auth methods (Agent, etc.)
            }
        }
        Ok(())
    }

    fn decrypt_fields(
        &mut self,
        encryption_service: &dyn EncryptionService,
    ) -> DatabaseResult<()> {
        // Decrypt auth_data fields based on auth method
        match &mut self.auth_data {
            AuthData::Password { password } => {
                // Decrypt password
                let decrypted = tokio::task::block_in_place(|| {
                    tokio::runtime::Handle::current().block_on(async {
                        encryption_service.decrypt_string(password, Some(&self.base.device_id)).await
                    })
                })?;
                *password = decrypted;
            }
            AuthData::PrivateKey { private_key, key_type: _, public_key: _ } => {
                // Decrypt private key
                let decrypted = tokio::task::block_in_place(|| {
                    tokio::runtime::Handle::current().block_on(async {
                        encryption_service.decrypt_string(private_key, Some(&self.base.device_id)).await
                    })
                })?;
                *private_key = decrypted;
            }
            AuthData::PrivateKeyWithPassphrase { private_key, passphrase, key_type: _, public_key: _ } => {
                // Decrypt both private key and passphrase
                let decrypted_key = tokio::task::block_in_place(|| {
                    tokio::runtime::Handle::current().block_on(async {
                        encryption_service.decrypt_string(private_key, Some(&self.base.device_id)).await
                    })
                })?;
                let decrypted_passphrase = tokio::task::block_in_place(|| {
                    tokio::runtime::Handle::current().block_on(async {
                        encryption_service.decrypt_string(passphrase, Some(&self.base.device_id)).await
                    })
                })?;
                *private_key = decrypted_key;
                *passphrase = decrypted_passphrase;
            }
            AuthData::Certificate { certificate, private_key, key_type: _, validity_period: _ } => {
                // Decrypt certificate and private key
                let decrypted_cert = tokio::task::block_in_place(|| {
                    tokio::runtime::Handle::current().block_on(async {
                        encryption_service.decrypt_string(certificate, Some(&self.base.device_id)).await
                    })
                })?;
                let decrypted_key = tokio::task::block_in_place(|| {
                    tokio::runtime::Handle::current().block_on(async {
                        encryption_service.decrypt_string(private_key, Some(&self.base.device_id)).await
                    })
                })?;
                *certificate = decrypted_cert;
                *private_key = decrypted_key;
            }
            _ => {
                // No decryption needed for other auth methods (Agent, etc.)
            }
        }
        Ok(())
    }

    fn has_encrypted_data(&self) -> bool {
        match &self.auth_data {
            AuthData::Password { .. }
            | AuthData::PrivateKey { .. }
            | AuthData::PrivateKeyWithPassphrase { .. }
            | AuthData::Certificate { .. } => true,
            AuthData::Agent { .. } | AuthData::Kerberos { .. } => false,
            AuthData::PKCS11 { pin, .. } => pin.is_some(),
        }
    }

    fn encryption_device_id(&self) -> Option<&str> {
        Some(&self.base.device_id)
    }
}

/// Request để tạo SSH profile mới
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
        profile.description = self.description;
        profile.tags = self.tags.unwrap_or_default();

        profile
    }
}

/// Request để update SSH profile
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
        // TODO: In a real implementation, we would encrypt the value here
        // For now, we serialize as-is since encryption/decryption is handled
        // by the Encryptable trait methods during save/load operations
        value.serialize(serializer)
    }

    pub fn deserialize<'de, D>(deserializer: D) -> Result<String, D::Error>
    where
        D: Deserializer<'de>,
    {
        // TODO: In a real implementation, we would decrypt the value here
        // For now, we deserialize as-is since encryption/decryption is handled
        // by the Encryptable trait methods during save/load operations
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
    fn base_model(&self) -> &crate::models::base::BaseModel {
        &self.base
    }
}
