mod auth;
mod command;
mod ssh;
pub mod sync_ops;
mod tunnel;

use async_trait::async_trait;
use sqlx::SqlitePool;
use std::sync::Arc;
use tokio::sync::RwLock;

use crate::database::{
    error::{DatabaseError, DatabaseResult},
    traits::{Database, DatabaseProviderType},
};

/// SQLite database provider implementation
pub struct SQLiteProvider {
    database_path: String,
    pool: Option<Arc<RwLock<SqlitePool>>>,
}

impl SQLiteProvider {
    /// Create new SQLite provider
    pub fn new(database_path: String) -> Self {
        Self {
            database_path,
            pool: None,
        }
    }

    /// Get database pool reference
    pub(crate) fn get_pool(&self) -> DatabaseResult<&Arc<RwLock<SqlitePool>>> {
        self.pool
            .as_ref()
            .ok_or_else(|| DatabaseError::ConnectionFailed("Database not connected".to_string()))
    }
}

#[async_trait]
impl Database for SQLiteProvider {
    async fn connect(&mut self) -> DatabaseResult<()> {
        if let Some(parent) = std::path::Path::new(&self.database_path).parent() {
            std::fs::create_dir_all(parent).map_err(|e| {
                DatabaseError::ConnectionFailed(format!(
                    "Failed to create database directory: {}",
                    e
                ))
            })?;
        }

        let connection_string = format!("sqlite://{}?mode=rwc", self.database_path);
        let pool = SqlitePool::connect(&connection_string)
            .await
            .map_err(|e| DatabaseError::ConnectionFailed(e.to_string()))?;

        self.pool = Some(Arc::new(RwLock::new(pool)));
        self.create_tables().await?;

        Ok(())
    }

