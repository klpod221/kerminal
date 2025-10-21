use crate::database::error::{EncryptionError, EncryptionResult};
use keyring::{Entry, Error as KeyringError};

use base64::{engine::general_purpose, Engine as _};
/// System keychain integration cho auto-unlock
pub struct KeychainManager {
    app_name: String,
}

#[allow(dead_code)]
impl KeychainManager {
    /// Create new keychain manager
    pub fn new(app_name: String) -> Self {
        Self { app_name }
    }

    /// Store master password trong keychain
    pub fn store_master_password(&self, device_id: &str, password: &str) -> EncryptionResult<()> {
        let service = format!("{}_master_password", self.app_name);

        let entry = Entry::new(&service, device_id)
            .map_err(|e| EncryptionError::KeychainError(e.to_string()))?;

        entry
            .set_password(password)
            .map_err(|e| EncryptionError::KeychainError(e.to_string()))?;

        Ok(())
    }

    /// Retrieve master password from keychain
    pub fn get_master_password(&self, device_id: &str) -> EncryptionResult<Option<String>> {
        let service = format!("{}_master_password", self.app_name);

        let entry = Entry::new(&service, device_id)
            .map_err(|e| EncryptionError::KeychainError(e.to_string()))?;

        match entry.get_password() {
            Ok(password) => Ok(Some(password)),
            Err(KeyringError::NoEntry) => Ok(None),
            Err(e) => Err(EncryptionError::KeychainError(e.to_string())),
        }
    }

    /// Delete master password from keychain
    pub fn delete_master_password(&self, device_id: &str) -> EncryptionResult<()> {
        let service = format!("{}_master_password", self.app_name);
        let entry = Entry::new(&service, device_id)
            .map_err(|e| EncryptionError::KeychainError(e.to_string()))?;

        match entry.delete_password() {
            Ok(()) => Ok(()),
            Err(KeyringError::NoEntry) => Ok(()), // Already deleted
            Err(e) => Err(EncryptionError::KeychainError(e.to_string())),
        }
    }

    /// Store device encryption key trong keychain
    pub fn store_device_key(&self, device_id: &str, key: &[u8]) -> EncryptionResult<()> {
        let service = format!("{}_device_key", self.app_name);
        let key_string = general_purpose::STANDARD.encode(key);

        let entry = Entry::new(&service, device_id)
            .map_err(|e| EncryptionError::KeychainError(e.to_string()))?;

        entry
            .set_password(&key_string)
            .map_err(|e| EncryptionError::KeychainError(e.to_string()))?;

        Ok(())
    }

    /// Retrieve device encryption key from keychain
    pub fn get_device_key(&self, device_id: &str) -> EncryptionResult<Option<Vec<u8>>> {
        let service = format!("{}_device_key", self.app_name);
        let entry = Entry::new(&service, device_id)
            .map_err(|e| EncryptionError::KeychainError(e.to_string()))?;

        match entry.get_password() {
            Ok(key_string) => {
                let key = general_purpose::STANDARD.decode(key_string).map_err(|e| {
                    EncryptionError::KeychainError(format!("Invalid key format: {}", e))
                })?;
                Ok(Some(key))
            }
            Err(KeyringError::NoEntry) => Ok(None),
            Err(e) => Err(EncryptionError::KeychainError(e.to_string())),
        }
    }

    /// Delete device encryption key from keychain
    pub fn delete_device_key(&self, device_id: &str) -> EncryptionResult<()> {
        let service = format!("{}_device_key", self.app_name);
        let entry = Entry::new(&service, device_id)
            .map_err(|e| EncryptionError::KeychainError(e.to_string()))?;

        match entry.delete_password() {
            Ok(()) => Ok(()),
            Err(KeyringError::NoEntry) => Ok(()), // Already deleted
            Err(e) => Err(EncryptionError::KeychainError(e.to_string())),
        }
    }

    pub fn is_available(&self) -> bool {
        let service = format!("{}_test", self.app_name);
        Entry::new(&service, "test").is_ok()
    }

    /// Clear all keychain entries for this app
    pub fn clear_all(&self, device_ids: &[String]) -> EncryptionResult<()> {
        let mut errors = Vec::new();

        for device_id in device_ids {
            if let Err(e) = self.delete_master_password(device_id) {
                errors.push(format!(
                    "Failed to delete master password for {}: {}",
                    device_id, e
                ));
            }
            if let Err(e) = self.delete_device_key(device_id) {
                errors.push(format!(
                    "Failed to delete device key for {}: {}",
                    device_id, e
                ));
            }
        }

        if !errors.is_empty() {
            eprintln!("Keychain cleanup warnings: {}", errors.join(", "));
        }

        Ok(())
    }
}
