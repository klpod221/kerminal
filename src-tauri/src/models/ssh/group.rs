use serde::{Deserialize, Serialize};

use crate::{
    database::{
        error::DatabaseResult,
        traits::{Encryptable, EncryptionService},
    },
    impl_syncable,
    models::base::BaseModel,
};

/// SSH Group to organize profiles
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct SSHGroup {
    /// Base model with sync metadata
    #[serde(flatten)]
    pub base: BaseModel,

    /// Group information
    pub name: String,
    pub description: Option<String>,

    /// UI customization
    pub color: Option<String>, // Hex color
}

impl SSHGroup {
    /// Create a new SSH group
    pub fn new(device_id: String, name: String) -> Self {
        Self {
            base: BaseModel::new(device_id),
            name,
            description: None,
            color: None,
        }
    }
}

impl_syncable!(SSHGroup, "ssh_groups");

impl Encryptable for SSHGroup {
    fn encrypted_fields() -> Vec<&'static str> {
        vec![]
    }

    fn encrypt_fields(
        &mut self,
        _encryption_service: &dyn EncryptionService,
    ) -> DatabaseResult<()> {
        Ok(())
    }

    fn decrypt_fields(
        &mut self,
        _encryption_service: &dyn EncryptionService,
    ) -> DatabaseResult<()> {
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
}

impl CreateSSHGroupRequest {
    pub fn to_group(self, device_id: String) -> SSHGroup {
        let mut group = SSHGroup::new(device_id, self.name);

        group.description = self.description;
        group.color = self.color;

        group
    }
}

/// Request to update SSH group
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct UpdateSSHGroupRequest {
    pub name: Option<String>,
    pub description: Option<Option<String>>, // Some(None) = clear description
    pub color: Option<Option<String>>,
    pub icon: Option<Option<String>>,
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

        if needs_touch {
            group.base.touch();
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