    async fn create_tables(&self) -> DatabaseResult<()> {
        let pool = self.get_pool()?;
        let pool = pool.read().await;

        sqlx::query(
            r#"
            CREATE TABLE IF NOT EXISTS ssh_profiles (
                id TEXT PRIMARY KEY,
                name TEXT NOT NULL,
                host TEXT NOT NULL,
                port INTEGER NOT NULL,
                username TEXT NOT NULL,
                group_id TEXT,
                auth_method TEXT NOT NULL,
                auth_data TEXT NOT NULL,
                description TEXT,
                color TEXT,
                timeout INTEGER,
                keep_alive BOOLEAN NOT NULL DEFAULT true,
                compression BOOLEAN NOT NULL DEFAULT false,
                created_at TEXT NOT NULL,
                updated_at TEXT NOT NULL,
                device_id TEXT NOT NULL,
                version INTEGER NOT NULL DEFAULT 1,
                sync_status TEXT NOT NULL DEFAULT 'Clean'
            )
        "#,
        )
        .execute(&*pool)
        .await
        .map_err(|e| DatabaseError::QueryFailed(e.to_string()))?;

        sqlx::query(
            r#"
            CREATE TABLE IF NOT EXISTS ssh_groups (
                id TEXT PRIMARY KEY,
                name TEXT NOT NULL,
                description TEXT,
                color TEXT,
                created_at TEXT NOT NULL,
                updated_at TEXT NOT NULL,
                device_id TEXT NOT NULL,
                version INTEGER NOT NULL DEFAULT 1,
                sync_status TEXT NOT NULL DEFAULT 'Clean'
            )
        "#,
        )
        .execute(&*pool)
        .await
        .map_err(|e| DatabaseError::QueryFailed(e.to_string()))?;

        sqlx::query(
            r#"
            CREATE TABLE IF NOT EXISTS ssh_keys (
                id TEXT PRIMARY KEY,
                name TEXT NOT NULL,
                key_type TEXT NOT NULL,
                private_key TEXT NOT NULL,
                public_key TEXT,
                passphrase TEXT,
                fingerprint TEXT NOT NULL,
                description TEXT,
                last_used TEXT,
                created_at TEXT NOT NULL,
                updated_at TEXT NOT NULL,
                device_id TEXT NOT NULL,
                version INTEGER NOT NULL DEFAULT 1,
                sync_status TEXT NOT NULL DEFAULT 'Clean'
            )
        "#,
        )
        .execute(&*pool)
        .await
        .map_err(|e| DatabaseError::QueryFailed(e.to_string()))?;

        sqlx::query(
            r#"
            CREATE INDEX IF NOT EXISTS idx_ssh_keys_fingerprint
            ON ssh_keys(fingerprint)
        "#,
        )
        .execute(&*pool)
        .await
        .map_err(|e| DatabaseError::QueryFailed(e.to_string()))?;

        sqlx::query(
            r#"
            CREATE TABLE IF NOT EXISTS ssh_tunnels (
                id TEXT PRIMARY KEY,
                name TEXT NOT NULL,
                description TEXT,
                profile_id TEXT NOT NULL,
                tunnel_type TEXT NOT NULL,
                local_host TEXT NOT NULL,
                local_port INTEGER NOT NULL,
                remote_host TEXT,
                remote_port INTEGER,
                auto_start BOOLEAN NOT NULL DEFAULT false,
                created_at TEXT NOT NULL,
                updated_at TEXT NOT NULL,
                device_id TEXT NOT NULL,
                version INTEGER NOT NULL DEFAULT 1,
                sync_status TEXT NOT NULL DEFAULT 'Clean',
                FOREIGN KEY (profile_id) REFERENCES ssh_profiles(id) ON DELETE CASCADE
            )
        "#,
        )
        .execute(&*pool)
        .await
        .map_err(|e| DatabaseError::QueryFailed(e.to_string()))?;

        sqlx::query(
            r#"
            CREATE INDEX IF NOT EXISTS idx_ssh_tunnels_profile_id
            ON ssh_tunnels(profile_id)
        "#,
        )
        .execute(&*pool)
        .await
        .map_err(|e| DatabaseError::QueryFailed(e.to_string()))?;

        sqlx::query(
            r#"
            CREATE TABLE IF NOT EXISTS master_passwords (
                device_id TEXT PRIMARY KEY,
                password_salt BLOB NOT NULL,
                verification_hash TEXT NOT NULL,
                auto_unlock BOOLEAN NOT NULL DEFAULT false,
                auto_lock_timeout INTEGER,
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
                device_id TEXT PRIMARY KEY,
                device_name TEXT NOT NULL,
                device_type TEXT NOT NULL,
                os_name TEXT NOT NULL,
                os_version TEXT NOT NULL,
                created_at TEXT NOT NULL,
                last_seen_at TEXT NOT NULL,
                is_current BOOLEAN NOT NULL DEFAULT false
            )
        "#,
        )
        .execute(&*pool)
        .await
        .map_err(|e| DatabaseError::QueryFailed(e.to_string()))?;

        sqlx::query(
            r#"
            CREATE TABLE IF NOT EXISTS sync_metadata (
                id TEXT PRIMARY KEY,
                table_name TEXT NOT NULL,
                record_id TEXT NOT NULL,
                last_sync_at TEXT NOT NULL,
                sync_hash TEXT NOT NULL,
                conflict_resolution TEXT,
                created_at TEXT NOT NULL,
                updated_at TEXT NOT NULL
            )
        "#,
        )
        .execute(&*pool)
        .await
        .map_err(|e| DatabaseError::QueryFailed(e.to_string()))?;

        sqlx::query(
            r#"
            CREATE TABLE IF NOT EXISTS saved_commands (
                id TEXT PRIMARY KEY,
                name TEXT NOT NULL,
                description TEXT,
                command TEXT NOT NULL,
                group_id TEXT,
                tags TEXT,
                is_favorite BOOLEAN DEFAULT 0,
                usage_count INTEGER DEFAULT 0,
                last_used_at TEXT,
                created_at TEXT NOT NULL,
                updated_at TEXT NOT NULL,
                device_id TEXT NOT NULL,
                version INTEGER NOT NULL DEFAULT 1,
                sync_status TEXT NOT NULL DEFAULT 'synced',
                FOREIGN KEY (group_id) REFERENCES saved_command_groups(id) ON DELETE SET NULL
            )
        "#,
        )
        .execute(&*pool)
        .await
        .map_err(|e| DatabaseError::QueryFailed(e.to_string()))?;

        sqlx::query(
            r#"
            CREATE TABLE IF NOT EXISTS saved_command_groups (
                id TEXT PRIMARY KEY,
                name TEXT NOT NULL,
                description TEXT,
                color TEXT,
                icon TEXT,
                created_at TEXT NOT NULL,
                updated_at TEXT NOT NULL,
                device_id TEXT NOT NULL,
                version INTEGER NOT NULL DEFAULT 1,
                sync_status TEXT NOT NULL DEFAULT 'synced'
            )
        "#,
        )
        .execute(&*pool)
        .await
        .map_err(|e| DatabaseError::QueryFailed(e.to_string()))?;

        sqlx::query(
            r#"
            CREATE TABLE IF NOT EXISTS external_databases (
                id TEXT PRIMARY KEY,
                name TEXT NOT NULL,
                db_type TEXT NOT NULL,
                connection_details_encrypted TEXT NOT NULL,
                sync_settings TEXT NOT NULL,
                is_active BOOLEAN NOT NULL DEFAULT false,
                auto_sync_enabled BOOLEAN NOT NULL DEFAULT false,
                last_sync_at TEXT,
                created_at TEXT NOT NULL,
                updated_at TEXT NOT NULL,
                device_id TEXT NOT NULL,
                version INTEGER NOT NULL DEFAULT 1,
                sync_status TEXT NOT NULL DEFAULT 'Pending'
            )
        "#,
        )
        .execute(&*pool)
        .await
        .map_err(|e| DatabaseError::QueryFailed(e.to_string()))?;

        sqlx::query(
            r#"
            CREATE TABLE IF NOT EXISTS sync_operations (
                id TEXT PRIMARY KEY,
                operation_type TEXT NOT NULL,
                entity_type TEXT NOT NULL,
                entity_id TEXT NOT NULL,
                source_db TEXT NOT NULL,
                target_db TEXT NOT NULL,
                status TEXT NOT NULL,
                error_message TEXT,
                started_at TEXT NOT NULL,
                completed_at TEXT
            )
        "#,
        )
        .execute(&*pool)
        .await
        .map_err(|e| DatabaseError::QueryFailed(e.to_string()))?;

        sqlx::query(
            r#"
            CREATE INDEX IF NOT EXISTS idx_sync_operations_entity
            ON sync_operations(entity_type, entity_id)
        "#,
        )
        .execute(&*pool)
        .await
        .map_err(|e| DatabaseError::QueryFailed(e.to_string()))?;

        sqlx::query(
            r#"
            CREATE INDEX IF NOT EXISTS idx_sync_operations_started_at
            ON sync_operations(started_at DESC)
        "#,
        )
        .execute(&*pool)
        .await
        .map_err(|e| DatabaseError::QueryFailed(e.to_string()))?;

        sqlx::query(
            r#"
            CREATE TABLE IF NOT EXISTS sync_conflicts (
                id TEXT PRIMARY KEY,
                entity_type TEXT NOT NULL,
                entity_id TEXT NOT NULL,
                local_version INTEGER NOT NULL,
                remote_version INTEGER NOT NULL,
                local_data TEXT NOT NULL,
                remote_data TEXT NOT NULL,
                resolution_strategy TEXT,
                resolved BOOLEAN NOT NULL DEFAULT false,
                created_at TEXT NOT NULL,
                resolved_at TEXT
            )
        "#,
        )
        .execute(&*pool)
        .await
        .map_err(|e| DatabaseError::QueryFailed(e.to_string()))?;

        sqlx::query(
            r#"
            CREATE INDEX IF NOT EXISTS idx_sync_conflicts_entity
            ON sync_conflicts(entity_type, entity_id)
        "#,
        )
        .execute(&*pool)
        .await
        .map_err(|e| DatabaseError::QueryFailed(e.to_string()))?;

        sqlx::query(
            r#"
            CREATE INDEX IF NOT EXISTS idx_sync_conflicts_resolved
            ON sync_conflicts(resolved)
        "#,
        )
        .execute(&*pool)
        .await
        .map_err(|e| DatabaseError::QueryFailed(e.to_string()))?;

        Ok(())
    }

