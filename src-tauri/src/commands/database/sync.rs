use tauri::State;

use crate::{
    models::sync::{
        ConflictResolutionStrategy, SyncConflict, SyncDirection, SyncLog, SyncOperation,
    },
    state::AppState,
};

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
pub async fn get_unresolved_conflicts(
    app_state: State<'_, AppState>,
) -> Result<Vec<SyncConflict>, String> {
    let database_service = app_state.database_service.lock().await;

    database_service
        .find_unresolved_conflicts()
        .await
        .map_err(|e| e.to_string())
}

#[tauri::command]
pub async fn sync_now(
    database_id: String,
    direction: SyncDirection,
    app_state: State<'_, AppState>,
) -> Result<SyncLog, String> {
    app_state
        .sync_service
        .sync(&database_id, direction)
        .await
        .map_err(|e| e.to_string())
}

#[tauri::command]
pub async fn get_sync_status(
    database_id: String,
    app_state: State<'_, AppState>,
) -> Result<crate::services::sync::SyncServiceStatus, String> {
    app_state
        .sync_service
        .get_status(&database_id)
        .await
        .map_err(|e| e.to_string())
}

#[tauri::command]
pub async fn get_sync_logs(
    _database_id: String,
    limit: Option<i32>,
    app_state: State<'_, AppState>,
) -> Result<Vec<SyncOperation>, String> {
    let database_service = app_state.database_service.lock().await;

    let limit_value = limit.unwrap_or(50);

    database_service
        .find_recent_sync_operations(limit_value)
        .await
        .map_err(|e| e.to_string())
}

#[tauri::command]
pub async fn enable_auto_sync(
    database_id: String,
    app_state: State<'_, AppState>,
) -> Result<(), String> {
    app_state
        .sync_service
        .enable_auto_sync(&database_id)
        .await
        .map_err(|e| e.to_string())
}

#[tauri::command]
pub async fn disable_auto_sync(
    database_id: String,
    app_state: State<'_, AppState>,
) -> Result<(), String> {
    app_state
        .sync_service
        .disable_auto_sync(&database_id)
        .await
        .map_err(|e| e.to_string())
}

#[tauri::command]
pub async fn get_sync_statistics(
    app_state: State<'_, AppState>,
) -> Result<crate::models::sync::SyncStats, String> {
    let database_service = app_state.database_service.lock().await;
    database_service
        .get_sync_stats()
        .await
        .map_err(|e| e.to_string())
}

#[tauri::command]
pub async fn get_sync_service_statistics(
    app_state: State<'_, AppState>,
) -> Result<crate::services::sync::SyncServiceStatistics, String> {
    Ok(app_state.sync_service.get_statistics().await)
}

#[tauri::command]
pub async fn get_current_device(
    app_state: State<'_, AppState>,
) -> Result<Option<crate::models::auth::Device>, String> {
    let database_service = app_state.database_service.lock().await;
    let local_db = database_service.get_local_database();
    let local_db_read = local_db.read().await;

    local_db_read
        .get_current_device()
        .await
        .map_err(|e| e.to_string())
}

#[tauri::command]
pub async fn get_all_devices(
    app_state: State<'_, AppState>,
) -> Result<Vec<crate::models::auth::Device>, String> {
    let database_service = app_state.database_service.lock().await;

    database_service
        .get_all_devices()
        .await
        .map_err(|e| e.to_string())
}

#[tauri::command]
pub async fn register_device(
    device_name: String,
    device_type: String,
    app_state: State<'_, AppState>,
) -> Result<crate::models::auth::Device, String> {
    use crate::models::auth::{Device, DeviceType};

    let database_service = app_state.database_service.lock().await;

    let device_type_enum = match device_type.as_str() {
        "Desktop" => DeviceType::Desktop,
        "Laptop" => DeviceType::Laptop,
        "Mobile" => DeviceType::Mobile,
        "Server" => DeviceType::Server,
        _ => DeviceType::Unknown,
    };

    let mut device = Device::new_current(device_name);
    device.device_type = device_type_enum;

    let local_db = database_service.get_local_database();
    local_db
        .write()
        .await
        .save_device(&device)
        .await
        .map_err(|e| e.to_string())?;

    Ok(device)
}

/// Get all unresolved conflict resolutions for manual resolution
#[tauri::command]
pub async fn get_unresolved_conflict_resolutions(
    app_state: State<'_, AppState>,
) -> Result<Vec<crate::models::sync::conflict::ConflictResolution>, String> {
    let database_service = app_state.database_service.lock().await;
    let local_db = database_service.get_local_database();
    let local_db_read = local_db.read().await;

    local_db_read
        .get_unresolved_conflict_resolutions()
        .await
        .map_err(|e| e.to_string())
}

/// Resolve a conflict resolution with specified strategy
#[tauri::command]
pub async fn resolve_conflict_resolution(
    id: String,
    strategy: ConflictResolutionStrategy,
    app_state: State<'_, AppState>,
) -> Result<(), String> {
    let database_service = app_state.database_service.lock().await;
    let local_db = database_service.get_local_database();
    let local_db_read = local_db.read().await;

    local_db_read
        .resolve_conflict_resolution(&id, strategy)
        .await
        .map_err(|e| e.to_string())
}

/// Cleanup resolved conflicts older than specified days
#[tauri::command]
pub async fn cleanup_resolved_conflicts(
    days: i64,
    app_state: State<'_, AppState>,
) -> Result<usize, String> {
    let database_service = app_state.database_service.lock().await;
    let local_db = database_service.get_local_database();
    let local_db_read = local_db.read().await;

    local_db_read
        .cleanup_resolved_conflicts(days)
        .await
        .map_err(|e| e.to_string())
}

/// Get global sync settings
#[tauri::command]
pub async fn get_global_sync_settings(
    app_state: State<'_, AppState>,
) -> Result<Option<crate::models::sync::SyncSettings>, String> {
    let database_service = app_state.database_service.lock().await;
    let local_db = database_service.get_local_database();
    let local_db_read = local_db.read().await;

    local_db_read
        .get_global_sync_settings()
        .await
        .map_err(|e| e.to_string())
}

/// Update global sync settings
#[tauri::command]
pub async fn update_global_sync_settings(
    settings: crate::models::sync::UpdateSyncSettingsRequest,
    app_state: State<'_, AppState>,
) -> Result<(), String> {
    let database_service = app_state.database_service.lock().await;
    let local_db = database_service.get_local_database();
    let local_db_write = local_db.write().await;

    local_db_write
        .update_sync_settings(&settings)
        .await
        .map_err(|e| e.to_string())
}
