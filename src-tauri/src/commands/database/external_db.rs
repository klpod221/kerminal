use tauri::State;

use crate::database::encryption::ExternalDbEncryptor;
use crate::models::sync::{ConnectionDetails, DatabaseType, ExternalDatabaseConfig};
use crate::state::AppState;

#[tauri::command]
pub async fn add_external_database(
    name: String,
    db_type: DatabaseType,
    connection_details: ConnectionDetails,
    auto_sync: bool,
    sync_interval_minutes: u64,
    conflict_resolution_strategy: String,
    app_state: State<'_, AppState>,
) -> Result<ExternalDatabaseConfig, String> {
    let database_service = app_state.database_service.lock().await;
    let device_id = database_service.get_device_id().to_string();

    let master_password_manager = database_service.get_master_password_manager_arc();
    let encryptor = ExternalDbEncryptor::new(master_password_manager);

    let encrypted_connection_details = encryptor
        .encrypt_connection_details(&connection_details)
        .await
        .map_err(|e| format!("Failed to encrypt connection details: {}", e))?;

    let sync_settings = crate::models::sync::SyncSettings {
        auto_sync,
        sync_interval_minutes,
        conflict_resolution_strategy: conflict_resolution_strategy
            .parse()
            .map_err(|e| format!("Invalid conflict resolution strategy: {}", e))?,
    };

    let encrypted_sync_settings = encryptor
        .encrypt_sync_settings(&sync_settings)
        .await
        .map_err(|e| format!("Failed to encrypt sync settings: {}", e))?;

    let config = ExternalDatabaseConfig::new(
        device_id,
        name,
        db_type,
        encrypted_connection_details,
        encrypted_sync_settings,
    );

    database_service
        .save_external_database(&config)
        .await
        .map_err(|e| format!("Failed to save external database: {}", e))?;

    Ok(config)
}

#[tauri::command]
pub async fn get_external_databases(
    app_state: State<'_, AppState>,
) -> Result<Vec<ExternalDatabaseConfig>, String> {
    let database_service = app_state.database_service.lock().await;

    database_service
        .find_all_external_databases()
        .await
        .map_err(|e| format!("Failed to retrieve external databases: {}", e))
}

#[tauri::command]
pub async fn delete_external_database(
    id: String,
    app_state: State<'_, AppState>,
) -> Result<(), String> {
    let database_service = app_state.database_service.lock().await;

    database_service
        .delete_external_database(&id)
        .await
        .map_err(|e| format!("Failed to delete external database: {}", e))
}

#[tauri::command]
pub async fn test_external_database_connection(
    id: String,
    app_state: State<'_, AppState>,
) -> Result<String, String> {
    let result = app_state
        .sync_service
        .test_connection(&id)
        .await
        .map_err(|e| format!("Failed to test connection: {}", e))?;

    if result {
        Ok("Connection successful".to_string())
    } else {
        Err("Connection failed".to_string())
    }
}

#[tauri::command]
pub async fn connect_to_database(
    database_id: String,
    app_state: State<'_, AppState>,
) -> Result<(), String> {
    app_state
        .sync_service
        .connect(&database_id)
        .await
        .map_err(|e| format!("Failed to connect to database: {}", e))
}

#[tauri::command]
pub async fn disconnect_from_database(
    database_id: String,
    app_state: State<'_, AppState>,
) -> Result<(), String> {
    app_state
        .sync_service
        .disconnect(&database_id)
        .await
        .map_err(|e| format!("Failed to disconnect from database: {}", e))
}
