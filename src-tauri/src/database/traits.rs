
use crate::database::error::DatabaseResult;
use async_trait::async_trait;
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

/// Core database trait that all providers must implement
#[allow(dead_code)]
#[async_trait]
pub trait Database: Send + Sync {
    /// Connect to the database
    async fn connect(&mut self) -> DatabaseResult<()>;

    /// Disconnect from the database
    async fn disconnect(&mut self) -> DatabaseResult<()>;

    /// Check if the database is connected
    fn is_connected(&self) -> bool;

    /// Test the database connection
    async fn test_connection(&self) -> DatabaseResult<()>;

    /// Concrete methods for specific types (object-safe)
    async fn save_ssh_profile(&self, model: &crate::models::ssh::SSHProfile) -> DatabaseResult<()>;
    async fn find_ssh_profile_by_id(
        &self,
        id: &str,
    ) -> DatabaseResult<Option<crate::models::ssh::SSHProfile>>;
    async fn find_all_ssh_profiles(&self) -> DatabaseResult<Vec<crate::models::ssh::SSHProfile>>;
    async fn update_ssh_profile(
        &self,
        model: &crate::models::ssh::SSHProfile,
    ) -> DatabaseResult<()>;
    async fn delete_ssh_profile(&self, id: &str) -> DatabaseResult<()>;

    async fn save_ssh_group(&self, model: &crate::models::ssh::SSHGroup) -> DatabaseResult<()>;
    async fn find_ssh_group_by_id(
        &self,
        id: &str,
    ) -> DatabaseResult<Option<crate::models::ssh::SSHGroup>>;
    async fn find_all_ssh_groups(&self) -> DatabaseResult<Vec<crate::models::ssh::SSHGroup>>;
    async fn update_ssh_group(&self, model: &crate::models::ssh::SSHGroup) -> DatabaseResult<()>;
    async fn delete_ssh_group(&self, id: &str) -> DatabaseResult<()>;

    async fn save_ssh_key(&self, model: &crate::models::ssh::SSHKey) -> DatabaseResult<()>;
    async fn find_ssh_key_by_id(
        &self,
        id: &str,
    ) -> DatabaseResult<Option<crate::models::ssh::SSHKey>>;
    async fn find_all_ssh_keys(&self) -> DatabaseResult<Vec<crate::models::ssh::SSHKey>>;
    async fn update_ssh_key(&self, model: &crate::models::ssh::SSHKey) -> DatabaseResult<()>;
    async fn delete_ssh_key(&self, id: &str) -> DatabaseResult<()>;
    async fn count_profiles_using_key(&self, key_id: &str) -> DatabaseResult<u32>;

    async fn save_ssh_tunnel(&self, model: &crate::models::ssh::SSHTunnel) -> DatabaseResult<()>;
    async fn find_ssh_tunnel_by_id(
        &self,
        id: &str,
    ) -> DatabaseResult<Option<crate::models::ssh::SSHTunnel>>;
    async fn find_all_ssh_tunnels(&self) -> DatabaseResult<Vec<crate::models::ssh::SSHTunnel>>;
    async fn find_ssh_tunnels_by_profile_id(
        &self,
        profile_id: &str,
    ) -> DatabaseResult<Vec<crate::models::ssh::SSHTunnel>>;
    async fn find_auto_start_ssh_tunnels(
        &self,
    ) -> DatabaseResult<Vec<crate::models::ssh::SSHTunnel>>;
    async fn update_ssh_tunnel(&self, model: &crate::models::ssh::SSHTunnel) -> DatabaseResult<()>;
    async fn delete_ssh_tunnel(&self, id: &str) -> DatabaseResult<()>;
    async fn delete_ssh_tunnels_by_profile_id(&self, profile_id: &str) -> DatabaseResult<()>;

    /// Saved Command operations
    async fn save_saved_command(
        &self,
        model: &crate::models::saved_command::SavedCommand,
    ) -> DatabaseResult<()>;
    async fn find_saved_command_by_id(
        &self,
        id: &str,
    ) -> DatabaseResult<Option<crate::models::saved_command::SavedCommand>>;
    async fn find_all_saved_commands(
        &self,
    ) -> DatabaseResult<Vec<crate::models::saved_command::SavedCommand>>;
    async fn update_saved_command(
        &self,
        model: &crate::models::saved_command::SavedCommand,
    ) -> DatabaseResult<()>;
    async fn delete_saved_command(&self, id: &str) -> DatabaseResult<()>;
    async fn increment_command_usage(&self, id: &str) -> DatabaseResult<()>;
    async fn toggle_command_favorite(&self, id: &str) -> DatabaseResult<()>;

    /// Saved Command Group operations
    async fn save_saved_command_group(
        &self,
        model: &crate::models::saved_command::SavedCommandGroup,
    ) -> DatabaseResult<()>;
    async fn find_saved_command_group_by_id(
        &self,
        id: &str,
    ) -> DatabaseResult<Option<crate::models::saved_command::SavedCommandGroup>>;
    async fn find_all_saved_command_groups(
        &self,
    ) -> DatabaseResult<Vec<crate::models::saved_command::SavedCommandGroup>>;
    async fn update_saved_command_group(
        &self,
        model: &crate::models::saved_command::SavedCommandGroup,
    ) -> DatabaseResult<()>;
    async fn delete_saved_command_group(&self, id: &str) -> DatabaseResult<()>;

