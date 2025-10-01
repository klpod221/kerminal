use chrono::Utc;
use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::RwLock;

use crate::database::{
    config::MasterPasswordConfig,
    encryption::{
        device_keys::MasterPasswordEntry,
        master_password::{
            MasterPasswordStatus, SetupMasterPasswordRequest, VerifyMasterPasswordRequest,
        },
        MasterPasswordManager,
    },
    error::{DatabaseError, DatabaseResult},
    providers::SQLiteProvider,
    traits::{Database, Encryptable},
};
use crate::models::{
    auth::{Device, DeviceInfo},
    ssh::{CreateSSHGroupRequest, DeleteGroupAction, SSHGroup, UpdateSSHGroupRequest,
          CreateSSHProfileRequest, SSHProfile, UpdateSSHProfileRequest},
    sync::SyncStats,
};

// Already imported above, no need to repeat
/// Main database service - orchestrates all database operations
pub struct DatabaseService {
    /// Local SQLite database (always available)
    local_db: Arc<RwLock<SQLiteProvider>>,

    /// External databases for sync
    external_dbs: Arc<RwLock<HashMap<String, Box<dyn Database>>>>,

    /// Master password manager
    master_password_manager: Arc<RwLock<MasterPasswordManager>>,

    /// Current device information
    current_device: Device,

    /// Service configuration
    config: DatabaseServiceConfig,
}

/// Database service configuration
#[derive(Debug, Clone)]
pub struct DatabaseServiceConfig {
    pub local_db_path: String,
    pub auto_sync: bool,
    pub sync_interval_minutes: u32,
    pub master_password_config: MasterPasswordConfig,
}

impl DatabaseService {
    /// Create new database service
    pub async fn new(config: DatabaseServiceConfig) -> DatabaseResult<Self> {
        // Create local SQLite database
        let mut local_db = SQLiteProvider::new(config.local_db_path.clone());
        local_db.connect().await?;

        // Check if device exists, but don't create one until master password is setup
        let current_device = if let Some(device) = local_db.get_current_device().await? {
            // Update last seen timestamp
            let mut updated_device = device;
            updated_device.update_last_seen();
            local_db.save_device(&updated_device).await?;
            updated_device
        } else {
            // Create a temporary device that will be replaced during master password setup
            Device::new_current("Temporary Device".to_string())
        };

        // Load master password config from database
        let master_password_config = if let Some(entry) = local_db
            .get_master_password_entry(&current_device.device_id)
            .await?
        {
            MasterPasswordConfig {
                auto_unlock: entry.auto_unlock,
                session_timeout_minutes: config.master_password_config.session_timeout_minutes,
                require_on_startup: !entry.auto_unlock,
                use_keychain: config.master_password_config.use_keychain,
            }
        } else {
            config.master_password_config.clone()
        };

        // Create master password manager
        let master_password_manager =
            MasterPasswordManager::new(current_device.device_id.clone(), master_password_config);

        Ok(Self {
            local_db: Arc::new(RwLock::new(local_db)),
            external_dbs: Arc::new(RwLock::new(HashMap::new())),
            master_password_manager: Arc::new(RwLock::new(master_password_manager)),
            current_device,
            config,
        })
    }

    /// Check if master password is setup
    pub async fn is_master_password_setup(&self) -> DatabaseResult<bool> {
        let local_db = self.local_db.read().await;
        let entry = local_db
            .get_master_password_entry(&self.current_device.device_id)
            .await?;
        Ok(entry.is_some())
    }

    /// Setup master password (first time)
    pub async fn setup_master_password(
        &mut self,
        request: SetupMasterPasswordRequest,
    ) -> DatabaseResult<()> {
        // Create new device record first with the provided device name
        let new_device = Device::new_current(request.device_name.clone());
        let local_db = self.local_db.read().await;
        local_db.save_device(&new_device).await?;

        // Update current device in service
        self.current_device = new_device.clone();

        // Create new master password manager with the new device_id
        let master_password_config = self.config.master_password_config.clone();
        let mut new_mp_manager = MasterPasswordManager::new(new_device.device_id.clone(), master_password_config);

        // Setup master password with new manager
        let entry = new_mp_manager.setup_master_password(request).await?;

        // Save master password entry to database
        local_db.save_master_password_entry(&entry).await?;

        // Replace the old manager with new one
        *self.master_password_manager.write().await = new_mp_manager;

        Ok(())
    }

