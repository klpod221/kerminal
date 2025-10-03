use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};

use crate::{
    database::{
        error::DatabaseResult,
        traits::{Encryptable, EncryptionService},
    },
    impl_syncable,
    models::base::BaseModel,
};

use super::profile::KeyType;

/// Resolved SSH key data for authentication (plain text, ready for use)
#[derive(Debug, Clone)]
pub struct ResolvedSSHKey {
    pub private_key: String,
    pub key_type: KeyType,
    pub public_key: Option<String>,
    pub passphrase: Option<String>,
}

/// SSH Key - Reusable authentication key for SSH connections
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct SSHKey {
    /// Base model with sync metadata
    #[serde(flatten)]
    pub base: BaseModel,

    /// Key information
    pub name: String,
    pub key_type: KeyType,

    /// Key data (encrypted)
    #[serde(with = "encrypted_string")]
    pub private_key: String,

    /// Public key (not encrypted - safe to display)
    pub public_key: Option<String>,

    /// Optional passphrase for the private key (encrypted)
    #[serde(with = "encrypted_option_string")]
    pub passphrase: Option<String>,

    /// Fingerprint for identification (SHA256 hash of public key)
    pub fingerprint: String,

    /// Notes
    pub description: Option<String>,

    /// Usage tracking
    pub last_used: Option<DateTime<Utc>>,
}

impl SSHKey {
    /// Create a new SSH key
    pub fn new(
        device_id: String,
        name: String,
        key_type: KeyType,
        private_key: String,
        public_key: Option<String>,
        passphrase: Option<String>,
    ) -> Self {
        let fingerprint = Self::calculate_fingerprint(&private_key, &public_key);

        Self {
            base: BaseModel::new(device_id),
            name,
            key_type,
            private_key,
            public_key,
            passphrase,
            fingerprint,
            description: None,
            last_used: None,
        }
    }

    /// Calculate SHA256 fingerprint from key data
    pub fn calculate_fingerprint(private_key: &str, public_key: &Option<String>) -> String {
        let key_data = if let Some(pub_key) = public_key {
            pub_key.as_bytes()
        } else {
            private_key.as_bytes()
        };

        let mut hasher = Sha256::new();
        hasher.update(key_data);
        let result = hasher.finalize();

        // Format as colon-separated hex (like SSH does)
        result
            .iter()
            .map(|b| format!("{:02x}", b))
            .collect::<Vec<_>>()
            .chunks(2)
            .map(|chunk| chunk.join(""))
            .collect::<Vec<_>>()
            .join(":")
    }

    /// Get key ID
    pub fn get_id(&self) -> &str {
        &self.base.id
    }

    /// Update key information
    pub fn update(&mut self, name: Option<String>, description: Option<Option<String>>) {
        if let Some(name) = name {
            self.name = name;
        }
        if let Some(description) = description {
            self.description = description;
        }
        self.base.touch();
    }

    /// Mark key as recently used
    pub fn mark_used(&mut self) {
        self.last_used = Some(Utc::now());
        self.base.touch();
    }

    /// Check if key has passphrase
    pub fn has_passphrase(&self) -> bool {
        self.passphrase.is_some() && !self.passphrase.as_ref().unwrap().is_empty()
    }

    /// Get display name for UI
    pub fn display_name(&self) -> String {
        if self.name.is_empty() {
            format!("{} Key ({})", self.key_type, &self.fingerprint[..16])
        } else {
            self.name.clone()
        }
    }

    /// Get short fingerprint for display (first 16 chars)
    pub fn short_fingerprint(&self) -> String {
        if self.fingerprint.len() > 16 {
            format!("{}...", &self.fingerprint[..16])
        } else {
            self.fingerprint.clone()
        }
    }
}

// Implement Syncable trait using macro
impl_syncable!(SSHKey, "ssh_keys");

