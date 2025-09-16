use std::collections::HashMap;
use argon2::{Argon2, PasswordHash, PasswordHasher, PasswordVerifier};
use argon2::password_hash::SaltString;
use chrono::{DateTime, Utc};

use crate::database::{
    encryption::{AESEncryption, KeychainManager},
    error::{EncryptionError, EncryptionResult},
    config::MasterPasswordConfig,
};

/// Device-specific encryption key manager
pub struct DeviceKeyManager {
    current_device_id: String,
    device_keys: HashMap<String, DeviceEncryptionKey>,
    keychain: KeychainManager,
}

/// Device encryption key information
#[derive(Debug, Clone)]
#[allow(dead_code)]
pub struct DeviceEncryptionKey {
    pub device_id: String,
    pub device_name: String,
    pub encryption_key: [u8; 32],
    pub key_salt: [u8; 32],
    pub key_version: u32,
    pub created_at: DateTime<Utc>,
    pub last_used_at: DateTime<Utc>,
}

/// Master password entry information
#[derive(Debug, Clone)]
pub struct MasterPasswordEntry {
    pub device_id: String,
    pub device_name: String,
    pub password_salt: [u8; 32],
    pub verification_hash: String,
    pub auto_unlock: bool,
    pub created_at: DateTime<Utc>,
    pub last_verified_at: Option<DateTime<Utc>>,
}

#[allow(dead_code)]
impl DeviceKeyManager {
    /// Create new device key manager
    pub fn new(current_device_id: String) -> Self {
        Self {
            current_device_id,
            device_keys: HashMap::new(),
            keychain: KeychainManager::new("kerminal".to_string()),
        }
    }

    /// Create master password entry cho current device
    pub fn create_master_password(
        &mut self,
        device_name: String,
        password: &str,
        config: &MasterPasswordConfig,
    ) -> EncryptionResult<MasterPasswordEntry> {
        // Generate salt
        let salt_bytes = AESEncryption::generate_salt();
        let salt = SaltString::encode_b64(&salt_bytes)
            .map_err(|e| EncryptionError::KeyDerivationFailed(e.to_string()))?;

        // Hash password for verification
        let argon2 = Argon2::default();
        let password_hash = argon2
            .hash_password(password.as_bytes(), &salt)
            .map_err(|e| EncryptionError::KeyDerivationFailed(e.to_string()))?
            .to_string();

        // Create device encryption key
        let device_key = self.derive_device_key(password, &salt_bytes)?;

        // Store in keychain if auto_unlock enabled
        if config.use_keychain && config.auto_unlock {
            if let Err(e) = self.keychain.store_master_password(&self.current_device_id, password) {
                eprintln!("Warning: Failed to store password in keychain: {}", e);
            }

            if let Err(e) = self.keychain.store_device_key(&self.current_device_id, &device_key.encryption_key) {
                eprintln!("Warning: Failed to store device key in keychain: {}", e);
            }
        }

        // Store device key in memory
        self.device_keys.insert(self.current_device_id.clone(), device_key);

        Ok(MasterPasswordEntry {
            device_id: self.current_device_id.clone(),
            device_name,
            password_salt: salt_bytes,
            verification_hash: password_hash,
            auto_unlock: config.auto_unlock && config.use_keychain,
            created_at: Utc::now(),
            last_verified_at: Some(Utc::now()),
        })
    }

    /// Verify master password và load device key
    pub fn verify_master_password(
        &mut self,
        password: &str,
        entry: &MasterPasswordEntry,
    ) -> EncryptionResult<bool> {
        // Verify password hash
        let parsed_hash = PasswordHash::new(&entry.verification_hash)
            .map_err(|_e| EncryptionError::MasterPasswordVerificationFailed)?;

        let argon2 = Argon2::default();
        let is_valid = argon2
            .verify_password(password.as_bytes(), &parsed_hash)
            .is_ok();

        if is_valid {
            // Derive device key và store in memory
            let device_key = self.derive_device_key(password, &entry.password_salt)?;
            self.device_keys.insert(entry.device_id.clone(), device_key);
        }

        Ok(is_valid)
    }

