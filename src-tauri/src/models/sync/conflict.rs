#![allow(dead_code)]
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

pub use super::external_db::ConflictResolutionStrategy;

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct SyncConflict {
    pub id: String,
    pub entity_type: String,
    pub entity_id: String,
    pub local_version: u64,
    pub remote_version: u64,
    pub local_data: String,
    pub remote_data: String,
    pub resolution_strategy: Option<ConflictResolutionStrategy>,
    pub resolved: bool,
    pub created_at: DateTime<Utc>,
    pub resolved_at: Option<DateTime<Utc>>,
}

/// Conflict resolution record stored in database
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ConflictResolution {
    pub id: String,
    pub entity_type: String,
    pub entity_id: String,
    pub local_data: serde_json::Value,
    pub remote_data: serde_json::Value,
    pub resolution_strategy: Option<ConflictResolutionStrategy>,
    pub resolved_at: Option<DateTime<Utc>>,
    pub created_at: DateTime<Utc>,
}

impl SyncConflict {
    pub fn new(
        entity_type: String,
        entity_id: String,
        local_version: u64,
        remote_version: u64,
        local_data: String,
        remote_data: String,
    ) -> Self {
        Self {
            id: Uuid::new_v4().to_string(),
            entity_type,
            entity_id,
            local_version,
            remote_version,
            local_data,
            remote_data,
            resolution_strategy: None,
            resolved: false,
            created_at: Utc::now(),
            resolved_at: None,
        }
    }

    pub fn resolve(&mut self, strategy: ConflictResolutionStrategy) {
        self.resolution_strategy = Some(strategy);
        self.resolved = true;
        self.resolved_at = Some(Utc::now());
    }

    pub fn is_resolved(&self) -> bool {
        self.resolved
    }
}
