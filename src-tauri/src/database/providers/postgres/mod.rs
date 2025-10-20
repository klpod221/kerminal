use async_trait::async_trait;
use sqlx::PgPool;
use std::sync::Arc;
use tokio::sync::RwLock;

use crate::database::{
    error::{DatabaseError, DatabaseResult},
    traits::{Database, SqlValue, ToSqlValue},
};

pub struct PostgreSQLProvider {
    connection_string: String,
    pool: Option<Arc<RwLock<PgPool>>>,
}

impl PostgreSQLProvider {
    pub fn new(connection_string: String) -> Self {
        Self {
            connection_string,
            pool: None,
        }
    }

    fn get_pool(&self) -> DatabaseResult<&Arc<RwLock<PgPool>>> {
        self.pool
            .as_ref()
            .ok_or_else(|| DatabaseError::ConnectionFailed("Database not connected".to_string()))
    }
}

#[async_trait]
impl Database for PostgreSQLProvider {
    async fn connect(&mut self) -> DatabaseResult<()> {
        let pool = PgPool::connect(&self.connection_string)
            .await
            .map_err(|e| DatabaseError::ConnectionFailed(e.to_string()))?;

        self.pool = Some(Arc::new(RwLock::new(pool)));
        Ok(())
    }

    async fn create_tables(&self) -> DatabaseResult<()> {
        Err(DatabaseError::NotImplemented(
            "PostgreSQL table creation not yet implemented".to_string(),
        ))
    }

    async fn disconnect(&mut self) -> DatabaseResult<()> {
        if let Some(pool_arc) = &self.pool {
            let pool = pool_arc.read().await;
            pool.close().await;
        }
        self.pool = None;
        Ok(())
    }

    fn is_connected(&self) -> bool {
        self.pool.is_some()
    }

    async fn test_connection(&self) -> DatabaseResult<()> {
        let pool_arc = self.get_pool()?;
        let pool = pool_arc.read().await;

        sqlx::query("SELECT 1")
            .execute(&*pool)
            .await
            .map_err(|e| DatabaseError::QueryFailed(e.to_string()))?;

        Ok(())
    }

    async fn execute_raw(&self, _query: &str, _params: &[&dyn ToSqlValue]) -> DatabaseResult<u64> {
        Err(DatabaseError::NotImplemented(
            "PostgreSQL execute_raw not yet implemented".to_string(),
        ))
    }

    async fn fetch_raw(
        &self,
        _query: &str,
        _params: &[&dyn ToSqlValue],
    ) -> DatabaseResult<Vec<std::collections::HashMap<String, SqlValue>>> {
        Err(DatabaseError::NotImplemented(
            "PostgreSQL fetch_raw not yet implemented".to_string(),
        ))
    }

    async fn save_ssh_profile(
        &self,
        _model: &crate::models::ssh::SSHProfile,
    ) -> DatabaseResult<()> {
        Err(DatabaseError::NotImplemented(
            "PostgreSQL save_ssh_profile not yet implemented".to_string(),
        ))
    }

    async fn find_ssh_profile_by_id(
        &self,
        _id: &str,
    ) -> DatabaseResult<Option<crate::models::ssh::SSHProfile>> {
        Err(DatabaseError::NotImplemented(
            "PostgreSQL find_ssh_profile_by_id not yet implemented".to_string(),
        ))
    }

    async fn find_all_ssh_profiles(&self) -> DatabaseResult<Vec<crate::models::ssh::SSHProfile>> {
        Err(DatabaseError::NotImplemented(
            "PostgreSQL find_all_ssh_profiles not yet implemented".to_string(),
        ))
    }

    async fn update_ssh_profile(
        &self,
        _model: &crate::models::ssh::SSHProfile,
    ) -> DatabaseResult<()> {
        Err(DatabaseError::NotImplemented(
            "PostgreSQL update_ssh_profile not yet implemented".to_string(),
        ))
    }

