use base64::{engine::general_purpose, Engine as _};
use chrono::{DateTime, Utc};
use std::sync::Arc;
use tokio::sync::RwLock;
use serde::{Deserialize, Serialize};

use crate::database::{
    config::MasterPasswordConfig,
    encryption::{device_keys::MasterPasswordEntry, DeviceKeyManager, KeychainManager},
    error::{EncryptionError, EncryptionResult},
    traits::EncryptionService,
};

/// Master password manager - orchestrates tất cả master password operations
pub struct MasterPasswordManager {
    device_key_manager: Arc<RwLock<DeviceKeyManager>>,
    keychain_manager: KeychainManager,
    current_device_id: String,
    config: MasterPasswordConfig,
    session_start: Option<DateTime<Utc>>,
}

/// Master password setup request
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SetupMasterPasswordRequest {
    #[serde(rename = "deviceName")]
    pub device_name: String,
    pub password: String,
    #[serde(rename = "confirmPassword")]
    pub confirm_password: String,
    #[serde(rename = "autoUnlock")]
    pub auto_unlock: bool,
    #[serde(rename = "useKeychain")]
    pub use_keychain: bool,
    #[serde(rename = "autoLockTimeout")]
    pub auto_lock_timeout: Option<u32>, // in minutes
}

/// Master password verification request
#[derive(Debug, Clone)]
pub struct VerifyMasterPasswordRequest {
    pub password: String,
    pub device_id: Option<String>,
}

/// Master password status
#[derive(Debug, Clone)]
#[allow(dead_code)]
pub struct MasterPasswordStatus {
    pub is_setup: bool,
    pub is_unlocked: bool,
    pub auto_unlock_enabled: bool,
    pub keychain_available: bool,
    pub session_active: bool,
    pub session_expires_at: Option<DateTime<Utc>>,
    pub loaded_device_count: usize,
}

#[allow(dead_code)]
impl MasterPasswordManager {
    /// Create new master password manager
    pub fn new(current_device_id: String, config: MasterPasswordConfig) -> Self {
        let device_key_manager = Arc::new(RwLock::new(DeviceKeyManager::new(
            current_device_id.clone(),
        )));

        Self {
            device_key_manager,
            keychain_manager: KeychainManager::new("kerminal".to_string()),
            current_device_id,
            config,
            session_start: None,
        }
    }

    /// Setup master password lần đầu tiên
    pub async fn setup_master_password(
        &mut self,
        request: SetupMasterPasswordRequest,
    ) -> EncryptionResult<MasterPasswordEntry> {
        // Validate request
        self.validate_setup_request(&request)?;

        // Update config based on request
        self.config.auto_unlock = request.auto_unlock && request.use_keychain;
        self.config.use_keychain = request.use_keychain;
        self.config.session_timeout_minutes = request.auto_lock_timeout;

        // Create master password entry
        let mut manager = self.device_key_manager.write().await;
        let entry =
            manager.create_master_password(&request.password, &self.config)?;

        // Only start session if auto unlock is enabled
        if request.auto_unlock && request.use_keychain {
            self.session_start = Some(Utc::now());
        }

        // Try to store in keychain if requested
        if request.use_keychain && request.auto_unlock {
            println!("MasterPasswordManager: Attempting to store password in keychain...");
            match self.keychain_manager.store_master_password(
                &self.current_device_id,
                &request.password,
            ) {
                Ok(()) => {
                    println!("MasterPasswordManager: Successfully stored password in keychain");
                }
                Err(e) => {
                    eprintln!("Warning: Failed to store password in keychain: {}", e);
                }
            }
        }

        Ok(entry)
    }

    /// Verify master password
    pub async fn verify_master_password(
        &mut self,
        request: VerifyMasterPasswordRequest,
        stored_entry: &MasterPasswordEntry,
    ) -> EncryptionResult<bool> {
        let device_id = request
            .device_id
            .as_ref()
            .unwrap_or(&self.current_device_id);

        // If verifying for different device, need the stored entry for that device
        if device_id != &self.current_device_id {
            return self
                .verify_device_password(&request.password, stored_entry)
                .await;
        }

        // Verify password
        let mut manager = self.device_key_manager.write().await;
        let is_valid = manager.verify_master_password(&request.password, stored_entry)?;

        if is_valid {
            self.session_start = Some(Utc::now());
        }

        Ok(is_valid)
    }

