use crate::models::base::BaseModel;
use serde::{Deserialize, Serialize};

/// Saved command group model
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct SavedCommandGroup {
    #[serde(flatten)]
    pub base: BaseModel,
    pub name: String,
    pub description: Option<String>,
    pub color: Option<String>,
    pub icon: Option<String>,
}

/// Request to create a new saved command group
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct CreateSavedCommandGroupRequest {
    pub name: String,
    pub description: Option<String>,
    pub color: Option<String>,
    pub icon: Option<String>,
}

/// Request to update an existing saved command group
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct UpdateSavedCommandGroupRequest {
    pub name: Option<String>,
    pub description: Option<String>,
    pub color: Option<String>,
    pub icon: Option<String>,
}

impl SavedCommandGroup {
    /// Create a new saved command group
    pub fn new(device_id: String, name: String) -> Self {
        Self {
            base: BaseModel::new(device_id),
            name,
            description: None,
            color: None,
            icon: None,
        }
    }
}
