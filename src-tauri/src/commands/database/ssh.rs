use crate::models::ssh::{
    CreateSSHGroupRequest, CreateSSHKeyRequest, CreateSSHProfileRequest, DeleteGroupAction,
    SSHConfigHost, SSHGroup, SSHKey, SSHProfile, TestSSHConnectionRequest, UpdateSSHGroupRequest,
    UpdateSSHKeyRequest, UpdateSSHProfileRequest,
};
use crate::services::ssh_config_parser;
use crate::state::AppState;
use serde::Deserialize;
use tauri::{Emitter, State};

use super::common::app_result;

/// Create new SSH group
#[tauri::command]
pub async fn create_ssh_group(
    state: State<'_, AppState>,
    request: CreateSSHGroupRequest,
    app_handle: tauri::AppHandle,
) -> Result<SSHGroup, String> {
    let group = app_result!(state.ssh_service.create_ssh_group(request).await)?;
    let _ = app_handle.emit("ssh_group_created", &group);
    Ok(group)
}

/// Get all SSH groups
#[tauri::command]
pub async fn get_ssh_groups(state: State<'_, AppState>) -> Result<Vec<SSHGroup>, String> {
    app_result!(state.ssh_service.get_ssh_groups().await)
}

/// Get SSH group by ID
#[tauri::command]
pub async fn get_ssh_group(state: State<'_, AppState>, id: String) -> Result<SSHGroup, String> {
    app_result!(state.ssh_service.get_ssh_group(&id).await)
}

/// Update SSH group
#[tauri::command]
pub async fn update_ssh_group(
    state: State<'_, AppState>,
    id: String,
    request: UpdateSSHGroupRequest,
    app_handle: tauri::AppHandle,
) -> Result<SSHGroup, String> {
    let group = app_result!(state.ssh_service.update_ssh_group(&id, request).await)?;
    let _ = app_handle.emit("ssh_group_updated", &group);
    Ok(group)
}

/// Delete SSH group with action for existing profiles
#[tauri::command]
pub async fn delete_ssh_group(
    state: State<'_, AppState>,
    id: String,
    action: DeleteGroupActionDto,
    app_handle: tauri::AppHandle,
) -> Result<(), String> {
    let action = action.into();
    app_result!(state.ssh_service.delete_ssh_group(&id, action).await)?;
    let _ = app_handle.emit("ssh_group_deleted", &serde_json::json!({ "id": id }));
    Ok(())
}

/// Create new SSH profile
#[tauri::command]
pub async fn create_ssh_profile(
    state: State<'_, AppState>,
    request: CreateSSHProfileRequest,
    app_handle: tauri::AppHandle,
) -> Result<SSHProfile, String> {
    let profile = app_result!(state.ssh_service.create_ssh_profile(request).await)?;
    let _ = app_handle.emit("ssh_profile_created", &profile);
    Ok(profile)
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
    app_handle: tauri::AppHandle,
) -> Result<SSHProfile, String> {
    let profile = app_result!(state.ssh_service.update_ssh_profile(&id, request).await)?;
    let _ = app_handle.emit("ssh_profile_updated", &profile);
    Ok(profile)
}

/// Delete SSH profile
#[tauri::command]
pub async fn delete_ssh_profile(
    state: State<'_, AppState>,
    id: String,
    app_handle: tauri::AppHandle,
) -> Result<(), String> {
    app_result!(state.ssh_service.delete_ssh_profile(&id).await)?;
    let _ = app_handle.emit("ssh_profile_deleted", &serde_json::json!({ "id": id }));
    Ok(())
}

