use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

/// Sync direction configuration
#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "camelCase")]
pub enum SyncDirection {
    /// Only push local changes to remote
    Push,
    /// Only pull remote changes to local
    Pull,
    /// Bidirectional sync (push and pull)
    Both,
}

impl std::fmt::Display for SyncDirection {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            SyncDirection::Push => write!(f, "Push"),
            SyncDirection::Pull => write!(f, "Pull"),
            SyncDirection::Both => write!(f, "Both"),
        }
    }
}

impl std::str::FromStr for SyncDirection {
    type Err = String;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            // PascalCase (original format)
            "Push" => Ok(SyncDirection::Push),
            "Pull" => Ok(SyncDirection::Pull),
            "Both" => Ok(SyncDirection::Both),
            // camelCase (serde format)
            "push" => Ok(SyncDirection::Push),
            "pull" => Ok(SyncDirection::Pull),
            "both" => Ok(SyncDirection::Both),
            _ => Err(format!("Unknown sync direction: {}", s)),
        }
    }
}

impl Default for SyncDirection {
    fn default() -> Self {
        SyncDirection::Both
    }
}

/// Sync operation status
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "camelCase")]
pub enum SyncStatus {
    /// Never synced
    Never,
    /// Sync in progress
    InProgress,
    /// Last sync successful
    Success,
    /// Last sync failed
    Failed,
}

impl std::fmt::Display for SyncStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            SyncStatus::Never => write!(f, "Never"),
            SyncStatus::InProgress => write!(f, "InProgress"),
            SyncStatus::Success => write!(f, "Success"),
            SyncStatus::Failed => write!(f, "Failed"),
        }
    }
}

impl std::str::FromStr for SyncStatus {
    type Err = String;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            "Never" => Ok(SyncStatus::Never),
            "InProgress" => Ok(SyncStatus::InProgress),
            "Success" => Ok(SyncStatus::Success),
            "Failed" => Ok(SyncStatus::Failed),
            _ => Err(format!("Unknown sync status: {}", s)),
        }
    }
}

impl Default for SyncStatus {
    fn default() -> Self {
        SyncStatus::Never
    }
}

/// Global sync settings (applies to ALL external databases)
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct SyncSettings {
    /// Always "global" - single settings instance
    pub id: String,

    /// Enable/disable sync globally
    pub is_active: bool,

    /// Enable automatic sync
    pub auto_sync_enabled: bool,

    /// Sync interval in minutes (for auto-sync)
    pub sync_interval_minutes: u32,

    /// Conflict resolution strategy
    pub conflict_strategy: super::external_db::ConflictResolutionStrategy,

    /// Sync direction
    pub sync_direction: SyncDirection,

    /// Last selected database ID (for UI persistence)
    pub selected_database_id: Option<String>,

    /// Last sync timestamp
    pub last_sync_at: Option<DateTime<Utc>>,

    /// Created timestamp
    pub created_at: DateTime<Utc>,

    /// Updated timestamp
    pub updated_at: DateTime<Utc>,
}

impl SyncSettings {
    /// Create new global sync settings with default values
    pub fn new() -> Self {
        let now = Utc::now();
        Self {
            id: "global".to_string(),
            is_active: false,
            auto_sync_enabled: false,
            sync_interval_minutes: 15,
            conflict_strategy: super::external_db::ConflictResolutionStrategy::Manual,
            sync_direction: SyncDirection::Both,
            selected_database_id: None,
            last_sync_at: None,
            created_at: now,
            updated_at: now,
        }
    }

    /// Update sync timestamp
    pub fn mark_sync_complete(&mut self) {
        self.last_sync_at = Some(Utc::now());
        self.updated_at = Utc::now();
    }

    /// Touch updated_at timestamp
    pub fn touch(&mut self) {
        self.updated_at = Utc::now();
    }
}

/// Request to update global sync settings
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct UpdateSyncSettingsRequest {
    pub is_active: Option<bool>,
    pub auto_sync_enabled: Option<bool>,
    pub sync_interval_minutes: Option<u32>,
    pub conflict_strategy: Option<String>,
    pub sync_direction: Option<String>,
    pub selected_database_id: Option<String>,
}
