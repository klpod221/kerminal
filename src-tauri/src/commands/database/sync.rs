use tauri::State;

use crate::{
    models::sync::{ConflictResolutionStrategy, SyncConflict, SyncOperation, SyncDirection},
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
) -> Result<String, String> {
    let database_service = app_state.database_service.lock().await;

    let config = database_service
        .find_external_database_by_id(&database_id)
        .await
        .map_err(|e| e.to_string())?
        .ok_or_else(|| "External database not found".to_string())?;

    drop(database_service);

    let sync_service = &app_state.sync_service;
    let engine = sync_service.engine();

    let log = match direction {
        SyncDirection::Pull => {
            engine
                .pull(&config)
                .await
                .map_err(|e| e.to_string())?
        }
        SyncDirection::Push => {
            engine
                .push(&config)
                .await
                .map_err(|e| e.to_string())?
        }
        SyncDirection::Bidirectional => {
            engine
                .sync(&config)
                .await
                .map_err(|e| e.to_string())?
        }
    };

    Ok(format!(
        "Sync completed: {} records synced, status: {:?}",
        log.records_synced, log.status
    ))
}

#[tauri::command]
pub async fn get_sync_status(
    database_id: String,
    app_state: State<'_, AppState>,
) -> Result<String, String> {
    let database_service = app_state.database_service.lock().await;

    let config = database_service
        .find_external_database_by_id(&database_id)
        .await
        .map_err(|e| e.to_string())?
        .ok_or_else(|| "External database not found".to_string())?;

    let status = if config.is_active {
        format!("Active - Last sync: {:?}", config.last_sync_at)
    } else {
        "Inactive".to_string()
    };

    Ok(status)
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
    let database_service = app_state.database_service.lock().await;

    database_service
        .toggle_external_database_active(&database_id, true)
        .await
        .map_err(|e| e.to_string())
}

#[tauri::command]
pub async fn disable_auto_sync(
    database_id: String,
    app_state: State<'_, AppState>,
) -> Result<(), String> {
    let database_service = app_state.database_service.lock().await;

    database_service
        .toggle_external_database_active(&database_id, false)
        .await
        .map_err(|e| e.to_string())
}

#[tauri::command]
pub async fn get_sync_statistics(
    app_state: State<'_, AppState>,
) -> Result<serde_json::Value, String> {
    let database_service = app_state.database_service.lock().await;

    let databases = database_service
        .find_all_external_databases()
        .await
        .map_err(|e| e.to_string())?;

    let total_databases = databases.len();
    let active_databases = databases.iter().filter(|db| db.is_active).count();
    let conflicts = database_service
        .find_unresolved_conflicts()
        .await
        .map_err(|e| e.to_string())?
        .len();

    Ok(serde_json::json!({
        "totalDatabases": total_databases,
        "activeDatabases": active_databases,
        "unresolvedConflicts": conflicts,
        "lastSyncTime": chrono::Utc::now().to_rfc3339()
    }))
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
