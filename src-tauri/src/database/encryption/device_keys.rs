use argon2::password_hash::SaltString;
use argon2::{Argon2, PasswordHash, PasswordHasher, PasswordVerifier};
use chrono::{DateTime, Utc};
use std::collections::HashMap;

use crate::database::{
    config::MasterPasswordConfig,
    encryption::{AESEncryption, KeychainManager},
    error::{EncryptionError, EncryptionResult},
};

/// Device-specific encryption key manager
pub struct DeviceKeyManager {
    instance_id: uuid::Uuid,
    current_device_id: String,
    device_keys: HashMap<String, DeviceEncryptionKey>,
    keychain: KeychainManager,
}

/// Device encryption key information
#[derive(Debug, Clone)]
pub struct DeviceEncryptionKey {
    #[allow(dead_code)]
    pub device_id: String,
    #[allow(dead_code)]
    pub device_name: String,
    pub encryption_key: [u8; 32],
    #[allow(dead_code)]
    pub key_salt: [u8; 32],
    #[allow(dead_code)]
    pub key_version: u32,
    #[allow(dead_code)]
    pub created_at: DateTime<Utc>,
    pub last_used_at: DateTime<Utc>,
}

/// Master password entry information
#[derive(Debug, Clone)]
pub struct MasterPasswordEntry {
    pub device_id: String,
    pub password_salt: [u8; 32],
    pub verification_hash: String,
    pub auto_unlock: bool,
    pub auto_lock_timeout: Option<u32>, // in minutes
    pub created_at: DateTime<Utc>,
    pub last_verified_at: Option<DateTime<Utc>>,
}

impl DeviceKeyManager {
    /// Create new device key manager
    pub fn new(current_device_id: String) -> Self {
        let instance_id = uuid::Uuid::new_v4();
        Self {
            instance_id,
            current_device_id,
            device_keys: HashMap::new(),
            keychain: KeychainManager::new("kerminal".to_string()),
        }
    }

    /// Create master password entry cho current device
    pub fn create_master_password(
        &mut self,
        password: &str,
        config: &MasterPasswordConfig,
    ) -> EncryptionResult<MasterPasswordEntry> {
        let salt_bytes = AESEncryption::generate_salt();
        let salt = SaltString::encode_b64(&salt_bytes)
            .map_err(|e| EncryptionError::KeyDerivationFailed(e.to_string()))?;

        let argon2 = Argon2::default();
        let password_hash = argon2
            .hash_password(password.as_bytes(), &salt)
            .map_err(|e| EncryptionError::KeyDerivationFailed(e.to_string()))?
            .to_string();

        let device_key = self.derive_device_key(password, &salt_bytes)?;

        if config.use_keychain && config.auto_unlock {
            match self
                .keychain
                .store_master_password(&self.current_device_id, password)
            {
                Ok(()) => {}
                Err(e) => {
                    eprintln!("Warning: Failed to store password in keychain: {}", e);
                }
            }

            match self
                .keychain
                .store_device_key(&self.current_device_id, &device_key.encryption_key)
            {
                Ok(()) => {}
                Err(e) => {
                    eprintln!("Warning: Failed to store device key in keychain: {}", e);
                }
            }
        }

        self.device_keys
            .insert(self.current_device_id.clone(), device_key);

        Ok(MasterPasswordEntry {
            device_id: self.current_device_id.clone(),
            password_salt: salt_bytes,
            verification_hash: password_hash,
            auto_unlock: config.auto_unlock && config.use_keychain,
            auto_lock_timeout: config.session_timeout_minutes,
            created_at: Utc::now(),
            last_verified_at: Some(Utc::now()),
        })
    }

    /// Verify master password and load device key
    pub fn verify_master_password(
        &mut self,
        password: &str,
        entry: &MasterPasswordEntry,
    ) -> EncryptionResult<bool> {
        let parsed_hash = PasswordHash::new(&entry.verification_hash)
            .map_err(|_e| EncryptionError::MasterPasswordVerificationFailed)?;

        let argon2 = Argon2::default();
        let is_valid = argon2
            .verify_password(password.as_bytes(), &parsed_hash)
            .is_ok();

        if is_valid {
            let device_key = self.derive_device_key(password, &entry.password_salt)?;
            self.device_keys.insert(entry.device_id.clone(), device_key);
        }

        Ok(is_valid)
    }

