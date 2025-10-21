
use crate::models::base::BaseModel;
use serde::{Deserialize, Serialize};

/// Saved command model
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct SavedCommand {
    #[serde(flatten)]
    pub base: BaseModel,
    pub name: String,
    pub description: Option<String>,
    pub command: String,
    pub group_id: Option<String>,
    pub tags: Option<String>, // JSON array as string
    pub is_favorite: bool,
    pub usage_count: u32,
    pub last_used_at: Option<String>,
}

/// Request to create a new saved command
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct CreateSavedCommandRequest {
    pub name: String,
    pub description: Option<String>,
    pub command: String,
    pub group_id: Option<String>,
    pub tags: Option<String>,
    pub is_favorite: Option<bool>,
}

/// Request to update an existing saved command
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct UpdateSavedCommandRequest {
    pub name: Option<String>,
    pub description: Option<String>,
    pub command: Option<String>,
    pub group_id: Option<String>,
    pub tags: Option<String>,
    pub is_favorite: Option<bool>,
}

impl SavedCommand {
    /// Create a new saved command
    pub fn new(device_id: String, name: String, command: String, group_id: Option<String>) -> Self {
        Self {
            base: BaseModel::new(device_id),
            name,
            description: None,
            command,
            group_id,
            tags: None,
            is_favorite: false,
            usage_count: 0,
            last_used_at: None,
        }
    }
}
