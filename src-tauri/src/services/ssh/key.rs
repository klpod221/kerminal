use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::Mutex;

use crate::database::{error::DatabaseResult, service::DatabaseService};
use crate::models::ssh::{CreateSSHKeyRequest, SSHKey, UpdateSSHKeyRequest};

/// SSH Key service for centralized key management with caching
pub struct SSHKeyService {
    database_service: Arc<Mutex<DatabaseService>>,
    key_cache: Arc<Mutex<HashMap<String, crate::models::ssh::key::ResolvedSSHKey>>>,
}

impl SSHKeyService {
    /// Create new SSHKeyService instance
    pub fn new(database_service: Arc<Mutex<DatabaseService>>) -> Self {
        Self {
            database_service,
            key_cache: Arc::new(Mutex::new(HashMap::new())),
        }
    }

    /// Clear key cache (call after key update/delete)
    #[allow(dead_code)]
    pub async fn clear_cache(&self) {
        let mut cache = self.key_cache.lock().await;
        cache.clear();
    }

    /// Clear specific key from cache
    async fn invalidate_key_cache(&self, key_id: &str) {
        let mut cache = self.key_cache.lock().await;
        cache.remove(key_id);
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
        let result = db_service.update_ssh_key(id, request).await;
        drop(db_service);

        if result.is_ok() {
            self.invalidate_key_cache(id).await;
        }
        result
    }

    /// Delete SSH key (checks for profile dependencies)
    pub async fn delete_ssh_key(&self, id: &str, force: bool) -> DatabaseResult<()> {
        let db_service = self.database_service.lock().await;
        let result = db_service.delete_ssh_key(id, force).await;
        drop(db_service);

        if result.is_ok() {
            self.invalidate_key_cache(id).await;
        }
        result
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
        let private_key = std::fs::read_to_string(file_path).map_err(|e| {
            crate::database::error::DatabaseError::Internal(anyhow::anyhow!(
                "Failed to read key file: {}",
                e
            ))
        })?;

        let key_type = Self::detect_key_type(&private_key)?;

        let public_key = Self::extract_public_key(file_path);

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
            if key_content.contains("ssh-ed25519") {
                Ok(KeyType::Ed25519)
            } else if key_content.contains("ecdsa") {
                Ok(KeyType::ECDSA)
            } else {
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
    #[allow(dead_code)]
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
    #[allow(dead_code)]
    pub async fn mark_key_used(&self, key_id: &str) -> DatabaseResult<()> {
        let db_service = self.database_service.lock().await;
        db_service.mark_key_used(key_id).await
    }

    /// Resolve SSH key reference to plain text key data for authentication (with caching)
    pub async fn resolve_key_for_auth(
        &self,
        key_id: &str,
    ) -> DatabaseResult<crate::models::ssh::key::ResolvedSSHKey> {
        {
            let cache = self.key_cache.lock().await;
            if let Some(cached_key) = cache.get(key_id) {
                return Ok(cached_key.clone());
            }
        }

        let ssh_key = self.get_ssh_key(key_id).await?;

        let resolved_key = crate::models::ssh::key::ResolvedSSHKey {
            private_key: ssh_key.private_key,
            passphrase: ssh_key.passphrase,
        };

        {
            let mut cache = self.key_cache.lock().await;
            cache.insert(key_id.to_string(), resolved_key.clone());
        }

        Ok(resolved_key)
    }
}