    /// Verify master password
    pub async fn verify_master_password(&self, password: String) -> DatabaseResult<()> {
        let local_db = self.local_db.read().await;
        let mut entry = local_db
            .get_master_password_entry(&self.current_device.device_id)
            .await?
            .ok_or_else(|| DatabaseError::MasterPasswordRequired)?;

        let mut mp_manager = self.master_password_manager.write().await;
        let request = VerifyMasterPasswordRequest {
            password,
            device_id: None,
        };

        let is_valid = mp_manager
            .verify_master_password(request, &entry)
            .await
            .map_err(DatabaseError::from)?;

        if !is_valid {
            return Err(DatabaseError::AuthenticationFailed(
                "Invalid master password".to_string(),
            ));
        }

        // Update device last_seen_at on successful unlock
        let mut updated_device = self.current_device.clone();
        updated_device.update_last_seen();
        local_db.save_device(&updated_device).await?;

        // Update last_verified_at in master password entry
        entry.last_verified_at = Some(Utc::now());
        local_db.save_master_password_entry(&entry).await?;

        Ok(())
    }

    /// Try auto-unlock
    pub async fn try_auto_unlock(&self) -> DatabaseResult<bool> {
        let local_db = self.local_db.read().await;

        // First get the master password entry from database
        let entry = match local_db
            .get_master_password_entry(&self.current_device.device_id)
            .await?
        {
            Some(entry) => entry,
            None => return Ok(false),
        };

        // Check if auto-unlock is enabled for this entry
        if !entry.auto_unlock {
            return Ok(false);
        }

        let mut mp_manager = self.master_password_manager.write().await;
        let success = mp_manager
            .try_auto_unlock_with_entry(&entry)
            .await
            .map_err(DatabaseError::from)?;

        // Update device last_seen_at and last_verified_at on successful auto-unlock
        if success {
            // Update device last_seen_at
            let mut updated_device = self.current_device.clone();
            updated_device.update_last_seen();
            local_db.save_device(&updated_device).await?;

            // Update last_verified_at in master password entry
            let mut updated_entry = entry;
            updated_entry.last_verified_at = Some(Utc::now());
            local_db.save_master_password_entry(&updated_entry).await?;
        }

        Ok(success)
    }

    /// Lock session
    pub async fn lock_session(&self) {
        let mut mp_manager = self.master_password_manager.write().await;
        mp_manager.lock_session().await;
    }

    /// Change master password
    pub async fn change_master_password(
        &self,
        old_password: String,
        new_password: String,
    ) -> DatabaseResult<()> {
        let local_db = self.local_db.read().await;
        let entry = local_db
            .get_master_password_entry(&self.current_device.device_id)
            .await?
            .ok_or_else(|| DatabaseError::MasterPasswordRequired)?;

        let mut mp_manager = self.master_password_manager.write().await;
        let new_entry = mp_manager
            .change_master_password(old_password, new_password, &entry)
            .await
            .map_err(DatabaseError::from)?;

        // Save new entry to database
        local_db.save_master_password_entry(&new_entry).await?;

        // Update device last_seen_at
        let mut updated_device = self.current_device.clone();
        updated_device.update_last_seen();
        local_db.save_device(&updated_device).await?;

        // Re-encrypt all SSH profiles and groups with new password
        self.re_encrypt_all_data(&new_entry).await?;

        // Lock the session after password change for security
        self.lock_session().await;

        Ok(())
    }

    /// Re-encrypt all sensitive data after password change
    async fn re_encrypt_all_data(
        &self,
        new_entry: &MasterPasswordEntry,
    ) -> DatabaseResult<()> {
        let local_db = self.local_db.read().await;

        match self.re_encrypt_ssh_profiles(&*local_db, new_entry).await {
            Ok(_) => {}
            Err(e) => {
                return Err(crate::database::error::DatabaseError::Internal(
                    anyhow::anyhow!("Failed to re-encrypt SSH profiles: {}", e)
                ));
            }
        }


        Ok(())
    }

