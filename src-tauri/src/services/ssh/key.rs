use std::sync::Arc;
use tokio::sync::Mutex;

use crate::database::{error::DatabaseResult, service::DatabaseService};
use crate::models::ssh::{CreateSSHKeyRequest, SSHKey, UpdateSSHKeyRequest};

/// SSH Key service for centralized key management
pub struct SSHKeyService {
    database_service: Arc<Mutex<DatabaseService>>,
}

impl SSHKeyService {
    /// Create new SSHKeyService instance
    pub fn new(database_service: Arc<Mutex<DatabaseService>>) -> Self {
        Self { database_service }
    }

    /// Create new SSH key
    pub async fn create_ssh_key(&self, request: CreateSSHKeyRequest) -> DatabaseResult<SSHKey> {
        let db_service = self.database_service.lock().await;
        db_service.create_ssh_key(request).await
    }

    /// Get all SSH keys
    pub async fn get_ssh_keys(&self) -> DatabaseResult<Vec<SSHKey>> {
        let db_service = self.database_service.lock().await;
        db_service.get_ssh_keys().await
    }

    /// Get SSH key by ID
    pub async fn get_ssh_key(&self, id: &str) -> DatabaseResult<SSHKey> {
        let db_service = self.database_service.lock().await;
        db_service.get_ssh_key(id).await
    }

    /// Update SSH key (updates all provided fields)
    pub async fn update_ssh_key(
        &self,
        id: &str,
        request: UpdateSSHKeyRequest,
    ) -> DatabaseResult<SSHKey> {
        let db_service = self.database_service.lock().await;
        db_service.update_ssh_key(id, request).await
    }

    /// Delete SSH key (checks for profile dependencies)
    pub async fn delete_ssh_key(&self, id: &str, force: bool) -> DatabaseResult<()> {
        let db_service = self.database_service.lock().await;
        db_service.delete_ssh_key(id, force).await
    }

    /// Count how many profiles are using this key
    pub async fn count_profiles_using_key(&self, key_id: &str) -> DatabaseResult<u32> {
        let db_service = self.database_service.lock().await;
        db_service.count_profiles_using_key(key_id).await
    }

    /// Import SSH key from file path
    pub async fn import_ssh_key_from_file(
        &self,
        name: String,
        file_path: &str,
        passphrase: Option<String>,
        description: Option<String>,
    ) -> DatabaseResult<SSHKey> {
        // Read key from file
        let private_key = std::fs::read_to_string(file_path).map_err(|e| {
            crate::database::error::DatabaseError::Internal(anyhow::anyhow!(
                "Failed to read key file: {}",
                e
            ))
        })?;

        // Detect key type from key content
        let key_type = Self::detect_key_type(&private_key)?;

        // Try to extract public key if it exists
        let public_key = Self::extract_public_key(file_path);

        // Create request and save
        let request = CreateSSHKeyRequest {
            name,
            key_type: Some(key_type),
            private_key,
            public_key,
            passphrase,
            description,
        };

        self.create_ssh_key(request).await
    }

    /// Detect SSH key type from key content
    fn detect_key_type(key_content: &str) -> DatabaseResult<crate::models::ssh::profile::KeyType> {
        use crate::models::ssh::profile::KeyType;

        if key_content.contains("BEGIN RSA PRIVATE KEY")
            || key_content.contains("BEGIN PRIVATE KEY")
        {
            Ok(KeyType::RSA)
        } else if key_content.contains("BEGIN OPENSSH PRIVATE KEY") {
            // OpenSSH format could be Ed25519, ECDSA, or RSA
            // Try to detect from the key data
            if key_content.contains("ssh-ed25519") {
                Ok(KeyType::Ed25519)
            } else if key_content.contains("ecdsa") {
                Ok(KeyType::ECDSA)
            } else {
                // Default to RSA if we can't determine
                Ok(KeyType::RSA)
            }
        } else if key_content.contains("BEGIN EC PRIVATE KEY") {
            Ok(KeyType::ECDSA)
        } else if key_content.contains("BEGIN DSA PRIVATE KEY") {
            Ok(KeyType::DSA)
        } else {
            Err(crate::database::error::DatabaseError::Internal(
                anyhow::anyhow!("Unable to detect SSH key type from key content"),
            ))
        }
    }

    /// Try to extract public key from .pub file
    fn extract_public_key(private_key_path: &str) -> Option<String> {
        let pub_key_path = format!("{}.pub", private_key_path);
        std::fs::read_to_string(&pub_key_path).ok()
    }

    /// Validate SSH key format (basic validation)
    pub fn validate_key_format(key_content: &str) -> Result<(), String> {
        if key_content.trim().is_empty() {
            return Err("SSH key cannot be empty".to_string());
        }

        if !key_content.contains("BEGIN") || !key_content.contains("PRIVATE KEY") {
            return Err("Invalid SSH key format: missing BEGIN PRIVATE KEY header".to_string());
        }

        if !key_content.contains("END") {
            return Err("Invalid SSH key format: missing END header".to_string());
        }

        Ok(())
    }

    /// Mark key as recently used (called when connecting with a profile)
    pub async fn mark_key_used(&self, key_id: &str) -> DatabaseResult<()> {
        let db_service = self.database_service.lock().await;
        db_service.mark_key_used(key_id).await
    }

    /// Resolve SSH key reference to plain text key data for authentication
    pub async fn resolve_key_for_auth(&self, key_id: &str) -> DatabaseResult<crate::models::ssh::key::ResolvedSSHKey> {
        // Get the SSH key
        let ssh_key = self.get_ssh_key(key_id).await?;

        // Return resolved key data
        Ok(crate::models::ssh::key::ResolvedSSHKey {
            private_key: ssh_key.private_key,
            key_type: ssh_key.key_type,
            public_key: ssh_key.public_key,
            passphrase: ssh_key.passphrase,
        })
    }
}
