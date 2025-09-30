use serde::{Deserialize, Serialize};

use crate::{
    database::{
        error::DatabaseResult,
        traits::{Encryptable, EncryptionService},
    },
    impl_syncable,
    models::base::BaseModel,
};

/// SSH Group để organize profiles
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct SSHGroup {
    /// Base model với sync metadata
    #[serde(flatten)]
    pub base: BaseModel,

    /// Group information
    pub name: String,
    pub description: Option<String>,

    /// UI customization
    pub color: Option<String>,       // Hex color

    /// Group settings
    pub is_expanded: bool,           // UI state - expanded/collapsed
    pub default_auth_method: Option<String>,  // Default auth for new profiles in group
}

impl SSHGroup {
    /// Create a new SSH group
    pub fn new(device_id: String, name: String) -> Self {
        Self {
            base: BaseModel::new(device_id),
            name,
            description: None,
            color: None,
            is_expanded: true,
            default_auth_method: None,
        }
    }

    /// Update group information
    pub fn update_info(&mut self, name: Option<String>, description: Option<String>) {
        if let Some(name) = name {
            self.name = name;
        }
        if let Some(description) = description {
            self.description = Some(description);
        }
        self.base.touch();
    }

    /// Update UI customization
    pub fn update_appearance(
        &mut self,
        color: Option<String>,
        icon: Option<String>,
        sort_order: Option<i32>,
    ) {
        if let Some(color) = color {
            self.color = Some(color);
        }
        self.base.touch();
    }

    /// Toggle expansion state
    pub fn toggle_expanded(&mut self) {
        self.is_expanded = !self.is_expanded;
        // Note: UI state - no need to sync
    }

    /// Set default authentication method for new profiles
    pub fn set_default_auth_method(&mut self, auth_method: Option<String>) {
        self.default_auth_method = auth_method;
        self.base.touch();
    }

    /// Get display name
    pub fn display_name(&self) -> &str {
        &self.name
    }
}

// Implement Syncable trait using macro
impl_syncable!(SSHGroup, "ssh_groups");

// SSH Groups không có encrypted data
impl Encryptable for SSHGroup {
    fn encrypted_fields() -> Vec<&'static str> {
        vec![]
    }

    fn encrypt_fields(&mut self, _encryption_service: &dyn EncryptionService) -> DatabaseResult<()> {
        Ok(())
    }

    fn decrypt_fields(&mut self, _encryption_service: &dyn EncryptionService) -> DatabaseResult<()> {
        Ok(())
    }

    fn has_encrypted_data(&self) -> bool {
        false
    }

    fn encryption_device_id(&self) -> Option<&str> {
        None
    }
}

/// Request to create new SSH group
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct CreateSSHGroupRequest {
    pub name: String,
    pub description: Option<String>,
    pub color: Option<String>,
    pub icon: Option<String>,
    #[serde(rename = "sortOrder")]
    pub sort_order: Option<i32>,
    #[serde(rename = "defaultAuthMethod")]
    pub default_auth_method: Option<String>,
}

impl CreateSSHGroupRequest {
    pub fn to_group(self, device_id: String) -> SSHGroup {
        let mut group = SSHGroup::new(device_id, self.name);

        group.description = self.description;
        group.color = self.color;
        group.default_auth_method = self.default_auth_method;

        group
    }
}

/// Request to update SSH group
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct UpdateSSHGroupRequest {
    pub name: Option<String>,
    pub description: Option<Option<String>>,  // Some(None) = clear description
    pub color: Option<Option<String>>,
    pub icon: Option<Option<String>>,
    #[serde(rename = "sortOrder")]
    pub sort_order: Option<i32>,
    #[serde(rename = "isExpanded")]
    pub is_expanded: Option<bool>,
    #[serde(rename = "defaultAuthMethod")]
    pub default_auth_method: Option<Option<String>>,
}

impl UpdateSSHGroupRequest {
    pub fn apply_to_group(self, group: &mut SSHGroup) {
        let mut needs_touch = false;

        if let Some(name) = self.name {
            group.name = name;
            needs_touch = true;
        }
        if let Some(description) = self.description {
            group.description = description;
            needs_touch = true;
        }
        if let Some(color) = self.color {
            group.color = color;
            needs_touch = true;
        }
        if let Some(is_expanded) = self.is_expanded {
            group.is_expanded = is_expanded;
            // UI state - không cần sync
        }
        if let Some(default_auth_method) = self.default_auth_method {
            group.default_auth_method = default_auth_method;
            needs_touch = true;
        }

        if needs_touch {
            group.base.touch();
        }
    }
}

/// Group with profile statistics
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SSHGroupWithStats {
    #[serde(flatten)]
    pub group: SSHGroup,
    pub profile_count: u32,
    pub profiles: Vec<String>,  // Profile IDs in this group
}

impl SSHGroupWithStats {
    pub fn new(group: SSHGroup, profile_count: u32, profiles: Vec<String>) -> Self {
        Self {
            group,
            profile_count,
            profiles,
        }
    }
}

/// Enum to handle profiles when deleting a group
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum DeleteGroupAction {
    /// Move profiles to another group
    MoveToGroup(String),
    /// Move profiles to ungrouped
    MoveToUngrouped,
    /// Delete all profiles in group
    DeleteProfiles,
}

impl Default for DeleteGroupAction {
    fn default() -> Self {
        Self::MoveToUngrouped
    }
}

impl crate::database::sync::strategies::HasBaseModel for SSHGroup {
    fn base_model(&self) -> &crate::models::base::BaseModel {
        &self.base
    }
}