    async fn test_connection(&self) -> DatabaseResult<()> {
        let pool_arc = self.get_pool()?;
        let pool = pool_arc.read().await;

        sqlx::query("SELECT 1")
            .fetch_one(&*pool)
            .await
            .map_err(|e| DatabaseError::ConnectionFailed(e.to_string()))?;

        Ok(())
    }

    async fn drop_tables(&self) -> DatabaseResult<()> {
        let pool_arc = self.get_pool()?;
        let pool = pool_arc.read().await;

        sqlx::query("DROP TABLE IF EXISTS ssh_profiles")
            .execute(&*pool)
            .await
            .ok();
        sqlx::query("DROP TABLE IF EXISTS ssh_groups")
            .execute(&*pool)
            .await
            .ok();
        sqlx::query("DROP TABLE IF EXISTS devices")
            .execute(&*pool)
            .await
            .ok();
        sqlx::query("DROP TABLE IF EXISTS master_passwords")
            .execute(&*pool)
            .await
            .ok();
        sqlx::query("DROP TABLE IF EXISTS sync_metadata")
            .execute(&*pool)
            .await
            .ok();

        Ok(())
    }

    async fn migrate(&self, version: u32) -> DatabaseResult<()> {
        match version {
            1 => {
                self.create_tables().await?;
                Ok(())
            }
            _ => {
                self.create_tables().await?;
                Ok(())
            }
        }
    }

