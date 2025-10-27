use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

pub use super::external_db::ConflictResolutionStrategy;

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
