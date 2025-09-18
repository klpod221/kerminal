use std::sync::Arc;
use tokio::sync::Mutex;

use crate::database::{
    error::DatabaseResult,
    service::DatabaseService,
};
use crate::models::ssh::{
    CreateSSHGroupRequest, DeleteGroupAction, SSHGroup, UpdateSSHGroupRequest,
    CreateSSHProfileRequest, SSHProfile, UpdateSSHProfileRequest,
};

/// SSH service for handling SSH profiles and groups
pub struct SSHService {
    database_service: Arc<Mutex<DatabaseService>>,
}

impl SSHService {
    /// Create new SSHService instance
    pub fn new(database_service: Arc<Mutex<DatabaseService>>) -> Self {
        Self { database_service }
    }

    // === SSH Group Management ===

    /// Create new SSH group
    pub async fn create_ssh_group(&self, request: CreateSSHGroupRequest) -> DatabaseResult<SSHGroup> {
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

    // === SSH Profile Management ===

    /// Create new SSH profile
    pub async fn create_ssh_profile(&self, request: CreateSSHProfileRequest) -> DatabaseResult<SSHProfile> {
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
}