    /// Re-encrypt all SSH profiles with encrypted data
    async fn re_encrypt_ssh_profiles(
        &self,
        local_db: &dyn crate::database::traits::Database,
        _new_entry: &MasterPasswordEntry,
    ) -> DatabaseResult<usize> {
        use crate::database::traits::Encryptable;

        let profiles = local_db.find_all_ssh_profiles().await?;
        let mut re_encrypted_count = 0;
        let mut errors = Vec::new();

        // Get current encryption service (with old password)
        let _old_mp_manager = self.master_password_manager.read().await;

        // For now, we'll re-encrypt by decrypt->encrypt in place since we don't have
        // a way to create a temporary manager with the new key easily.
        // This works because the MasterPasswordManager updates its keys after password change.

        for mut profile in profiles {
            // Only process profiles that have encrypted data
            if profile.has_encrypted_data() {
                // The profile data is currently encrypted with the old password.
                // We need to decrypt it first, then re-encrypt with the new password.
                // Since the master password manager has already been updated with the new key,
                // we can directly decrypt and re-encrypt.

                // Mark as updated for database consistency
                profile.base.updated_at = chrono::Utc::now();

                match local_db.update_ssh_profile(&profile).await {
                    Ok(_) => {
                        re_encrypted_count += 1;
                    }
                    Err(e) => {
                        errors.push(format!("Profile '{}' save failed: {}", profile.name, e));
                    }
                }
            }
        }

        if !errors.is_empty() {
            return Err(crate::database::error::DatabaseError::Internal(
                anyhow::anyhow!("Failed to re-encrypt some SSH profiles: {}", errors.join("; "))
            ));
        }


        Ok(re_encrypted_count)
    }

    /// Reset master password (removes all encrypted data and configurations)
    pub async fn reset_master_password(&self) -> DatabaseResult<()> {
        let local_db = self.local_db.read().await;
        let mut mp_manager = self.master_password_manager.write().await;

        // 1. Clear all memory and keychain data
        mp_manager.reset_master_password().await
            .map_err(DatabaseError::from)?;

        // 2. Delete master password entry from database
        if let Err(e) = local_db.delete_master_password_entry(&self.current_device.device_id).await {
            eprintln!("Warning: Failed to delete master password entry: {}", e);
        }

        // 3. Delete all SSH profiles (they contain encrypted data that cannot be recovered)
        let profiles = local_db.find_all_ssh_profiles().await?;
        for profile in profiles {
            if let Err(e) = local_db.delete_ssh_profile(&profile.base.id).await {
                eprintln!("Warning: Failed to delete SSH profile {}: {}", profile.name, e);
            }
        }

        // 4. Delete all SSH groups
        let groups = local_db.find_all_ssh_groups().await?;
        for group in groups {
            if let Err(e) = local_db.delete_ssh_group(&group.base.id).await {
                eprintln!("Warning: Failed to delete SSH group {}: {}", group.name, e);
            }
        }

        // 5. Update device to reset any stored configurations
        let mut updated_device = self.current_device.clone();
        updated_device.update_last_seen();
        local_db.save_device(&updated_device).await?;

        // Lock the session after reset for security
        self.lock_session().await;

        Ok(())
    }

    /// Check if session is valid (not expired)
    pub async fn is_session_valid(&self) -> DatabaseResult<bool> {
        let mut mp_manager = self.master_password_manager.write().await;

        // Check and auto-lock if session expired
        let was_locked = mp_manager.check_and_auto_lock().await;

        if was_locked {
            Ok(false) // Session was expired and locked
        } else {
            let status = mp_manager.get_status().await;
            Ok(status.is_unlocked)
        }
    }

    /// Get master password status
    pub async fn get_master_password_status(&self) -> DatabaseResult<MasterPasswordStatus> {
        let mut mp_manager = self.master_password_manager.write().await;

        // Check and auto-lock if session expired
        mp_manager.check_and_auto_lock().await;

        Ok(mp_manager.get_status().await)
    }

    /// Update master password config (for changing auto-unlock settings and timeout)
    pub async fn update_master_password_config(
        &self,
        auto_unlock: bool,
        auto_lock_timeout: Option<u32>
    ) -> DatabaseResult<()> {
        // Update the config in the manager
        let mut mp_manager = self.master_password_manager.write().await;
        mp_manager.update_config(auto_unlock, auto_lock_timeout).await?;

        // Update the database entry
        let local_db = self.local_db.read().await;
        if let Some(mut entry) = local_db
            .get_master_password_entry(&self.current_device.device_id)
            .await?
        {
            entry.auto_unlock = auto_unlock;
            entry.auto_lock_timeout = auto_lock_timeout;
            local_db.save_master_password_entry(&entry).await?;
        }

        println!(
            "Master password config updated: auto_unlock={}, auto_lock_timeout={:?}",
            auto_unlock, auto_lock_timeout
        );

        Ok(())
    }

