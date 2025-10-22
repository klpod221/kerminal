use serde::{Deserialize, Serialize};

use crate::models::base::BaseModel;

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "camelCase")]
pub enum DatabaseType {
    #[serde(rename = "mysql")]
    MySQL,
    #[serde(rename = "postgresql")]
    PostgreSQL,
    #[serde(rename = "mongodb")]
    MongoDB,
}

impl std::fmt::Display for DatabaseType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            DatabaseType::MySQL => write!(f, "mysql"),
            DatabaseType::PostgreSQL => write!(f, "postgresql"),
            DatabaseType::MongoDB => write!(f, "mongodb"),
        }
    }
}

impl std::str::FromStr for DatabaseType {
    type Err = String;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s.to_lowercase().as_str() {
            "mysql" => Ok(DatabaseType::MySQL),
            "postgresql" | "postgres" => Ok(DatabaseType::PostgreSQL),
            "mongodb" | "mongo" => Ok(DatabaseType::MongoDB),
            _ => Err(format!("Unknown database type: {}", s)),
        }
    }
}

/// Connection details for external database
/// Note: Password should be encrypted before storage
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ConnectionDetails {
    pub host: String,
    pub port: u16,
    pub username: String,
    pub password: String,
    pub database_name: String,
    #[serde(default)]
    pub ssl_enabled: bool,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub ssl_cert: Option<String>,
}

impl ConnectionDetails {
    /// Build connection string for the database type
    pub fn to_connection_string(&self, db_type: &DatabaseType) -> String {
        match db_type {
            DatabaseType::MySQL => {
                format!(
                    "mysql://{}:{}@{}:{}/{}",
                    self.username, self.password, self.host, self.port, self.database_name
                )
            }
            DatabaseType::PostgreSQL => {
                let ssl_mode = if self.ssl_enabled { "require" } else { "prefer" };
                format!(
                    "postgresql://{}:{}@{}:{}/{}?sslmode={}",
                    self.username, self.password, self.host, self.port, self.database_name, ssl_mode
                )
            }
            DatabaseType::MongoDB => {
                let ssl_param = if self.ssl_enabled { "&ssl=true" } else { "" };
                format!(
                    "mongodb://{}:{}@{}:{}/{}?authSource=admin{}",
                    self.username, self.password, self.host, self.port, self.database_name, ssl_param
                )
            }
        }
    }
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "camelCase")]
pub enum ConflictResolutionStrategy {
    LastWriteWins,
    FirstWriteWins,
    Manual,
    LocalWins,
    RemoteWins,
}

impl std::fmt::Display for ConflictResolutionStrategy {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            ConflictResolutionStrategy::LastWriteWins => write!(f, "LastWriteWins"),
            ConflictResolutionStrategy::FirstWriteWins => write!(f, "FirstWriteWins"),
            ConflictResolutionStrategy::Manual => write!(f, "Manual"),
            ConflictResolutionStrategy::LocalWins => write!(f, "LocalWins"),
            ConflictResolutionStrategy::RemoteWins => write!(f, "RemoteWins"),
        }
    }
}

impl std::str::FromStr for ConflictResolutionStrategy {
    type Err = String;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            // PascalCase (for database storage and Display)
            "LastWriteWins" => Ok(ConflictResolutionStrategy::LastWriteWins),
            "FirstWriteWins" => Ok(ConflictResolutionStrategy::FirstWriteWins),
            "Manual" => Ok(ConflictResolutionStrategy::Manual),
            "LocalWins" => Ok(ConflictResolutionStrategy::LocalWins),
            "RemoteWins" => Ok(ConflictResolutionStrategy::RemoteWins),
            // camelCase (for frontend compatibility)
            "lastWriteWins" => Ok(ConflictResolutionStrategy::LastWriteWins),
            "firstWriteWins" => Ok(ConflictResolutionStrategy::FirstWriteWins),
            "manual" => Ok(ConflictResolutionStrategy::Manual),
            "localWins" => Ok(ConflictResolutionStrategy::LocalWins),
            "remoteWins" => Ok(ConflictResolutionStrategy::RemoteWins),
            _ => Err(format!("Unknown conflict resolution strategy: {}", s)),
        }
    }
}

/// External database configuration
/// Stores connection information only (not sync settings)
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ExternalDatabaseConfig {
    #[serde(flatten)]
    pub base: BaseModel,

    /// Display name for this database
    pub name: String,

    /// Database type
    pub db_type: DatabaseType,

    /// Encrypted connection details (ConnectionDetails serialized + encrypted)
    pub connection_details_encrypted: String,

    /// Whether this database connection is currently active
    #[serde(default)]
    pub is_active: bool,
}

impl ExternalDatabaseConfig {
    /// Create new external database configuration
    pub fn new(
        device_id: String,
        name: String,
        db_type: DatabaseType,
        connection_details_encrypted: String,
    ) -> Self {
        Self {
            base: BaseModel::new(device_id),
            name,
            db_type,
            connection_details_encrypted,
            is_active: false, // Default to inactive
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct AddExternalDatabaseRequest {
    pub name: String,
    pub db_type: DatabaseType,
    pub connection_details: ConnectionDetails,
    pub auto_sync: bool,
    pub sync_interval_minutes: u64,
    pub conflict_resolution_strategy: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct UpdateExternalDatabaseRequest {
    pub id: String,
    pub name: Option<String>,
    pub connection_details: Option<ConnectionDetails>,
    pub auto_sync: Option<bool>,
    pub sync_interval_minutes: Option<u64>,
    pub conflict_resolution_strategy: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct TestConnectionRequest {
    pub db_type: DatabaseType,
    pub connection_details: ConnectionDetails,
    pub database_id: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ExternalDatabaseWithDetails {
    pub config: ExternalDatabaseConfig,
    pub connection_details: ConnectionDetails,
}
