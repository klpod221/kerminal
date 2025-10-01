use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// Sync metadata for tracking sync operations
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SyncMetadata {
    pub id: String,
    pub database_id: String, // External database ID
    pub last_sync_at: DateTime<Utc>,
    pub sync_direction: SyncDirection,
    pub records_synced: u32,
    pub conflicts_detected: u32,
    pub conflicts_resolved: u32,
    pub sync_duration_ms: u64,
    pub sync_status: SyncOperationStatus,
    pub error_message: Option<String>,
    pub created_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum SyncDirection {
    Push,          // Local to remote
    Pull,          // Remote to local
    Bidirectional, // Both directions
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum SyncOperationStatus {
    InProgress,
    Completed,
    Failed,
    PartialSuccess, // Some records synced, some failed
}

/// Conflict record for tracking and resolution
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ConflictRecord {
    pub id: String,
    pub table_name: String,
    pub record_id: String,
    pub conflict_type: ConflictType,
    pub local_version: u64,
    pub remote_version: u64,
    pub local_data: serde_json::Value,
    pub remote_data: serde_json::Value,
    pub local_device_id: String,
    pub remote_device_id: String,
    pub detected_at: DateTime<Utc>,
    pub resolved_at: Option<DateTime<Utc>>,
    pub resolution: Option<ConflictResolution>,
    pub resolved_by: Option<String>, // Device ID or "auto"
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum ConflictType {
    /// Same record modified by different devices
    ConcurrentModification,
    /// Record deleted locally but modified remotely
    DeletedModified,
    /// Record modified locally but deleted remotely
    ModifiedDeleted,
    /// Different field values
    FieldMismatch,
    /// Schema version mismatch
    SchemaMismatch,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum ConflictResolution {
    UseLocal,
    UseRemote,
    Merge(MergeStrategy),
    Skip,
    Manual(serde_json::Value), // User-provided resolution
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MergeStrategy {
    pub field_resolutions: HashMap<String, FieldResolution>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum FieldResolution {
    UseLocal,
    UseRemote,
    Combine, // For arrays/lists
    Manual(serde_json::Value),
}

impl SyncMetadata {
    pub fn new(database_id: String, direction: SyncDirection) -> Self {
        Self {
            id: uuid::Uuid::new_v4().to_string(),
            database_id,
            last_sync_at: Utc::now(),
            sync_direction: direction,
            records_synced: 0,
            conflicts_detected: 0,
            conflicts_resolved: 0,
            sync_duration_ms: 0,
            sync_status: SyncOperationStatus::InProgress,
            error_message: None,
            created_at: Utc::now(),
        }
    }

    pub fn mark_completed(
        &mut self,
        records_synced: u32,
        conflicts_detected: u32,
        conflicts_resolved: u32,
        duration_ms: u64,
    ) {
        self.records_synced = records_synced;
        self.conflicts_detected = conflicts_detected;
        self.conflicts_resolved = conflicts_resolved;
        self.sync_duration_ms = duration_ms;
        self.sync_status = if conflicts_detected > 0 && conflicts_resolved < conflicts_detected {
            SyncOperationStatus::PartialSuccess
        } else {
            SyncOperationStatus::Completed
        };
    }

    pub fn mark_failed(&mut self, error: String, duration_ms: u64) {
        self.sync_status = SyncOperationStatus::Failed;
        self.error_message = Some(error);
        self.sync_duration_ms = duration_ms;
    }
}

impl ConflictRecord {
    pub fn new(
        table_name: String,
        record_id: String,
        conflict_type: ConflictType,
        local_version: u64,
        remote_version: u64,
        local_data: serde_json::Value,
        remote_data: serde_json::Value,
        local_device_id: String,
        remote_device_id: String,
    ) -> Self {
        Self {
            id: uuid::Uuid::new_v4().to_string(),
            table_name,
            record_id,
            conflict_type,
            local_version,
            remote_version,
            local_data,
            remote_data,
            local_device_id,
            remote_device_id,
            detected_at: Utc::now(),
            resolved_at: None,
            resolution: None,
            resolved_by: None,
        }
    }

    pub fn resolve(&mut self, resolution: ConflictResolution, resolved_by: String) {
        self.resolution = Some(resolution);
        self.resolved_at = Some(Utc::now());
        self.resolved_by = Some(resolved_by);
    }

    pub fn is_resolved(&self) -> bool {
        self.resolution.is_some()
    }

    pub fn age_hours(&self) -> i64 {
        (Utc::now() - self.detected_at).num_hours()
    }
}

/// Sync statistics to display in UI
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SyncStats {
    pub total_records: u32,
    pub synced_records: u32,
    pub pending_records: u32,
    pub failed_records: u32,
    pub conflicts: u32,
    pub last_sync: Option<DateTime<Utc>>,
    pub sync_enabled: bool,
    pub databases: Vec<DatabaseSyncStats>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DatabaseSyncStats {
    pub database_id: String,
    pub database_name: String,
    pub last_sync: Option<DateTime<Utc>>,
    pub records_synced: u32,
    pub sync_status: SyncOperationStatus,
    pub error_message: Option<String>,
}

impl Default for SyncStats {
    fn default() -> Self {
        Self {
            total_records: 0,
            synced_records: 0,
            pending_records: 0,
            failed_records: 0,
            conflicts: 0,
            last_sync: None,
            sync_enabled: false,
            databases: vec![],
        }
    }
}

impl std::fmt::Display for ConflictType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            ConflictType::ConcurrentModification => write!(f, "Concurrent Modification"),
            ConflictType::DeletedModified => write!(f, "Deleted vs Modified"),
            ConflictType::ModifiedDeleted => write!(f, "Modified vs Deleted"),
            ConflictType::FieldMismatch => write!(f, "Field Mismatch"),
            ConflictType::SchemaMismatch => write!(f, "Schema Mismatch"),
        }
    }
}

impl std::fmt::Display for SyncOperationStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            SyncOperationStatus::InProgress => write!(f, "In Progress"),
            SyncOperationStatus::Completed => write!(f, "Completed"),
            SyncOperationStatus::Failed => write!(f, "Failed"),
            SyncOperationStatus::PartialSuccess => write!(f, "Partial Success"),
        }
    }
}