    async fn delete_ssh_profile(&self, _id: &str) -> DatabaseResult<()> {
        Err(DatabaseError::NotImplemented(
            "PostgreSQL delete_ssh_profile not yet implemented".to_string(),
        ))
    }

    async fn save_ssh_group(&self, _model: &crate::models::ssh::SSHGroup) -> DatabaseResult<()> {
        Err(DatabaseError::NotImplemented(
            "PostgreSQL save_ssh_group not yet implemented".to_string(),
        ))
    }

    async fn find_ssh_group_by_id(
        &self,
        _id: &str,
    ) -> DatabaseResult<Option<crate::models::ssh::SSHGroup>> {
        Err(DatabaseError::NotImplemented(
            "PostgreSQL find_ssh_group_by_id not yet implemented".to_string(),
        ))
    }

    async fn find_all_ssh_groups(&self) -> DatabaseResult<Vec<crate::models::ssh::SSHGroup>> {
        Err(DatabaseError::NotImplemented(
            "PostgreSQL find_all_ssh_groups not yet implemented".to_string(),
        ))
    }

    async fn update_ssh_group(&self, _model: &crate::models::ssh::SSHGroup) -> DatabaseResult<()> {
        Err(DatabaseError::NotImplemented(
            "PostgreSQL update_ssh_group not yet implemented".to_string(),
        ))
    }

    async fn delete_ssh_group(&self, _id: &str) -> DatabaseResult<()> {
        Err(DatabaseError::NotImplemented(
            "PostgreSQL delete_ssh_group not yet implemented".to_string(),
        ))
    }

    async fn save_ssh_key(&self, _model: &crate::models::ssh::SSHKey) -> DatabaseResult<()> {
        Err(DatabaseError::NotImplemented(
            "PostgreSQL save_ssh_key not yet implemented".to_string(),
        ))
    }

    async fn find_ssh_key_by_id(
        &self,
        _id: &str,
    ) -> DatabaseResult<Option<crate::models::ssh::SSHKey>> {
        Err(DatabaseError::NotImplemented(
            "PostgreSQL find_ssh_key_by_id not yet implemented".to_string(),
        ))
    }

    async fn find_all_ssh_keys(&self) -> DatabaseResult<Vec<crate::models::ssh::SSHKey>> {
        Err(DatabaseError::NotImplemented(
            "PostgreSQL find_all_ssh_keys not yet implemented".to_string(),
        ))
    }

    async fn update_ssh_key(&self, _model: &crate::models::ssh::SSHKey) -> DatabaseResult<()> {
        Err(DatabaseError::NotImplemented(
            "PostgreSQL update_ssh_key not yet implemented".to_string(),
        ))
    }

    async fn delete_ssh_key(&self, _id: &str) -> DatabaseResult<()> {
        Err(DatabaseError::NotImplemented(
            "PostgreSQL delete_ssh_key not yet implemented".to_string(),
        ))
    }

    async fn count_profiles_using_key(&self, _key_id: &str) -> DatabaseResult<u32> {
        Err(DatabaseError::NotImplemented(
            "PostgreSQL count_profiles_using_key not yet implemented".to_string(),
        ))
    }

    async fn save_ssh_tunnel(&self, _model: &crate::models::ssh::SSHTunnel) -> DatabaseResult<()> {
        Err(DatabaseError::NotImplemented(
            "PostgreSQL save_ssh_tunnel not yet implemented".to_string(),
        ))
    }

    async fn find_ssh_tunnel_by_id(
        &self,
        _id: &str,
    ) -> DatabaseResult<Option<crate::models::ssh::SSHTunnel>> {
        Err(DatabaseError::NotImplemented(
            "PostgreSQL find_ssh_tunnel_by_id not yet implemented".to_string(),
        ))
    }

    async fn find_all_ssh_tunnels(&self) -> DatabaseResult<Vec<crate::models::ssh::SSHTunnel>> {
        Err(DatabaseError::NotImplemented(
            "PostgreSQL find_all_ssh_tunnels not yet implemented".to_string(),
        ))
    }

