use async_trait::async_trait;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use chrono::{DateTime, Utc};
use crate::database::error::DatabaseResult;

/// Core database trait that all providers must implement
#[async_trait]
pub trait Database: Send + Sync {
    /// Connect to the database
    async fn connect(&mut self) -> DatabaseResult<()>;

    /// Disconnect from the database
    async fn disconnect(&mut self) -> DatabaseResult<()>;

    /// Check if the database is connected
    fn is_connected(&self) -> bool;

    /// Test the database connection
    async fn test_connection(&self) -> DatabaseResult<()>;

    /// Execute raw SQL query (for SQL databases)
    async fn execute_raw(&self, query: &str, params: &[&dyn ToSqlValue]) -> DatabaseResult<u64>;

    /// Fetch raw query results (for SQL databases)
    async fn fetch_raw(&self, query: &str, params: &[&dyn ToSqlValue]) -> DatabaseResult<Vec<HashMap<String, SqlValue>>>;

    /// Concrete methods for specific types (object-safe)
    async fn save_ssh_profile(&self, model: &crate::database::models::ssh_profile::SSHProfile) -> DatabaseResult<()>;
    async fn find_ssh_profile_by_id(&self, id: &str) -> DatabaseResult<Option<crate::database::models::ssh_profile::SSHProfile>>;
    async fn find_all_ssh_profiles(&self) -> DatabaseResult<Vec<crate::database::models::ssh_profile::SSHProfile>>;
    async fn update_ssh_profile(&self, model: &crate::database::models::ssh_profile::SSHProfile) -> DatabaseResult<()>;
    async fn delete_ssh_profile(&self, id: &str) -> DatabaseResult<()>;

    async fn save_ssh_group(&self, model: &crate::database::models::ssh_group::SSHGroup) -> DatabaseResult<()>;
    async fn find_ssh_group_by_id(&self, id: &str) -> DatabaseResult<Option<crate::database::models::ssh_group::SSHGroup>>;
    async fn find_all_ssh_groups(&self) -> DatabaseResult<Vec<crate::database::models::ssh_group::SSHGroup>>;
    async fn update_ssh_group(&self, model: &crate::database::models::ssh_group::SSHGroup) -> DatabaseResult<()>;
    async fn delete_ssh_group(&self, id: &str) -> DatabaseResult<()>;

    /// Transaction support

    /// Schema management
    async fn create_tables(&self) -> DatabaseResult<()>;
    async fn drop_tables(&self) -> DatabaseResult<()>;
    async fn migrate(&self, version: u32) -> DatabaseResult<()>;

    /// Provider-specific info
    fn provider_type(&self) -> DatabaseProviderType;
    fn connection_info(&self) -> String;
}

/// Transaction trait for atomic operations
#[async_trait]
pub trait DatabaseTransaction: Send + Sync {
    async fn commit(&mut self) -> DatabaseResult<()>;
    async fn rollback(&mut self) -> DatabaseResult<()>;

    async fn save<T>(&mut self, model: &T) -> DatabaseResult<()>
    where
        T: Syncable + Serialize + Send + Sync;

    async fn update<T>(&mut self, model: &T) -> DatabaseResult<()>
    where
        T: Syncable + Serialize + Send + Sync;

    async fn delete<T>(&mut self, id: &str) -> DatabaseResult<()>
    where
        T: Syncable + Send + Sync;
}

/// Trait for models that can be synchronized across databases
pub trait Syncable: Send + Sync {
    /// Get the table/collection name for this model
    fn table_name() -> &'static str;

    /// Get the unique identifier for this model
    fn id(&self) -> &str;

    /// Get the device ID that created/modified this record
    fn device_id(&self) -> &str;

    /// Get the creation timestamp
    fn created_at(&self) -> DateTime<Utc>;

    /// Get the last modification timestamp
    fn updated_at(&self) -> DateTime<Utc>;

    /// Get the version number for conflict resolution
    fn version(&self) -> u64;

    /// Set the version number
    fn set_version(&mut self, version: u64);

    /// Get sync status
    fn sync_status(&self) -> &SyncStatus;

    /// Set sync status
    fn set_sync_status(&mut self, status: SyncStatus);

    /// Generate a checksum for change detection
    fn checksum(&self) -> String;

    /// Check if this model should be synced to external databases
    fn should_sync(&self) -> bool {
        true
    }

    /// Get fields that should be excluded from sync
    fn excluded_fields() -> Vec<&'static str> {
        vec![]
    }
}