/// Move profile to different group
#[tauri::command]
pub async fn move_profile_to_group(
    state: State<'_, AppState>,
    profile_id: String,
    group_id: Option<String>,
) -> Result<(), String> {
    app_result!(
        state
            .ssh_service
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

/// Test SSH connection with a profile
#[tauri::command]
pub async fn test_ssh_connection(
    state: State<'_, AppState>,
    request: TestSSHConnectionRequest,
) -> Result<(), String> {
    app_result!(state.ssh_service.test_ssh_connection(request).await)
}

/// DTO for delete group action from frontend
#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct DeleteGroupActionDto {
    pub action_type: String,
    pub target_group_id: Option<String>,
}

impl From<DeleteGroupActionDto> for DeleteGroupAction {
    fn from(dto: DeleteGroupActionDto) -> Self {
        match dto.action_type.as_str() {
            "moveToGroup" => {
                DeleteGroupAction::MoveToGroup(dto.target_group_id.unwrap_or_default())
            }
            "moveToUngrouped" => DeleteGroupAction::MoveToUngrouped,
            "deleteProfiles" => DeleteGroupAction::DeleteProfiles,
            _ => DeleteGroupAction::MoveToUngrouped,
        }
    }
}

/// Create new SSH key
#[tauri::command]
pub async fn create_ssh_key(
    state: State<'_, AppState>,
    request: CreateSSHKeyRequest,
    app_handle: tauri::AppHandle,
) -> Result<SSHKey, String> {
    let service = state.ssh_key_service.lock().await;
    let key = app_result!(service.create_ssh_key(request).await)?;
    let _ = app_handle.emit("ssh_key_created", &key);
    Ok(key)
}

/// Get all SSH keys
#[tauri::command]
pub async fn get_ssh_keys(state: State<'_, AppState>) -> Result<Vec<SSHKey>, String> {
    let service = state.ssh_key_service.lock().await;
    app_result!(service.get_ssh_keys().await)
}

/// Update SSH key (all provided fields)
#[tauri::command]
pub async fn update_ssh_key(
    state: State<'_, AppState>,
    id: String,
    request: UpdateSSHKeyRequest,
    app_handle: tauri::AppHandle,
) -> Result<SSHKey, String> {
    let service = state.ssh_key_service.lock().await;
    let key = app_result!(service.update_ssh_key(&id, request).await)?;
    let _ = app_handle.emit("ssh_key_updated", &key);
    Ok(key)
}

/// Delete SSH key
#[tauri::command]
pub async fn delete_ssh_key(
    state: State<'_, AppState>,
    id: String,
    force: bool,
    app_handle: tauri::AppHandle,
) -> Result<(), String> {
    let service = state.ssh_key_service.lock().await;
    app_result!(service.delete_ssh_key(&id, force).await)?;
    let _ = app_handle.emit("ssh_key_deleted", &serde_json::json!({ "id": id }));
    Ok(())
}

/// Count profiles using a specific key
#[tauri::command]
pub async fn count_profiles_using_key(
    state: State<'_, AppState>,
    key_id: String,
) -> Result<u32, String> {
    let service = state.ssh_key_service.lock().await;
    app_result!(service.count_profiles_using_key(&key_id).await)
}

/// Import SSH key from file path
#[tauri::command]
pub async fn import_ssh_key_from_file(
    state: State<'_, AppState>,
    name: String,
    file_path: String,
    passphrase: Option<String>,
    description: Option<String>,
) -> Result<SSHKey, String> {
    let service = state.ssh_key_service.lock().await;
    app_result!(
        service
            .import_ssh_key_from_file(name, &file_path, passphrase, description)
            .await
    )
}

#[tauri::command]
pub async fn cleanup_idle_connections(state: State<'_, AppState>) -> Result<(), String> {
    state.ssh_connection_pool.cleanup_idle().await;
    Ok(())
}

#[tauri::command]
pub async fn clear_connection_pool(state: State<'_, AppState>) -> Result<(), String> {
    state.ssh_connection_pool.clear().await;
    Ok(())
}

#[tauri::command]
pub async fn get_connection_pool_size(state: State<'_, AppState>) -> Result<usize, String> {
    Ok(state.ssh_connection_pool.pool_size().await)
}

/// Get SSH config hosts from ~/.ssh/config
#[tauri::command]
pub async fn get_ssh_config_hosts(
    _state: State<'_, AppState>,
) -> Result<Vec<SSHConfigHost>, String> {
    match ssh_config_parser::parse_ssh_config(None).await {
        Ok(hosts) => Ok(hosts),
        Err(e) => Err(e.to_string()),
    }
}
