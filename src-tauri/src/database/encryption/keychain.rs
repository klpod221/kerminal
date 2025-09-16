use keyring::{Entry, Error as KeyringError};
use crate::database::error::{EncryptionError, EncryptionResult};

/// System keychain integration cho auto-unlock
pub struct KeychainManager {
    app_name: String,
}

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

    /// Retrieve master password từ keychain
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

    /// Delete master password từ keychain
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
        let key_string = base64::encode(key);

        let entry = Entry::new(&service, device_id)
            .map_err(|e| EncryptionError::KeychainError(e.to_string()))?;

        entry
            .set_password(&key_string)
            .map_err(|e| EncryptionError::KeychainError(e.to_string()))?;

        Ok(())
    }

    /// Retrieve device encryption key từ keychain
    pub fn get_device_key(&self, device_id: &str) -> EncryptionResult<Option<Vec<u8>>> {
        let service = format!("{}_device_key", self.app_name);
        let entry = Entry::new(&service, device_id)
            .map_err(|e| EncryptionError::KeychainError(e.to_string()))?;

        match entry.get_password() {
            Ok(key_string) => {
                let key = base64::decode(key_string)
                    .map_err(|e| EncryptionError::KeychainError(format!("Invalid key format: {}", e)))?;
                Ok(Some(key))
            },
            Err(KeyringError::NoEntry) => Ok(None),
            Err(e) => Err(EncryptionError::KeychainError(e.to_string())),
        }
    }

    /// Delete device encryption key từ keychain
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

    /// Check if keychain is available
    pub fn is_available(&self) -> bool {
        // Try to create a test entry
        let service = format!("{}_test", self.app_name);
        match Entry::new(&service, "test") {
            Ok(_) => true,
            Err(_) => false,
        }
    }

    /// Clear all keychain entries for this app
    pub fn clear_all(&self, device_ids: &[String]) -> EncryptionResult<()> {
        for device_id in device_ids {
            let _ = self.delete_master_password(device_id);
            let _ = self.delete_device_key(device_id);
        }
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_keychain_availability() {
        let keychain = KeychainManager::new("kerminal_test".to_string());
        // Note: Keychain có thể không available trong CI environment
        println!("Keychain available: {}", keychain.is_available());
    }

    #[test]
    fn test_master_password_operations() {
        let keychain = KeychainManager::new("kerminal_test".to_string());
        let device_id = "test_device";
        let password = "test_password_123";

        if !keychain.is_available() {
            println!("Keychain not available, skipping test");
            return;
        }

        // Store password
        keychain.store_master_password(device_id, password).unwrap();

        // Retrieve password
        let retrieved = keychain.get_master_password(device_id).unwrap();
        assert_eq!(retrieved, Some(password.to_string()));

        // Delete password
        keychain.delete_master_password(device_id).unwrap();

        // Verify deletion
        let after_delete = keychain.get_master_password(device_id).unwrap();
        assert_eq!(after_delete, None);
    }

    #[test]
    fn test_device_key_operations() {
        let keychain = KeychainManager::new("kerminal_test".to_string());
        let device_id = "test_device";
        let key = [1, 2, 3, 4, 5, 6, 7, 8, 9, 10, 11, 12, 13, 14, 15, 16,
                   17, 18, 19, 20, 21, 22, 23, 24, 25, 26, 27, 28, 29, 30, 31, 32];

        if !keychain.is_available() {
            println!("Keychain not available, skipping test");
            return;
        }

        // Store key
        keychain.store_device_key(device_id, &key).unwrap();

        // Retrieve key
        let retrieved = keychain.get_device_key(device_id).unwrap();
        assert_eq!(retrieved, Some(key.to_vec()));

        // Delete key
        keychain.delete_device_key(device_id).unwrap();

        // Verify deletion
        let after_delete = keychain.get_device_key(device_id).unwrap();
        assert_eq!(after_delete, None);
    }
}