    /// Try to auto-unlock using keychain
    pub fn try_auto_unlock(&mut self, device_id: &str) -> EncryptionResult<bool> {
        if !self.keychain.is_available() {
            return Ok(false);
        }

        // Try to get device key directly from keychain
        if let Some(key_bytes) = self.keychain.get_device_key(device_id)? {
            if key_bytes.len() == 32 {
                let mut key_array = [0u8; 32];
                key_array.copy_from_slice(&key_bytes);

                let device_key = DeviceEncryptionKey {
                    device_id: device_id.to_string(),
                    device_name: "Auto-unlocked".to_string(),
                    encryption_key: key_array,
                    key_salt: [0u8; 32], // Not needed for direct key
                    key_version: 1,
                    created_at: Utc::now(),
                    last_used_at: Utc::now(),
                };

                self.device_keys.insert(device_id.to_string(), device_key);
                return Ok(true);
            }
        }

        // Fallback: try to get password from keychain
    if let Some(_password) = self.keychain.get_master_password(device_id)? {
            // Need master password entry to verify - this should be loaded from database
            // For now, return false and require manual unlock
            return Ok(false);
        }

        Ok(false)
    }

    /// Add device key từ sync (khi gặp encrypted data từ device khác)
    pub fn add_device_key(
        &mut self,
        device_id: String,
        device_name: String,
        password: &str,
        password_entry: &MasterPasswordEntry,
    ) -> EncryptionResult<()> {
        // Verify password first
        if !self.verify_password_for_device(password, password_entry)? {
            return Err(EncryptionError::MasterPasswordVerificationFailed);
        }

        // Derive device key
        let device_key = DeviceEncryptionKey {
            device_id: device_id.clone(),
            device_name,
            encryption_key: self.derive_key_from_password(password, &password_entry.password_salt)?,
            key_salt: password_entry.password_salt,
            key_version: 1,
            created_at: Utc::now(),
            last_used_at: Utc::now(),
        };

        // Store in memory
        self.device_keys.insert(device_id, device_key);

        Ok(())
    }

    /// Get device encryption key
    pub fn get_device_key(&mut self, device_id: &str) -> Option<&DeviceEncryptionKey> {
        if let Some(key) = self.device_keys.get_mut(device_id) {
            key.last_used_at = Utc::now();
            Some(key)
        } else {
            None
        }
    }

    /// Encrypt data với device key
    pub fn encrypt_with_device(
        &mut self,
        data: &[u8],
        device_id: Option<&str>,
    ) -> EncryptionResult<Vec<u8>> {
        let device_id_str = device_id.unwrap_or(&self.current_device_id).to_string();

        let device_key = self.get_device_key(&device_id_str)
            .ok_or_else(|| EncryptionError::UnknownDeviceKey(device_id_str.clone()))?;

        AESEncryption::encrypt(&device_key.encryption_key, data)
    }

    /// Decrypt data với device key
    pub fn decrypt_with_device(
        &mut self,
        encrypted_data: &[u8],
        device_id: Option<&str>,
    ) -> EncryptionResult<Vec<u8>> {
        let device_id_str = device_id.unwrap_or(&self.current_device_id).to_string();

        let device_key = self.get_device_key(&device_id_str)
            .ok_or_else(|| EncryptionError::UnknownDeviceKey(device_id_str.clone()))?;

        AESEncryption::decrypt(&device_key.encryption_key, encrypted_data)
    }

    /// Try to decrypt với tất cả known device keys
    pub fn try_decrypt_with_any_device(&mut self, encrypted_data: &[u8]) -> EncryptionResult<(Vec<u8>, String)> {
        for (device_id, device_key) in &self.device_keys {
            if let Ok(decrypted) = AESEncryption::decrypt(&device_key.encryption_key, encrypted_data) {
                return Ok((decrypted, device_id.clone()));
            }
        }

        Err(EncryptionError::DecryptionFailed("No device key could decrypt this data".to_string()))
    }

