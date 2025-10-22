mod sync;

use async_trait::async_trait;
use chrono::{DateTime, Utc};
use serde_json::Value;
use sqlx::MySqlPool;
use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::RwLock;

use crate::database::{
    error::{DatabaseError, DatabaseResult},
    traits_sync::SyncTarget,
};

/// MySQL provider for sync operations only
pub struct MySQLProvider {
    connection_string: String,
    pool: Option<Arc<RwLock<MySqlPool>>>,
}

impl MySQLProvider {
    pub fn new(connection_string: String) -> Self {
        Self {
            connection_string,
            pool: None,
        }
    }

    pub(crate) fn get_pool(&self) -> DatabaseResult<&Arc<RwLock<MySqlPool>>> {
        self.pool
            .as_ref()
            .ok_or_else(|| DatabaseError::ConnectionFailed("Database not connected".to_string()))
    }

    /// Create sync tables in MySQL database
    async fn create_sync_tables(&self) -> DatabaseResult<()> {
        let pool = self.get_pool()?;
        let pool = pool.read().await;

        let tables = vec![
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
                proxy TEXT,
                created_at DATETIME NOT NULL,
                updated_at DATETIME NOT NULL,
                device_id VARCHAR(255) NOT NULL,
                version BIGINT NOT NULL DEFAULT 1,
                sync_status VARCHAR(50) NOT NULL DEFAULT 'Synced',
                INDEX idx_ssh_profiles_group_id (group_id),
                INDEX idx_ssh_profiles_updated_at (updated_at)
            ) ENGINE=InnoDB DEFAULT CHARSET=utf8mb4
            "#,
            r#"
            CREATE TABLE IF NOT EXISTS ssh_groups (
                id VARCHAR(36) PRIMARY KEY,
                name VARCHAR(255) NOT NULL,
                description TEXT,
                color VARCHAR(50),
                created_at DATETIME NOT NULL,
                updated_at DATETIME NOT NULL,
                device_id VARCHAR(255) NOT NULL,
                version BIGINT NOT NULL DEFAULT 1,
                sync_status VARCHAR(50) NOT NULL DEFAULT 'Synced',
                INDEX idx_ssh_groups_updated_at (updated_at)
            ) ENGINE=InnoDB DEFAULT CHARSET=utf8mb4
            "#,
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
                last_used DATETIME,
                created_at DATETIME NOT NULL,
                updated_at DATETIME NOT NULL,
                device_id VARCHAR(255) NOT NULL,
                version BIGINT NOT NULL DEFAULT 1,
                sync_status VARCHAR(50) NOT NULL DEFAULT 'Synced',
                INDEX idx_ssh_keys_fingerprint (fingerprint),
                INDEX idx_ssh_keys_updated_at (updated_at)
            ) ENGINE=InnoDB DEFAULT CHARSET=utf8mb4
            "#,
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
                created_at DATETIME NOT NULL,
                updated_at DATETIME NOT NULL,
                device_id VARCHAR(255) NOT NULL,
                version BIGINT NOT NULL DEFAULT 1,
                sync_status VARCHAR(50) NOT NULL DEFAULT 'Synced',
                INDEX idx_ssh_tunnels_profile_id (profile_id),
                INDEX idx_ssh_tunnels_updated_at (updated_at),
                FOREIGN KEY (profile_id) REFERENCES ssh_profiles(id) ON DELETE CASCADE
            ) ENGINE=InnoDB DEFAULT CHARSET=utf8mb4
            "#,
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
                last_used_at DATETIME,
                created_at DATETIME NOT NULL,
                updated_at DATETIME NOT NULL,
                device_id VARCHAR(255) NOT NULL,
                version BIGINT NOT NULL DEFAULT 1,
                sync_status VARCHAR(50) NOT NULL DEFAULT 'Synced',
                INDEX idx_saved_commands_group_id (group_id),
                INDEX idx_saved_commands_updated_at (updated_at)
            ) ENGINE=InnoDB DEFAULT CHARSET=utf8mb4
            "#,
            r#"
            CREATE TABLE IF NOT EXISTS saved_command_groups (
                id VARCHAR(36) PRIMARY KEY,
                name VARCHAR(255) NOT NULL,
                description TEXT,
                color VARCHAR(50),
                icon VARCHAR(50),
                created_at DATETIME NOT NULL,
                updated_at DATETIME NOT NULL,
                device_id VARCHAR(255) NOT NULL,
                version BIGINT NOT NULL DEFAULT 1,
                sync_status VARCHAR(50) NOT NULL DEFAULT 'Synced',
                INDEX idx_saved_command_groups_updated_at (updated_at)
            ) ENGINE=InnoDB DEFAULT CHARSET=utf8mb4
            "#,
        ];

        for table_sql in tables {
            sqlx::query(table_sql)
                .execute(&*pool)
                .await
                .map_err(|e| DatabaseError::QueryFailed(e.to_string()))?;
        }

        Ok(())
    }
}

#[async_trait]
impl SyncTarget for MySQLProvider {
    async fn connect(&mut self) -> DatabaseResult<()> {
        let pool = MySqlPool::connect(&self.connection_string)
            .await
            .map_err(|e| DatabaseError::ConnectionFailed(e.to_string()))?;

        self.pool = Some(Arc::new(RwLock::new(pool)));
        self.create_sync_tables().await?;
        Ok(())
    }

    async fn test_connection(&self) -> DatabaseResult<()> {
        let pool = self.get_pool()?;
        let pool = pool.read().await;

        sqlx::query("SELECT 1")
            .execute(&*pool)
            .await
            .map_err(|e| DatabaseError::QueryFailed(e.to_string()))?;

        Ok(())
    }

    async fn push_records(&self, table: &str, records: Vec<Value>) -> DatabaseResult<usize> {
        sync::push_records(self, table, records).await
    }

    async fn pull_records(
        &self,
        table: &str,
        since: Option<DateTime<Utc>>,
    ) -> DatabaseResult<Vec<Value>> {
        sync::pull_records(self, table, since).await
    }

    async fn get_record_versions(
        &self,
        table: &str,
        ids: Vec<String>,
    ) -> DatabaseResult<HashMap<String, u64>> {
        sync::get_record_versions(self, table, ids).await
    }
}