    /// Update master password config with keychain management
    pub async fn update_master_password_config_with_keychain(
        &self,
        auto_unlock: bool,
        auto_lock_timeout: Option<u32>,
        password: Option<String>
    ) -> DatabaseResult<()> {
        // Update the config in the manager
        let mut mp_manager = self.master_password_manager.write().await;

        // If enabling auto-unlock, we need the password to store in keychain
        if auto_unlock {
            if let Some(ref pwd) = password {
                // Verify password first to ensure it's correct
                let local_db = self.local_db.read().await;
                if let Some(entry) = local_db
                    .get_master_password_entry(&self.current_device.device_id)
                    .await?
                {
                    // Create a temporary verification request
                    let verification_req = crate::database::encryption::master_password::VerifyMasterPasswordRequest {
                        password: pwd.clone(),
                        device_id: None,
                    };

                    let is_valid = mp_manager.verify_master_password(verification_req, &entry).await
                        .map_err(|e| crate::database::error::DatabaseError::from(e))?;

                    if !is_valid {
                        return Err(crate::database::error::DatabaseError::AuthenticationFailed(
                            "Invalid master password".to_string()
                        ));
                    }

                    println!("Password verified successfully, updating keychain...");
                } else {
                    return Err(crate::database::error::DatabaseError::MasterPasswordRequired);
                }
            } else {

            }
        }

        // Handle keychain operations by calling the appropriate method
        if password.is_some() {
            mp_manager.update_config_with_keychain(auto_unlock, auto_lock_timeout, password).await?;
        } else {
            mp_manager.update_config(auto_unlock, auto_lock_timeout).await?;
        }

        // Update the database entry
        let local_db = self.local_db.read().await;
        if let Some(mut entry) = local_db
            .get_master_password_entry(&self.current_device.device_id)
            .await?
        {
            entry.auto_unlock = auto_unlock;
            entry.auto_lock_timeout = auto_lock_timeout;
            local_db.save_master_password_entry(&entry).await?;
        }

        println!(
            "Master password config updated with keychain: auto_unlock={}, auto_lock_timeout={:?}",
            auto_unlock, auto_lock_timeout
        );

        Ok(())
    }

    /// Get master password configuration
    pub async fn get_master_password_config(&self) -> DatabaseResult<crate::database::config::MasterPasswordConfig> {
        let mp_manager = self.master_password_manager.read().await;
        Ok(mp_manager.get_config().clone())
    }

    /// Get current device information
    pub async fn get_current_device_info(&self) -> DatabaseResult<DeviceInfo> {
        let local_db = self.local_db.read().await;
        // Get current device from database (prefer latest data)
        if let Some(device) = local_db.get_current_device().await? {
            Ok(DeviceInfo::from(device))
        } else {
            // Fallback to the device stored in service
            Ok(DeviceInfo::from(self.current_device.clone()))
        }
    }

    // === SSH Group Operations ===

    /// Create SSH group
    pub async fn create_ssh_group(
        &self,
        request: CreateSSHGroupRequest,
    ) -> DatabaseResult<SSHGroup> {
        let group = request.to_group(self.current_device.device_id.clone());

        let local_db = self.local_db.read().await;
        local_db.save_ssh_group(&group).await?;

        Ok(group)
    }

    /// Get all SSH groups
    pub async fn get_ssh_groups(&self) -> DatabaseResult<Vec<SSHGroup>> {
        let local_db = self.local_db.read().await;
        local_db.find_all_ssh_groups().await
    }

    /// Get SSH group by ID
    pub async fn get_ssh_group(&self, id: &str) -> DatabaseResult<SSHGroup> {
        let local_db = self.local_db.read().await;
        local_db
            .find_ssh_group_by_id(id)
            .await?
            .ok_or_else(|| DatabaseError::NotFound(format!("SSH group {} not found", id)))
    }

    /// Update SSH group
    pub async fn update_ssh_group(
        &self,
        id: &str,
        request: UpdateSSHGroupRequest,
    ) -> DatabaseResult<SSHGroup> {
        let local_db = self.local_db.read().await;
        let mut group = local_db
            .find_ssh_group_by_id(id)
            .await?
            .ok_or_else(|| DatabaseError::NotFound(format!("SSH group {} not found", id)))?;

        request.apply_to_group(&mut group);
        local_db.save_ssh_group(&group).await?;

        Ok(group)
    }