    /// Try to auto-unlock using keychain with stored password
    pub fn try_auto_unlock_with_password(
        &mut self,
        device_id: &str,
        entry: &MasterPasswordEntry,
    ) -> EncryptionResult<bool> {
        if !self.keychain.is_available() {
            return Ok(false);
        }

        match self.keychain.get_master_password(device_id) {
            Ok(Some(password)) => {
                if self.verify_password_for_device(&password, entry)? {
                    let device_key = self.derive_device_key(&password, &entry.password_salt)?;
                    self.device_keys.insert(device_id.to_string(), device_key);
                    return Ok(true);
                }
            }
            Ok(None) => {}
            Err(_) => {}
        }

        match self.keychain.get_device_key(device_id) {
            Ok(Some(key_bytes)) => {
                if key_bytes.len() == 32 {
                    let mut key_array = [0u8; 32];
                    key_array.copy_from_slice(&key_bytes);

                    let device_key = DeviceEncryptionKey {
                        device_id: device_id.to_string(),
                        device_name: "Auto-unlocked".to_string(),
                        encryption_key: key_array,
                        key_salt: [0u8; 32],
                        key_version: 1,
                        created_at: Utc::now(),
                        last_used_at: Utc::now(),
                    };

                    self.device_keys.insert(device_id.to_string(), device_key);
                    return Ok(true);
                }
            }
            Ok(None) => {}
            Err(e) => {
                eprintln!(
                    "DeviceKeyManager: Error retrieving device key from keychain: {}",
                    e
                );
            }
        }

        Ok(false)
    }

    /// Try to auto-unlock using keychain
    #[allow(dead_code)]
    pub fn try_auto_unlock(&mut self, device_id: &str) -> EncryptionResult<bool> {
        if !self.keychain.is_available() {
            return Ok(false);
        }

        if let Some(key_bytes) = self.keychain.get_device_key(device_id)? {
            if key_bytes.len() == 32 {
                let mut key_array = [0u8; 32];
                key_array.copy_from_slice(&key_bytes);

                let device_key = DeviceEncryptionKey {
                    device_id: device_id.to_string(),
                    device_name: "Auto-unlocked".to_string(),
                    encryption_key: key_array,
                    key_salt: [0u8; 32],
                    key_version: 1,
                    created_at: Utc::now(),
                    last_used_at: Utc::now(),
                };

                self.device_keys.insert(device_id.to_string(), device_key);
                return Ok(true);
            }
        }

        Ok(false)
    }

