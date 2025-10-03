use async_trait::async_trait;
use sqlx::{Row, SqlitePool};
use std::sync::Arc;
use tokio::sync::RwLock;

use crate::{
    database::{
        encryption::device_keys::MasterPasswordEntry,
        error::{DatabaseError, DatabaseResult},
        traits::{Database, DatabaseProviderType, SqlValue, ToSqlValue},
    },
    models::{
        ssh::{SSHGroup, SSHProfile},
    },
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
    fn get_pool(&self) -> DatabaseResult<&Arc<RwLock<SqlitePool>>> {
        self.pool
            .as_ref()
            .ok_or_else(|| DatabaseError::ConnectionFailed("Database not connected".to_string()))
    }
}

#[async_trait]
impl Database for SQLiteProvider {
    async fn connect(&mut self) -> DatabaseResult<()> {
        // Ensure parent directory exists
        if let Some(parent) = std::path::Path::new(&self.database_path).parent() {
            std::fs::create_dir_all(parent).map_err(|e| {
                DatabaseError::ConnectionFailed(format!(
                    "Failed to create database directory: {}",
                    e
                ))
            })?;
        }

        // Connect to SQLite database - will create file if not exists
        let connection_string = format!("sqlite://{}?mode=rwc", self.database_path);
        let pool = SqlitePool::connect(&connection_string)
            .await
            .map_err(|e| DatabaseError::ConnectionFailed(e.to_string()))?;

        self.pool = Some(Arc::new(RwLock::new(pool)));

        // Create tables if they don't exist
        self.create_tables().await?;

        Ok(())
    }

    /// Create required tables
    async fn create_tables(&self) -> DatabaseResult<()> {
        let pool = self.get_pool()?;
        let pool = pool.read().await;

        // Create SSH profiles table
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

        // Create SSH groups table
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

        // Create SSH keys table
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

        // Create index on fingerprint for faster lookups
        sqlx::query(
            r#"
            CREATE INDEX IF NOT EXISTS idx_ssh_keys_fingerprint
            ON ssh_keys(fingerprint)
        "#,
        )
        .execute(&*pool)
        .await
        .map_err(|e| DatabaseError::QueryFailed(e.to_string()))?;

        // Create SSH tunnels table
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

        // Create index on profile_id for faster lookups
        sqlx::query(
            r#"
            CREATE INDEX IF NOT EXISTS idx_ssh_tunnels_profile_id
            ON ssh_tunnels(profile_id)
        "#,
        )
        .execute(&*pool)
        .await
        .map_err(|e| DatabaseError::QueryFailed(e.to_string()))?;

        // Create master passwords table
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

        // Create devices table
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

        // Create sync metadata table
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
            .fetch_one(&*pool)
            .await
            .map_err(|e| DatabaseError::ConnectionFailed(e.to_string()))?;

        Ok(())
    }

    async fn execute_raw(&self, query: &str, _params: &[&dyn ToSqlValue]) -> DatabaseResult<u64> {
        let pool_arc = self.get_pool()?;
        let pool = pool_arc.read().await;

        let result = sqlx::query(query)
            .execute(&*pool)
            .await
            .map_err(|e| DatabaseError::QueryFailed(e.to_string()))?;

        Ok(result.rows_affected())
    }