/// Trait for models with encrypted fields
pub trait Encryptable: Send + Sync {
    /// Get fields that should be encrypted
    fn encrypted_fields() -> Vec<&'static str>;

    /// Encrypt sensitive fields
    fn encrypt_fields(&mut self, encryption_service: &dyn EncryptionService) -> DatabaseResult<()>;

    /// Decrypt sensitive fields
    fn decrypt_fields(&mut self, encryption_service: &dyn EncryptionService) -> DatabaseResult<()>;

    /// Check if the model has encrypted data
    fn has_encrypted_data(&self) -> bool;

    /// Get the device ID that encrypted this data (for multi-password support)
    fn encryption_device_id(&self) -> Option<&str>;
}

/// Encryption service trait
#[async_trait]
pub trait EncryptionService: Send + Sync {
    async fn encrypt(&self, data: &[u8], device_id: Option<&str>) -> DatabaseResult<Vec<u8>>;
    async fn decrypt(&self, encrypted_data: &[u8], device_id: Option<&str>) -> DatabaseResult<Vec<u8>>;
    async fn encrypt_string(&self, data: &str, device_id: Option<&str>) -> DatabaseResult<String>;
    async fn decrypt_string(&self, encrypted_data: &str, device_id: Option<&str>) -> DatabaseResult<String>;
}

/// Query criteria for filtering data
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct QueryCriteria {
    pub filters: HashMap<String, QueryFilter>,
    pub sort_by: Option<String>,
    pub sort_direction: SortDirection,
    pub limit: Option<u32>,
    pub offset: Option<u32>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum QueryFilter {
    Equals(SqlValue),
    NotEquals(SqlValue),
    In(Vec<SqlValue>),
    NotIn(Vec<SqlValue>),
    Like(String),
    NotLike(String),
    GreaterThan(SqlValue),
    LessThan(SqlValue),
    GreaterThanOrEqual(SqlValue),
    LessThanOrEqual(SqlValue),
    Between(SqlValue, SqlValue),
    IsNull,
    IsNotNull,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum SortDirection {
    Asc,
    Desc,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum SqlValue {
    Text(String),
    Integer(i64),
    Real(f64),
    Boolean(bool),
    DateTime(DateTime<Utc>),
    Null,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum SyncStatus {
    Pending,    // Waiting to be synced
    Syncing,    // Currently syncing
    Synced,     // Successfully synced
    Failed,     // Sync failed
    Conflict,   // Conflict detected
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum DatabaseProviderType {
    SQLite,
    MySQL,
    PostgreSQL,
    MongoDB,
}

/// Trait for converting values to SQL parameters
pub trait ToSqlValue {
    fn to_sql_value(&self) -> SqlValue;
}

// Implementations for common types
impl ToSqlValue for String {
    fn to_sql_value(&self) -> SqlValue {
        SqlValue::Text(self.clone())
    }
}

impl ToSqlValue for &str {
    fn to_sql_value(&self) -> SqlValue {
        SqlValue::Text(self.to_string())
    }
}

impl ToSqlValue for i64 {
    fn to_sql_value(&self) -> SqlValue {
        SqlValue::Integer(*self)
    }
}

impl ToSqlValue for f64 {
    fn to_sql_value(&self) -> SqlValue {
        SqlValue::Real(*self)
    }
}

impl ToSqlValue for bool {
    fn to_sql_value(&self) -> SqlValue {
        SqlValue::Boolean(*self)
    }
}

impl ToSqlValue for DateTime<Utc> {
    fn to_sql_value(&self) -> SqlValue {
        SqlValue::DateTime(*self)
    }
}

impl Default for QueryCriteria {
    fn default() -> Self {
        Self {
            filters: HashMap::new(),
            sort_by: None,
            sort_direction: SortDirection::Asc,
            limit: None,
            offset: None,
        }
    }
}

impl QueryCriteria {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn filter(mut self, field: &str, filter: QueryFilter) -> Self {
        self.filters.insert(field.to_string(), filter);
        self
    }

    pub fn sort_by(mut self, field: &str, direction: SortDirection) -> Self {
        self.sort_by = Some(field.to_string());
        self.sort_direction = direction;
        self
    }

    pub fn limit(mut self, limit: u32) -> Self {
        self.limit = Some(limit);
        self
    }

    pub fn offset(mut self, offset: u32) -> Self {
        self.offset = Some(offset);
        self
    }
}