    /// Master Password operations
    async fn save_master_password_entry(
        &self,
        entry: &crate::database::encryption::device_keys::MasterPasswordEntry,
    ) -> DatabaseResult<()>;
    async fn get_master_password_entry(
        &self,
    ) -> DatabaseResult<Option<crate::database::encryption::device_keys::MasterPasswordEntry>>;
    async fn update_master_password_last_verified(&self, device_id: &str) -> DatabaseResult<()>;
    async fn delete_master_password_entry(&self, device_id: &str) -> DatabaseResult<()>;

    /// Device operations
    async fn save_device(&self, device: &crate::models::auth::Device) -> DatabaseResult<()>;
    async fn get_device_by_id(
        &self,
        device_id: &str,
    ) -> DatabaseResult<Option<crate::models::auth::Device>>;
    async fn get_current_device(&self) -> DatabaseResult<Option<crate::models::auth::Device>>;
    async fn get_all_devices(&self) -> DatabaseResult<Vec<crate::models::auth::Device>>;
    async fn update_device_last_seen(&self, device_id: &str) -> DatabaseResult<()>;
    async fn delete_device(&self, device_id: &str) -> DatabaseResult<()>;

    /// Schema management
    async fn create_tables(&self) -> DatabaseResult<()>;
    async fn drop_tables(&self) -> DatabaseResult<()>;
    async fn migrate(&self, version: u32) -> DatabaseResult<()>;

    /// Provider-specific info
    fn provider_type(&self) -> DatabaseProviderType;
    fn connection_info(&self) -> String;
}

/// Trait for models that can be synchronized across databases
#[allow(dead_code)]
pub trait Syncable: Send + Sync {
    /// Get the table/collection name for this model
    fn table_name() -> &'static str;

    /// Get the unique identifier for this model
    fn id(&self) -> &str;

    /// Get the device ID that created/modified this record
    fn device_id(&self) -> &str;

    /// Get the creation timestamp
    fn created_at(&self) -> DateTime<Utc>;

    /// Get the last modification timestamp
    fn updated_at(&self) -> DateTime<Utc>;

    /// Get the version number for conflict resolution
    fn version(&self) -> u64;

    /// Set the version number
    fn set_version(&mut self, version: u64);

    /// Get sync status
    fn sync_status(&self) -> &SyncStatus;

    /// Set sync status
    fn set_sync_status(&mut self, status: SyncStatus);

    /// Generate a checksum for change detection
    fn checksum(&self) -> String;

    /// Check if this model should be synced to external databases
    fn should_sync(&self) -> bool {
        true
    }

    /// Get fields that should be excluded from sync
    fn excluded_fields() -> Vec<&'static str> {
        vec![]
    }
}

/// Trait for models with encrypted fields
#[allow(dead_code)]
pub trait Encryptable: Send + Sync {
    /// Get fields that should be encrypted
    fn encrypted_fields() -> Vec<&'static str>;

    /// Encrypt sensitive fields
    fn encrypt_fields(&mut self, encryption_service: &dyn EncryptionService) -> DatabaseResult<()>;

    /// Decrypt sensitive fields
    fn decrypt_fields(&mut self, encryption_service: &dyn EncryptionService) -> DatabaseResult<()>;

    /// Check if the model has encrypted data
    fn has_encrypted_data(&self) -> bool;

    /// Get the device ID that encrypted this data (for multi-password support)
    fn encryption_device_id(&self) -> Option<&str>;
}

/// Encryption service trait
#[async_trait]
#[allow(dead_code)]
pub trait EncryptionService: Send + Sync {
    async fn encrypt(&self, data: &[u8], device_id: Option<&str>) -> DatabaseResult<Vec<u8>>;
    async fn decrypt(
        &self,
        encrypted_data: &[u8],
        device_id: Option<&str>,
    ) -> DatabaseResult<Vec<u8>>;
    async fn encrypt_string(&self, data: &str, device_id: Option<&str>) -> DatabaseResult<String>;
    async fn decrypt_string(
        &self,
        encrypted_data: &str,
        device_id: Option<&str>,
    ) -> DatabaseResult<String>;
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum SyncStatus {
    Pending,  // Waiting to be synced
    Syncing,  // Currently syncing
    Synced,   // Successfully synced
    Failed,   // Sync failed
    Conflict, // Conflict detected
}

impl std::fmt::Display for SyncStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            SyncStatus::Pending => write!(f, "Pending"),
            SyncStatus::Syncing => write!(f, "Syncing"),
            SyncStatus::Synced => write!(f, "Synced"),
            SyncStatus::Failed => write!(f, "Failed"),
            SyncStatus::Conflict => write!(f, "Conflict"),
        }
    }
}

impl std::str::FromStr for SyncStatus {
    type Err = String;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            "Pending" => Ok(SyncStatus::Pending),
            "Syncing" => Ok(SyncStatus::Syncing),
            "Synced" => Ok(SyncStatus::Synced),
            "Failed" => Ok(SyncStatus::Failed),
            "Conflict" => Ok(SyncStatus::Conflict),
            _ => Err(format!("Unknown sync status: {}", s)),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum DatabaseProviderType {
    SQLite,
    MySQL,
    PostgreSQL,
    MongoDB,
}
