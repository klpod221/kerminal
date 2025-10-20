use tauri::State;

use crate::{
    models::sync::{ConflictResolutionStrategy, SyncConflict, SyncOperation},
    state::AppState,
};

#[tauri::command]
pub async fn get_sync_history(
    limit: i32,
    app_state: State<'_, AppState>,
) -> Result<Vec<SyncOperation>, String> {
    let database_service = app_state.database_service.lock().await;

    database_service
        .find_recent_sync_operations(limit)
        .await
        .map_err(|e| e.to_string())
}

#[tauri::command]
pub async fn get_sync_conflicts(
    app_state: State<'_, AppState>,
) -> Result<Vec<SyncConflict>, String> {
    let database_service = app_state.database_service.lock().await;

    database_service
        .find_unresolved_conflicts()
        .await
        .map_err(|e| e.to_string())
}

#[tauri::command]
pub async fn resolve_conflict(
    conflict_id: String,
    strategy: ConflictResolutionStrategy,
    app_state: State<'_, AppState>,
) -> Result<(), String> {
    let database_service = app_state.database_service.lock().await;

    database_service
        .resolve_conflict(&conflict_id, strategy)
        .await
        .map_err(|e| e.to_string())
}

#[tauri::command]
pub async fn delete_conflict(
    conflict_id: String,
    app_state: State<'_, AppState>,
) -> Result<(), String> {
    let database_service = app_state.database_service.lock().await;

    database_service
        .delete_conflict(&conflict_id)
        .await
        .map_err(|e| e.to_string())
}

#[tauri::command]
pub async fn get_sync_operations_by_entity(
    entity_type: String,
    entity_id: String,
    app_state: State<'_, AppState>,
) -> Result<Vec<SyncOperation>, String> {
    let database_service = app_state.database_service.lock().await;

    database_service
        .find_sync_operations_by_entity(&entity_type, &entity_id)
        .await
        .map_err(|e| e.to_string())
}