    /// Change master password cho current device
    pub fn change_master_password(
        &mut self,
        old_password: &str,
        new_password: &str,
        entry: &MasterPasswordEntry,
        config: &MasterPasswordConfig,
    ) -> EncryptionResult<MasterPasswordEntry> {
        // Verify old password first
        if !self.verify_password_for_device(old_password, entry)? {
            return Err(EncryptionError::MasterPasswordVerificationFailed);
        }

        // Create new master password entry
        self.create_master_password(entry.device_name.clone(), new_password, config)
    }

    /// Clear all device keys từ memory
    pub fn clear_all_keys(&mut self) {
        self.device_keys.clear();
    }

    /// Get loaded device IDs
    pub fn get_loaded_device_ids(&self) -> Vec<String> {
        self.device_keys.keys().cloned().collect()
    }

    /// Derive device encryption key từ master password
    fn derive_device_key(&self, password: &str, salt: &[u8; 32]) -> EncryptionResult<DeviceEncryptionKey> {
        let encryption_key = self.derive_key_from_password(password, salt)?;

        Ok(DeviceEncryptionKey {
            device_id: self.current_device_id.clone(),
            device_name: "Current Device".to_string(),
            encryption_key,
            key_salt: *salt,
            key_version: 1,
            created_at: Utc::now(),
            last_used_at: Utc::now(),
        })
    }

    /// Derive encryption key từ password và salt
    fn derive_key_from_password(&self, password: &str, salt: &[u8; 32]) -> EncryptionResult<[u8; 32]> {
        let mut key = [0u8; 32];

        // Use PBKDF2 để derive key
        pbkdf2::pbkdf2::<hmac::Hmac<sha2::Sha256>>(
            password.as_bytes(),
            salt,
            100_000, // 100k iterations
            &mut key,
        );

        Ok(key)
    }

    /// Verify password cho specific device
    fn verify_password_for_device(
        &self,
        password: &str,
        entry: &MasterPasswordEntry,
    ) -> EncryptionResult<bool> {
        let parsed_hash = PasswordHash::new(&entry.verification_hash)
            .map_err(|_| EncryptionError::MasterPasswordVerificationFailed)?;

        let argon2 = Argon2::default();
        Ok(argon2.verify_password(password.as_bytes(), &parsed_hash).is_ok())
    }
}

#[allow(dead_code)]
impl DeviceEncryptionKey {
    /// Check if key is expired (for security)
    pub fn is_expired(&self, max_age_days: i64) -> bool {
        let max_age = chrono::Duration::days(max_age_days);
        (Utc::now() - self.created_at) > max_age
    }

    /// Check if key was used recently
    pub fn is_recently_used(&self, threshold_hours: i64) -> bool {
        let threshold = chrono::Duration::hours(threshold_hours);
        (Utc::now() - self.last_used_at) < threshold
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::database::config::MasterPasswordConfig;

    #[test]
    fn test_master_password_creation_and_verification() {
        let mut manager = DeviceKeyManager::new("test_device".to_string());
        let config = MasterPasswordConfig::default();
        let password = "test_password_123";

        // Create master password
        let entry = manager.create_master_password(
            "Test Device".to_string(),
            password,
            &config,
        ).unwrap();

        // Verify correct password
        let is_valid = manager.verify_master_password(password, &entry).unwrap();
        assert!(is_valid);

        // Verify incorrect password
        let is_invalid = manager.verify_master_password("wrong_password", &entry).unwrap();
        assert!(!is_invalid);
    }

    #[test]
    fn test_encryption_decryption() {
        let mut manager = DeviceKeyManager::new("test_device".to_string());
        let config = MasterPasswordConfig::default();
        let password = "test_password_123";
        let data = b"Hello, World!";

        // Create master password
        let _master_password_entry = manager.create_master_password(
            "Test Device".to_string(),
            password,
            &config,
        ).unwrap();

        // Encrypt data
        let encrypted = manager.encrypt_with_device(data, None).unwrap();

        // Decrypt data
        let decrypted = manager.decrypt_with_device(&encrypted, None).unwrap();

        assert_eq!(data, decrypted.as_slice());
    }
}
