use crate::database::error::DatabaseError;
use crate::database::models::{
    ssh_profile::{SSHProfile, CreateSSHProfileRequest, UpdateSSHProfileRequest},
    ssh_group::{SSHGroup, CreateSSHGroupRequest, UpdateSSHGroupRequest, DeleteGroupAction},
    device::DeviceInfo,
};
use crate::database::encryption::master_password::SetupMasterPasswordRequest;
use crate::database::service::DatabaseStats;
use crate::state::AppState;
use tauri::State;
use serde::Deserialize;

// === Error handling ===

/// Convert DatabaseError to String for Tauri
impl From<DatabaseError> for String {
    fn from(error: DatabaseError) -> Self {
        match error {
            DatabaseError::ConnectionFailed(msg) => format!("Connection failed: {}", msg),
            DatabaseError::AuthenticationFailed(msg) => format!("Authentication failed: {}", msg),
            DatabaseError::QueryFailed(msg) => format!("Query failed: {}", msg),
            DatabaseError::TransactionFailed(msg) => format!("Transaction failed: {}", msg),
            DatabaseError::NotFound(msg) => format!("Not found: {}", msg),
            DatabaseError::ValidationError(msg) => format!("Validation error: {}", msg),
            DatabaseError::ParseError(msg) => format!("Parse error: {}", msg),
            DatabaseError::SerializationError(err) => format!("Serialization error: {}", err),
            DatabaseError::EncryptionError(err) => format!("Encryption error: {}", err),
            DatabaseError::SyncError(msg) => format!("Sync error: {}", msg),
            DatabaseError::ConfigError(msg) => format!("Configuration error: {}", msg),
            DatabaseError::MigrationError(msg) => format!("Migration error: {}", msg),
            DatabaseError::ConflictResolutionRequired => "Conflict resolution required".to_string(),
            DatabaseError::MasterPasswordRequired => "Master password required".to_string(),
            DatabaseError::UnsupportedProvider(msg) => format!("Unsupported provider: {}", msg),
            DatabaseError::Internal(err) => format!("Internal error: {}", err),
            DatabaseError::NotImplemented(msg) => format!("Not implemented: {}", msg),
        }
    }
}

/// Helper macro to convert DatabaseResult to Result<T, String>
macro_rules! db_result {
    ($expr:expr) => {
        $expr.map_err(|e| e.into())
    };
}

// === Master Password Commands ===

#[tauri::command]
pub async fn is_master_password_setup(state: State<'_, AppState>) -> Result<bool, String> {
    let db_service = state.database_service.lock().await;
    db_result!(db_service.is_master_password_setup().await)
}

#[tauri::command]
pub async fn setup_master_password(
    state: State<'_, AppState>,
    password: String,
    confirm_password: String,
    device_name: String,
    auto_unlock: bool,
) -> Result<(), String> {
    let db_service = state.database_service.lock().await;
    let request = SetupMasterPasswordRequest {
        device_name,
        password,
        confirm_password,
        auto_unlock,
        use_keychain: true,
    };
    db_result!(db_service.setup_master_password(request).await)
}

#[tauri::command]
pub async fn verify_master_password(
    state: State<'_, AppState>,
    password: String,
) -> Result<bool, String> {
    let db_service = state.database_service.lock().await;
    db_result!(db_service.verify_master_password(password).await)
}

#[tauri::command]
pub async fn try_auto_unlock(state: State<'_, AppState>) -> Result<bool, String> {
    let db_service = state.database_service.lock().await;
    db_result!(db_service.try_auto_unlock().await)
}

#[tauri::command]
pub async fn lock_session(state: State<'_, AppState>) -> Result<(), String> {
    let db_service = state.database_service.lock().await;
    db_service.lock_session().await;
    Ok(())
}

// === SSH Group Commands ===

#[tauri::command]
pub async fn create_ssh_group(
    state: State<'_, AppState>,
    name: String,
    description: Option<String>,
    color: Option<String>,
    icon: Option<String>,
) -> Result<SSHGroup, String> {
    let db_service = state.database_service.lock().await;
    let request = CreateSSHGroupRequest {
        name,
        description,
        color,
        icon,
        sort_order: None,
        default_auth_method: None,
    };
    db_result!(db_service.create_ssh_group(request).await)
}

#[tauri::command]
pub async fn get_ssh_groups(state: State<'_, AppState>) -> Result<Vec<SSHGroup>, String> {
    let db_service = state.database_service.lock().await;
    db_result!(db_service.get_ssh_groups().await)
}

