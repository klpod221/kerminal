use std::sync::Arc;
use tokio::sync::Mutex;

use crate::database::{
    encryption::master_password::SetupMasterPasswordRequest,
    error::{DatabaseError, DatabaseResult},
    service::DatabaseService,
};

/// Authentication service for handling master password and device management
pub struct AuthService {
    database_service: Arc<Mutex<DatabaseService>>,
}

impl AuthService {
    /// Create new AuthService instance
    pub fn new(database_service: Arc<Mutex<DatabaseService>>) -> Self {
        Self { database_service }
    }

    /// Setup master password for first time
    pub async fn setup_master_password(
        &self,
        request: SetupMasterPasswordRequest,
    ) -> DatabaseResult<()> {
        let db_service = self.database_service.lock().await;
        db_service.setup_master_password(request).await
    }

    /// Verify master password
    pub async fn verify_master_password(&self, password: String) -> DatabaseResult<()> {
        let db_service = self.database_service.lock().await;
        db_service.verify_master_password(password).await
    }

    /// Try auto unlock with keychain
    pub async fn try_auto_unlock(&self) -> DatabaseResult<bool> {
        let db_service = self.database_service.lock().await;
        db_service.try_auto_unlock().await
    }

    /// Lock current session
    pub async fn lock_session(&self) {
        let db_service = self.database_service.lock().await;
        db_service.lock_session().await;
    }

    /// Check if master password is setup
    pub async fn is_master_password_setup(&self) -> DatabaseResult<bool> {
        let db_service = self.database_service.lock().await;
        db_service.is_master_password_setup().await
    }

    /// Get master password status
    pub async fn get_master_password_status(&self) -> DatabaseResult<serde_json::Value> {
        let db_service = self.database_service.lock().await;
        let is_setup = db_service.is_master_password_setup().await?;
        let actual_status = db_service.get_master_password_status().await?;

        let status = serde_json::json!({
            "isSetup": is_setup,
            "isUnlocked": actual_status.is_unlocked,
            "autoUnlockEnabled": actual_status.auto_unlock_enabled,
            "keychainAvailable": actual_status.keychain_available,
            "sessionActive": actual_status.session_active,
            "sessionExpiresAt": actual_status.session_expires_at,
            "loadedDeviceCount": actual_status.loaded_device_count
        });

        Ok(status)
    }

    /// Get current device information
    pub async fn get_current_device(&self) -> DatabaseResult<serde_json::Value> {
        let db_service = self.database_service.lock().await;
        let device_info = db_service.get_current_device_info().await?;

        let device = serde_json::json!({
            "device_id": device_info.device_id,
            "device_name": device_info.device_name,
            "device_type": device_info.device_type.to_string(),
            "os_name": device_info.os_info.os_type,
            "os_version": device_info.os_info.os_version,
            "created_at": device_info.created_at.to_rfc3339()
        });

        Ok(device)
    }

    /// Change master password with validation
    pub async fn change_master_password(
        &self,
        old_password: String,
        new_password: String,
    ) -> DatabaseResult<()> {
        // Validate that passwords are provided
        if old_password.is_empty() {
            return Err(DatabaseError::ValidationError("Current password is required".to_string()));
        }

        if new_password.is_empty() {
            return Err(DatabaseError::ValidationError("New password is required".to_string()));
        }

        // Validate new password strength (basic checks)
        if new_password.len() < 8 {
            return Err(DatabaseError::ValidationError(
                "New password must be at least 8 characters long".to_string()
            ));
        }

        if old_password == new_password {
            return Err(DatabaseError::ValidationError(
                "New password must be different from current password".to_string()
            ));
        }

        let db_service = self.database_service.lock().await;
        db_service.change_master_password(old_password, new_password).await
    }

    /// Reset master password (dangerous operation)
    pub async fn reset_master_password(&self) -> DatabaseResult<()> {
        let db_service = self.database_service.lock().await;
        db_service.reset_master_password().await
    }

    /// Update master password configuration
    pub async fn update_master_password_config(
        &self,
        auto_unlock: bool,
        auto_lock_timeout: Option<u32>,
    ) -> DatabaseResult<()> {
        let db_service = self.database_service.lock().await;
        db_service.update_master_password_config(auto_unlock, auto_lock_timeout).await
    }

    /// Get master password configuration
    pub async fn get_master_password_config(&self) -> DatabaseResult<serde_json::Value> {
        let db_service = self.database_service.lock().await;
        let config = db_service.get_master_password_config().await?;

        let config_json = serde_json::json!({
            "autoUnlock": config.auto_unlock,
            "sessionTimeoutMinutes": config.session_timeout_minutes,
            "requireOnStartup": config.require_on_startup,
            "useKeychain": config.use_keychain
        });

        Ok(config_json)
    }
}
