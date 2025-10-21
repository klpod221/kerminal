#![allow(dead_code)]

use chrono::{DateTime, Utc};
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

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ConnectionDetails {
    pub host: String,
    pub port: u16,
    pub username: String,
    pub password: String,
    pub database: String,
}

impl ConnectionDetails {
    pub fn to_connection_string(&self, db_type: &DatabaseType) -> String {
        match db_type {
            DatabaseType::MySQL => {
                format!(
                    "mysql://{}:{}@{}:{}/{}",
                    self.username, self.password, self.host, self.port, self.database
                )
            }
            DatabaseType::PostgreSQL => {
                format!(
                    "postgresql://{}:{}@{}:{}/{}",
                    self.username, self.password, self.host, self.port, self.database
                )
            }
            DatabaseType::MongoDB => {
                format!(
                    "mongodb://{}:{}@{}:{}/{}",
                    self.username, self.password, self.host, self.port, self.database
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
            "LastWriteWins" => Ok(ConflictResolutionStrategy::LastWriteWins),
            "FirstWriteWins" => Ok(ConflictResolutionStrategy::FirstWriteWins),
            "Manual" => Ok(ConflictResolutionStrategy::Manual),
            "LocalWins" => Ok(ConflictResolutionStrategy::LocalWins),
            "RemoteWins" => Ok(ConflictResolutionStrategy::RemoteWins),
            _ => Err(format!("Unknown conflict resolution strategy: {}", s)),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct SyncSettings {
    pub auto_sync: bool,
    pub sync_interval_minutes: u64,
    pub conflict_resolution_strategy: ConflictResolutionStrategy,
}

impl Default for SyncSettings {
    fn default() -> Self {
        Self {
            auto_sync: false,
            sync_interval_minutes: 30,
            conflict_resolution_strategy: ConflictResolutionStrategy::Manual,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ExternalDatabaseConfig {
    #[serde(flatten)]
    pub base: BaseModel,
    pub name: String,
    pub db_type: DatabaseType,
    pub connection_details_encrypted: String,
    pub sync_settings: String,
    pub is_active: bool,
    pub last_sync_at: Option<DateTime<Utc>>,
}

impl ExternalDatabaseConfig {
    pub fn new(
        device_id: String,
        name: String,
        db_type: DatabaseType,
        connection_details_encrypted: String,
        sync_settings: String,
    ) -> Self {
        Self {
            base: BaseModel::new(device_id),
            name,
            db_type,
            connection_details_encrypted,
            sync_settings,
            is_active: false,
            last_sync_at: None,
        }
    }

    pub fn parse_sync_settings(&self) -> Result<SyncSettings, serde_json::Error> {
        serde_json::from_str(&self.sync_settings)
    }
}
