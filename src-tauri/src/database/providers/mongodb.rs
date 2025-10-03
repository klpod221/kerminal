// MongoDB database provider implementation
// Note: This is a placeholder for future MongoDB support

use crate::{
    database::{
        error::DatabaseResult,
        traits::{Database, DatabaseProviderType, SqlValue, ToSqlValue},
    },
    models::{ssh::SSHGroup, ssh::SSHProfile},
};
use async_trait::async_trait;
use std::collections::HashMap;

/// MongoDB database provider (placeholder implementation)
#[allow(dead_code)]
pub struct MongoDBProvider {
    connection_string: String,
}

#[allow(dead_code)]
impl MongoDBProvider {
    pub fn new(connection_string: String) -> Self {
        Self { connection_string }
    }
}

#[async_trait]
impl Database for MongoDBProvider {
    async fn connect(&mut self) -> DatabaseResult<()> {
        unimplemented!("MongoDB provider not yet implemented")
    }

    async fn disconnect(&mut self) -> DatabaseResult<()> {
        unimplemented!("MongoDB provider not yet implemented")
    }

    fn is_connected(&self) -> bool {
        false
    }

    async fn test_connection(&self) -> DatabaseResult<()> {
        unimplemented!("MongoDB provider not yet implemented")
    }

    async fn execute_raw(&self, _query: &str, _params: &[&dyn ToSqlValue]) -> DatabaseResult<u64> {
        unimplemented!("MongoDB provider not yet implemented")
    }

    async fn fetch_raw(
        &self,
        _query: &str,
        _params: &[&dyn ToSqlValue],
    ) -> DatabaseResult<Vec<HashMap<String, SqlValue>>> {
        unimplemented!("MongoDB provider not yet implemented")
    }

    async fn save_ssh_profile(&self, _model: &SSHProfile) -> DatabaseResult<()> {
        unimplemented!("MongoDB provider not yet implemented")
    }

    async fn find_ssh_profile_by_id(&self, _id: &str) -> DatabaseResult<Option<SSHProfile>> {
        unimplemented!("MongoDB provider not yet implemented")
    }

    async fn find_all_ssh_profiles(&self) -> DatabaseResult<Vec<SSHProfile>> {
        unimplemented!("MongoDB provider not yet implemented")
    }

    async fn update_ssh_profile(&self, _model: &SSHProfile) -> DatabaseResult<()> {
        unimplemented!("MongoDB provider not yet implemented")
    }

    async fn delete_ssh_profile(&self, _id: &str) -> DatabaseResult<()> {
        unimplemented!("MongoDB provider not yet implemented")
    }

    async fn save_ssh_group(&self, _model: &SSHGroup) -> DatabaseResult<()> {
        unimplemented!("MongoDB provider not yet implemented")
    }

    async fn find_ssh_group_by_id(&self, _id: &str) -> DatabaseResult<Option<SSHGroup>> {
        unimplemented!("MongoDB provider not yet implemented")
    }

    async fn find_all_ssh_groups(&self) -> DatabaseResult<Vec<SSHGroup>> {
        unimplemented!("MongoDB provider not yet implemented")
    }

    async fn update_ssh_group(&self, _model: &SSHGroup) -> DatabaseResult<()> {
        unimplemented!("MongoDB provider not yet implemented")
    }

    async fn delete_ssh_group(&self, _id: &str) -> DatabaseResult<()> {
        unimplemented!("MongoDB provider not yet implemented")
    }

    async fn save_ssh_key(&self, _model: &crate::models::ssh::SSHKey) -> DatabaseResult<()> {
        unimplemented!("MongoDB provider not yet implemented")
    }

    async fn find_ssh_key_by_id(&self, _id: &str) -> DatabaseResult<Option<crate::models::ssh::SSHKey>> {
        unimplemented!("MongoDB provider not yet implemented")
    }

    async fn find_all_ssh_keys(&self) -> DatabaseResult<Vec<crate::models::ssh::SSHKey>> {
        unimplemented!("MongoDB provider not yet implemented")
    }

    async fn update_ssh_key(&self, _model: &crate::models::ssh::SSHKey) -> DatabaseResult<()> {
        unimplemented!("MongoDB provider not yet implemented")
    }

    async fn delete_ssh_key(&self, _id: &str) -> DatabaseResult<()> {
        unimplemented!("MongoDB provider not yet implemented")
    }

    async fn count_profiles_using_key(&self, _key_id: &str) -> DatabaseResult<u32> {
        unimplemented!("MongoDB provider not yet implemented")
    }

    async fn create_tables(&self) -> DatabaseResult<()> {
        unimplemented!("MongoDB provider not yet implemented")
    }

    async fn drop_tables(&self) -> DatabaseResult<()> {
        unimplemented!("MongoDB provider not yet implemented")
    }

    async fn migrate(&self, _version: u32) -> DatabaseResult<()> {
        unimplemented!("MongoDB provider not yet implemented")
    }

    fn provider_type(&self) -> DatabaseProviderType {
        DatabaseProviderType::MongoDB
    }

    fn connection_info(&self) -> String {
        format!("MongoDB: {}", self.connection_string)
    }
}
