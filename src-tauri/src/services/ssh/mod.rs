pub mod connection_pool;
pub mod key;

use anyhow;
use std::sync::Arc;
use tokio::sync::Mutex;

pub use connection_pool::SSHConnectionPool;
pub use key::SSHKeyService;

use crate::database::{error::DatabaseResult, service::DatabaseService};
use crate::models::ssh::{
    CreateSSHGroupRequest, CreateSSHProfileRequest, DeleteGroupAction, SSHGroup, SSHProfile,
    TestSSHConnectionRequest, UpdateSSHGroupRequest, UpdateSSHProfileRequest,
};

/// SSH service for handling SSH profiles and groups
pub struct SSHService {
    database_service: Arc<Mutex<DatabaseService>>,
    ssh_key_service: Arc<Mutex<SSHKeyService>>,
}

impl SSHService {
    /// Create new SSHService instance
    pub fn new(
        database_service: Arc<Mutex<DatabaseService>>,
        ssh_key_service: Arc<Mutex<SSHKeyService>>,
    ) -> Self {
        Self {
            database_service,
            ssh_key_service,
        }
    }

    /// Create new SSH group
    pub async fn create_ssh_group(
        &self,
        request: CreateSSHGroupRequest,
    ) -> DatabaseResult<SSHGroup> {
        let db_service = self.database_service.lock().await;
        db_service.create_ssh_group(request).await
    }

    /// Get all SSH groups
    pub async fn get_ssh_groups(&self) -> DatabaseResult<Vec<SSHGroup>> {
        let db_service = self.database_service.lock().await;
        db_service.get_ssh_groups().await
    }

    /// Get SSH group by ID
    pub async fn get_ssh_group(&self, id: &str) -> DatabaseResult<SSHGroup> {
        let db_service = self.database_service.lock().await;
        db_service.get_ssh_group(id).await
    }

    /// Update SSH group
    pub async fn update_ssh_group(
        &self,
        id: &str,
        request: UpdateSSHGroupRequest,
    ) -> DatabaseResult<SSHGroup> {
        let db_service = self.database_service.lock().await;
        db_service.update_ssh_group(id, request).await
    }

    /// Delete SSH group with action for existing profiles
    pub async fn delete_ssh_group(
        &self,
        id: &str,
        action: DeleteGroupAction,
    ) -> DatabaseResult<()> {
        let db_service = self.database_service.lock().await;
        db_service.delete_ssh_group(id, action).await
    }

    /// Create new SSH profile
    pub async fn create_ssh_profile(
        &self,
        request: CreateSSHProfileRequest,
    ) -> DatabaseResult<SSHProfile> {
        let db_service = self.database_service.lock().await;
        db_service.create_ssh_profile(request).await
    }

    /// Get all SSH profiles
    pub async fn get_ssh_profiles(&self) -> DatabaseResult<Vec<SSHProfile>> {
        let db_service = self.database_service.lock().await;
        db_service.get_ssh_profiles(None).await
    }

    /// Get SSH profile by ID
    pub async fn get_ssh_profile(&self, id: &str) -> DatabaseResult<SSHProfile> {
        let db_service = self.database_service.lock().await;
        db_service.get_ssh_profile(id).await
    }

    /// Update SSH profile
    pub async fn update_ssh_profile(
        &self,
        id: &str,
        request: UpdateSSHProfileRequest,
    ) -> DatabaseResult<SSHProfile> {
        let db_service = self.database_service.lock().await;
        db_service.update_ssh_profile(id, request).await
    }

    /// Delete SSH profile
    pub async fn delete_ssh_profile(&self, id: &str) -> DatabaseResult<()> {
        let db_service = self.database_service.lock().await;
        db_service.delete_ssh_profile(id).await
    }

    /// Move profile to different group
    pub async fn move_profile_to_group(
        &self,
        profile_id: &str,
        group_id: Option<&str>,
    ) -> DatabaseResult<()> {
        let db_service = self.database_service.lock().await;
        db_service.move_profile_to_group(profile_id, group_id).await
    }

    /// Duplicate SSH profile with new name
    pub async fn duplicate_ssh_profile(
        &self,
        id: &str,
        new_name: String,
    ) -> DatabaseResult<SSHProfile> {
        let db_service = self.database_service.lock().await;
        db_service.duplicate_ssh_profile(id, new_name).await
    }

    /// Test SSH connection with a profile
    pub async fn test_ssh_connection(
        &self,
        request: TestSSHConnectionRequest,
    ) -> DatabaseResult<()> {
        use crate::core::terminal::ssh::SSHTerminal;
        use crate::models::ssh::profile::AuthData;
        use crate::models::terminal::{TerminalConfig, TerminalType};

        let device_id = {
            let db_service = self.database_service.lock().await;
            db_service.get_device_id().to_string()
        };

        let profile = request.to_profile(device_id);

        let resolved_key = match &profile.auth_data {
            AuthData::KeyReference { key_id } => {
                let key_service = self.ssh_key_service.lock().await;
                Some(
                    key_service
                        .resolve_key_for_auth(key_id)
                        .await
                        .map_err(|e| {
                            crate::database::error::DatabaseError::Internal(anyhow::anyhow!(
                                e.to_string()
                            ))
                        })?,
                )
            }
            AuthData::Password { .. } | AuthData::Certificate { .. } => None,
        };

        let config = TerminalConfig {
            terminal_type: TerminalType::SSH,
            local_config: None,
            ssh_profile_id: Some(profile.base.id.clone()),
            ssh_config_host: None,
            ssh_config_password: None,
        };

        let mut ssh_terminal = SSHTerminal::new(
            "test-connection".to_string(),
            config,
            profile,
            Some(self.database_service.clone()),
        )
        .map_err(|e| {
            crate::database::error::DatabaseError::Internal(anyhow::anyhow!(e.to_string()))
        })?;

        let connect_result = if let Some(resolved_key) = resolved_key {
            ssh_terminal
                .connect_with_resolved_data(Some(resolved_key))
                .await
        } else {
            ssh_terminal.connect().await
        };

        let _ = ssh_terminal.disconnect().await;

        connect_result.map_err(|e| {
            crate::database::error::DatabaseError::Internal(anyhow::anyhow!(e.to_string()))
        })?;

        Ok(())
    }
}