impl Encryptable for SSHKey {
    fn encrypted_fields() -> Vec<&'static str> {
        vec!["private_key", "passphrase"]
    }

    fn encrypt_fields(
        &mut self,
        encryption_service: &dyn EncryptionService,
    ) -> DatabaseResult<()> {
        // Encrypt private key
        let encrypted_key = tokio::task::block_in_place(|| {
            tokio::runtime::Handle::current().block_on(async {
                encryption_service
                    .encrypt_string(&self.private_key, Some(&self.base.device_id))
                    .await
            })
        })?;
        self.private_key = encrypted_key;

        // Encrypt passphrase if present
        if let Some(passphrase) = &self.passphrase {
            if !passphrase.is_empty() {
                let encrypted_passphrase = tokio::task::block_in_place(|| {
                    tokio::runtime::Handle::current().block_on(async {
                        encryption_service
                            .encrypt_string(passphrase, Some(&self.base.device_id))
                            .await
                    })
                })?;
                self.passphrase = Some(encrypted_passphrase);
            }
        }

        Ok(())
    }

    fn decrypt_fields(
        &mut self,
        encryption_service: &dyn EncryptionService,
    ) -> DatabaseResult<()> {
        // Decrypt private key
        let decrypted_key = tokio::task::block_in_place(|| {
            tokio::runtime::Handle::current().block_on(async {
                encryption_service
                    .decrypt_string(&self.private_key, Some(&self.base.device_id))
                    .await
            })
        })?;
        self.private_key = decrypted_key;

        // Decrypt passphrase if present
        if let Some(passphrase) = &self.passphrase {
            if !passphrase.is_empty() {
                let decrypted_passphrase = tokio::task::block_in_place(|| {
                    tokio::runtime::Handle::current().block_on(async {
                        encryption_service
                            .decrypt_string(passphrase, Some(&self.base.device_id))
                            .await
                    })
                })?;
                self.passphrase = Some(decrypted_passphrase);
            }
        }

        Ok(())
    }

    fn has_encrypted_data(&self) -> bool {
        true // Always has private key
    }

    fn encryption_device_id(&self) -> Option<&str> {
        Some(&self.base.device_id)
    }
}

/// Request to create new SSH key
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct CreateSSHKeyRequest {
    pub name: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub key_type: Option<KeyType>,
    pub private_key: String,
    pub public_key: Option<String>,
    pub passphrase: Option<String>,
    pub description: Option<String>,
}

impl CreateSSHKeyRequest {
    pub fn to_key(self, device_id: String) -> SSHKey {
        // Auto-detect key type if not provided
        let key_type = self.key_type.unwrap_or_else(|| {
            Self::detect_key_type(&self.private_key).unwrap_or(KeyType::RSA)
        });

        let mut key = SSHKey::new(
            device_id,
            self.name,
            key_type,
            self.private_key,
            self.public_key,
            self.passphrase,
        );

        key.description = self.description;
        key
    }

    /// Detect SSH key type from private key content
    fn detect_key_type(key_content: &str) -> Option<KeyType> {
        if key_content.contains("BEGIN RSA PRIVATE KEY")
            || key_content.contains("BEGIN PRIVATE KEY")
        {
            Some(KeyType::RSA)
        } else if key_content.contains("BEGIN OPENSSH PRIVATE KEY") {
            // OpenSSH format could be Ed25519, ECDSA, or RSA
            if key_content.contains("ssh-ed25519") {
                Some(KeyType::Ed25519)
            } else if key_content.contains("ecdsa") {
                Some(KeyType::ECDSA)
            } else {
                Some(KeyType::RSA)
            }
        } else if key_content.contains("BEGIN EC PRIVATE KEY") {
            Some(KeyType::ECDSA)
        } else if key_content.contains("BEGIN DSA PRIVATE KEY") {
            Some(KeyType::DSA)
        } else {
            None
        }
    }
}

/// Request to update SSH key
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct UpdateSSHKeyRequest {
    pub name: Option<String>,
    pub description: Option<Option<String>>,
    pub private_key: Option<String>,
    pub public_key: Option<Option<String>>,
    pub passphrase: Option<Option<String>>,
}

impl UpdateSSHKeyRequest {
    pub fn apply_to_key(self, key: &mut SSHKey) {
        if let Some(name) = self.name {
            key.name = name;
        }
        if let Some(description) = self.description {
            key.description = description;
        }
        if let Some(private_key) = self.private_key {
            key.private_key = private_key;
        }
        if let Some(public_key) = self.public_key {
            key.public_key = public_key;
        }
        if let Some(passphrase) = self.passphrase {
            key.passphrase = passphrase;
        }
        key.base.touch();
    }
}

/// Module to handle encrypted string serialization
mod encrypted_string {
    use serde::{Deserialize, Deserializer, Serialize, Serializer};

    pub fn serialize<S>(value: &String, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        // Encryption is handled by the Encryptable trait during save operations
        value.serialize(serializer)
    }

    pub fn deserialize<'de, D>(deserializer: D) -> Result<String, D::Error>
    where
        D: Deserializer<'de>,
    {
        // Decryption is handled by the Encryptable trait during load operations
        String::deserialize(deserializer)
    }
}

mod encrypted_option_string {
    use serde::{Deserialize, Deserializer, Serialize, Serializer};

    pub fn serialize<S>(value: &Option<String>, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        // Encryption is handled by the Encryptable trait during save operations
        value.serialize(serializer)
    }

    pub fn deserialize<'de, D>(deserializer: D) -> Result<Option<String>, D::Error>
    where
        D: Deserializer<'de>,
    {
        // Decryption is handled by the Encryptable trait during load operations
        Option::<String>::deserialize(deserializer)
    }
}

impl crate::database::sync::strategies::HasBaseModel for SSHKey {
    fn base_model(&self) -> &crate::models::base::BaseModel {
        &self.base
    }
}