    /// Try auto-unlock từ keychain
    pub async fn try_auto_unlock(&mut self) -> EncryptionResult<bool> {
        if !self.config.auto_unlock || !self.config.use_keychain {
            return Ok(false);
        }

        let mut manager = self.device_key_manager.write().await;
        let success = manager.try_auto_unlock(&self.current_device_id)?;

        if success {
            self.session_start = Some(Utc::now());
        }

        Ok(success)
    }

    /// Try auto-unlock với stored password entry từ database
    pub async fn try_auto_unlock_with_entry(&mut self, entry: &crate::database::encryption::device_keys::MasterPasswordEntry) -> EncryptionResult<bool> {
        println!("MasterPasswordManager: Checking auto-unlock config:");
        println!("  - config.auto_unlock: {}", self.config.auto_unlock);
        println!("  - config.use_keychain: {}", self.config.use_keychain);
        println!("  - keychain_available: {}", self.keychain_manager.is_available());

        if !self.config.auto_unlock || !self.config.use_keychain {
            println!("MasterPasswordManager: Auto-unlock disabled in config");
            return Ok(false);
        }

        println!("MasterPasswordManager: Attempting auto-unlock with device key manager...");
        let mut manager = self.device_key_manager.write().await;
        let success = manager.try_auto_unlock_with_password(&self.current_device_id, entry)?;

        if success {
            println!("MasterPasswordManager: Auto-unlock successful, starting session");
            self.session_start = Some(Utc::now());
        } else {
            println!("MasterPasswordManager: Auto-unlock failed");
        }

        Ok(success)
    }

    /// Add device password cho multi-device support
    pub async fn add_device_password(
        &mut self,
        device_id: String,
        device_name: String,
        password: String,
        stored_entry: &MasterPasswordEntry,
    ) -> EncryptionResult<()> {
        let mut manager = self.device_key_manager.write().await;
        manager.add_device_key(device_id, device_name, &password, stored_entry)
    }

    /// Change master password
    pub async fn change_master_password(
        &mut self,
        old_password: String,
        new_password: String,
        stored_entry: &MasterPasswordEntry,
    ) -> EncryptionResult<MasterPasswordEntry> {
        // Validate new password
        self.validate_password(&new_password)?;

        let mut manager = self.device_key_manager.write().await;
        let new_entry = manager.change_master_password(
            &old_password,
            &new_password,
            stored_entry,
            &self.config,
        )?;

        Ok(new_entry)
    }

    /// Lock session (clear keys từ memory)
    pub async fn lock_session(&mut self) {
        let mut manager = self.device_key_manager.write().await;
        manager.clear_all_keys();
        self.session_start = None;
    }

    /// Check if session is expired
    pub fn is_session_expired(&self) -> bool {
        if let Some(start_time) = self.session_start {
            if let Some(timeout_minutes) = self.config.session_timeout_minutes {
                let timeout_duration = chrono::Duration::minutes(timeout_minutes as i64);
                (Utc::now() - start_time) > timeout_duration
            } else {
                false // No timeout configured
            }
        } else {
            true // No active session
        }
    }

    /// Check and auto-lock if session expired
    pub async fn check_and_auto_lock(&mut self) -> bool {
        if self.is_session_expired() && self.session_start.is_some() {
            self.lock_session().await;
            true // Session was locked
        } else {
            false // Session is still valid
        }
    }

    /// Get master password status
    pub async fn get_status(&self) -> MasterPasswordStatus {
        let manager = self.device_key_manager.read().await;
        let loaded_devices = manager.get_loaded_device_ids();

        let session_expires_at = if let Some(start_time) = self.session_start {
            self.config
                .session_timeout_minutes
                .map(|timeout| start_time + chrono::Duration::minutes(timeout as i64))
        } else {
            None
        };

        let is_unlocked = self.session_start.is_some() && !self.is_session_expired();

        MasterPasswordStatus {
            is_setup: false, // Sẽ được set từ database check
            is_unlocked,
            auto_unlock_enabled: self.config.auto_unlock,
            keychain_available: self.keychain_manager.is_available(),
            session_active: self.session_start.is_some(),
            session_expires_at,
            loaded_device_count: loaded_devices.len(),
        }
    }

