mod auth;
mod command;
mod ssh;
mod tunnel;

use async_trait::async_trait;
use sqlx::PgPool;
use std::sync::Arc;
use tokio::sync::RwLock;

use crate::database::{
    error::{DatabaseError, DatabaseResult},
    traits::Database,
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

    pub(crate) fn get_pool(&self) -> DatabaseResult<&Arc<RwLock<PgPool>>> {
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
        let pool = self.get_pool()?;
        let pool = pool.read().await;

        sqlx::query(
            r#"
            CREATE TABLE IF NOT EXISTS ssh_profiles (
                id VARCHAR(36) PRIMARY KEY,
                name VARCHAR(255) NOT NULL,
                host VARCHAR(255) NOT NULL,
                port INT NOT NULL,
                username VARCHAR(255) NOT NULL,
                group_id VARCHAR(36),
                auth_method TEXT NOT NULL,
                auth_data TEXT NOT NULL,
                description TEXT,
                color VARCHAR(50),
                timeout INT,
                keep_alive BOOLEAN NOT NULL DEFAULT TRUE,
                compression BOOLEAN NOT NULL DEFAULT FALSE,
                created_at TEXT NOT NULL,
                updated_at TEXT NOT NULL,
                device_id VARCHAR(255) NOT NULL,
                version BIGINT NOT NULL DEFAULT 1,
                sync_status TEXT NOT NULL DEFAULT 'Synced'
            )
        "#,
        )
        .execute(&*pool)
        .await
        .map_err(|e| DatabaseError::QueryFailed(e.to_string()))?;

        sqlx::query("CREATE INDEX IF NOT EXISTS idx_ssh_profiles_group_id ON ssh_profiles(group_id)")
            .execute(&*pool)
            .await
            .map_err(|e| DatabaseError::QueryFailed(e.to_string()))?;

        sqlx::query(
            r#"
            CREATE TABLE IF NOT EXISTS ssh_groups (
                id VARCHAR(36) PRIMARY KEY,
                name VARCHAR(255) NOT NULL,
                description TEXT,
                color VARCHAR(50),
                created_at TEXT NOT NULL,
                updated_at TEXT NOT NULL,
                device_id VARCHAR(255) NOT NULL,
                version BIGINT NOT NULL DEFAULT 1,
                sync_status TEXT NOT NULL DEFAULT 'Synced'
            )
        "#,
        )
        .execute(&*pool)
        .await
        .map_err(|e| DatabaseError::QueryFailed(e.to_string()))?;

        sqlx::query(
            r#"
            CREATE TABLE IF NOT EXISTS ssh_keys (
                id VARCHAR(36) PRIMARY KEY,
                name VARCHAR(255) NOT NULL,
                key_type TEXT NOT NULL,
                private_key TEXT NOT NULL,
                public_key TEXT,
                passphrase TEXT,
                fingerprint VARCHAR(255) NOT NULL,
                description TEXT,
                last_used TEXT,
                created_at TEXT NOT NULL,
                updated_at TEXT NOT NULL,
                device_id VARCHAR(255) NOT NULL,
                version BIGINT NOT NULL DEFAULT 1,
                sync_status TEXT NOT NULL DEFAULT 'Synced'
            )
        "#,
        )
        .execute(&*pool)
        .await
        .map_err(|e| DatabaseError::QueryFailed(e.to_string()))?;

        sqlx::query("CREATE INDEX IF NOT EXISTS idx_ssh_keys_fingerprint ON ssh_keys(fingerprint)")
            .execute(&*pool)
            .await
            .map_err(|e| DatabaseError::QueryFailed(e.to_string()))?;

        sqlx::query(
            r#"
            CREATE TABLE IF NOT EXISTS ssh_tunnels (
                id VARCHAR(36) PRIMARY KEY,
                name VARCHAR(255) NOT NULL,
                description TEXT,
                profile_id VARCHAR(36) NOT NULL,
                tunnel_type TEXT NOT NULL,
                local_host VARCHAR(255) NOT NULL,
                local_port INT NOT NULL,
                remote_host VARCHAR(255),
                remote_port INT,
                auto_start BOOLEAN NOT NULL DEFAULT FALSE,
                created_at TEXT NOT NULL,
                updated_at TEXT NOT NULL,
                device_id VARCHAR(255) NOT NULL,
                version BIGINT NOT NULL DEFAULT 1,
                sync_status TEXT NOT NULL DEFAULT 'Synced',
                FOREIGN KEY (profile_id) REFERENCES ssh_profiles(id) ON DELETE CASCADE
            )
        "#,
        )
        .execute(&*pool)
        .await
        .map_err(|e| DatabaseError::QueryFailed(e.to_string()))?;

        sqlx::query("CREATE INDEX IF NOT EXISTS idx_ssh_tunnels_profile_id ON ssh_tunnels(profile_id)")
            .execute(&*pool)
            .await
            .map_err(|e| DatabaseError::QueryFailed(e.to_string()))?;

        sqlx::query(
            r#"
            CREATE TABLE IF NOT EXISTS saved_commands (
                id VARCHAR(36) PRIMARY KEY,
                name VARCHAR(255) NOT NULL,
                description TEXT,
                command TEXT NOT NULL,
                group_id VARCHAR(36),
                tags TEXT,
                is_favorite BOOLEAN NOT NULL DEFAULT FALSE,
                usage_count BIGINT NOT NULL DEFAULT 0,
                last_used_at TEXT,
                created_at TEXT NOT NULL,
                updated_at TEXT NOT NULL,
                device_id VARCHAR(255) NOT NULL,
                version BIGINT NOT NULL DEFAULT 1,
                sync_status TEXT NOT NULL DEFAULT 'Synced'
            )
        "#,
        )
        .execute(&*pool)
        .await
        .map_err(|e| DatabaseError::QueryFailed(e.to_string()))?;

        sqlx::query("CREATE INDEX IF NOT EXISTS idx_saved_commands_group_id ON saved_commands(group_id)")
            .execute(&*pool)
            .await
            .map_err(|e| DatabaseError::QueryFailed(e.to_string()))?;

        sqlx::query(
            r#"
            CREATE TABLE IF NOT EXISTS saved_command_groups (
                id VARCHAR(36) PRIMARY KEY,
                name VARCHAR(255) NOT NULL,
                description TEXT,
                color VARCHAR(50),
                icon VARCHAR(50),
                created_at TEXT NOT NULL,
                updated_at TEXT NOT NULL,
                device_id VARCHAR(255) NOT NULL,
                version BIGINT NOT NULL DEFAULT 1,
                sync_status TEXT NOT NULL DEFAULT 'Synced'
            )
        "#,
        )
        .execute(&*pool)
        .await
        .map_err(|e| DatabaseError::QueryFailed(e.to_string()))?;

        sqlx::query(
            r#"
            CREATE TABLE IF NOT EXISTS master_passwords (
                device_id VARCHAR(255) PRIMARY KEY,
                password_salt BYTEA NOT NULL,
                verification_hash TEXT NOT NULL,
                auto_unlock BOOLEAN NOT NULL DEFAULT FALSE,
                auto_lock_timeout INT,
                created_at TEXT NOT NULL,
                last_verified_at TEXT
            )
        "#,
        )
        .execute(&*pool)
        .await
        .map_err(|e| DatabaseError::QueryFailed(e.to_string()))?;

        sqlx::query(
            r#"
            CREATE TABLE IF NOT EXISTS devices (
                device_id VARCHAR(255) PRIMARY KEY,
                device_name VARCHAR(255) NOT NULL,
                device_type TEXT NOT NULL,
                os_name VARCHAR(255),
                os_version VARCHAR(255),
                created_at TEXT NOT NULL,
                last_seen_at TEXT NOT NULL,
                is_current BOOLEAN NOT NULL DEFAULT FALSE
            )
        "#,
        )
        .execute(&*pool)
        .await
        .map_err(|e| DatabaseError::QueryFailed(e.to_string()))?;

        Ok(())
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

    async fn save_ssh_profile(
        &self,
        model: &crate::models::ssh::SSHProfile,
    ) -> DatabaseResult<()> {
        ssh::save_ssh_profile(self, model).await
    }

    async fn find_ssh_profile_by_id(
        &self,
        id: &str,
    ) -> DatabaseResult<Option<crate::models::ssh::SSHProfile>> {
        ssh::find_ssh_profile_by_id(self, id).await
    }

    async fn find_all_ssh_profiles(&self) -> DatabaseResult<Vec<crate::models::ssh::SSHProfile>> {
        ssh::find_all_ssh_profiles(self).await
    }

    async fn update_ssh_profile(
        &self,
        model: &crate::models::ssh::SSHProfile,
    ) -> DatabaseResult<()> {
        ssh::update_ssh_profile(self, model).await
    }

    async fn delete_ssh_profile(&self, id: &str) -> DatabaseResult<()> {
        ssh::delete_ssh_profile(self, id).await
    }

    async fn save_ssh_group(&self, model: &crate::models::ssh::SSHGroup) -> DatabaseResult<()> {
        ssh::save_ssh_group(self, model).await
    }

    async fn find_ssh_group_by_id(
        &self,
        id: &str,
    ) -> DatabaseResult<Option<crate::models::ssh::SSHGroup>> {
        ssh::find_ssh_group_by_id(self, id).await
    }

    async fn find_all_ssh_groups(&self) -> DatabaseResult<Vec<crate::models::ssh::SSHGroup>> {
        ssh::find_all_ssh_groups(self).await
    }

    async fn update_ssh_group(&self, model: &crate::models::ssh::SSHGroup) -> DatabaseResult<()> {
        ssh::update_ssh_group(self, model).await
    }

    async fn delete_ssh_group(&self, id: &str) -> DatabaseResult<()> {
        ssh::delete_ssh_group(self, id).await
    }

    async fn save_ssh_key(&self, model: &crate::models::ssh::SSHKey) -> DatabaseResult<()> {
        ssh::save_ssh_key(self, model).await
    }

    async fn find_ssh_key_by_id(
        &self,
        id: &str,
    ) -> DatabaseResult<Option<crate::models::ssh::SSHKey>> {
        ssh::find_ssh_key_by_id(self, id).await
    }

    async fn find_all_ssh_keys(&self) -> DatabaseResult<Vec<crate::models::ssh::SSHKey>> {
        ssh::find_all_ssh_keys(self).await
    }

    async fn update_ssh_key(&self, model: &crate::models::ssh::SSHKey) -> DatabaseResult<()> {
        ssh::update_ssh_key(self, model).await
    }

    async fn delete_ssh_key(&self, id: &str) -> DatabaseResult<()> {
        ssh::delete_ssh_key(self, id).await
    }

    async fn count_profiles_using_key(&self, key_id: &str) -> DatabaseResult<u32> {
        ssh::count_profiles_using_key(self, key_id).await
    }

    async fn save_ssh_tunnel(&self, model: &crate::models::ssh::SSHTunnel) -> DatabaseResult<()> {
        tunnel::save_ssh_tunnel(self, model).await
    }

    async fn find_ssh_tunnel_by_id(
        &self,
        id: &str,
    ) -> DatabaseResult<Option<crate::models::ssh::SSHTunnel>> {
        tunnel::find_ssh_tunnel_by_id(self, id).await
    }

    async fn find_all_ssh_tunnels(&self) -> DatabaseResult<Vec<crate::models::ssh::SSHTunnel>> {
        tunnel::find_all_ssh_tunnels(self).await
    }

    async fn find_ssh_tunnels_by_profile_id(
        &self,
        profile_id: &str,
    ) -> DatabaseResult<Vec<crate::models::ssh::SSHTunnel>> {
        tunnel::find_ssh_tunnels_by_profile_id(self, profile_id).await
    }

    async fn find_auto_start_ssh_tunnels(
        &self,
    ) -> DatabaseResult<Vec<crate::models::ssh::SSHTunnel>> {
        tunnel::find_auto_start_ssh_tunnels(self).await
    }

    async fn update_ssh_tunnel(
        &self,
        model: &crate::models::ssh::SSHTunnel,
    ) -> DatabaseResult<()> {
        tunnel::update_ssh_tunnel(self, model).await
    }

    async fn delete_ssh_tunnel(&self, id: &str) -> DatabaseResult<()> {
        tunnel::delete_ssh_tunnel(self, id).await
    }

    async fn delete_ssh_tunnels_by_profile_id(&self, profile_id: &str) -> DatabaseResult<()> {
        tunnel::delete_ssh_tunnels_by_profile_id(self, profile_id).await
    }

    async fn save_saved_command(
        &self,
        model: &crate::models::saved_command::SavedCommand,
    ) -> DatabaseResult<()> {
        command::save_saved_command(self, model).await
    }

    async fn find_saved_command_by_id(
        &self,
        id: &str,
    ) -> DatabaseResult<Option<crate::models::saved_command::SavedCommand>> {
        command::find_saved_command_by_id(self, id).await
    }

    async fn find_all_saved_commands(
        &self,
    ) -> DatabaseResult<Vec<crate::models::saved_command::SavedCommand>> {
        command::find_all_saved_commands(self).await
    }

    async fn update_saved_command(
        &self,
        model: &crate::models::saved_command::SavedCommand,
    ) -> DatabaseResult<()> {
        command::update_saved_command(self, model).await
    }

    async fn delete_saved_command(&self, id: &str) -> DatabaseResult<()> {
        command::delete_saved_command(self, id).await
    }

    async fn increment_command_usage(&self, id: &str) -> DatabaseResult<()> {
        command::increment_command_usage(self, id).await
    }

    async fn toggle_command_favorite(&self, id: &str) -> DatabaseResult<()> {
        command::toggle_command_favorite(self, id).await
    }

    async fn save_saved_command_group(
        &self,
        model: &crate::models::saved_command::SavedCommandGroup,
    ) -> DatabaseResult<()> {
        command::save_saved_command_group(self, model).await
    }

    async fn find_saved_command_group_by_id(
        &self,
        id: &str,
    ) -> DatabaseResult<Option<crate::models::saved_command::SavedCommandGroup>> {
        command::find_saved_command_group_by_id(self, id).await
    }

    async fn find_all_saved_command_groups(
        &self,
    ) -> DatabaseResult<Vec<crate::models::saved_command::SavedCommandGroup>> {
        command::find_all_saved_command_groups(self).await
    }

    async fn update_saved_command_group(
        &self,
        model: &crate::models::saved_command::SavedCommandGroup,
    ) -> DatabaseResult<()> {
        command::update_saved_command_group(self, model).await
    }

    async fn delete_saved_command_group(&self, id: &str) -> DatabaseResult<()> {
        command::delete_saved_command_group(self, id).await
    }

    async fn save_master_password_entry(
        &self,
        entry: &crate::database::encryption::device_keys::MasterPasswordEntry,
    ) -> DatabaseResult<()> {
        auth::save_master_password_entry(self, entry).await
    }

    async fn get_master_password_entry(
        &self,
    ) -> DatabaseResult<Option<crate::database::encryption::device_keys::MasterPasswordEntry>> {
        auth::get_master_password_entry(self).await
    }

    async fn update_master_password_last_verified(&self, device_id: &str) -> DatabaseResult<()> {
        auth::update_master_password_last_verified(self, device_id).await
    }

    async fn delete_master_password_entry(&self, device_id: &str) -> DatabaseResult<()> {
        auth::delete_master_password_entry(self, device_id).await
    }

    async fn save_device(&self, device: &crate::models::auth::Device) -> DatabaseResult<()> {
        auth::save_device(self, device).await
    }

    async fn get_device_by_id(
        &self,
        device_id: &str,
    ) -> DatabaseResult<Option<crate::models::auth::Device>> {
        auth::get_device_by_id(self, device_id).await
    }

    async fn get_current_device(&self) -> DatabaseResult<Option<crate::models::auth::Device>> {
        auth::get_current_device(self).await
    }

    async fn get_all_devices(&self) -> DatabaseResult<Vec<crate::models::auth::Device>> {
        auth::get_all_devices(self).await
    }

    async fn update_device_last_seen(&self, device_id: &str) -> DatabaseResult<()> {
        auth::update_device_last_seen(self, device_id).await
    }

    async fn delete_device(&self, device_id: &str) -> DatabaseResult<()> {
        auth::delete_device(self, device_id).await
    }

    async fn drop_tables(&self) -> DatabaseResult<()> {
        let pool = self.get_pool()?;
        let pool = pool.read().await;

        let tables = vec![
            "ssh_tunnels",
            "ssh_profiles",
            "ssh_groups",
            "ssh_keys",
            "saved_commands",
            "saved_command_groups",
            "master_passwords",
            "devices",
        ];

        for table in tables {
            sqlx::query(&format!("DROP TABLE IF EXISTS {}", table))
                .execute(&*pool)
                .await
                .map_err(|e| DatabaseError::QueryFailed(e.to_string()))?;
        }

        Ok(())
    }

    async fn migrate(&self, _version: u32) -> DatabaseResult<()> {
        Ok(())
    }

    fn provider_type(&self) -> crate::database::traits::DatabaseProviderType {
        crate::database::traits::DatabaseProviderType::MySQL
    }

    fn connection_info(&self) -> String {
        "MySQL connection".to_string()
    }
}