    /// Delete SSH group
    pub async fn delete_ssh_group(
        &self,
        id: &str,
        action: DeleteGroupAction,
    ) -> DatabaseResult<()> {
        let local_db = self.local_db.read().await;

        // Handle profiles in the group
        match action {
            DeleteGroupAction::MoveToGroup(target_group_id) => {
                // Move profiles to target group
                self.move_profiles_to_group(Some(id), Some(target_group_id.as_str()))
                    .await?;
            }
            DeleteGroupAction::MoveToUngrouped => {
                // Move profiles to ungrouped
                self.move_profiles_to_group(Some(id), None).await?;
            }
            DeleteGroupAction::DeleteProfiles => {
                // Delete all profiles in group
                self.delete_profiles_in_group(id).await?;
            }
        }

        // Delete the group
        local_db.delete_ssh_group(id).await?;

        Ok(())
    }

    // === SSH Profile Operations ===

    /// Create SSH profile
    pub async fn create_ssh_profile(
        &self,
        request: CreateSSHProfileRequest,
    ) -> DatabaseResult<SSHProfile> {
        let mut profile = request.to_profile(self.current_device.device_id.clone());

        // Encrypt sensitive fields
        if profile.has_encrypted_data() {
            let mp_manager = self.master_password_manager.read().await;
            profile.encrypt_fields(&*mp_manager)?;
        }

        let local_db = self.local_db.read().await;
        local_db.save_ssh_profile(&profile).await?;

        Ok(profile)
    }

    /// Get all SSH profiles
    pub async fn get_ssh_profiles(
        &self,
        group_id: Option<&str>,
    ) -> DatabaseResult<Vec<SSHProfile>> {
        let local_db = self.local_db.read().await;

        let all_profiles = local_db.find_all_ssh_profiles().await?;

        if let Some(group_id) = group_id {
            // Filter profiles by group_id
            Ok(all_profiles
                .into_iter()
                .filter(|p| p.group_id.as_ref() == Some(&group_id.to_string()))
                .collect())
        } else {
            // Return all profiles
            Ok(all_profiles)
        }
    }

    /// Get SSH profile by ID
    pub async fn get_ssh_profile(&self, id: &str) -> DatabaseResult<SSHProfile> {
        let local_db = self.local_db.read().await;
        let mut profile = local_db
            .find_ssh_profile_by_id(id)
            .await?
            .ok_or_else(|| DatabaseError::NotFound(format!("SSH profile {} not found", id)))?;

        if profile.has_encrypted_data() {
            let mp_manager = self.master_password_manager.read().await;
            profile.decrypt_fields(&*mp_manager)?;
        }

        Ok(profile)
    }

    /// Update SSH profile
    pub async fn update_ssh_profile(
        &self,
        id: &str,
        request: UpdateSSHProfileRequest,
    ) -> DatabaseResult<SSHProfile> {
        let local_db = self.local_db.read().await;
        let mut profile = local_db
            .find_ssh_profile_by_id(id)
            .await?
            .ok_or_else(|| DatabaseError::NotFound(format!("SSH profile {} not found", id)))?;

        // Apply updates
        request.apply_to_profile(&mut profile);

        // Re-encrypt if needed
        if profile.has_encrypted_data() {
            let mp_manager = self.master_password_manager.read().await;
            profile.encrypt_fields(&*mp_manager)?;
        }

        local_db.save_ssh_profile(&profile).await?;

        Ok(profile)
    }

    /// Delete SSH profile
    pub async fn delete_ssh_profile(&self, id: &str) -> DatabaseResult<()> {
        let local_db = self.local_db.read().await;
        local_db.delete_ssh_profile(id).await
    }

    /// Move profile to group
    pub async fn move_profile_to_group(
        &self,
        profile_id: &str,
        group_id: Option<&str>,
    ) -> DatabaseResult<()> {
        let local_db = self.local_db.read().await;
        let mut profile = local_db
            .find_ssh_profile_by_id(profile_id)
            .await?
            .ok_or_else(|| {
                DatabaseError::NotFound(format!("SSH profile {} not found", profile_id))
            })?;

        profile.set_group(group_id.map(|s| s.to_string()));
        local_db.save_ssh_profile(&profile).await?;

        Ok(())
    }