    /// Add device key from sync
    #[allow(dead_code)]
    pub fn add_device_key(
        &mut self,
        device_id: String,
        device_name: String,
        password: &str,
        password_entry: &MasterPasswordEntry,
    ) -> EncryptionResult<()> {
        if !self.verify_password_for_device(password, password_entry)? {
            return Err(EncryptionError::MasterPasswordVerificationFailed);
        }

        let device_key = DeviceEncryptionKey {
            device_id: device_id.clone(),
            device_name,
            encryption_key: self.derive_key_from_password(password, &password_entry.password_salt)?,
            key_salt: password_entry.password_salt,
            key_version: 1,
            created_at: Utc::now(),
            last_used_at: Utc::now(),
        };

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

    /// Encrypt data with device key
    pub fn encrypt_with_device(
        &mut self,
        data: &[u8],
        device_id: Option<&str>,
    ) -> EncryptionResult<Vec<u8>> {
        let device_id_str = device_id.unwrap_or(&self.current_device_id).to_string();

        let device_key = self
            .get_device_key(&device_id_str)
            .ok_or_else(|| EncryptionError::UnknownDeviceKey(device_id_str.clone()))?;

        AESEncryption::encrypt(&device_key.encryption_key, data)
    }

    /// Decrypt data with device key
    pub fn decrypt_with_device(
        &mut self,
        encrypted_data: &[u8],
        device_id: Option<&str>,
    ) -> EncryptionResult<Vec<u8>> {
        let device_id_str = device_id.unwrap_or(&self.current_device_id).to_string();

        let device_key = self
            .get_device_key(&device_id_str)
            .ok_or_else(|| EncryptionError::UnknownDeviceKey(device_id_str.clone()))?;

        AESEncryption::decrypt(&device_key.encryption_key, encrypted_data)
    }

    /// Try to decrypt with all known device keys
    pub fn try_decrypt_with_any_device(
        &mut self,
        encrypted_data: &[u8],
    ) -> EncryptionResult<(Vec<u8>, String)> {
        for (device_id, device_key) in &self.device_keys {
            if let Ok(decrypted) =
                AESEncryption::decrypt(&device_key.encryption_key, encrypted_data)
            {
                return Ok((decrypted, device_id.clone()));
            }
        }

                Err(EncryptionError::DecryptionFailed(
            "No device key could decrypt this data".to_string(),
        ))
    }

    /// Ensure shared device key exists for cross-device encryption
    pub fn ensure_shared_device_key(
        &mut self,
        password: &str,
        password_entry: &MasterPasswordEntry,
    ) -> EncryptionResult<()> {
        if self.device_keys.contains_key("__shared__") {
            return Ok(());
        }

        let mut key_material = [0u8; 32];
        let _ = pbkdf2::pbkdf2::<hmac::Hmac<sha2::Sha256>>(
            password.as_bytes(),
            &password_entry.password_salt,
            100_000, // 100k iterations
            &mut key_material,
        );

        let shared_key = DeviceEncryptionKey {
            device_id: "__shared__".to_string(),
            device_name: "Shared Cross-Device Key".to_string(),
            encryption_key: key_material,
            key_salt: password_entry.password_salt,
            key_version: 1,
            created_at: Utc::now(),
            last_used_at: Utc::now(),
        };

        self.device_keys
            .insert("__shared__".to_string(), shared_key);

        Ok(())
    }

    /// Change master password cho current device
    pub fn change_master_password(
        &mut self,
        old_password: &str,
        new_password: &str,
        entry: &MasterPasswordEntry,
        config: &MasterPasswordConfig,
    ) -> EncryptionResult<MasterPasswordEntry> {
        if !self.verify_password_for_device(old_password, entry)? {
            return Err(EncryptionError::MasterPasswordVerificationFailed);
        }

        self.create_master_password(new_password, config)
    }

    /// Clear all device keys from memory
    pub fn clear_all_keys(&mut self) {
        self.device_keys.clear();
    }

    /// Get loaded device IDs
    pub fn get_loaded_device_ids(&self) -> Vec<String> {
        self.device_keys.keys().cloned().collect()
    }

    /// Try to ensure shared key after auto-unlock
    pub fn ensure_shared_key_from_keychain(
        &mut self,
        device_id: &str,
        entry: &MasterPasswordEntry,
    ) -> EncryptionResult<()> {
        if self.device_keys.contains_key("__shared__") {
            return Ok(());
        }

        if let Ok(Some(password)) = self.keychain.get_master_password(device_id) {
            self.ensure_shared_device_key(&password, entry)?;
        }

        Ok(())
    }

    /// Check if device key exists
    pub fn has_device_key(&self, device_id: &str) -> bool {
        self.device_keys.contains_key(device_id)
    }

    /// Ensure shared device key from current device key
    /// Since both are derived from same password+salt, they should be identical
    pub fn ensure_shared_device_key_from_current(&mut self) -> EncryptionResult<()> {
        if self.device_keys.contains_key("__shared__") {
            return Ok(());
        }

        let current_key = self.device_keys.get(&self.current_device_id)
            .ok_or_else(|| {
                eprintln!("DeviceKeyManager[{}]::ensure_shared_device_key_from_current: Current device key not found!",
                    self.instance_id);
                EncryptionError::UnknownDeviceKey(self.current_device_id.clone())
            })?;

        let shared_key = DeviceEncryptionKey {
            device_id: "__shared__".to_string(),
            device_name: "Shared Cross-Device Key".to_string(),
            encryption_key: current_key.encryption_key,
            key_salt: current_key.key_salt,
            key_version: current_key.key_version,
            created_at: Utc::now(),
            last_used_at: Utc::now(),
        };

        self.device_keys
            .insert("__shared__".to_string(), shared_key);

        Ok(())
    }

    /// Derive device encryption key from master password
    fn derive_device_key(
        &self,
        password: &str,
        salt: &[u8; 32],
    ) -> EncryptionResult<DeviceEncryptionKey> {
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

    /// Derive encryption key from password and salt
    fn derive_key_from_password(
        &self,
        password: &str,
        salt: &[u8; 32],
    ) -> EncryptionResult<[u8; 32]> {
        let mut key = [0u8; 32];

        let _ = pbkdf2::pbkdf2::<hmac::Hmac<sha2::Sha256>>(
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
        Ok(argon2
            .verify_password(password.as_bytes(), &parsed_hash)
            .is_ok())
    }
}