    fn provider_type(&self) -> DatabaseProviderType {
        DatabaseProviderType::SQLite
    }

    fn connection_info(&self) -> String {
        format!("SQLite: {}", self.database_path)
    }

    async fn save_ssh_profile(&self, model: &crate::models::ssh::SSHProfile) -> DatabaseResult<()> {
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

    async fn find_auto_start_ssh_tunnels(
        &self,
    ) -> DatabaseResult<Vec<crate::models::ssh::SSHTunnel>> {
        tunnel::find_auto_start_ssh_tunnels(self).await
    }

    async fn delete_ssh_tunnel(&self, id: &str) -> DatabaseResult<()> {
        tunnel::delete_ssh_tunnel(self, id).await
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
        command::save_saved_command(self, model).await
    }

    async fn delete_saved_command(&self, id: &str) -> DatabaseResult<()> {
        command::delete_saved_command(self, id).await
    }

    async fn increment_command_usage(&self, _id: &str) -> DatabaseResult<()> {
        Err(DatabaseError::NotImplemented(
            "increment_command_usage not yet implemented".to_string(),
        ))
    }

    async fn toggle_command_favorite(&self, _id: &str) -> DatabaseResult<()> {
        Err(DatabaseError::NotImplemented(
            "toggle_command_favorite not yet implemented".to_string(),
        ))
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
        command::save_saved_command_group(self, model).await
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
        let device = auth::get_current_device(self).await?;
        match device {
            Some(d) => auth::get_master_password_entry(self, &d.device_id).await,
            None => Ok(None),
        }
    }

    async fn update_master_password_last_verified(&self, _device_id: &str) -> DatabaseResult<()> {
        Err(DatabaseError::NotImplemented(
            "update_master_password_last_verified not yet implemented".to_string(),
        ))
    }

    async fn delete_master_password_entry(&self, device_id: &str) -> DatabaseResult<()> {
        auth::delete_master_password_entry(self, device_id).await
    }

    async fn save_device(&self, device: &crate::models::auth::Device) -> DatabaseResult<()> {
        auth::save_device(self, device).await
    }

    async fn get_current_device(&self) -> DatabaseResult<Option<crate::models::auth::Device>> {
        auth::get_current_device(self).await
    }

    async fn get_all_devices(&self) -> DatabaseResult<Vec<crate::models::auth::Device>> {
        auth::get_all_devices(self).await
    }
}

impl SQLiteProvider {
    pub async fn get_all_external_databases(
        &self,
    ) -> DatabaseResult<Vec<crate::models::sync::external_db::ExternalDatabaseConfig>> {
        self.find_all_external_databases().await
    }

    pub async fn get_sync_logs(
        &self,
        _database_id: &str,
        _limit: Option<i32>,
    ) -> DatabaseResult<Vec<crate::models::sync::log::SyncLog>> {
        Ok(Vec::new())
    }

    pub async fn save_sync_log(
        &self,
        _log: &crate::models::sync::log::SyncLog,
    ) -> DatabaseResult<()> {
        Ok(())
    }

    pub async fn save_conflict_resolution(
        &self,
        resolution: &crate::models::sync::conflict::ConflictResolution,
    ) -> DatabaseResult<()> {
        let pool_arc = self.get_pool()?;
        let pool = pool_arc.read().await;

        sqlx::query(
            r#"
            INSERT INTO sync_conflicts (
                id, entity_type, entity_id, local_version, remote_version,
                local_data, remote_data, resolution_strategy, resolved, created_at, resolved_at
            ) VALUES (?, ?, ?, 0, 0, ?, ?, NULL, false, ?, NULL)
            "#,
        )
        .bind(&resolution.id)
        .bind(&resolution.entity_type)
        .bind(&resolution.entity_id)
        .bind(resolution.local_data.to_string())
        .bind(resolution.remote_data.to_string())
        .bind(resolution.created_at.to_rfc3339())
        .execute(&*pool)
        .await
        .map_err(|e| DatabaseError::QueryFailed(e.to_string()))?;

        Ok(())
    }
}
