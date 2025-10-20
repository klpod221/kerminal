use std::sync::Arc;
use tokio::sync::RwLock;

use crate::{
    database::{
        encryption::MasterPasswordManager,
        error::{DatabaseResult, EncryptionError},
        traits::EncryptionService,
    },
    models::sync::{ConnectionDetails, SyncSettings},
};

pub struct ExternalDbEncryptor {
    master_password_manager: Arc<RwLock<MasterPasswordManager>>,
}

impl ExternalDbEncryptor {
    pub fn new(master_password_manager: Arc<RwLock<MasterPasswordManager>>) -> Self {
        Self {
            master_password_manager,
        }
    }

    pub async fn encrypt_connection_details(
        &self,
        details: &ConnectionDetails,
    ) -> DatabaseResult<String> {
        let json = serde_json::to_string(details).map_err(|e| {
            EncryptionError::EncryptionFailed(format!(
                "Failed to serialize connection details: {}",
                e
            ))
        })?;

        let manager = self.master_password_manager.read().await;
        let encrypted = manager.encrypt_string(&json, None).await?;

        Ok(encrypted)
    }

    pub async fn decrypt_connection_details(
        &self,
        encrypted: &str,
    ) -> DatabaseResult<ConnectionDetails> {
        let manager = self.master_password_manager.read().await;
        let decrypted = manager.decrypt_string(encrypted, None).await?;

        let details: ConnectionDetails = serde_json::from_str(&decrypted).map_err(|e| {
            EncryptionError::DecryptionFailed(format!(
                "Failed to deserialize connection details: {}",
                e
            ))
        })?;

        Ok(details)
    }

    pub async fn encrypt_sync_settings(&self, settings: &SyncSettings) -> DatabaseResult<String> {
        let json = serde_json::to_string(settings).map_err(|e| {
            EncryptionError::EncryptionFailed(format!("Failed to serialize sync settings: {}", e))
        })?;

        let manager = self.master_password_manager.read().await;
        let encrypted = manager.encrypt_string(&json, None).await?;

        Ok(encrypted)
    }

    pub async fn decrypt_sync_settings(&self, encrypted: &str) -> DatabaseResult<SyncSettings> {
        let manager = self.master_password_manager.read().await;
        let decrypted = manager.decrypt_string(encrypted, None).await?;

        let settings: SyncSettings = serde_json::from_str(&decrypted).map_err(|e| {
            EncryptionError::DecryptionFailed(format!("Failed to deserialize sync settings: {}", e))
        })?;

        Ok(settings)
    }
}