    async fn fetch_raw(
        &self,
        query: &str,
        _params: &[&dyn ToSqlValue],
    ) -> DatabaseResult<Vec<std::collections::HashMap<String, SqlValue>>> {
        let pool_arc = self.get_pool()?;
        let pool = pool_arc.read().await;

        let rows = sqlx::query(query)
            .fetch_all(&*pool)
            .await
            .map_err(|e| DatabaseError::QueryFailed(e.to_string()))?;

        let mut results = Vec::new();
        for _row in rows {
            let mut map = std::collections::HashMap::new();
            // Note: This is simplified - in real implementation you'd iterate over columns
            map.insert(
                "placeholder".to_string(),
                SqlValue::Text("placeholder".to_string()),
            );
            results.push(map);
        }

        Ok(results)
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
        // Simple migration system - version 1 includes all necessary tables
        match version {
            1 => {
                // Version 1: Create all tables with final schema
                self.create_tables().await?;
                Ok(())
            }
            _ => {
                // For backward compatibility, default to version 1
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

    // Concrete implementations for object safety
    async fn save_ssh_profile(&self, model: &SSHProfile) -> DatabaseResult<()> {
        let pool_arc = self.get_pool()?;
        let pool = pool_arc.read().await;

        sqlx::query(
            r#"
            INSERT OR REPLACE INTO ssh_profiles (
                id, name, host, port, username, group_id, auth_method, auth_data,
                description, color, timeout, keep_alive, compression, created_at, updated_at,
                device_id, version, sync_status
            ) VALUES (?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?)
        "#,
        )
        .bind(&model.base.id)
        .bind(&model.name)
        .bind(&model.host)
        .bind(model.port as i32)
        .bind(&model.username)
        .bind(&model.group_id)
        .bind(serde_json::to_string(&model.auth_method).unwrap_or_default())
        .bind(serde_json::to_string(&model.auth_data).unwrap_or_default())
        .bind(&model.description)
        .bind(&model.color)
        .bind(model.timeout.map(|t| t as i32))
        .bind(model.keep_alive)
        .bind(model.compression)
        .bind(model.base.created_at)
        .bind(model.base.updated_at)
        .bind(&model.base.device_id)
        .bind(model.base.version as i64)
        .bind(serde_json::to_string(&model.base.sync_status).unwrap_or_default())
        .execute(&*pool)
        .await
        .map_err(|e| DatabaseError::QueryFailed(e.to_string()))?;

        Ok(())
    }

    async fn find_ssh_profile_by_id(&self, id: &str) -> DatabaseResult<Option<SSHProfile>> {
        let pool = self.get_pool()?;
        let pool = pool.read().await;

        let row = sqlx::query(
            "SELECT id, name, host, port, username, group_id, auth_method, auth_data, description, color, timeout, keep_alive, compression, created_at, updated_at, device_id, version, sync_status FROM ssh_profiles WHERE id = ?"
        )
        .bind(id)
        .fetch_optional(&*pool)
        .await
        .map_err(|e| DatabaseError::QueryFailed(e.to_string()))?;

        if let Some(row) = row {
            let profile = SSHProfile {
                base: crate::models::base::BaseModel {
                    id: row.get("id"),
                    created_at: chrono::DateTime::parse_from_rfc3339(
                        &row.get::<String, _>("created_at"),
                    )
                    .map_err(|e| DatabaseError::ParseError(format!("Parse error: {}", e)))?
                    .with_timezone(&chrono::Utc),
                    updated_at: chrono::DateTime::parse_from_rfc3339(
                        &row.get::<String, _>("updated_at"),
                    )
                    .map_err(|e| DatabaseError::ParseError(format!("Parse error: {}", e)))?
                    .with_timezone(&chrono::Utc),
                    device_id: row.get("device_id"),
                    version: row.get::<i64, _>("version") as u64,
                    sync_status: serde_json::from_str(&row.get::<String, _>("sync_status"))
                        .unwrap_or(crate::database::traits::SyncStatus::Synced),
                },
                name: row.get("name"),
                host: row.get("host"),
                port: row.get::<i32, _>("port") as u16,
                username: row.get("username"),
                group_id: row.get("group_id"),
                auth_method: serde_json::from_str(&row.get::<String, _>("auth_method"))
                    .map_err(|e| DatabaseError::SerializationError(e))?,
                auth_data: serde_json::from_str(&row.get::<String, _>("auth_data"))
                    .map_err(|e| DatabaseError::SerializationError(e))?,
                timeout: row.get::<Option<i32>, _>("timeout").map(|t| t as u32),
                keep_alive: row.get("keep_alive"),
                compression: row.get("compression"),
                color: row.get("color"),
                description: row.get("description"),
                proxy: None,
            };
            Ok(Some(profile))
        } else {
            Ok(None)
        }
    }

    async fn find_all_ssh_profiles(&self) -> DatabaseResult<Vec<SSHProfile>> {
        let pool = self.get_pool()?;
        let pool = pool.read().await;

        let rows = sqlx::query(
            "SELECT id, name, host, port, username, group_id, auth_method, auth_data, description, color, timeout, keep_alive, compression, created_at, updated_at, device_id, version, sync_status FROM ssh_profiles ORDER BY name"
        )
        .fetch_all(&*pool)
        .await
        .map_err(|e| DatabaseError::QueryFailed(e.to_string()))?;

        let mut profiles = Vec::new();
        for row in rows {
            let profile = SSHProfile {
                base: crate::models::base::BaseModel {
                    id: row.get("id"),
                    created_at: chrono::DateTime::parse_from_rfc3339(
                        &row.get::<String, _>("created_at"),
                    )
                    .map_err(|e| DatabaseError::ParseError(format!("Parse error: {}", e)))?
                    .with_timezone(&chrono::Utc),
                    updated_at: chrono::DateTime::parse_from_rfc3339(
                        &row.get::<String, _>("updated_at"),
                    )
                    .map_err(|e| DatabaseError::ParseError(format!("Parse error: {}", e)))?
                    .with_timezone(&chrono::Utc),
                    device_id: row.get("device_id"),
                    version: row.get::<i64, _>("version") as u64,
                    sync_status: serde_json::from_str(&row.get::<String, _>("sync_status"))
                        .unwrap_or(crate::database::traits::SyncStatus::Synced),
                },
                name: row.get("name"),
                host: row.get("host"),
                port: row.get::<i32, _>("port") as u16,
                username: row.get("username"),
                group_id: row.get("group_id"),
                auth_method: serde_json::from_str(&row.get::<String, _>("auth_method"))
                    .map_err(|e| DatabaseError::SerializationError(e))?,
                auth_data: serde_json::from_str(&row.get::<String, _>("auth_data"))
                    .map_err(|e| DatabaseError::SerializationError(e))?,
                timeout: row.get::<Option<i32>, _>("timeout").map(|t| t as u32),
                keep_alive: row.get("keep_alive"),
                compression: row.get("compression"),
                color: row.get("color"),
                description: row.get("description"),
                proxy: None, // TODO: Implement proxy support in database
            };
            profiles.push(profile);
        }

        Ok(profiles)
    }

    async fn update_ssh_profile(&self, model: &SSHProfile) -> DatabaseResult<()> {
        // For now, just call save which does INSERT OR REPLACE
        self.save_ssh_profile(model).await
    }

    async fn delete_ssh_profile(&self, id: &str) -> DatabaseResult<()> {
        let pool_arc = self.get_pool()?;
        let pool = pool_arc.read().await;

        sqlx::query("DELETE FROM ssh_profiles WHERE id = ?")
            .bind(id)
            .execute(&*pool)
            .await
            .map_err(|e| DatabaseError::QueryFailed(e.to_string()))?;

        Ok(())
    }

    async fn save_ssh_group(&self, model: &SSHGroup) -> DatabaseResult<()> {
        let pool_arc = self.get_pool()?;
        let pool = pool_arc.read().await;

        sqlx::query(
            r#"
            INSERT OR REPLACE INTO ssh_groups (
                id, name, description, color,
                created_at, updated_at, device_id, version, sync_status
            ) VALUES (?, ?, ?, ?, ?, ?, ?, ?, ?)
        "#,
        )
        .bind(&model.base.id)
        .bind(&model.name)
        .bind(&model.description)
        .bind(&model.color)
        .bind(model.base.created_at)
        .bind(model.base.updated_at)
        .bind(&model.base.device_id)
        .bind(model.base.version as i64)
        .bind(serde_json::to_string(&model.base.sync_status).unwrap_or_default())
        .execute(&*pool)
        .await
        .map_err(|e| DatabaseError::QueryFailed(e.to_string()))?;

        Ok(())
    }

    async fn find_ssh_group_by_id(&self, id: &str) -> DatabaseResult<Option<SSHGroup>> {
        let pool = self.get_pool()?;
        let pool = pool.read().await;

        let row = sqlx::query(
            "SELECT id, name, description, color, created_at, updated_at, device_id, version, sync_status FROM ssh_groups WHERE id = ?"
        )
        .bind(id)
        .fetch_optional(&*pool)
        .await
        .map_err(|e| DatabaseError::QueryFailed(e.to_string()))?;

        if let Some(row) = row {
            let group = SSHGroup {
                base: crate::models::base::BaseModel {
                    id: row.get("id"),
                    created_at: chrono::DateTime::parse_from_rfc3339(
                        &row.get::<String, _>("created_at"),
                    )
                    .map_err(|e| DatabaseError::ParseError(format!("Parse error: {}", e)))?
                    .with_timezone(&chrono::Utc),
                    updated_at: chrono::DateTime::parse_from_rfc3339(
                        &row.get::<String, _>("updated_at"),
                    )
                    .map_err(|e| DatabaseError::ParseError(format!("Parse error: {}", e)))?
                    .with_timezone(&chrono::Utc),
                    device_id: row.get("device_id"),
                    version: row.get::<i64, _>("version") as u64,
                    sync_status: serde_json::from_str(&row.get::<String, _>("sync_status"))
                        .unwrap_or(crate::database::traits::SyncStatus::Synced),
                },
                name: row.get("name"),
                description: row.get("description"),
                color: row.get("color"),
            };
            Ok(Some(group))
        } else {
            Ok(None)
        }
    }

    async fn find_all_ssh_groups(&self) -> DatabaseResult<Vec<SSHGroup>> {
        let pool = self.get_pool()?;
        let pool = pool.read().await;

        let rows = sqlx::query(
            "SELECT id, name, description, color, created_at, updated_at, device_id, version, sync_status FROM ssh_groups ORDER BY name"
        )
        .fetch_all(&*pool)
        .await
        .map_err(|e| DatabaseError::QueryFailed(e.to_string()))?;

        let mut groups = Vec::new();
        for row in rows {
            let group = SSHGroup {
                base: crate::models::base::BaseModel {
                    id: row.get("id"),
                    created_at: chrono::DateTime::parse_from_rfc3339(
                        &row.get::<String, _>("created_at"),
                    )
                    .map_err(|e| DatabaseError::ParseError(format!("Parse error: {}", e)))?
                    .with_timezone(&chrono::Utc),
                    updated_at: chrono::DateTime::parse_from_rfc3339(
                        &row.get::<String, _>("updated_at"),
                    )
                    .map_err(|e| DatabaseError::ParseError(format!("Parse error: {}", e)))?
                    .with_timezone(&chrono::Utc),
                    device_id: row.get("device_id"),
                    version: row.get::<i64, _>("version") as u64,
                    sync_status: serde_json::from_str(&row.get::<String, _>("sync_status"))
                        .unwrap_or(crate::database::traits::SyncStatus::Synced),
                },
                name: row.get("name"),
                description: row.get("description"),
                color: row.get("color"),
            };
            groups.push(group);
        }

        Ok(groups)
    }

    async fn update_ssh_group(&self, model: &SSHGroup) -> DatabaseResult<()> {
        // For now, just call save which does INSERT OR REPLACE
        self.save_ssh_group(model).await
    }

    async fn delete_ssh_group(&self, id: &str) -> DatabaseResult<()> {
        let pool_arc = self.get_pool()?;
        let pool = pool_arc.read().await;

        sqlx::query("DELETE FROM ssh_groups WHERE id = ?")
            .bind(id)
            .execute(&*pool)
            .await
            .map_err(|e| DatabaseError::QueryFailed(e.to_string()))?;

        Ok(())
    }

    async fn save_ssh_key(
        &self,
        model: &crate::models::ssh::SSHKey,
    ) -> DatabaseResult<()> {
        let pool_arc = self.get_pool()?;
        let pool = pool_arc.read().await;

        sqlx::query(
            r#"
            INSERT OR REPLACE INTO ssh_keys (
                id, name, key_type, private_key, public_key, passphrase,
                fingerprint, description, last_used, created_at, updated_at,
                device_id, version, sync_status
            ) VALUES (?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?)
        "#,
        )
        .bind(&model.base.id)
        .bind(&model.name)
        .bind(serde_json::to_string(&model.key_type).unwrap())
        .bind(&model.private_key)
        .bind(&model.public_key)
        .bind(&model.passphrase)
        .bind(&model.fingerprint)
        .bind(&model.description)
        .bind(model.last_used.map(|dt| dt.to_rfc3339()))
        .bind(model.base.created_at.to_rfc3339())
        .bind(model.base.updated_at.to_rfc3339())
        .bind(&model.base.device_id)
        .bind(model.base.version as i64)
        .bind(serde_json::to_string(&model.base.sync_status).unwrap())
        .execute(&*pool)
        .await
        .map_err(|e| DatabaseError::QueryFailed(e.to_string()))?;

        Ok(())
    }

    async fn find_ssh_key_by_id(
        &self,
        id: &str,
    ) -> DatabaseResult<Option<crate::models::ssh::SSHKey>> {
        use crate::models::ssh::SSHKey;

        let pool = self.get_pool()?;
        let pool = pool.read().await;

        let result = sqlx::query(
            "SELECT id, name, key_type, private_key, public_key, passphrase, fingerprint, description, last_used, created_at, updated_at, device_id, version, sync_status FROM ssh_keys WHERE id = ?"
        )
        .bind(id)
        .fetch_optional(&*pool)
        .await
        .map_err(|e| DatabaseError::QueryFailed(e.to_string()))?;

        if let Some(row) = result {
            let key = SSHKey {
                base: crate::models::base::BaseModel {
                    id: row.get("id"),
                    created_at: chrono::DateTime::parse_from_rfc3339(
                        &row.get::<String, _>("created_at"),
                    )
                    .map_err(|e| DatabaseError::ParseError(format!("Parse error: {}", e)))?
                    .with_timezone(&chrono::Utc),
                    updated_at: chrono::DateTime::parse_from_rfc3339(
                        &row.get::<String, _>("updated_at"),
                    )
                    .map_err(|e| DatabaseError::ParseError(format!("Parse error: {}", e)))?
                    .with_timezone(&chrono::Utc),
                    device_id: row.get("device_id"),
                    version: row.get::<i64, _>("version") as u64,
                    sync_status: serde_json::from_str(&row.get::<String, _>("sync_status"))
                        .unwrap_or(crate::database::traits::SyncStatus::Synced),
                },
                name: row.get("name"),
                key_type: serde_json::from_str(&row.get::<String, _>("key_type"))
                    .map_err(|e| DatabaseError::SerializationError(e))?,
                private_key: row.get("private_key"),
                public_key: row.get("public_key"),
                passphrase: row.get("passphrase"),
                fingerprint: row.get("fingerprint"),
                description: row.get("description"),
                last_used: row
                    .get::<Option<String>, _>("last_used")
                    .and_then(|s| chrono::DateTime::parse_from_rfc3339(&s).ok())
                    .map(|dt| dt.with_timezone(&chrono::Utc)),
            };
            Ok(Some(key))
        } else {
            Ok(None)
        }
    }

    async fn find_all_ssh_keys(&self) -> DatabaseResult<Vec<crate::models::ssh::SSHKey>> {
        use crate::models::ssh::SSHKey;

        let pool = self.get_pool()?;
        let pool = pool.read().await;

        let rows = sqlx::query(
            "SELECT id, name, key_type, private_key, public_key, passphrase, fingerprint, description, last_used, created_at, updated_at, device_id, version, sync_status FROM ssh_keys ORDER BY name"
        )
        .fetch_all(&*pool)
        .await
        .map_err(|e| DatabaseError::QueryFailed(e.to_string()))?;

        let mut keys = Vec::new();
        for row in rows {
            let key = SSHKey {
                base: crate::models::base::BaseModel {
                    id: row.get("id"),
                    created_at: chrono::DateTime::parse_from_rfc3339(
                        &row.get::<String, _>("created_at"),
                    )
                    .map_err(|e| DatabaseError::ParseError(format!("Parse error: {}", e)))?
                    .with_timezone(&chrono::Utc),
                    updated_at: chrono::DateTime::parse_from_rfc3339(
                        &row.get::<String, _>("updated_at"),
                    )
                    .map_err(|e| DatabaseError::ParseError(format!("Parse error: {}", e)))?
                    .with_timezone(&chrono::Utc),
                    device_id: row.get("device_id"),
                    version: row.get::<i64, _>("version") as u64,
                    sync_status: serde_json::from_str(&row.get::<String, _>("sync_status"))
                        .unwrap_or(crate::database::traits::SyncStatus::Synced),
                },
                name: row.get("name"),
                key_type: serde_json::from_str(&row.get::<String, _>("key_type"))
                    .map_err(|e| DatabaseError::SerializationError(e))?,
                private_key: row.get("private_key"),
                public_key: row.get("public_key"),
                passphrase: row.get("passphrase"),
                fingerprint: row.get("fingerprint"),
                description: row.get("description"),
                last_used: row
                    .get::<Option<String>, _>("last_used")
                    .and_then(|s| chrono::DateTime::parse_from_rfc3339(&s).ok())
                    .map(|dt| dt.with_timezone(&chrono::Utc)),
            };
            keys.push(key);
        }

        Ok(keys)
    }

    async fn update_ssh_key(
        &self,
        model: &crate::models::ssh::SSHKey,
    ) -> DatabaseResult<()> {
        // For now, just call save which does INSERT OR REPLACE
        self.save_ssh_key(model).await
    }

    async fn delete_ssh_key(&self, id: &str) -> DatabaseResult<()> {
        let pool_arc = self.get_pool()?;
        let pool = pool_arc.read().await;

        sqlx::query("DELETE FROM ssh_keys WHERE id = ?")
            .bind(id)
            .execute(&*pool)
            .await
            .map_err(|e| DatabaseError::QueryFailed(e.to_string()))?;

        Ok(())
    }

    async fn count_profiles_using_key(&self, key_id: &str) -> DatabaseResult<u32> {
        let pool = self.get_pool()?;
        let pool = pool.read().await;

        // Search for profiles that have KeyReference auth_data containing this key_id
        let count = sqlx::query_scalar::<_, i64>(
            r#"
            SELECT COUNT(*) FROM ssh_profiles
            WHERE auth_data LIKE '%"key_id":"' || ? || '"%'
        "#,
        )
        .bind(key_id)
        .fetch_one(&*pool)
        .await
        .map_err(|e| DatabaseError::QueryFailed(e.to_string()))?;

        Ok(count as u32)
    }

    // === SSH Tunnel Methods ===

    async fn save_ssh_tunnel(&self, model: &crate::models::ssh::SSHTunnel) -> DatabaseResult<()> {
        let pool = self.get_pool()?;
        let pool = pool.read().await;

        sqlx::query(
            r#"
            INSERT OR REPLACE INTO ssh_tunnels (
                id, name, description, profile_id, tunnel_type, local_host, local_port,
                remote_host, remote_port, auto_start, created_at, updated_at,
                device_id, version, sync_status
            ) VALUES (?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?)
        "#,
        )
        .bind(&model.base.id)
        .bind(&model.name)
        .bind(&model.description)
        .bind(&model.profile_id)
        .bind(serde_json::to_string(&model.tunnel_type).unwrap())
        .bind(&model.local_host)
        .bind(model.local_port as i32)
        .bind(&model.remote_host)
        .bind(model.remote_port.map(|p| p as i32))
        .bind(model.auto_start)
        .bind(model.base.created_at.to_rfc3339())
        .bind(model.base.updated_at.to_rfc3339())
        .bind(&model.base.device_id)
        .bind(model.base.version as i64)
        .bind(serde_json::to_string(&model.base.sync_status).unwrap())
        .execute(&*pool)
        .await
        .map_err(|e| DatabaseError::QueryFailed(e.to_string()))?;

        Ok(())
    }

    async fn find_ssh_tunnel_by_id(
        &self,
        id: &str,
    ) -> DatabaseResult<Option<crate::models::ssh::SSHTunnel>> {
        let pool = self.get_pool()?;
        let pool = pool.read().await;

        let row = sqlx::query("SELECT * FROM ssh_tunnels WHERE id = ?")
            .bind(id)
            .fetch_optional(&*pool)
            .await
            .map_err(|e| DatabaseError::QueryFailed(e.to_string()))?;

        if let Some(row) = row {
            let tunnel = crate::models::ssh::SSHTunnel {
                base: crate::models::base::BaseModel {
                    id: row.get("id"),
                    created_at: chrono::DateTime::parse_from_rfc3339(
                        &row.get::<String, _>("created_at"),
                    )
                    .map_err(|e| DatabaseError::QueryFailed(e.to_string()))?
                    .with_timezone(&chrono::Utc),
                    updated_at: chrono::DateTime::parse_from_rfc3339(
                        &row.get::<String, _>("updated_at"),
                    )
                    .map_err(|e| DatabaseError::QueryFailed(e.to_string()))?
                    .with_timezone(&chrono::Utc),
                    device_id: row.get("device_id"),
                    version: row.get::<i64, _>("version") as u64,
                    sync_status: serde_json::from_str(&row.get::<String, _>("sync_status"))
                        .unwrap_or(crate::database::traits::SyncStatus::Synced),
                },
                name: row.get("name"),
                description: row.get("description"),
                profile_id: row.get("profile_id"),
                tunnel_type: serde_json::from_str(&row.get::<String, _>("tunnel_type"))
                    .map_err(|e| DatabaseError::QueryFailed(e.to_string()))?,
                local_host: row.get("local_host"),
                local_port: row.get::<i32, _>("local_port") as u16,
                remote_host: row.get("remote_host"),
                remote_port: row.get::<Option<i32>, _>("remote_port").map(|p| p as u16),
                auto_start: row.get("auto_start"),
                status: crate::models::ssh::TunnelStatus::default(),
                error_message: None,
            };
            Ok(Some(tunnel))
        } else {
            Ok(None)
        }
    }

    async fn find_all_ssh_tunnels(
        &self,
    ) -> DatabaseResult<Vec<crate::models::ssh::SSHTunnel>> {
        let pool = self.get_pool()?;
        let pool = pool.read().await;

        let rows = sqlx::query("SELECT * FROM ssh_tunnels ORDER BY name")
            .fetch_all(&*pool)
            .await
            .map_err(|e| DatabaseError::QueryFailed(e.to_string()))?;

        let mut tunnels = Vec::new();
        for row in rows {
            let tunnel = crate::models::ssh::SSHTunnel {
                base: crate::models::base::BaseModel {
                    id: row.get("id"),
                    created_at: chrono::DateTime::parse_from_rfc3339(
                        &row.get::<String, _>("created_at"),
                    )
                    .map_err(|e| DatabaseError::QueryFailed(e.to_string()))?
                    .with_timezone(&chrono::Utc),
                    updated_at: chrono::DateTime::parse_from_rfc3339(
                        &row.get::<String, _>("updated_at"),
                    )
                    .map_err(|e| DatabaseError::QueryFailed(e.to_string()))?
                    .with_timezone(&chrono::Utc),
                    device_id: row.get("device_id"),
                    version: row.get::<i64, _>("version") as u64,
                    sync_status: serde_json::from_str(&row.get::<String, _>("sync_status"))
                        .unwrap_or(crate::database::traits::SyncStatus::Synced),
                },
                name: row.get("name"),
                description: row.get("description"),
                profile_id: row.get("profile_id"),
                tunnel_type: serde_json::from_str(&row.get::<String, _>("tunnel_type"))
                    .map_err(|e| DatabaseError::QueryFailed(e.to_string()))?,
                local_host: row.get("local_host"),
                local_port: row.get::<i32, _>("local_port") as u16,
                remote_host: row.get("remote_host"),
                remote_port: row.get::<Option<i32>, _>("remote_port").map(|p| p as u16),
                auto_start: row.get("auto_start"),
                status: crate::models::ssh::TunnelStatus::default(),
                error_message: None,
            };
            tunnels.push(tunnel);
        }

        Ok(tunnels)
    }

    async fn find_ssh_tunnels_by_profile_id(
        &self,
        profile_id: &str,
    ) -> DatabaseResult<Vec<crate::models::ssh::SSHTunnel>> {
        let pool = self.get_pool()?;
        let pool = pool.read().await;

        let rows = sqlx::query("SELECT * FROM ssh_tunnels WHERE profile_id = ? ORDER BY name")
            .bind(profile_id)
            .fetch_all(&*pool)
            .await
            .map_err(|e| DatabaseError::QueryFailed(e.to_string()))?;

        let mut tunnels = Vec::new();
        for row in rows {
            let tunnel = crate::models::ssh::SSHTunnel {
                base: crate::models::base::BaseModel {
                    id: row.get("id"),
                    created_at: chrono::DateTime::parse_from_rfc3339(
                        &row.get::<String, _>("created_at"),
                    )
                    .map_err(|e| DatabaseError::QueryFailed(e.to_string()))?
                    .with_timezone(&chrono::Utc),
                    updated_at: chrono::DateTime::parse_from_rfc3339(
                        &row.get::<String, _>("updated_at"),
                    )
                    .map_err(|e| DatabaseError::QueryFailed(e.to_string()))?
                    .with_timezone(&chrono::Utc),
                    device_id: row.get("device_id"),
                    version: row.get::<i64, _>("version") as u64,
                    sync_status: serde_json::from_str(&row.get::<String, _>("sync_status"))
                        .unwrap_or(crate::database::traits::SyncStatus::Synced),
                },
                name: row.get("name"),
                description: row.get("description"),
                profile_id: row.get("profile_id"),
                tunnel_type: serde_json::from_str(&row.get::<String, _>("tunnel_type"))
                    .map_err(|e| DatabaseError::QueryFailed(e.to_string()))?,
                local_host: row.get("local_host"),
                local_port: row.get::<i32, _>("local_port") as u16,
                remote_host: row.get("remote_host"),
                remote_port: row.get::<Option<i32>, _>("remote_port").map(|p| p as u16),
                auto_start: row.get("auto_start"),
                status: crate::models::ssh::TunnelStatus::default(),
                error_message: None,
            };
            tunnels.push(tunnel);
        }

        Ok(tunnels)
    }

    async fn find_auto_start_ssh_tunnels(
        &self,
    ) -> DatabaseResult<Vec<crate::models::ssh::SSHTunnel>> {
        let pool = self.get_pool()?;
        let pool = pool.read().await;

        let rows = sqlx::query("SELECT * FROM ssh_tunnels WHERE auto_start = true ORDER BY name")
            .fetch_all(&*pool)
            .await
            .map_err(|e| DatabaseError::QueryFailed(e.to_string()))?;

        let mut tunnels = Vec::new();
        for row in rows {
            let tunnel = crate::models::ssh::SSHTunnel {
                base: crate::models::base::BaseModel {
                    id: row.get("id"),
                    created_at: chrono::DateTime::parse_from_rfc3339(
                        &row.get::<String, _>("created_at"),
                    )
                    .map_err(|e| DatabaseError::QueryFailed(e.to_string()))?
                    .with_timezone(&chrono::Utc),
                    updated_at: chrono::DateTime::parse_from_rfc3339(
                        &row.get::<String, _>("updated_at"),
                    )
                    .map_err(|e| DatabaseError::QueryFailed(e.to_string()))?
                    .with_timezone(&chrono::Utc),
                    device_id: row.get("device_id"),
                    version: row.get::<i64, _>("version") as u64,
                    sync_status: serde_json::from_str(&row.get::<String, _>("sync_status"))
                        .unwrap_or(crate::database::traits::SyncStatus::Synced),
                },
                name: row.get("name"),
                description: row.get("description"),
                profile_id: row.get("profile_id"),
                tunnel_type: serde_json::from_str(&row.get::<String, _>("tunnel_type"))
                    .map_err(|e| DatabaseError::QueryFailed(e.to_string()))?,
                local_host: row.get("local_host"),
                local_port: row.get::<i32, _>("local_port") as u16,
                remote_host: row.get("remote_host"),
                remote_port: row.get::<Option<i32>, _>("remote_port").map(|p| p as u16),
                auto_start: row.get("auto_start"),
                status: crate::models::ssh::TunnelStatus::default(),
                error_message: None,
            };
            tunnels.push(tunnel);
        }

        Ok(tunnels)
    }

    async fn update_ssh_tunnel(&self, model: &crate::models::ssh::SSHTunnel) -> DatabaseResult<()> {
        self.save_ssh_tunnel(model).await
    }

    async fn delete_ssh_tunnel(&self, id: &str) -> DatabaseResult<()> {
        let pool = self.get_pool()?;
        let pool = pool.read().await;

        sqlx::query("DELETE FROM ssh_tunnels WHERE id = ?")
            .bind(id)
            .execute(&*pool)
            .await
            .map_err(|e| DatabaseError::QueryFailed(e.to_string()))?;

        Ok(())
    }

    async fn delete_ssh_tunnels_by_profile_id(&self, profile_id: &str) -> DatabaseResult<()> {
        let pool = self.get_pool()?;
        let pool = pool.read().await;

        sqlx::query("DELETE FROM ssh_tunnels WHERE profile_id = ?")
            .bind(profile_id)
            .execute(&*pool)
            .await
            .map_err(|e| DatabaseError::QueryFailed(e.to_string()))?;

        Ok(())
    }
}

impl SQLiteProvider {
    /// Save master password entry
    pub async fn save_master_password_entry(
        &self,
        entry: &MasterPasswordEntry,
    ) -> DatabaseResult<()> {
        let pool_arc = self.get_pool()?;
        let pool = pool_arc.read().await;

        sqlx::query(
            r#"
            INSERT OR REPLACE INTO master_passwords (
                device_id, password_salt, verification_hash, auto_unlock,
                auto_lock_timeout, created_at, last_verified_at
            ) VALUES (?, ?, ?, ?, ?, ?, ?)
        "#,
        )
        .bind(&entry.device_id)
        .bind(&entry.password_salt.to_vec())
        .bind(&entry.verification_hash)
        .bind(entry.auto_unlock)
        .bind(entry.auto_lock_timeout.map(|t| t as i64))
        .bind(entry.created_at)
        .bind(entry.last_verified_at)
        .execute(&*pool)
        .await
        .map_err(|e| DatabaseError::QueryFailed(e.to_string()))?;

        Ok(())
    }

    /// Get master password entry by device ID
    pub async fn get_master_password_entry(
        &self,
        device_id: &str,
    ) -> DatabaseResult<Option<MasterPasswordEntry>> {
        let pool_arc = self.get_pool()?;
        let pool = pool_arc.read().await;

        let row = sqlx::query("SELECT * FROM master_passwords WHERE device_id = ?")
            .bind(device_id)
            .fetch_optional(&*pool)
            .await
            .map_err(|e| DatabaseError::QueryFailed(e.to_string()))?;

        if let Some(row) = row {
            let salt_bytes: Vec<u8> = row.get("password_salt");
            let mut salt_array = [0u8; 32];
            salt_array.copy_from_slice(&salt_bytes[..32]);

            let entry = MasterPasswordEntry {
                device_id: row.get("device_id"),
                password_salt: salt_array,
                verification_hash: row.get("verification_hash"),
                auto_unlock: row.get("auto_unlock"),
                auto_lock_timeout: row.get::<Option<i64>, _>("auto_lock_timeout").map(|t| t as u32),
                created_at: row.get("created_at"),
                last_verified_at: row.get("last_verified_at"),
            };
            Ok(Some(entry))
        } else {
            Ok(None)
        }
    }

    /// Delete master password entry
    pub async fn delete_master_password_entry(&self, device_id: &str) -> DatabaseResult<()> {
        let pool_arc = self.get_pool()?;
        let pool = pool_arc.read().await;

        sqlx::query("DELETE FROM master_passwords WHERE device_id = ?")
            .bind(device_id)
            .execute(&*pool)
            .await
            .map_err(|e| DatabaseError::QueryFailed(e.to_string()))?;

        Ok(())
    }

    /// Get current device from database
    pub async fn get_current_device(
        &self,
    ) -> DatabaseResult<Option<crate::models::auth::Device>> {
        let pool_arc = self.get_pool()?;
        let pool = pool_arc.read().await;

        let row = sqlx::query("SELECT * FROM devices WHERE is_current = true LIMIT 1")
            .fetch_optional(&*pool)
            .await
            .map_err(|e| DatabaseError::QueryFailed(e.to_string()))?;

        if let Some(row) = row {
            let device = crate::models::auth::Device {
                device_id: row.get("device_id"),
                device_name: row.get("device_name"),
                device_type: serde_json::from_str(&row.get::<String, _>("device_type"))
                    .unwrap_or(crate::models::auth::DeviceType::Unknown),
                os_info: crate::models::auth::OsInfo {
                    os_type: row.get("os_name"),
                    os_version: row.get("os_version"),
                    arch: "".to_string(), // Will be updated when we enhance schema
                    hostname: "".to_string(), // Will be updated when we enhance schema
                },
                app_version: env!("CARGO_PKG_VERSION").to_string(),
                created_at: chrono::DateTime::parse_from_rfc3339(
                    &row.get::<String, _>("created_at"),
                )
                .map_err(|e| DatabaseError::QueryFailed(e.to_string()))?
                .with_timezone(&chrono::Utc),
                last_seen: chrono::DateTime::parse_from_rfc3339(
                    &row.get::<String, _>("last_seen_at"),
                )
                .map_err(|e| DatabaseError::QueryFailed(e.to_string()))?
                .with_timezone(&chrono::Utc),
                is_current: row.get("is_current"),
            };
            Ok(Some(device))
        } else {
            Ok(None)
        }
    }

    /// Save device to database
    pub async fn save_device(
        &self,
        device: &crate::models::auth::Device,
    ) -> DatabaseResult<()> {
        let pool_arc = self.get_pool()?;
        let pool = pool_arc.read().await;

        // First set all devices as not current
        sqlx::query("UPDATE devices SET is_current = false")
            .execute(&*pool)
            .await
            .map_err(|e| DatabaseError::QueryFailed(e.to_string()))?;

        // Insert or replace the new current device
        sqlx::query(
            r#"
            INSERT OR REPLACE INTO devices (
                device_id, device_name, device_type, os_name, os_version,
                created_at, last_seen_at, is_current
            ) VALUES (?, ?, ?, ?, ?, ?, ?, ?)
            "#,
        )
        .bind(&device.device_id)
        .bind(&device.device_name)
        .bind(
            serde_json::to_string(&device.device_type)
                .unwrap_or_else(|_| "\"Unknown\"".to_string()),
        )
        .bind(&device.os_info.os_type)
        .bind(&device.os_info.os_version)
        .bind(device.created_at.to_rfc3339())
        .bind(device.last_seen.to_rfc3339())
        .bind(device.is_current)
        .execute(&*pool)
        .await
        .map_err(|e| DatabaseError::QueryFailed(e.to_string()))?;

        Ok(())
    }
}
