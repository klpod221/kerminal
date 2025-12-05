use crate::error::AppError;
use crate::models::terminal::profile::{
    CreateTerminalProfileRequest, TerminalProfile, UpdateTerminalProfileRequest,
};
use crate::state::AppState;
use tauri::State;

#[tauri::command]
pub async fn create_terminal_profile(
    app_state: State<'_, AppState>,
    request: CreateTerminalProfileRequest,
) -> Result<TerminalProfile, AppError> {
    let db = app_state.database_service.lock().await;
    db.create_terminal_profile(request)
        .await
        .map_err(|e| AppError::Database(e.to_string()))
}

#[tauri::command]
pub async fn get_terminal_profile(
    app_state: State<'_, AppState>,
    id: String,
) -> Result<TerminalProfile, AppError> {
    let db = app_state.database_service.lock().await;
    db.get_terminal_profile(&id)
        .await
        .map_err(|e| AppError::Database(e.to_string()))
}

#[tauri::command]
pub async fn list_terminal_profiles(
    app_state: State<'_, AppState>,
) -> Result<Vec<TerminalProfile>, AppError> {
    let db = app_state.database_service.lock().await;
    db.list_terminal_profiles()
        .await
        .map_err(|e| AppError::Database(e.to_string()))
}

#[tauri::command]
pub async fn update_terminal_profile(
    app_state: State<'_, AppState>,
    id: String,
    request: UpdateTerminalProfileRequest,
) -> Result<TerminalProfile, AppError> {
    let db = app_state.database_service.lock().await;
    db.update_terminal_profile(&id, request)
        .await
        .map_err(|e| AppError::Database(e.to_string()))
}

#[tauri::command]
pub async fn delete_terminal_profile(
    app_state: State<'_, AppState>,
    id: String,
) -> Result<(), AppError> {
    let db = app_state.database_service.lock().await;
    db.delete_terminal_profile(&id)
        .await
        .map_err(|e| AppError::Database(e.to_string()))
}