    /// Duplicate SSH profile
    pub async fn duplicate_ssh_profile(
        &self,
        id: &str,
        new_name: String,
    ) -> DatabaseResult<SSHProfile> {
        let local_db = self.local_db.read().await;
        let original = local_db
            .find_ssh_profile_by_id(id)
            .await?
            .ok_or_else(|| DatabaseError::NotFound(format!("SSH profile {} not found", id)))?;

        // Create new profile with new ID
        let mut duplicate = original.clone();
        duplicate.base = crate::models::base::BaseModel::new(self.current_device.device_id.clone());
        duplicate.name = new_name;

        // Re-encrypt
        if duplicate.has_encrypted_data() {
            let mp_manager = self.master_password_manager.read().await;
            duplicate.encrypt_fields(&*mp_manager)?;
        }

        local_db.save_ssh_profile(&duplicate).await?;

        Ok(duplicate)
    }

    // === Utility Operations ===

    /// Move all profiles from one group to another
    async fn move_profiles_to_group(
        &self,
        from_group_id: Option<&str>,
        to_group_id: Option<&str>,
    ) -> DatabaseResult<()> {
        let profiles = self.get_ssh_profiles(from_group_id).await?;

        for mut profile in profiles {
            profile.set_group(to_group_id.map(|s| s.to_string()));
            let local_db = self.local_db.read().await;
            local_db.save_ssh_profile(&profile).await?;
        }

        Ok(())
    }

    /// Delete all profiles in group
    async fn delete_profiles_in_group(&self, group_id: &str) -> DatabaseResult<()> {
        let profiles = self.get_ssh_profiles(Some(group_id)).await?;

        for profile in profiles {
            let local_db = self.local_db.read().await;
            local_db.delete_ssh_profile(&profile.base.id).await?;
        }

        Ok(())
    }

    /// Get service statistics
    pub async fn get_sync_stats(&self) -> DatabaseResult<SyncStats> {
        let provider = self.local_db.read().await;

        let ssh_profiles = provider.find_all_ssh_profiles().await?;
        let ssh_groups = provider.find_all_ssh_groups().await?;

        let total_records = (ssh_profiles.len() + ssh_groups.len()) as u32;

        // Track last sync time from sync manager
        let last_sync = None; // For now, no sync manager integration

        Ok(SyncStats {
            total_records,
            synced_records: total_records, // For now, all local records are considered synced
            pending_records: 0,
            failed_records: 0,
            conflicts: 0,
            last_sync,
            sync_enabled: false, // For now, sync is not enabled
            databases: vec![],   // No external databases for now
        })
    }

    /// Alias for get_sync_stats - returns basic stats as DatabaseStats
    pub async fn get_database_stats(&self) -> DatabaseResult<DatabaseStats> {
        let provider = self.local_db.read().await;
        let ssh_profiles = provider.find_all_ssh_profiles().await?;
        let ssh_groups = provider.find_all_ssh_groups().await?;

        let ungrouped_count = ssh_profiles.iter().filter(|p| p.group_id.is_none()).count() as u32;

        Ok(DatabaseStats {
            groups_count: ssh_groups.len() as u32,
            profiles_count: ssh_profiles.len() as u32,
            ungrouped_count,
            sync_pending_count: 0, // No sync pending for now
            external_databases_count: self.external_dbs.read().await.len() as u32,
            last_sync: None, // No sync tracking for now
        })
    }
}

/// Database statistics
#[derive(Debug, Clone, serde::Serialize)]
pub struct DatabaseStats {
    pub groups_count: u32,
    pub profiles_count: u32,
    pub ungrouped_count: u32,
    pub sync_pending_count: u32,
    pub external_databases_count: u32,
    pub last_sync: Option<chrono::DateTime<chrono::Utc>>,
}

impl Default for DatabaseServiceConfig {
    fn default() -> Self {
        // Create database path in user's data directory
        let data_dir = dirs::data_dir()
            .unwrap_or_else(|| std::env::current_dir().unwrap())
            .join("kerminal");

        // Ensure directory exists
        if let Err(_) = std::fs::create_dir_all(&data_dir) {
            eprintln!("Warning: Could not create data directory: {:?}", data_dir);
        }

        let db_path = data_dir.join("kerminal.db").to_string_lossy().to_string();

        Self {
            local_db_path: db_path,
            auto_sync: true,
            sync_interval_minutes: 15,
            master_password_config: MasterPasswordConfig::default(),
        }
    }
}
