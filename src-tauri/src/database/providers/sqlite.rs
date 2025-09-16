use async_trait::async_trait;
use sqlx::{SqlitePool, Row};
use std::sync::Arc;
use tokio::sync::RwLock;

use crate::database::{
    traits::{Database, DatabaseProviderType, ToSqlValue, SqlValue, QueryCriteria},
    error::{DatabaseError, DatabaseResult},
    models::{
        ssh_profile::SSHProfile,
        ssh_group::SSHGroup,
        device::{Device, DeviceInfo},
        sync_metadata::{SyncMetadata, ConflictRecord},
    },
    encryption::device_keys::MasterPasswordEntry,
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
        self.pool.as_ref()
            .ok_or_else(|| DatabaseError::ConnectionFailed("Database not connected".to_string()))
    }
}

#[async_trait]
impl Database for SQLiteProvider {
    async fn connect(&mut self) -> DatabaseResult<()> {
        // Ensure parent directory exists
        if let Some(parent) = std::path::Path::new(&self.database_path).parent() {
            std::fs::create_dir_all(parent)
                .map_err(|e| DatabaseError::ConnectionFailed(
                    format!("Failed to create database directory: {}", e)
                ))?;
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
        sqlx::query(r#"
            CREATE TABLE IF NOT EXISTS ssh_profiles (
                id TEXT PRIMARY KEY,
                name TEXT NOT NULL,
                host TEXT NOT NULL,
                port INTEGER NOT NULL,
                username TEXT NOT NULL,
                group_id TEXT,
                auth_method TEXT NOT NULL,
                auth_data TEXT NOT NULL,
                tags TEXT NOT NULL DEFAULT '[]',
                description TEXT,
                color TEXT,
                timeout INTEGER,
                keep_alive BOOLEAN NOT NULL DEFAULT true,
                compression BOOLEAN NOT NULL DEFAULT false,
                sort_order INTEGER NOT NULL DEFAULT 0,
                created_at TEXT NOT NULL,
                updated_at TEXT NOT NULL,
                device_id TEXT NOT NULL,
                version INTEGER NOT NULL DEFAULT 1,
                sync_status TEXT NOT NULL DEFAULT 'Clean'
            )
        "#)
        .execute(&*pool)
        .await
        .map_err(|e| DatabaseError::QueryFailed(e.to_string()))?;

        // Create SSH groups table
        sqlx::query(r#"
            CREATE TABLE IF NOT EXISTS ssh_groups (
                id TEXT PRIMARY KEY,
                name TEXT NOT NULL,
                description TEXT,
                color TEXT,
                icon TEXT,
                sort_order INTEGER NOT NULL DEFAULT 0,
                is_expanded BOOLEAN NOT NULL DEFAULT true,
                default_auth_method TEXT,
                created_at TEXT NOT NULL,
                updated_at TEXT NOT NULL,
                device_id TEXT NOT NULL,
                version INTEGER NOT NULL DEFAULT 1,
                sync_status TEXT NOT NULL DEFAULT 'Clean'
            )
        "#)
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

    async fn fetch_raw(&self, query: &str, _params: &[&dyn ToSqlValue]) -> DatabaseResult<Vec<std::collections::HashMap<String, SqlValue>>> {
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
            map.insert("placeholder".to_string(), SqlValue::Text("placeholder".to_string()));
            results.push(map);
        }

        Ok(results)
    }

    async fn drop_tables(&self) -> DatabaseResult<()> {
        let pool_arc = self.get_pool()?;
        let pool = pool_arc.read().await;

        sqlx::query("DROP TABLE IF EXISTS ssh_profiles").execute(&*pool).await.ok();
        sqlx::query("DROP TABLE IF EXISTS ssh_groups").execute(&*pool).await.ok();
        sqlx::query("DROP TABLE IF EXISTS devices").execute(&*pool).await.ok();
        sqlx::query("DROP TABLE IF EXISTS master_passwords").execute(&*pool).await.ok();
        sqlx::query("DROP TABLE IF EXISTS sync_metadata").execute(&*pool).await.ok();

        Ok(())
    }

    async fn migrate(&self, version: u32) -> DatabaseResult<()> {
        // Basic migration system - for now just ensure tables exist
        match version {
            1 => {
                // Version 1: Create initial tables (already done in create_tables)
                self.create_tables().await?;
                Ok(())
            }
            _ => {
                // Unknown version - do nothing for now
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

        sqlx::query(r#"
            INSERT OR REPLACE INTO ssh_profiles (
                id, name, host, port, username, group_id, auth_method, auth_data,
                tags, notes, color, is_favorite, created_at, updated_at,
                device_id, version, sync_status
            ) VALUES (?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?)
        "#)
        .bind(&model.base.id)
        .bind(&model.name)
        .bind(&model.host)
        .bind(model.port as i32)
        .bind(&model.username)
        .bind(&model.group_id)
        .bind(serde_json::to_string(&model.auth_method).unwrap_or_default())
        .bind(serde_json::to_string(&model.auth_data).unwrap_or_default())
        .bind(serde_json::to_string(&model.tags).unwrap_or_default())
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

    async fn find_ssh_profile_by_id(&self, id: &str) -> DatabaseResult<Option<SSHProfile>> {
        let pool = self.get_pool()?;
        let pool = pool.read().await;

        let row = sqlx::query(
            "SELECT id, name, host, port, username, group_id, auth_method, auth_data, tags, description, color, timeout, keep_alive, compression, sort_order, created_at, updated_at, device_id, version, sync_status FROM ssh_profiles WHERE id = ?"
        )
        .bind(id)
        .fetch_optional(&*pool)
        .await
        .map_err(|e| DatabaseError::QueryFailed(e.to_string()))?;

        if let Some(row) = row {
            let profile = SSHProfile {
                base: crate::database::models::base::BaseModel {
                    id: row.get("id"),
                    created_at: chrono::DateTime::parse_from_rfc3339(&row.get::<String, _>("created_at"))
                        .map_err(|e| DatabaseError::ParseError(format!("Parse error: {}", e)))?
                        .with_timezone(&chrono::Utc),
                    updated_at: chrono::DateTime::parse_from_rfc3339(&row.get::<String, _>("updated_at"))
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
                icon: None, // Not stored in current schema
                sort_order: row.get::<i32, _>("sort_order"),
                description: row.get("description"),
                tags: serde_json::from_str(&row.get::<String, _>("tags"))
                    .unwrap_or_default(),
                proxy: None, // TODO: Implement proxy support in database
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
            "SELECT id, name, host, port, username, group_id, auth_method, auth_data, tags, description, color, timeout, keep_alive, compression, sort_order, created_at, updated_at, device_id, version, sync_status FROM ssh_profiles ORDER BY sort_order, name"
        )
        .fetch_all(&*pool)
        .await
        .map_err(|e| DatabaseError::QueryFailed(e.to_string()))?;

        let mut profiles = Vec::new();
        for row in rows {
            let profile = SSHProfile {
                base: crate::database::models::base::BaseModel {
                    id: row.get("id"),
                    created_at: chrono::DateTime::parse_from_rfc3339(&row.get::<String, _>("created_at"))
                        .map_err(|e| DatabaseError::ParseError(format!("Parse error: {}", e)))?
                        .with_timezone(&chrono::Utc),
                    updated_at: chrono::DateTime::parse_from_rfc3339(&row.get::<String, _>("updated_at"))
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
                icon: None, // Not stored in current schema
                sort_order: row.get::<i32, _>("sort_order"),
                description: row.get("description"),
                tags: serde_json::from_str(&row.get::<String, _>("tags"))
                    .unwrap_or_default(),
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

        sqlx::query(r#"
            INSERT OR REPLACE INTO ssh_groups (
                id, name, description, color, parent_id, sort_order,
                created_at, updated_at, device_id, version, sync_status
            ) VALUES (?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?)
        "#)
        .bind(&model.base.id)
        .bind(&model.name)
        .bind(&model.description)
        .bind(&model.color)
        .bind(model.sort_order as i32)
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
            "SELECT id, name, description, color, icon, sort_order, is_expanded, default_auth_method, created_at, updated_at, device_id, version, sync_status FROM ssh_groups WHERE id = ?"
        )
        .bind(id)
        .fetch_optional(&*pool)
        .await
        .map_err(|e| DatabaseError::QueryFailed(e.to_string()))?;

        if let Some(row) = row {
            let group = SSHGroup {
                base: crate::database::models::base::BaseModel {
                    id: row.get("id"),
                    created_at: chrono::DateTime::parse_from_rfc3339(&row.get::<String, _>("created_at"))
                        .map_err(|e| DatabaseError::ParseError(format!("Parse error: {}", e)))?
                        .with_timezone(&chrono::Utc),
                    updated_at: chrono::DateTime::parse_from_rfc3339(&row.get::<String, _>("updated_at"))
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
                icon: row.get("icon"),
                sort_order: row.get::<i32, _>("sort_order"),
                is_expanded: row.get("is_expanded"),
                default_auth_method: row.get("default_auth_method"),
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
            "SELECT id, name, description, color, icon, sort_order, is_expanded, default_auth_method, created_at, updated_at, device_id, version, sync_status FROM ssh_groups ORDER BY sort_order, name"
        )
        .fetch_all(&*pool)
        .await
        .map_err(|e| DatabaseError::QueryFailed(e.to_string()))?;

        let mut groups = Vec::new();
        for row in rows {
            let group = SSHGroup {
                base: crate::database::models::base::BaseModel {
                    id: row.get("id"),
                    created_at: chrono::DateTime::parse_from_rfc3339(&row.get::<String, _>("created_at"))
                        .map_err(|e| DatabaseError::ParseError(format!("Parse error: {}", e)))?
                        .with_timezone(&chrono::Utc),
                    updated_at: chrono::DateTime::parse_from_rfc3339(&row.get::<String, _>("updated_at"))
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
                icon: row.get("icon"),
                sort_order: row.get::<i32, _>("sort_order"),
                is_expanded: row.get("is_expanded"),
                default_auth_method: row.get("default_auth_method"),
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
}

impl SQLiteProvider {
    /// Save master password entry
    pub async fn save_master_password_entry(&self, entry: &MasterPasswordEntry) -> DatabaseResult<()> {
        let pool_arc = self.get_pool()?;
        let pool = pool_arc.read().await;

        sqlx::query(r#"
            INSERT OR REPLACE INTO master_passwords (
                device_id, device_name, password_salt, verification_hash, auto_unlock,
                created_at, last_verified_at
            ) VALUES (?, ?, ?, ?, ?, ?, ?)
        "#)
        .bind(&entry.device_id)
        .bind(&entry.device_name)
        .bind(&entry.password_salt.to_vec())
        .bind(&entry.verification_hash)
        .bind(entry.auto_unlock)
        .bind(entry.created_at)
        .bind(entry.last_verified_at)
        .execute(&*pool)
        .await
        .map_err(|e| DatabaseError::QueryFailed(e.to_string()))?;

        Ok(())
    }

    /// Get master password entry by device ID
    pub async fn get_master_password_entry(&self, device_id: &str) -> DatabaseResult<Option<MasterPasswordEntry>> {
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
                device_name: row.get("device_name"),
                password_salt: salt_array,
                verification_hash: row.get("verification_hash"),
                auto_unlock: row.get("auto_unlock"),
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
}
