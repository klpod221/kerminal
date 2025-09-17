use crate::database::error::DatabaseResult;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// Database configuration for different providers
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DatabaseConfig {
    pub id: String,
    pub name: String,
    pub provider: DatabaseProvider,
    pub connection: ConnectionConfig,
    pub sync_settings: SyncSettings,
    pub enabled: bool,
}

/// Supported database providers
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum DatabaseProvider {
    SQLite,
    MySQL,
    PostgreSQL,
    MongoDB,
}

/// Connection configuration for each provider
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum ConnectionConfig {
    SQLite {
        file_path: String,
    },
    MySQL {
        host: String,
        port: u16,
        username: String,
        password: String, // Will be encrypted
        database: String,
        ssl_mode: Option<String>,
    },
    PostgreSQL {
        host: String,
        port: u16,
        username: String,
        password: String, // Will be encrypted
        database: String,
        ssl_mode: Option<String>,
    },
    MongoDB {
        connection_string: String, // Will be encrypted
        database: String,
    },
}

/// Sync configuration settings
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SyncSettings {
    pub strategy: SyncStrategy,
    pub auto_sync: bool,
    pub sync_interval_minutes: u32,
    pub conflict_resolution: ConflictResolution,
    pub enabled_models: Vec<String>, // Which models to sync
}

/// Sync strategies
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum SyncStrategy {
    LastWriteWins,
    FirstWriteWins,
    ManualResolve,
    DevicePriority(Vec<String>), // Ordered list of device IDs
}

/// Conflict resolution strategies
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum ConflictResolution {
    UseLocal,
    UseRemote,
    MergeFields,
    AskUser,
    Skip,
}

/// Master password configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MasterPasswordConfig {
    pub auto_unlock: bool,
    pub session_timeout_minutes: Option<u32>,
    pub require_on_startup: bool,
    pub use_keychain: bool,
}

impl DatabaseConfig {
    /// Create a new SQLite configuration (always local)
    pub fn new_sqlite(file_path: String) -> Self {
        Self {
            id: "local_sqlite".to_string(),
            name: "Local Database".to_string(),
            provider: DatabaseProvider::SQLite,
            connection: ConnectionConfig::SQLite { file_path },
            sync_settings: SyncSettings {
                strategy: SyncStrategy::LastWriteWins,
                auto_sync: false, // SQLite is local only
                sync_interval_minutes: 0,
                conflict_resolution: ConflictResolution::UseLocal,
                enabled_models: vec![],
            },
            enabled: true,
        }
    }

    /// Create a new MySQL configuration
    pub fn new_mysql(
        name: String,
        host: String,
        port: u16,
        username: String,
        password: String,
        database: String,
    ) -> Self {
        Self {
            id: uuid::Uuid::new_v4().to_string(),
            name,
            provider: DatabaseProvider::MySQL,
            connection: ConnectionConfig::MySQL {
                host,
                port,
                username,
                password,
                database,
                ssl_mode: Some("REQUIRED".to_string()),
            },
            sync_settings: SyncSettings::default(),
            enabled: true,
        }
    }

    /// Create a new PostgreSQL configuration
    pub fn new_postgresql(
        name: String,
        host: String,
        port: u16,
        username: String,
        password: String,
        database: String,
    ) -> Self {
        Self {
            id: uuid::Uuid::new_v4().to_string(),
            name,
            provider: DatabaseProvider::PostgreSQL,
            connection: ConnectionConfig::PostgreSQL {
                host,
                port,
                username,
                password,
                database,
                ssl_mode: Some("require".to_string()),
            },
            sync_settings: SyncSettings::default(),
            enabled: true,
        }
    }

    /// Create a new MongoDB configuration
    pub fn new_mongodb(name: String, connection_string: String, database: String) -> Self {
        Self {
            id: uuid::Uuid::new_v4().to_string(),
            name,
            provider: DatabaseProvider::MongoDB,
            connection: ConnectionConfig::MongoDB {
                connection_string,
                database,
            },
            sync_settings: SyncSettings::default(),
            enabled: true,
        }
    }

    /// Get connection string for SQL databases
    pub fn get_connection_string(&self) -> DatabaseResult<String> {
        match &self.connection {
            ConnectionConfig::SQLite { file_path } => Ok(format!("sqlite:{}", file_path)),
            ConnectionConfig::MySQL {
                host,
                port,
                username,
                password,
                database,
                ssl_mode,
            } => {
                let ssl = ssl_mode
                    .as_ref()
                    .map(|s| format!("?sslmode={}", s))
                    .unwrap_or_default();
                Ok(format!(
                    "mysql://{}:{}@{}:{}/{}{}",
                    username, password, host, port, database, ssl
                ))
            }
            ConnectionConfig::PostgreSQL {
                host,
                port,
                username,
                password,
                database,
                ssl_mode,
            } => {
                let ssl = ssl_mode
                    .as_ref()
                    .map(|s| format!("?sslmode={}", s))
                    .unwrap_or_default();
                Ok(format!(
                    "postgres://{}:{}@{}:{}/{}{}",
                    username, password, host, port, database, ssl
                ))
            }
            ConnectionConfig::MongoDB {
                connection_string, ..
            } => Ok(connection_string.clone()),
        }
    }
}

impl Default for SyncSettings {
    fn default() -> Self {
        Self {
            strategy: SyncStrategy::LastWriteWins,
            auto_sync: true,
            sync_interval_minutes: 15,
            conflict_resolution: ConflictResolution::MergeFields,
            enabled_models: vec!["SSHProfile".to_string(), "SSHGroup".to_string()],
        }
    }
}

impl Default for MasterPasswordConfig {
    fn default() -> Self {
        Self {
            auto_unlock: false,
            session_timeout_minutes: Some(60),
            require_on_startup: true,
            use_keychain: true,
        }
    }
}