    async fn find_ssh_tunnels_by_profile_id(
        &self,
        _profile_id: &str,
    ) -> DatabaseResult<Vec<crate::models::ssh::SSHTunnel>> {
        Err(DatabaseError::NotImplemented(
            "PostgreSQL find_ssh_tunnels_by_profile_id not yet implemented".to_string(),
        ))
    }

    async fn find_auto_start_ssh_tunnels(
        &self,
    ) -> DatabaseResult<Vec<crate::models::ssh::SSHTunnel>> {
        Err(DatabaseError::NotImplemented(
            "PostgreSQL find_auto_start_ssh_tunnels not yet implemented".to_string(),
        ))
    }

    async fn update_ssh_tunnel(
        &self,
        _model: &crate::models::ssh::SSHTunnel,
    ) -> DatabaseResult<()> {
        Err(DatabaseError::NotImplemented(
            "PostgreSQL update_ssh_tunnel not yet implemented".to_string(),
        ))
    }

    async fn delete_ssh_tunnel(&self, _id: &str) -> DatabaseResult<()> {
        Err(DatabaseError::NotImplemented(
            "PostgreSQL delete_ssh_tunnel not yet implemented".to_string(),
        ))
    }

    async fn save_saved_command(
        &self,
        _model: &crate::models::saved_command::SavedCommand,
    ) -> DatabaseResult<()> {
        Err(DatabaseError::NotImplemented(
            "PostgreSQL save_saved_command not yet implemented".to_string(),
        ))
    }

    async fn find_saved_command_by_id(
        &self,
        _id: &str,
    ) -> DatabaseResult<Option<crate::models::saved_command::SavedCommand>> {
        Err(DatabaseError::NotImplemented(
            "PostgreSQL find_saved_command_by_id not yet implemented".to_string(),
        ))
    }

    async fn find_all_saved_commands(
        &self,
    ) -> DatabaseResult<Vec<crate::models::saved_command::SavedCommand>> {
        Err(DatabaseError::NotImplemented(
            "PostgreSQL find_all_saved_commands not yet implemented".to_string(),
        ))
    }

    async fn update_saved_command(
        &self,
        _model: &crate::models::saved_command::SavedCommand,
    ) -> DatabaseResult<()> {
        Err(DatabaseError::NotImplemented(
            "PostgreSQL update_saved_command not yet implemented".to_string(),
        ))
    }

    async fn delete_saved_command(&self, _id: &str) -> DatabaseResult<()> {
        Err(DatabaseError::NotImplemented(
            "PostgreSQL delete_saved_command not yet implemented".to_string(),
        ))
    }

    async fn increment_command_usage(&self, _id: &str) -> DatabaseResult<()> {
        Err(DatabaseError::NotImplemented(
            "PostgreSQL increment_command_usage not yet implemented".to_string(),
        ))
    }

    async fn toggle_command_favorite(&self, _id: &str) -> DatabaseResult<()> {
        Err(DatabaseError::NotImplemented(
            "PostgreSQL toggle_command_favorite not yet implemented".to_string(),
        ))
    }

    async fn save_saved_command_group(
        &self,
        _model: &crate::models::saved_command::SavedCommandGroup,
    ) -> DatabaseResult<()> {
        Err(DatabaseError::NotImplemented(
            "PostgreSQL save_saved_command_group not yet implemented".to_string(),
        ))
    }

    async fn find_saved_command_group_by_id(
        &self,
        _id: &str,
    ) -> DatabaseResult<Option<crate::models::saved_command::SavedCommandGroup>> {
        Err(DatabaseError::NotImplemented(
            "PostgreSQL find_saved_command_group_by_id not yet implemented".to_string(),
        ))
    }

    async fn find_all_saved_command_groups(
        &self,
    ) -> DatabaseResult<Vec<crate::models::saved_command::SavedCommandGroup>> {
        Err(DatabaseError::NotImplemented(
            "PostgreSQL find_all_saved_command_groups not yet implemented".to_string(),
        ))
    }

