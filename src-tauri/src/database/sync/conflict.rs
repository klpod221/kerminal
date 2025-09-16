use crate::database::error::DatabaseResult;
use crate::database::sync::strategies::{SyncStrategy, SyncResolution, HasBaseModel};
use chrono::{DateTime, Utc};
use serde::{Serialize, Deserialize};

#[derive(Debug)]
#[allow(dead_code)]
pub struct ConflictResolver {
    default_strategy: SyncStrategy,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum ConflictType {
    /// Data fields differ between local and remote
    DataConflict,
    /// Version mismatch
    VersionConflict,
    /// Device ID mismatch
    DeviceConflict,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ConflictRecord {
    pub id: String,
    pub entity_type: String,
    pub entity_id: String,
    pub conflict_type: ConflictType,
    pub local_version: u64,
    pub remote_version: u64,
    pub detected_at: DateTime<Utc>,
    pub resolved_at: Option<DateTime<Utc>>,
    pub resolution: Option<ConflictResolution>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum ConflictResolution {
    UseLocal,
    UseRemote,
    Manual(String), // Description of manual resolution
}

#[allow(dead_code)]
impl ConflictResolver {
    pub fn new(default_strategy: SyncStrategy) -> Self {
        Self { default_strategy }
    }

    pub fn resolve_conflict<T>(&self, local: &T, remote: &T) -> DatabaseResult<SyncResolution>
    where
        T: HasBaseModel,
    {
        let resolution = self.default_strategy.resolve_conflict(local, remote);
        Ok(resolution)
    }

    pub fn detect_conflict<T>(&self, local: &T, remote: &T) -> Option<ConflictType>
    where
        T: HasBaseModel,
    {
        let local_base = local.base_model();
        let remote_base = remote.base_model();

        if local_base.version != remote_base.version {
            Some(ConflictType::VersionConflict)
        } else if local_base.device_id != remote_base.device_id
                  && local_base.updated_at != remote_base.updated_at {
            Some(ConflictType::DataConflict)
        } else {
            None
        }
    }
}

impl Default for ConflictResolver {
    fn default() -> Self {
        Self::new(SyncStrategy::default())
    }
}
