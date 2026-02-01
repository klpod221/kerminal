mod auth;
mod command;
mod ssh;
pub mod sync_ops;
mod terminal;
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
                command TEXT,
                working_dir TEXT,
                env TEXT,
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
            CREATE TABLE IF NOT EXISTS sync_settings (
                id TEXT PRIMARY KEY DEFAULT 'global',
                is_active BOOLEAN NOT NULL DEFAULT false,
                auto_sync_enabled BOOLEAN NOT NULL DEFAULT false,
                sync_interval_minutes INTEGER NOT NULL DEFAULT 15,
                conflict_strategy TEXT NOT NULL DEFAULT 'Manual',
                sync_direction TEXT NOT NULL DEFAULT 'Bidirectional',
                selected_database_id TEXT,
                last_sync_at TEXT,
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
            CREATE TABLE IF NOT EXISTS sync_logs (
                id TEXT PRIMARY KEY,
                database_id TEXT NOT NULL,
                device_id TEXT NOT NULL,
                direction TEXT NOT NULL,
                status TEXT NOT NULL,
                started_at TEXT NOT NULL,
                completed_at TEXT,
                records_synced INTEGER NOT NULL DEFAULT 0,
                conflicts_resolved INTEGER NOT NULL DEFAULT 0,
                manual_conflicts INTEGER NOT NULL DEFAULT 0,
                error_message TEXT
            )
        "#,
        )
        .execute(&*pool)
        .await
        .map_err(|e| DatabaseError::QueryFailed(e.to_string()))?;

        sqlx::query(
            r#"
            CREATE INDEX IF NOT EXISTS idx_sync_logs_database_id
            ON sync_logs(database_id, started_at DESC)
        "#,
        )
        .execute(&*pool)
        .await
        .map_err(|e| DatabaseError::QueryFailed(e.to_string()))?;

        sqlx::query(
            r#"
            CREATE TABLE IF NOT EXISTS conflict_resolutions (
                id TEXT PRIMARY KEY,
                entity_type TEXT NOT NULL,
                entity_id TEXT NOT NULL,
                local_data TEXT NOT NULL,
                remote_data TEXT NOT NULL,
                resolution_strategy TEXT,
                resolved_at TEXT,
                created_at TEXT NOT NULL
            )
        "#,
        )
        .execute(&*pool)
        .await
        .map_err(|e| DatabaseError::QueryFailed(e.to_string()))?;

        sqlx::query(
            r#"
            CREATE INDEX IF NOT EXISTS idx_conflict_resolutions_entity
            ON conflict_resolutions(entity_type, entity_id)
        "#,
        )
        .execute(&*pool)
        .await
        .map_err(|e| DatabaseError::QueryFailed(e.to_string()))?;

        sqlx::query(
            r#"
            CREATE TABLE IF NOT EXISTS session_recordings (
                id TEXT PRIMARY KEY,
                terminal_id TEXT,
                session_name TEXT NOT NULL,
                terminal_type TEXT NOT NULL,
                started_at TEXT NOT NULL,
                ended_at TEXT,
                duration_ms INTEGER,
                file_path TEXT NOT NULL,
                file_size INTEGER NOT NULL DEFAULT 0,
                width INTEGER NOT NULL DEFAULT 80,
                height INTEGER NOT NULL DEFAULT 24,
                metadata TEXT,
                created_at TEXT NOT NULL
            )
            "#,
        )
        .execute(&*pool)
        .await
        .map_err(|e| DatabaseError::QueryFailed(e.to_string()))?;

        sqlx::query(
            r#"
            CREATE INDEX IF NOT EXISTS idx_recordings_started_at
            ON session_recordings(started_at DESC)
            "#,
        )
        .execute(&*pool)
        .await
        .map_err(|e| DatabaseError::QueryFailed(e.to_string()))?;

        sqlx::query(
            r#"
            CREATE TABLE IF NOT EXISTS terminal_profiles (
                id TEXT PRIMARY KEY,
                name TEXT NOT NULL,
                shell TEXT NOT NULL,
                working_dir TEXT,
                env TEXT,
                icon TEXT,
                color TEXT,
                command TEXT,
                created_at INTEGER NOT NULL,
                updated_at INTEGER NOT NULL
            )
            "#,
        )
        .execute(&*pool)
        .await
        .map_err(|e| DatabaseError::QueryFailed(e.to_string()))?;

        // Add command column if it doesn't exist (migration)
        sqlx::query("ALTER TABLE terminal_profiles ADD COLUMN command TEXT")
            .execute(&*pool)
            .await
            .ok(); // Ignore error if column already exists

        // Add SSH profile columns migration
        sqlx::query("ALTER TABLE ssh_profiles ADD COLUMN command TEXT")
            .execute(&*pool)
            .await
            .ok();
        sqlx::query("ALTER TABLE ssh_profiles ADD COLUMN working_dir TEXT")
            .execute(&*pool)
            .await
            .ok();
        sqlx::query("ALTER TABLE ssh_profiles ADD COLUMN env TEXT")
            .execute(&*pool)
            .await
            .ok();

        // Add jump_hosts column migration
        sqlx::query("ALTER TABLE ssh_profiles ADD COLUMN jump_hosts TEXT")
            .execute(&*pool)
            .await
            .ok();

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
            None => Ok(std::option::Option::None),
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
    pub async fn save_terminal_profile(
        &self,
        profile: &crate::models::terminal::profile::TerminalProfile,
    ) -> DatabaseResult<()> {
        terminal::save_terminal_profile(self, profile).await
    }

    pub async fn find_terminal_profile_by_id(
        &self,
        id: &str,
    ) -> DatabaseResult<Option<crate::models::terminal::profile::TerminalProfile>> {
        terminal::find_terminal_profile_by_id(self, id).await
    }

    pub async fn find_all_terminal_profiles(
        &self,
    ) -> DatabaseResult<Vec<crate::models::terminal::profile::TerminalProfile>> {
        terminal::find_all_terminal_profiles(self).await
    }

    pub async fn delete_terminal_profile(&self, id: &str) -> DatabaseResult<()> {
        terminal::delete_terminal_profile(self, id).await
    }

    pub async fn get_all_external_databases(
        &self,
    ) -> DatabaseResult<Vec<crate::models::sync::external_db::ExternalDatabaseConfig>> {
        self.find_all_external_databases().await
    }

    pub async fn get_sync_logs(
        &self,
        database_id: &str,
        limit: Option<i32>,
    ) -> DatabaseResult<Vec<crate::models::sync::log::SyncLog>> {
        let pool = self.pool.as_ref().ok_or_else(|| {
            DatabaseError::ConnectionFailed("Database not initialized".to_string())
        })?;
        let pool_guard = pool.read().await;

        let query = if let Some(limit_value) = limit {
            sqlx::query_as::<
                _,
                (
                    String,
                    String,
                    String,
                    String,
                    String,
                    String,
                    Option<String>,
                    i32,
                    i32,
                    i32,
                    Option<String>,
                ),
            >(
                r#"
                SELECT id, database_id, device_id, direction, status, started_at, completed_at,
                       records_synced, conflicts_resolved, manual_conflicts, error_message
                FROM sync_logs
                WHERE database_id = ?
                ORDER BY started_at DESC
                LIMIT ?
                "#,
            )
            .bind(database_id)
            .bind(limit_value)
            .fetch_all(&*pool_guard)
            .await
        } else {
            sqlx::query_as::<
                _,
                (
                    String,
                    String,
                    String,
                    String,
                    String,
                    String,
                    Option<String>,
                    i32,
                    i32,
                    i32,
                    Option<String>,
                ),
            >(
                r#"
                SELECT id, database_id, device_id, direction, status, started_at, completed_at,
                       records_synced, conflicts_resolved, manual_conflicts, error_message
                FROM sync_logs
                WHERE database_id = ?
                ORDER BY started_at DESC
                "#,
            )
            .bind(database_id)
            .fetch_all(&*pool_guard)
            .await
        };

        let rows = query.map_err(|e| DatabaseError::QueryFailed(e.to_string()))?;

        let mut logs = Vec::new();
        for row in rows {
            use std::str::FromStr;

            let direction = crate::models::sync::log::SyncDirection::from_str(&row.3)
                .map_err(DatabaseError::ParseError)?;
            let status = match row.4.as_str() {
                "InProgress" => crate::models::sync::log::SyncStatus::InProgress,
                "Completed" => crate::models::sync::log::SyncStatus::Completed,
                "Failed" => crate::models::sync::log::SyncStatus::Failed,
                "Cancelled" => crate::models::sync::log::SyncStatus::Cancelled,
                _ => crate::models::sync::log::SyncStatus::Failed,
            };

            let started_at = chrono::DateTime::parse_from_rfc3339(&row.5)
                .map_err(|e| DatabaseError::ParseError(e.to_string()))?
                .with_timezone(&chrono::Utc);

            let completed_at = if let Some(completed_str) = &row.6 {
                Some(
                    chrono::DateTime::parse_from_rfc3339(completed_str)
                        .map_err(|e| DatabaseError::ParseError(e.to_string()))?
                        .with_timezone(&chrono::Utc),
                )
            } else {
                None
            };

            logs.push(crate::models::sync::log::SyncLog {
                id: row.0,
                database_id: row.1,
                device_id: row.2,
                direction,
                status,
                started_at,
                completed_at,
                records_synced: row.7,
                conflicts_resolved: row.8,
                manual_conflicts: row.9,
                error_message: row.10,
            });
        }

        Ok(logs)
    }

    pub async fn save_sync_log(
        &self,
        log: &crate::models::sync::log::SyncLog,
    ) -> DatabaseResult<()> {
        let pool = self.pool.as_ref().ok_or_else(|| {
            DatabaseError::ConnectionFailed("Database not initialized".to_string())
        })?;
        let pool_guard = pool.read().await;

        sqlx::query(
            r#"
            INSERT OR REPLACE INTO sync_logs (
                id, database_id, device_id, direction, status, started_at, completed_at,
                records_synced, conflicts_resolved, manual_conflicts, error_message
            ) VALUES (?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?)
            "#,
        )
        .bind(&log.id)
        .bind(&log.database_id)
        .bind(&log.device_id)
        .bind(log.direction.to_string())
        .bind(match log.status {
            crate::models::sync::log::SyncStatus::InProgress => "InProgress",
            crate::models::sync::log::SyncStatus::Completed => "Completed",
            crate::models::sync::log::SyncStatus::Failed => "Failed",
            crate::models::sync::log::SyncStatus::Cancelled => "Cancelled",
        })
        .bind(log.started_at.to_rfc3339())
        .bind(log.completed_at.map(|dt| dt.to_rfc3339()))
        .bind(log.records_synced)
        .bind(log.conflicts_resolved)
        .bind(log.manual_conflicts)
        .bind(&log.error_message)
        .execute(&*pool_guard)
        .await
        .map_err(|e| DatabaseError::QueryFailed(e.to_string()))?;

        Ok(())
    }

    /// Delete sync logs older than specified days
    pub async fn delete_old_sync_logs(&self, days: i64) -> DatabaseResult<u64> {
        let pool = self.pool.as_ref().ok_or_else(|| {
            DatabaseError::ConnectionFailed("Database not initialized".to_string())
        })?;
        let pool_guard = pool.read().await;

        let cutoff_date = (chrono::Utc::now() - chrono::Duration::days(days)).to_rfc3339();

        let result = sqlx::query(
            r#"
            DELETE FROM sync_logs
            WHERE started_at < ?
            "#,
        )
        .bind(&cutoff_date)
        .execute(&*pool_guard)
        .await
        .map_err(|e| DatabaseError::QueryFailed(e.to_string()))?;

        Ok(result.rows_affected())
    }

    // Session recording operations
    pub async fn save_session_recording(
        &self,
        recording: &crate::models::recording::SessionRecording,
    ) -> DatabaseResult<()> {
        let pool = self.get_pool()?;
        let pool_guard = pool.read().await;

        sqlx::query(
            r#"
            INSERT OR REPLACE INTO session_recordings (
                id, terminal_id, session_name, terminal_type, started_at, ended_at,
                duration_ms, file_path, file_size, width, height, metadata, created_at
            ) VALUES (?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?)
            "#,
        )
        .bind(&recording.id)
        .bind(&recording.terminal_id)
        .bind(&recording.session_name)
        .bind(&recording.terminal_type)
        .bind(recording.started_at.to_rfc3339())
        .bind(recording.ended_at.as_ref().map(|dt| dt.to_rfc3339()))
        .bind(recording.duration_ms)
        .bind(&recording.file_path)
        .bind(recording.file_size)
        .bind(recording.width as i32)
        .bind(recording.height as i32)
        .bind(&recording.metadata)
        .bind(recording.created_at.to_rfc3339())
        .execute(&*pool_guard)
        .await
        .map_err(|e| DatabaseError::QueryFailed(e.to_string()))?;

        Ok(())
    }

    pub async fn get_session_recording(
        &self,
        id: &str,
    ) -> DatabaseResult<Option<crate::models::recording::SessionRecording>> {
        let pool = self.get_pool()?;
        let pool_guard = pool.read().await;

        let result = sqlx::query_as::<
            _,
            (
                String,
                String,
                String,
                String,
                String,
                Option<String>,
                Option<i64>,
                String,
                i64,
                i32,
                i32,
                Option<String>,
                String,
            ),
        >(
            r#"
            SELECT id, terminal_id, session_name, terminal_type, started_at, ended_at,
                   duration_ms, file_path, file_size, width, height, metadata, created_at
            FROM session_recordings
            WHERE id = ?
            "#,
        )
        .bind(id)
        .fetch_optional(&*pool_guard)
        .await
        .map_err(|e| DatabaseError::QueryFailed(e.to_string()))?;

        Ok(
            result.map(|row| crate::models::recording::SessionRecording {
                id: row.0,
                terminal_id: row.1,
                session_name: row.2,
                terminal_type: row.3,
                started_at: chrono::DateTime::parse_from_rfc3339(&row.4)
                    .unwrap()
                    .with_timezone(&chrono::Utc),
                ended_at: row.5.and_then(|s| {
                    chrono::DateTime::parse_from_rfc3339(&s)
                        .ok()
                        .map(|dt| dt.with_timezone(&chrono::Utc))
                }),
                duration_ms: row.6,
                file_path: row.7,
                file_size: row.8,
                width: row.9 as u16,
                height: row.10 as u16,
                metadata: row.11,
                created_at: chrono::DateTime::parse_from_rfc3339(&row.12)
                    .unwrap()
                    .with_timezone(&chrono::Utc),
            }),
        )
    }

    pub async fn list_session_recordings(
        &self,
    ) -> DatabaseResult<Vec<crate::models::recording::SessionRecording>> {
        let pool = self.get_pool()?;
        let pool_guard = pool.read().await;

        let results = sqlx::query_as::<
            _,
            (
                String,
                String,
                String,
                String,
                String,
                Option<String>,
                Option<i64>,
                String,
                i64,
                i32,
                i32,
                Option<String>,
                String,
            ),
        >(
            r#"
            SELECT id, terminal_id, session_name, terminal_type, started_at, ended_at,
                   duration_ms, file_path, file_size, width, height, metadata, created_at
            FROM session_recordings
            ORDER BY started_at DESC
            "#,
        )
        .fetch_all(&*pool_guard)
        .await
        .map_err(|e| DatabaseError::QueryFailed(e.to_string()))?;

        Ok(results
            .into_iter()
            .map(|row| crate::models::recording::SessionRecording {
                id: row.0,
                terminal_id: row.1,
                session_name: row.2,
                terminal_type: row.3,
                started_at: chrono::DateTime::parse_from_rfc3339(&row.4)
                    .unwrap()
                    .with_timezone(&chrono::Utc),
                ended_at: row.5.and_then(|s| {
                    chrono::DateTime::parse_from_rfc3339(&s)
                        .ok()
                        .map(|dt| dt.with_timezone(&chrono::Utc))
                }),
                duration_ms: row.6,
                file_path: row.7,
                file_size: row.8,
                width: row.9 as u16,
                height: row.10 as u16,
                metadata: row.11,
                created_at: chrono::DateTime::parse_from_rfc3339(&row.12)
                    .unwrap()
                    .with_timezone(&chrono::Utc),
            })
            .collect())
    }

    pub async fn delete_session_recording(&self, id: &str) -> DatabaseResult<()> {
        let pool = self.get_pool()?;
        let pool_guard = pool.read().await;

        sqlx::query("DELETE FROM session_recordings WHERE id = ?")
            .bind(id)
            .execute(&*pool_guard)
            .await
            .map_err(|e| DatabaseError::QueryFailed(e.to_string()))?;

        Ok(())
    }
}
