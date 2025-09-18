use base64::{engine::general_purpose, Engine as _};
use chrono::{DateTime, Utc};
use std::sync::Arc;
use tokio::sync::RwLock;

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
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct SetupMasterPasswordRequest {
    pub device_name: String,
    pub password: String,
    pub confirm_password: String,
    pub auto_unlock: bool,
    pub use_keychain: bool,
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
            manager.create_master_password(request.device_name, &request.password, &self.config)?;

        // Start session
        self.session_start = Some(Utc::now());

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

        MasterPasswordStatus {
            is_setup: false, // Sẽ được set từ database check
            is_unlocked: self.session_start.is_some() && !self.is_session_expired(),
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
        }

        Ok(())
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
        let manager = self.device_key_manager.read().await;
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

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_master_password_setup() {
        let config = MasterPasswordConfig::default();
        let mut manager = MasterPasswordManager::new("test_device".to_string(), config);

        let request = SetupMasterPasswordRequest {
            device_name: "Test Device".to_string(),
            password: "TestPassword123".to_string(),
            confirm_password: "TestPassword123".to_string(),
            auto_unlock: false,
            use_keychain: false,
            auto_lock_timeout: None,
        };

        let entry = manager.setup_master_password(request).await.unwrap();
        assert_eq!(entry.device_id, "test_device");
        assert_eq!(entry.device_name, "Test Device");
    }

    #[tokio::test]
    async fn test_password_validation() {
        let config = MasterPasswordConfig::default();
        let manager = MasterPasswordManager::new("test_device".to_string(), config);

        // Test weak password
        assert!(manager.validate_password("weak").is_err());

        // Test good password
        assert!(manager.validate_password("StrongPassword123").is_ok());

        // Test common password
        assert!(manager.validate_password("password123").is_err());
    }

    #[tokio::test]
    async fn test_session_management() {
        let mut config = MasterPasswordConfig::default();
        config.session_timeout_minutes = Some(1);

        let mut manager = MasterPasswordManager::new("test_device".to_string(), config);

        // No active session
        assert!(manager.is_session_expired());

        // Setup master password (starts session)
        let request = SetupMasterPasswordRequest {
            device_name: "Test Device".to_string(),
            password: "TestPassword123".to_string(),
            confirm_password: "TestPassword123".to_string(),
            auto_unlock: false,
            use_keychain: false,
            auto_lock_timeout: None,
        };

        let _password_entry = manager.setup_master_password(request).await.unwrap();

        // Session should be active
        assert!(!manager.is_session_expired());

        // Lock session
        manager.lock_session().await;
        assert!(manager.is_session_expired());
    }
}