    /// Update master password configuration
    pub async fn update_config(
        &mut self,
        auto_unlock: bool,
        auto_lock_timeout: Option<u32>
    ) -> EncryptionResult<()> {
        self.config.auto_unlock = auto_unlock;
        self.config.require_on_startup = !auto_unlock;

        // Update session timeout if provided
        if let Some(timeout) = auto_lock_timeout {
            self.config.session_timeout_minutes = if timeout == 0 { None } else { Some(timeout) };
        }

        if !auto_unlock {
            // Remove từ keychain nếu tắt auto-unlock
            println!("Auto-unlock disabled, removing credentials from keychain");
            if let Err(e) = self
                .keychain_manager
                .delete_master_password(&self.current_device_id)
            {
                eprintln!("Warning: Failed to remove password from keychain: {}", e);
            }

            if let Err(e) = self
                .keychain_manager
                .delete_device_key(&self.current_device_id)
            {
                eprintln!("Warning: Failed to remove device key from keychain: {}", e);
            }
        } else {
            println!("Auto-unlock enabled, but password needs to be provided separately to store in keychain");
        }

        Ok(())
    }

    /// Update master password configuration with keychain update
    pub async fn update_config_with_keychain(
        &mut self,
        auto_unlock: bool,
        auto_lock_timeout: Option<u32>,
        password: Option<String>
    ) -> EncryptionResult<()> {
        self.config.auto_unlock = auto_unlock;
        self.config.require_on_startup = !auto_unlock;

        // Update session timeout if provided
        if let Some(timeout) = auto_lock_timeout {
            self.config.session_timeout_minutes = if timeout == 0 { None } else { Some(timeout) };
        }

        if !auto_unlock {
            // Remove từ keychain nếu tắt auto-unlock
            println!("Auto-unlock disabled, removing credentials from keychain");
            if let Err(e) = self
                .keychain_manager
                .delete_master_password(&self.current_device_id)
            {
                eprintln!("Warning: Failed to remove password from keychain: {}", e);
            }

            if let Err(e) = self
                .keychain_manager
                .delete_device_key(&self.current_device_id)
            {
                eprintln!("Warning: Failed to remove device key from keychain: {}", e);
            }
        } else if let Some(pwd) = password {
            // Store password in keychain when enabling auto-unlock
            println!("Auto-unlock enabled, storing password in keychain");
            if let Err(e) = self
                .keychain_manager
                .store_master_password(&self.current_device_id, &pwd)
            {
                eprintln!("Warning: Failed to store password in keychain: {}", e);
                return Err(EncryptionError::KeychainError(format!("Failed to store password: {}", e)));
            }

            // Also store device key if we have an active session
            let mut device_key_manager = self.device_key_manager.write().await;
            if let Some(device_key) = device_key_manager.get_device_key(&self.current_device_id) {
                if let Err(e) = self
                    .keychain_manager
                    .store_device_key(&self.current_device_id, &device_key.encryption_key)
                {
                    eprintln!("Warning: Failed to store device key in keychain: {}", e);
                }
            }
        }

        Ok(())
    }

    /// Get current configuration
    pub fn get_config(&self) -> &MasterPasswordConfig {
        &self.config
    }

    /// Reset master password (removes all encrypted data)
    pub async fn reset_master_password(&mut self) -> EncryptionResult<()> {
        // Clear tất cả device keys
        let mut manager = self.device_key_manager.write().await;
        let loaded_devices = manager.get_loaded_device_ids();
        manager.clear_all_keys();

        // Clear keychain
        if let Err(e) = self.keychain_manager.clear_all(&loaded_devices) {
            eprintln!("Warning: Failed to clear keychain: {}", e);
        }

        // Reset session
        self.session_start = None;

        Ok(())
    }

    /// Verify password cho different device
    async fn verify_device_password(
        &self,
        password: &str,
        stored_entry: &MasterPasswordEntry,
    ) -> EncryptionResult<bool> {
        let _manager = self.device_key_manager.read().await;
        // Use temporary verification
        use argon2::{Argon2, PasswordHash, PasswordVerifier};

        let parsed_hash = PasswordHash::new(&stored_entry.verification_hash)
            .map_err(|_| EncryptionError::MasterPasswordVerificationFailed)?;

        let argon2 = Argon2::default();
        Ok(argon2
            .verify_password(password.as_bytes(), &parsed_hash)
            .is_ok())
    }