#[tauri::command]
pub async fn get_ssh_group(
    state: State<'_, AppState>,
    id: String,
) -> Result<Option<SSHGroup>, String> {
    let db_service = state.database_service.lock().await;
    db_result!(db_service.get_ssh_group(&id).await)
}

#[tauri::command]
pub async fn update_ssh_group(
    state: State<'_, AppState>,
    id: String,
    name: Option<String>,
    description: Option<String>,
    color: Option<String>,
    icon: Option<String>,
) -> Result<SSHGroup, String> {
    let db_service = state.database_service.lock().await;
    let request = UpdateSSHGroupRequest {
        name,
        description: description.map(Some),
        color: color.map(Some),
        icon: icon.map(Some),
        sort_order: None,
        is_expanded: None,
        default_auth_method: None,
    };
    db_result!(db_service.update_ssh_group(&id, request).await)
}

#[tauri::command]
pub async fn delete_ssh_group(
    state: State<'_, AppState>,
    id: String,
    action: DeleteGroupActionDto,
) -> Result<(), String> {
    let db_service = state.database_service.lock().await;
    let action = action.into();
    db_result!(db_service.delete_ssh_group(&id, action).await)
}

// === DTOs for Frontend ===

#[derive(Debug, Deserialize)]
pub struct DeleteGroupActionDto {
    pub action_type: String,
    pub target_group_id: Option<String>,
}

impl From<DeleteGroupActionDto> for DeleteGroupAction {
    fn from(dto: DeleteGroupActionDto) -> Self {
        match dto.action_type.as_str() {
            "move_to_group" => DeleteGroupAction::MoveToGroup(
                dto.target_group_id.unwrap_or_default()
            ),
            "move_to_ungrouped" => DeleteGroupAction::MoveToUngrouped,
            "delete_profiles" => DeleteGroupAction::DeleteProfiles,
            _ => DeleteGroupAction::MoveToUngrouped,
        }
    }
}

// === SSH Profile Commands ===

#[tauri::command]
pub async fn create_ssh_profile(
    state: State<'_, AppState>,
    request: CreateSSHProfileRequest,
) -> Result<SSHProfile, String> {
    let db_service = state.database_service.lock().await;
    db_result!(db_service.create_ssh_profile(request).await)
}

#[tauri::command]
pub async fn get_ssh_profiles(state: State<'_, AppState>) -> Result<Vec<SSHProfile>, String> {
    let db_service = state.database_service.lock().await;
    db_result!(db_service.get_ssh_profiles(None).await)
}

#[tauri::command]
pub async fn get_ssh_profile(
    state: State<'_, AppState>,
    id: String,
) -> Result<SSHProfile, String> {
    let db_service = state.database_service.lock().await;
    let profile = db_service.get_ssh_profile(&id).await
        .map_err(|e| e.to_string())?;

    profile.ok_or_else(|| "SSH profile not found".to_string())
}

#[tauri::command]
pub async fn update_ssh_profile(
    state: State<'_, AppState>,
    id: String,
    request: UpdateSSHProfileRequest,
) -> Result<SSHProfile, String> {
    let db_service = state.database_service.lock().await;
    db_result!(db_service.update_ssh_profile(&id, request).await)
}

#[tauri::command]
pub async fn delete_ssh_profile(
    state: State<'_, AppState>,
    id: String,
) -> Result<(), String> {
    let db_service = state.database_service.lock().await;
    db_result!(db_service.delete_ssh_profile(&id).await)
}

#[tauri::command]
pub async fn move_profile_to_group(
    state: State<'_, AppState>,
    profile_id: String,
    group_id: Option<String>,
) -> Result<(), String> {
    let db_service = state.database_service.lock().await;
    db_result!(db_service.move_profile_to_group(&profile_id, group_id.as_deref()).await)
}

#[tauri::command]
pub async fn duplicate_ssh_profile(
    state: State<'_, AppState>,
    id: String,
    new_name: String,
) -> Result<SSHProfile, String> {
    let db_service = state.database_service.lock().await;
    db_result!(db_service.duplicate_ssh_profile(&id, new_name).await)
}

// === System Commands ===

#[tauri::command]
pub async fn get_database_stats(state: State<'_, AppState>) -> Result<DatabaseStats, String> {
    let db_service = state.database_service.lock().await;
    db_result!(db_service.get_database_stats().await)
}

#[tauri::command]
pub async fn get_current_device(state: State<'_, AppState>) -> Result<DeviceInfo, String> {
    let db_service = state.database_service.lock().await;
    let device = db_service.get_current_device();
    Ok(device)
}

// === DTOs ===