    async fn update_saved_command_group(
        &self,
        _model: &crate::models::saved_command::SavedCommandGroup,
    ) -> DatabaseResult<()> {
        Err(DatabaseError::NotImplemented(
            "PostgreSQL update_saved_command_group not yet implemented".to_string(),
        ))
    }

    async fn delete_saved_command_group(&self, _id: &str) -> DatabaseResult<()> {
        Err(DatabaseError::NotImplemented(
            "PostgreSQL delete_saved_command_group not yet implemented".to_string(),
        ))
    }

    async fn save_master_password_entry(
        &self,
        _entry: &crate::database::encryption::device_keys::MasterPasswordEntry,
    ) -> DatabaseResult<()> {
        Err(DatabaseError::NotImplemented(
            "PostgreSQL save_master_password_entry not yet implemented".to_string(),
        ))
    }

    async fn get_master_password_entry(
        &self,
    ) -> DatabaseResult<Option<crate::database::encryption::device_keys::MasterPasswordEntry>> {
        Err(DatabaseError::NotImplemented(
            "PostgreSQL get_master_password_entry not yet implemented".to_string(),
        ))
    }

    async fn update_master_password_last_verified(&self, _device_id: &str) -> DatabaseResult<()> {
        Err(DatabaseError::NotImplemented(
            "PostgreSQL update_master_password_last_verified not yet implemented".to_string(),
        ))
    }

    async fn delete_master_password_entry(&self, _device_id: &str) -> DatabaseResult<()> {
        Err(DatabaseError::NotImplemented(
            "PostgreSQL delete_master_password_entry not yet implemented".to_string(),
        ))
    }

    async fn save_device(&self, _device: &crate::models::auth::Device) -> DatabaseResult<()> {
        Err(DatabaseError::NotImplemented(
            "PostgreSQL save_device not yet implemented".to_string(),
        ))
    }

    async fn get_device_by_id(
        &self,
        _device_id: &str,
    ) -> DatabaseResult<Option<crate::models::auth::Device>> {
        Err(DatabaseError::NotImplemented(
            "PostgreSQL get_device_by_id not yet implemented".to_string(),
        ))
    }

    async fn get_current_device(&self) -> DatabaseResult<Option<crate::models::auth::Device>> {
        Err(DatabaseError::NotImplemented(
            "PostgreSQL get_current_device not yet implemented".to_string(),
        ))
    }

    async fn get_all_devices(&self) -> DatabaseResult<Vec<crate::models::auth::Device>> {
        Err(DatabaseError::NotImplemented(
            "PostgreSQL get_all_devices not yet implemented".to_string(),
        ))
    }

    async fn update_device_last_seen(&self, _device_id: &str) -> DatabaseResult<()> {
        Err(DatabaseError::NotImplemented(
            "PostgreSQL update_device_last_seen not yet implemented".to_string(),
        ))
    }

    async fn delete_device(&self, _device_id: &str) -> DatabaseResult<()> {
        Err(DatabaseError::NotImplemented(
            "PostgreSQL delete_device not yet implemented".to_string(),
        ))
    }

    async fn delete_ssh_tunnels_by_profile_id(&self, _profile_id: &str) -> DatabaseResult<()> {
        Err(DatabaseError::NotImplemented(
            "PostgreSQL delete_ssh_tunnels_by_profile_id not yet implemented".to_string(),
        ))
    }

    async fn drop_tables(&self) -> DatabaseResult<()> {
        Err(DatabaseError::NotImplemented(
            "PostgreSQL drop_tables not yet implemented".to_string(),
        ))
    }

    async fn migrate(&self, _version: u32) -> DatabaseResult<()> {
        Err(DatabaseError::NotImplemented(
            "PostgreSQL migrate not yet implemented".to_string(),
        ))
    }

    fn provider_type(&self) -> crate::database::traits::DatabaseProviderType {
        crate::database::traits::DatabaseProviderType::PostgreSQL
    }

    fn connection_info(&self) -> String {
        "PostgreSQL connection".to_string()
    }
}