    /// Validate setup request
    fn validate_setup_request(&self, request: &SetupMasterPasswordRequest) -> EncryptionResult<()> {
        // Check password confirmation
        if request.password != request.confirm_password {
            return Err(EncryptionError::InvalidKey(
                "Passwords do not match".to_string(),
            ));
        }

        // Validate password strength
        self.validate_password(&request.password)?;

        // Check device name
        if request.device_name.trim().is_empty() {
            return Err(EncryptionError::InvalidKey(
                "Device name cannot be empty".to_string(),
            ));
        }

        // Check keychain availability if auto_unlock requested
        if request.auto_unlock && request.use_keychain && !self.keychain_manager.is_available() {
            return Err(EncryptionError::KeychainError(
                "Keychain not available".to_string(),
            ));
        }

        Ok(())
    }

    /// Validate password strength
    fn validate_password(&self, password: &str) -> EncryptionResult<()> {
        if password.len() < 8 {
            return Err(EncryptionError::InvalidKey(
                "Password must be at least 8 characters".to_string(),
            ));
        }

        if password.len() > 128 {
            return Err(EncryptionError::InvalidKey(
                "Password must be less than 128 characters".to_string(),
            ));
        }

        // Check for at least one uppercase, lowercase, number
        let has_upper = password.chars().any(|c| c.is_uppercase());
        let has_lower = password.chars().any(|c| c.is_lowercase());
        let has_digit = password.chars().any(|c| c.is_ascii_digit());

        if !has_upper || !has_lower || !has_digit {
            return Err(EncryptionError::InvalidKey(
                "Password must contain uppercase, lowercase, and numbers".to_string(),
            ));
        }

        // Check for common passwords (basic check)
        let common_passwords = ["password", "12345678", "qwerty", "admin", "letmein"];
        if common_passwords
            .iter()
            .any(|&common| password.to_lowercase().contains(common))
        {
            return Err(EncryptionError::InvalidKey(
                "Password is too common".to_string(),
            ));
        }

        Ok(())
    }
}

/// Implement EncryptionService trait
#[async_trait::async_trait]
impl EncryptionService for MasterPasswordManager {
    async fn encrypt(
        &self,
        data: &[u8],
        device_id: Option<&str>,
    ) -> crate::database::error::DatabaseResult<Vec<u8>> {
        let mut manager = self.device_key_manager.write().await;
        manager
            .encrypt_with_device(data, device_id)
            .map_err(|e| e.into())
    }

    async fn decrypt(
        &self,
        encrypted_data: &[u8],
        device_id: Option<&str>,
    ) -> crate::database::error::DatabaseResult<Vec<u8>> {
        let mut manager = self.device_key_manager.write().await;

        // Try specific device first
        if let Some(device_id) = device_id {
            if let Ok(data) = manager.decrypt_with_device(encrypted_data, Some(device_id)) {
                return Ok(data);
            }
        }

        // Fallback: try any device key
        manager
            .try_decrypt_with_any_device(encrypted_data)
            .map(|(data, _device_id)| data)
            .map_err(|e| e.into())
    }

    async fn encrypt_string(
        &self,
        data: &str,
        device_id: Option<&str>,
    ) -> crate::database::error::DatabaseResult<String> {
        let encrypted = self.encrypt(data.as_bytes(), device_id).await?;
        Ok(general_purpose::STANDARD.encode(encrypted))
    }

    async fn decrypt_string(
        &self,
        encrypted_data: &str,
        device_id: Option<&str>,
    ) -> crate::database::error::DatabaseResult<String> {
        let encrypted_bytes = general_purpose::STANDARD
            .decode(encrypted_data)
            .map_err(|_e| EncryptionError::InvalidFormat)?;

        let decrypted = self.decrypt(&encrypted_bytes, device_id).await?;

        String::from_utf8(decrypted)
            .map_err(|e| EncryptionError::DecryptionFailed(format!("Invalid UTF-8: {}", e)).into())
    }
}
