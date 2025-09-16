use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};
use uuid::Uuid;

use crate::database::traits::{Syncable, SyncStatus};

/// Base model that provides common fields for all syncable models
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BaseModel {
    pub id: String,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
    pub device_id: String,
    pub version: u64,
    pub sync_status: SyncStatus,
}

impl BaseModel {
    /// Create a new base model with current timestamp and device ID
    pub fn new(device_id: String) -> Self {
        let now = Utc::now();
        Self {
            id: Uuid::new_v4().to_string(),
            created_at: now,
            updated_at: now,
            device_id,
            version: 1,
            sync_status: SyncStatus::Pending,
        }
    }

    /// Update the timestamp and increment version
    pub fn touch(&mut self) {
        self.updated_at = Utc::now();
        self.version += 1;
        self.sync_status = SyncStatus::Pending;
    }

    /// Generate a checksum for this model (without metadata fields)
    pub fn generate_checksum<T: Serialize>(&self, model: &T) -> String {
        // Serialize the model and create a hash
        let json = serde_json::to_string(model).unwrap_or_default();
        let mut hasher = Sha256::new();
        hasher.update(json.as_bytes());
        format!("{:x}", hasher.finalize())
    }
}

impl Default for BaseModel {
    fn default() -> Self {
        Self::new("unknown".to_string())
    }
}



/// Macro to automatically implement Syncable trait for models with BaseModel
#[macro_export]
macro_rules! impl_syncable {
    ($model:ty, $table:expr) => {
        impl $crate::database::traits::Syncable for $model {
            fn table_name() -> &'static str {
                $table
            }

            fn id(&self) -> &str {
                &self.base.id
            }

            fn device_id(&self) -> &str {
                &self.base.device_id
            }

            fn created_at(&self) -> chrono::DateTime<chrono::Utc> {
                self.base.created_at
            }

            fn updated_at(&self) -> chrono::DateTime<chrono::Utc> {
                self.base.updated_at
            }

            fn version(&self) -> u64 {
                self.base.version
            }

            fn set_version(&mut self, version: u64) {
                self.base.version = version;
            }

            fn sync_status(&self) -> &$crate::database::traits::SyncStatus {
                &self.base.sync_status
            }

            fn set_sync_status(&mut self, status: $crate::database::traits::SyncStatus) {
                self.base.sync_status = status;
            }

            fn checksum(&self) -> String {
                self.base.generate_checksum(self)
            }
        }
    };
}

/// Device information for tracking which device created/modified records
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DeviceInfo {
    pub device_id: String,
    pub device_name: String,
    pub device_type: DeviceType,
    pub os_info: String,
    pub app_version: String,
    pub created_at: DateTime<Utc>,
    pub last_seen: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum DeviceType {
    Desktop,
    Laptop,
    Mobile,
    Server,
    Unknown,
}

impl DeviceInfo {
    pub fn new(device_name: String) -> Self {
        let now = Utc::now();
        Self {
            device_id: Uuid::new_v4().to_string(),
            device_name,
            device_type: DeviceType::Desktop, // Can be detected later
            os_info: std::env::consts::OS.to_string(),
            app_version: env!("CARGO_PKG_VERSION").to_string(),
            created_at: now,
            last_seen: now,
        }
    }

    pub fn update_last_seen(&mut self) {
        self.last_seen = Utc::now();
    }
}
