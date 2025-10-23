use chrono::Utc;
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
    auth::Device,
    saved_command::{
        CreateSavedCommandGroupRequest, CreateSavedCommandRequest, SavedCommand, SavedCommandGroup,
        UpdateSavedCommandGroupRequest, UpdateSavedCommandRequest,
    },
    ssh::{
        CreateSSHGroupRequest, CreateSSHKeyRequest, CreateSSHProfileRequest, DeleteGroupAction,
        SSHGroup, SSHKey, SSHProfile, UpdateSSHGroupRequest, UpdateSSHKeyRequest,
        UpdateSSHProfileRequest,
    },
    sync::SyncStats,
};

/// Main database service - orchestrates all database operations
pub struct DatabaseService {
    /// Local SQLite database (always available)
    local_db: Arc<RwLock<SQLiteProvider>>,

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
    pub master_password_config: MasterPasswordConfig,
}

impl DatabaseService {
    /// Create new database service
    pub async fn new(config: DatabaseServiceConfig) -> DatabaseResult<Self> {
        let mut local_db = SQLiteProvider::new(config.local_db_path.clone());
        local_db.connect().await?;

        let current_device = if let Some(device) = local_db.get_current_device().await? {
            let mut updated_device = device;
            updated_device.update_last_seen();
            local_db.save_device(&updated_device).await?;
            updated_device
        } else {
            Device::new_current("Temporary Device".to_string())
        };

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

        let master_password_manager =
            MasterPasswordManager::new(current_device.device_id.clone(), master_password_config);

        Ok(Self {
            local_db: Arc::new(RwLock::new(local_db)),
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

    /// Get current device ID
    pub fn get_device_id(&self) -> &str {
        &self.current_device.device_id
    }

    /// Get master password manager Arc for encryption operations
    pub fn get_master_password_manager_arc(&self) -> Arc<RwLock<MasterPasswordManager>> {
        self.master_password_manager.clone()
    }

    /// Get local database Arc for sync operations
    pub fn get_local_database(&self) -> Arc<RwLock<SQLiteProvider>> {
        self.local_db.clone()
    }

    /// Setup master password (first time)
    pub async fn setup_master_password(
        &mut self,
        request: SetupMasterPasswordRequest,
    ) -> DatabaseResult<()> {
        let new_device = Device::new_current(request.device_name.clone());
        let local_db = self.local_db.read().await;
        local_db.save_device(&new_device).await?;

        self.current_device = new_device.clone();

        let master_password_config = self.config.master_password_config.clone();
        let mut new_mp_manager =
            MasterPasswordManager::new(new_device.device_id.clone(), master_password_config);

        let entry = new_mp_manager.setup_master_password(request).await?;

        local_db.save_master_password_entry(&entry).await?;

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

        let mut updated_device = self.current_device.clone();
        updated_device.update_last_seen();
        local_db.save_device(&updated_device).await?;

        entry.last_verified_at = Some(Utc::now());
        local_db.save_master_password_entry(&entry).await?;

        Ok(())
    }

    /// Try auto-unlock
    pub async fn try_auto_unlock(&self) -> DatabaseResult<bool> {
        let local_db = self.local_db.read().await;

        let entry = match local_db
            .get_master_password_entry(&self.current_device.device_id)
            .await?
        {
            Some(entry) => entry,
            None => return Ok(false),
        };

        if !entry.auto_unlock {
            return Ok(false);
        }

        let mut mp_manager = self.master_password_manager.write().await;
        let success = mp_manager
            .try_auto_unlock_with_entry(&entry)
            .await
            .map_err(DatabaseError::from)?;

        if success {
            let mut updated_device = self.current_device.clone();
            updated_device.update_last_seen();
            local_db.save_device(&updated_device).await?;

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

        local_db.save_master_password_entry(&new_entry).await?;

        let mut updated_device = self.current_device.clone();
        updated_device.update_last_seen();
        local_db.save_device(&updated_device).await?;

        self.re_encrypt_all_data(&new_entry).await?;

        self.lock_session().await;

        Ok(())
    }

    /// Re-encrypt all sensitive data after password change
    async fn re_encrypt_all_data(&self, new_entry: &MasterPasswordEntry) -> DatabaseResult<()> {
        let local_db = self.local_db.read().await;

        match self.re_encrypt_ssh_profiles(&*local_db, new_entry).await {
            Ok(_) => {}
            Err(e) => {
                return Err(crate::database::error::DatabaseError::Internal(
                    anyhow::anyhow!("Failed to re-encrypt SSH profiles: {}", e),
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

        let _old_mp_manager = self.master_password_manager.read().await;

        for mut profile in profiles {
            if profile.has_encrypted_data() {
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
                anyhow::anyhow!(
                    "Failed to re-encrypt some SSH profiles: {}",
                    errors.join("; ")
                ),
            ));
        }

        Ok(re_encrypted_count)
    }

    /// Reset master password (removes all encrypted data and configurations)
    pub async fn reset_master_password(&self) -> DatabaseResult<()> {
        let local_db = self.local_db.read().await;
        let mut mp_manager = self.master_password_manager.write().await;

        mp_manager
            .reset_master_password()
            .await
            .map_err(DatabaseError::from)?;

        if let Err(e) = local_db
            .delete_master_password_entry(&self.current_device.device_id)
            .await
        {
            eprintln!("Warning: Failed to delete master password entry: {}", e);
        }

        let profiles = local_db.find_all_ssh_profiles().await?;
        for profile in profiles {
            if let Err(e) = local_db.delete_ssh_profile(&profile.base.id).await {
                eprintln!(
                    "Warning: Failed to delete SSH profile {}: {}",
                    profile.name, e
                );
            }
        }

        let groups = local_db.find_all_ssh_groups().await?;
        for group in groups {
            if let Err(e) = local_db.delete_ssh_group(&group.base.id).await {
                eprintln!("Warning: Failed to delete SSH group {}: {}", group.name, e);
            }
        }

        let mut updated_device = self.current_device.clone();
        updated_device.update_last_seen();
        local_db.save_device(&updated_device).await?;

        self.lock_session().await;

        Ok(())
    }

    /// Check if session is valid (not expired)
    pub async fn is_session_valid(&self) -> DatabaseResult<bool> {
        let mut mp_manager = self.master_password_manager.write().await;

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

        mp_manager.check_and_auto_lock().await;

        Ok(mp_manager.get_status().await)
    }

    /// Update master password config (for changing auto-unlock settings and timeout)
    pub async fn update_master_password_config(
        &self,
        auto_unlock: bool,
        auto_lock_timeout: Option<u32>,
    ) -> DatabaseResult<()> {
        let mut mp_manager = self.master_password_manager.write().await;
        mp_manager
            .update_config(auto_unlock, auto_lock_timeout)
            .await?;

        let local_db = self.local_db.read().await;
        if let Some(mut entry) = local_db
            .get_master_password_entry(&self.current_device.device_id)
            .await?
        {
            entry.auto_unlock = auto_unlock;
            entry.auto_lock_timeout = auto_lock_timeout;
            local_db.save_master_password_entry(&entry).await?;
        }

        Ok(())
    }

    /// Update master password config with keychain management
    pub async fn update_master_password_config_with_keychain(
        &self,
        auto_unlock: bool,
        auto_lock_timeout: Option<u32>,
        password: Option<String>,
    ) -> DatabaseResult<()> {
        let mut mp_manager = self.master_password_manager.write().await;

        if auto_unlock {
            if let Some(ref pwd) = password {
                let local_db = self.local_db.read().await;
                if let Some(entry) = local_db
                    .get_master_password_entry(&self.current_device.device_id)
                    .await?
                {
                    let verification_req =
                        crate::database::encryption::master_password::VerifyMasterPasswordRequest {
                            password: pwd.clone(),
                            device_id: None,
                        };

                    let is_valid = mp_manager
                        .verify_master_password(verification_req, &entry)
                        .await
                        .map_err(crate::database::error::DatabaseError::from)?;

                    if !is_valid {
                        return Err(crate::database::error::DatabaseError::AuthenticationFailed(
                            "Invalid master password".to_string(),
                        ));
                    }
                } else {
                    return Err(crate::database::error::DatabaseError::MasterPasswordRequired);
                }
            }
        }

        if password.is_some() {
            mp_manager
                .update_config_with_keychain(auto_unlock, auto_lock_timeout, password)
                .await?;
        } else {
            mp_manager
                .update_config(auto_unlock, auto_lock_timeout)
                .await?;
        }

        let local_db = self.local_db.read().await;
        if let Some(mut entry) = local_db
            .get_master_password_entry(&self.current_device.device_id)
            .await?
        {
            entry.auto_unlock = auto_unlock;
            entry.auto_lock_timeout = auto_lock_timeout;
            local_db.save_master_password_entry(&entry).await?;
        }

        Ok(())
    }

    /// Get master password configuration
    pub async fn get_master_password_config(
        &self,
    ) -> DatabaseResult<crate::database::config::MasterPasswordConfig> {
        let mp_manager = self.master_password_manager.read().await;
        Ok(mp_manager.get_config().clone())
    }

    /// Get all devices from database
    pub async fn get_all_devices(&self) -> DatabaseResult<Vec<crate::models::auth::Device>> {
        let local_db = self.local_db.read().await;
        local_db.get_all_devices().await
    }

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

        match action {
            DeleteGroupAction::MoveToGroup(target_group_id) => {
                self.move_profiles_to_group(Some(id), Some(target_group_id.as_str()))
                    .await?;
            }
            DeleteGroupAction::MoveToUngrouped => {
                self.move_profiles_to_group(Some(id), None).await?;
            }
            DeleteGroupAction::DeleteProfiles => {
                self.delete_profiles_in_group(id).await?;
            }
        }

        local_db.delete_ssh_group(id).await?;

        Ok(())
    }

    /// Create SSH profile
    pub async fn create_ssh_profile(
        &self,
        request: CreateSSHProfileRequest,
    ) -> DatabaseResult<SSHProfile> {
        let mut profile = request.to_profile(self.current_device.device_id.clone());

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
            Ok(all_profiles
                .into_iter()
                .filter(|p| p.group_id.as_ref() == Some(&group_id.to_string()))
                .collect())
        } else {
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

        request.apply_to_profile(&mut profile);

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

        let mut duplicate = original.clone();
        duplicate.base = crate::models::base::BaseModel::new(self.current_device.device_id.clone());
        duplicate.name = new_name;

        if duplicate.has_encrypted_data() {
            let mp_manager = self.master_password_manager.read().await;
            duplicate.encrypt_fields(&*mp_manager)?;
        }

        local_db.save_ssh_profile(&duplicate).await?;

        Ok(duplicate)
    }

    /// Create SSH key
    pub async fn create_ssh_key(&self, request: CreateSSHKeyRequest) -> DatabaseResult<SSHKey> {
        let mut key = request.to_key(self.current_device.device_id.clone());

        let mp_manager = self.master_password_manager.read().await;
        key.encrypt_fields(&*mp_manager)?;

        let local_db = self.local_db.read().await;
        local_db.save_ssh_key(&key).await?;

        Ok(key)
    }

    /// Get all SSH keys
    pub async fn get_ssh_keys(&self) -> DatabaseResult<Vec<SSHKey>> {
        let local_db = self.local_db.read().await;
        let keys = local_db.find_all_ssh_keys().await?;
        Ok(keys)
    }

    /// Get SSH key by ID
    pub async fn get_ssh_key(&self, id: &str) -> DatabaseResult<SSHKey> {
        let local_db = self.local_db.read().await;
        let mut key = local_db
            .find_ssh_key_by_id(id)
            .await?
            .ok_or_else(|| DatabaseError::NotFound(format!("SSH key {} not found", id)))?;

        let mp_manager = self.master_password_manager.read().await;
        key.decrypt_fields(&*mp_manager)?;

        Ok(key)
    }

    /// Update SSH key (metadata only - name, description)
    pub async fn update_ssh_key(
        &self,
        id: &str,
        request: UpdateSSHKeyRequest,
    ) -> DatabaseResult<SSHKey> {
        let local_db = self.local_db.read().await;
        let mut key = local_db
            .find_ssh_key_by_id(id)
            .await?
            .ok_or_else(|| DatabaseError::NotFound(format!("SSH key {} not found", id)))?;

        request.apply_to_key(&mut key);

        local_db.save_ssh_key(&key).await?;

        Ok(key)
    }

    /// Delete SSH key
    pub async fn delete_ssh_key(&self, id: &str, force: bool) -> DatabaseResult<()> {
        let count = self.count_profiles_using_key(id).await?;

        if count > 0 && !force {
            return Err(DatabaseError::Conflict(format!(
                "Cannot delete SSH key: {} profile(s) are using it. Use force=true to delete anyway.",
                count
            )));
        }

        let local_db = self.local_db.read().await;
        local_db.delete_ssh_key(id).await
    }

    /// Count profiles using a specific key
    pub async fn count_profiles_using_key(&self, key_id: &str) -> DatabaseResult<u32> {
        let local_db = self.local_db.read().await;
        local_db.count_profiles_using_key(key_id).await
    }

    /// Mark SSH key as recently used
    pub async fn mark_key_used(&self, key_id: &str) -> DatabaseResult<()> {
        let local_db = self.local_db.read().await;
        let mut key = local_db
            .find_ssh_key_by_id(key_id)
            .await?
            .ok_or_else(|| DatabaseError::NotFound(format!("SSH key {} not found", key_id)))?;

        key.mark_used();
        local_db.save_ssh_key(&key).await
    }

    /// Create SSH tunnel
    pub async fn create_ssh_tunnel(
        &self,
        request: crate::models::ssh::CreateSSHTunnelRequest,
    ) -> DatabaseResult<crate::models::ssh::SSHTunnel> {
        let _profile = self.get_ssh_profile(&request.profile_id).await?;

        let tunnel = crate::models::ssh::SSHTunnel::new(
            self.current_device.device_id.clone(),
            request.name,
            request.profile_id,
            request.tunnel_type,
            request.local_host,
            request.local_port,
            request.remote_host,
            request.remote_port,
        );

        let mut tunnel = tunnel;
        tunnel.description = request.description;
        tunnel.auto_start = request.auto_start.unwrap_or(false);

        tunnel.validate().map_err(DatabaseError::ValidationError)?;

        let local_db = self.local_db.read().await;
        local_db.save_ssh_tunnel(&tunnel).await?;

        Ok(tunnel)
    }

    /// Get all SSH tunnels
    pub async fn get_ssh_tunnels(&self) -> DatabaseResult<Vec<crate::models::ssh::SSHTunnel>> {
        let local_db = self.local_db.read().await;
        local_db.find_all_ssh_tunnels().await
    }

    /// Get SSH tunnel by ID
    pub async fn get_ssh_tunnel(&self, id: &str) -> DatabaseResult<crate::models::ssh::SSHTunnel> {
        let local_db = self.local_db.read().await;
        local_db
            .find_ssh_tunnel_by_id(id)
            .await?
            .ok_or_else(|| DatabaseError::NotFound(format!("SSH tunnel {} not found", id)))
    }

    /// Get SSH tunnels that have auto-start enabled
    pub async fn get_auto_start_ssh_tunnels(
        &self,
    ) -> DatabaseResult<Vec<crate::models::ssh::SSHTunnel>> {
        let local_db = self.local_db.read().await;
        local_db.find_auto_start_ssh_tunnels().await
    }

    /// Update SSH tunnel
    pub async fn update_ssh_tunnel(
        &self,
        id: &str,
        request: crate::models::ssh::UpdateSSHTunnelRequest,
    ) -> DatabaseResult<crate::models::ssh::SSHTunnel> {
        let local_db = self.local_db.read().await;
        let mut tunnel = local_db
            .find_ssh_tunnel_by_id(id)
            .await?
            .ok_or_else(|| DatabaseError::NotFound(format!("SSH tunnel {} not found", id)))?;

        if let Some(name) = request.name {
            tunnel.name = name;
        }
        if let Some(description) = request.description {
            tunnel.description = Some(description);
        }
        if let Some(profile_id) = request.profile_id {
            let _profile = self.get_ssh_profile(&profile_id).await?;
            tunnel.profile_id = profile_id;
        }
        if let Some(tunnel_type) = request.tunnel_type {
            tunnel.tunnel_type = tunnel_type;
        }
        if let Some(local_host) = request.local_host {
            tunnel.local_host = local_host;
        }
        if let Some(local_port) = request.local_port {
            tunnel.local_port = local_port;
        }
        if let Some(remote_host) = request.remote_host {
            tunnel.remote_host = Some(remote_host);
        }
        if let Some(remote_port) = request.remote_port {
            tunnel.remote_port = Some(remote_port);
        }
        if let Some(auto_start) = request.auto_start {
            tunnel.auto_start = auto_start;
        }

        tunnel.validate().map_err(DatabaseError::ValidationError)?;

        tunnel.base.touch();

        local_db.save_ssh_tunnel(&tunnel).await?;

        Ok(tunnel)
    }

    /// Delete SSH tunnel
    pub async fn delete_ssh_tunnel(&self, id: &str) -> DatabaseResult<()> {
        let local_db = self.local_db.read().await;
        local_db.delete_ssh_tunnel(id).await
    }

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

    /// Create saved command
    pub async fn create_saved_command(
        &self,
        request: CreateSavedCommandRequest,
    ) -> DatabaseResult<SavedCommand> {
        let command = SavedCommand::new(
            self.current_device.device_id.clone(),
            request.name,
            request.command,
            request.group_id,
        );

        let mut command = command;
        command.description = request.description;
        command.tags = request.tags;
        command.is_favorite = request.is_favorite.unwrap_or(false);

        let local_db = self.local_db.read().await;
        local_db.save_saved_command(&command).await?;

        Ok(command)
    }

    /// Get all saved commands
    pub async fn get_saved_commands(&self) -> DatabaseResult<Vec<SavedCommand>> {
        let local_db = self.local_db.read().await;
        local_db.find_all_saved_commands().await
    }

    /// Get saved command by ID
    pub async fn get_saved_command(&self, id: &str) -> DatabaseResult<SavedCommand> {
        let local_db = self.local_db.read().await;
        local_db
            .find_saved_command_by_id(id)
            .await?
            .ok_or_else(|| DatabaseError::NotFound(format!("Saved command {} not found", id)))
    }

    /// Update saved command
    pub async fn update_saved_command(
        &self,
        id: &str,
        request: UpdateSavedCommandRequest,
    ) -> DatabaseResult<SavedCommand> {
        let local_db = self.local_db.read().await;
        let mut command = local_db
            .find_saved_command_by_id(id)
            .await?
            .ok_or_else(|| DatabaseError::NotFound(format!("Saved command {} not found", id)))?;

        if let Some(name) = request.name {
            command.name = name;
        }
        if let Some(description) = request.description {
            command.description = Some(description);
        }
        if let Some(cmd) = request.command {
            command.command = cmd;
        }
        if let Some(group_id) = request.group_id {
            command.group_id = Some(group_id);
        }
        if let Some(tags) = request.tags {
            command.tags = Some(tags);
        }
        if let Some(is_favorite) = request.is_favorite {
            command.is_favorite = is_favorite;
        }

        command.base.updated_at = Utc::now();
        command.base.version += 1;

        local_db.save_saved_command(&command).await?;
        Ok(command)
    }

    /// Delete saved command
    pub async fn delete_saved_command(&self, id: &str) -> DatabaseResult<()> {
        let local_db = self.local_db.read().await;
        local_db.delete_saved_command(id).await
    }

    /// Increment command usage count
    pub async fn increment_command_usage(&self, id: &str) -> DatabaseResult<()> {
        let local_db = self.local_db.read().await;
        let mut command = local_db
            .find_saved_command_by_id(id)
            .await?
            .ok_or_else(|| DatabaseError::NotFound(format!("Saved command {} not found", id)))?;

        command.usage_count += 1;
        command.last_used_at = Some(Utc::now().to_rfc3339());
        command.base.updated_at = Utc::now();
        command.base.version += 1;

        local_db.save_saved_command(&command).await?;
        Ok(())
    }

    /// Toggle command favorite status
    pub async fn toggle_command_favorite(&self, id: &str) -> DatabaseResult<SavedCommand> {
        let local_db = self.local_db.read().await;
        let mut command = local_db
            .find_saved_command_by_id(id)
            .await?
            .ok_or_else(|| DatabaseError::NotFound(format!("Saved command {} not found", id)))?;

        command.is_favorite = !command.is_favorite;
        command.base.updated_at = Utc::now();
        command.base.version += 1;

        local_db.save_saved_command(&command).await?;
        Ok(command)
    }

    /// Create saved command group
    pub async fn create_saved_command_group(
        &self,
        request: CreateSavedCommandGroupRequest,
    ) -> DatabaseResult<SavedCommandGroup> {
        let group = SavedCommandGroup::new(self.current_device.device_id.clone(), request.name);

        let mut group = group;
        group.description = request.description;
        group.color = request.color;
        group.icon = request.icon;

        let local_db = self.local_db.read().await;
        local_db.save_saved_command_group(&group).await?;

        Ok(group)
    }

    /// Get all saved command groups
    pub async fn get_saved_command_groups(&self) -> DatabaseResult<Vec<SavedCommandGroup>> {
        let local_db = self.local_db.read().await;
        local_db.find_all_saved_command_groups().await
    }

    /// Get saved command group by ID
    pub async fn get_saved_command_group(&self, id: &str) -> DatabaseResult<SavedCommandGroup> {
        let local_db = self.local_db.read().await;
        local_db
            .find_saved_command_group_by_id(id)
            .await?
            .ok_or_else(|| DatabaseError::NotFound(format!("Saved command group {} not found", id)))
    }

    /// Update saved command group
    pub async fn update_saved_command_group(
        &self,
        id: &str,
        request: UpdateSavedCommandGroupRequest,
    ) -> DatabaseResult<SavedCommandGroup> {
        let local_db = self.local_db.read().await;
        let mut group = local_db
            .find_saved_command_group_by_id(id)
            .await?
            .ok_or_else(|| {
                DatabaseError::NotFound(format!("Saved command group {} not found", id))
            })?;

        if let Some(name) = request.name {
            group.name = name;
        }
        if let Some(description) = request.description {
            group.description = Some(description);
        }
        if let Some(color) = request.color {
            group.color = Some(color);
        }
        if let Some(icon) = request.icon {
            group.icon = Some(icon);
        }

        group.base.updated_at = Utc::now();
        group.base.version += 1;

        local_db.save_saved_command_group(&group).await?;
        Ok(group)
    }

    /// Delete saved command group
    pub async fn delete_saved_command_group(&self, id: &str) -> DatabaseResult<()> {
        let local_db = self.local_db.read().await;

        let commands = local_db.find_saved_commands_by_group_id(Some(id)).await?;
        for mut command in commands {
            command.group_id = None;
            command.base.updated_at = Utc::now();
            command.base.version += 1;
            local_db.save_saved_command(&command).await?;
        }

        local_db.delete_saved_command_group(id).await
    }

    /// Get service statistics
    pub async fn get_sync_stats(&self) -> DatabaseResult<SyncStats> {
        let provider = self.local_db.read().await;

        let ssh_profiles = provider.find_all_ssh_profiles().await?;
        let ssh_groups = provider.find_all_ssh_groups().await?;

        let total_records = (ssh_profiles.len() + ssh_groups.len()) as u32;

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
}

impl Default for DatabaseServiceConfig {
    fn default() -> Self {
        let data_dir = dirs::data_dir()
            .unwrap_or_else(|| std::env::current_dir().unwrap())
            .join("kerminal");

        if std::fs::create_dir_all(&data_dir).is_err() {
            eprintln!("Warning: Could not create data directory: {:?}", data_dir);
        }

        let db_path = data_dir.join("kerminal.db").to_string_lossy().to_string();

        Self {
            local_db_path: db_path,
            master_password_config: MasterPasswordConfig::default(),
        }
    }
}

impl DatabaseService {
    pub async fn save_external_database(
        &self,
        config: &crate::models::sync::ExternalDatabaseConfig,
    ) -> DatabaseResult<()> {
        let db = self.local_db.read().await;
        db.save_external_database(config).await
    }

    #[allow(dead_code)]
    pub async fn find_external_database_by_id(
        &self,
        id: &str,
    ) -> DatabaseResult<Option<crate::models::sync::ExternalDatabaseConfig>> {
        let db = self.local_db.read().await;
        db.find_external_database_by_id(id).await
    }

    pub async fn find_all_external_databases(
        &self,
    ) -> DatabaseResult<Vec<crate::models::sync::ExternalDatabaseConfig>> {
        let db = self.local_db.read().await;
        db.find_all_external_databases().await
    }

    pub async fn delete_external_database(&self, id: &str) -> DatabaseResult<()> {
        let db = self.local_db.read().await;
        db.delete_external_database(id).await
    }

    pub async fn find_unresolved_conflicts(
        &self,
    ) -> DatabaseResult<Vec<crate::models::sync::conflict::SyncConflict>> {
        let db = self.local_db.read().await;
        db.find_unresolved_conflicts().await
    }

    pub async fn resolve_conflict(
        &self,
        conflict_id: &str,
        strategy: crate::models::sync::external_db::ConflictResolutionStrategy,
    ) -> DatabaseResult<()> {
        let db = self.local_db.read().await;
        db.resolve_conflict(conflict_id, strategy).await
    }

    pub async fn find_recent_sync_operations(
        &self,
        limit: i32,
    ) -> DatabaseResult<Vec<crate::models::sync::operation::SyncOperation>> {
        let db = self.local_db.read().await;
        db.find_recent_sync_operations(limit).await
    }
}
