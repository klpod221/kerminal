use crate::models::saved_command::{
    CreateSavedCommandGroupRequest, CreateSavedCommandRequest, SavedCommand, SavedCommandGroup,
    UpdateSavedCommandGroupRequest, UpdateSavedCommandRequest,
};
use crate::state::AppState;
use tauri::State;

use crate::commands::database::common::app_result;

/// Macro for saved command service results that return String errors
macro_rules! saved_command_result {
    ($expr:expr) => {
        $expr
    };
}

// === Saved Command Commands ===

/// Create new saved command
#[tauri::command]
pub async fn create_saved_command(
    state: State<'_, AppState>,
    request: CreateSavedCommandRequest,
) -> Result<SavedCommand, String> {
    app_result!(state.saved_command_service.create_command(request).await)
}

/// Get all saved commands
#[tauri::command]
pub async fn get_saved_commands(state: State<'_, AppState>) -> Result<Vec<SavedCommand>, String> {
    app_result!(state.saved_command_service.get_commands().await)
}

/// Update saved command
#[tauri::command]
pub async fn update_saved_command(
    state: State<'_, AppState>,
    id: String,
    request: UpdateSavedCommandRequest,
) -> Result<SavedCommand, String> {
    app_result!(
        state
            .saved_command_service
            .update_command(&id, request)
            .await
    )
}

/// Delete saved command
#[tauri::command]
pub async fn delete_saved_command(state: State<'_, AppState>, id: String) -> Result<(), String> {
    app_result!(state.saved_command_service.delete_command(&id).await)
}

/// Increment command usage count
#[tauri::command]
pub async fn increment_command_usage(state: State<'_, AppState>, id: String) -> Result<(), String> {
    app_result!(state.saved_command_service.increment_usage(&id).await)
}

/// Toggle command favorite status
#[tauri::command]
pub async fn toggle_command_favorite(
    state: State<'_, AppState>,
    id: String,
) -> Result<SavedCommand, String> {
    app_result!(state.saved_command_service.toggle_favorite(&id).await)
}

// === Saved Command Group Commands ===

/// Create new saved command group
#[tauri::command]
pub async fn create_saved_command_group(
    state: State<'_, AppState>,
    request: CreateSavedCommandGroupRequest,
) -> Result<SavedCommandGroup, String> {
    app_result!(state.saved_command_service.create_group(request).await)
}

/// Get all saved command groups
#[tauri::command]
pub async fn get_saved_command_groups(
    state: State<'_, AppState>,
) -> Result<Vec<SavedCommandGroup>, String> {
    app_result!(state.saved_command_service.get_groups().await)
}

/// Update saved command group
#[tauri::command]
pub async fn update_saved_command_group(
    state: State<'_, AppState>,
    id: String,
    request: UpdateSavedCommandGroupRequest,
) -> Result<SavedCommandGroup, String> {
    app_result!(state.saved_command_service.update_group(&id, request).await)
}

/// Delete saved command group
#[tauri::command]
pub async fn delete_saved_command_group(
    state: State<'_, AppState>,
    id: String,
) -> Result<(), String> {
    app_result!(state.saved_command_service.delete_group(&id).await)
}
