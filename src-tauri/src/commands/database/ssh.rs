use crate::models::ssh::{
    CreateSSHGroupRequest, DeleteGroupAction, SSHGroup, UpdateSSHGroupRequest,
    CreateSSHProfileRequest, SSHProfile, UpdateSSHProfileRequest,
};
use crate::state::AppState;
use serde::Deserialize;
use tauri::State;

use super::common::app_result;

// === SSH Group Commands ===

/// Create new SSH group
#[tauri::command]
pub async fn create_ssh_group(
    state: State<'_, AppState>,
    name: String,
    description: Option<String>,
    color: Option<String>,
    icon: Option<String>,
) -> Result<SSHGroup, String> {
    let request = CreateSSHGroupRequest {
        name,
        description,
        color,
        icon,
        sort_order: None,
        default_auth_method: None,
    };
    app_result!(state.ssh_service.create_ssh_group(request).await)
}

/// Get all SSH groups
#[tauri::command]
pub async fn get_ssh_groups(state: State<'_, AppState>) -> Result<Vec<SSHGroup>, String> {
    app_result!(state.ssh_service.get_ssh_groups().await)
}

/// Get SSH group by ID
#[tauri::command]
pub async fn get_ssh_group(
    state: State<'_, AppState>,
    id: String,
) -> Result<SSHGroup, String> {
    app_result!(state.ssh_service.get_ssh_group(&id).await)
}

/// Update SSH group
#[tauri::command]
pub async fn update_ssh_group(
    state: State<'_, AppState>,
    id: String,
    name: Option<String>,
    description: Option<String>,
    color: Option<String>,
    icon: Option<String>,
) -> Result<SSHGroup, String> {
    let request = UpdateSSHGroupRequest {
        name,
        description: description.map(Some),
        color: color.map(Some),
        icon: icon.map(Some),
        sort_order: None,
        is_expanded: None,
        default_auth_method: None,
    };
    app_result!(state.ssh_service.update_ssh_group(&id, request).await)
}

/// Delete SSH group with action for existing profiles
#[tauri::command]
pub async fn delete_ssh_group(
    state: State<'_, AppState>,
    id: String,
    action: DeleteGroupActionDto,
) -> Result<(), String> {
    let action = action.into();
    app_result!(state.ssh_service.delete_ssh_group(&id, action).await)
}

// === SSH Profile Commands ===

/// Create new SSH profile
#[tauri::command]
pub async fn create_ssh_profile(
    state: State<'_, AppState>,
    request: CreateSSHProfileRequest,
) -> Result<SSHProfile, String> {
    app_result!(state.ssh_service.create_ssh_profile(request).await)
}

/// Get all SSH profiles
#[tauri::command]
pub async fn get_ssh_profiles(state: State<'_, AppState>) -> Result<Vec<SSHProfile>, String> {
    app_result!(state.ssh_service.get_ssh_profiles().await)
}

/// Get SSH profile by ID
#[tauri::command]
pub async fn get_ssh_profile(state: State<'_, AppState>, id: String) -> Result<SSHProfile, String> {
    app_result!(state.ssh_service.get_ssh_profile(&id).await)
}

/// Update SSH profile
#[tauri::command]
pub async fn update_ssh_profile(
    state: State<'_, AppState>,
    id: String,
    request: UpdateSSHProfileRequest,
) -> Result<SSHProfile, String> {
    app_result!(state.ssh_service.update_ssh_profile(&id, request).await)
}

/// Delete SSH profile
#[tauri::command]
pub async fn delete_ssh_profile(state: State<'_, AppState>, id: String) -> Result<(), String> {
    app_result!(state.ssh_service.delete_ssh_profile(&id).await)
}

/// Move profile to different group
#[tauri::command]
pub async fn move_profile_to_group(
    state: State<'_, AppState>,
    profile_id: String,
    group_id: Option<String>,
) -> Result<(), String> {
    app_result!(
        state.ssh_service
            .move_profile_to_group(&profile_id, group_id.as_deref())
            .await
    )
}

/// Duplicate SSH profile with new name
#[tauri::command]
pub async fn duplicate_ssh_profile(
    state: State<'_, AppState>,
    id: String,
    new_name: String,
) -> Result<SSHProfile, String> {
    app_result!(state.ssh_service.duplicate_ssh_profile(&id, new_name).await)
}

// === DTOs for Frontend ===

/// DTO for delete group action from frontend
#[derive(Debug, Deserialize)]
pub struct DeleteGroupActionDto {
    pub action_type: String,
    pub target_group_id: Option<String>,
}

impl From<DeleteGroupActionDto> for DeleteGroupAction {
    fn from(dto: DeleteGroupActionDto) -> Self {
        match dto.action_type.as_str() {
            "move_to_group" => {
                DeleteGroupAction::MoveToGroup(dto.target_group_id.unwrap_or_default())
            }
            "move_to_ungrouped" => DeleteGroupAction::MoveToUngrouped,
            "delete_profiles" => DeleteGroupAction::DeleteProfiles,
            _ => DeleteGroupAction::MoveToUngrouped,
        }
    }
}