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
pub async fn get_external_database(
    id: String,
    app_state: State<'_, AppState>,
) -> Result<Option<ExternalDatabaseConfig>, String> {
    let database_service = app_state.database_service.lock().await;

    database_service
        .find_external_database_by_id(&id)
        .await
        .map_err(|e| format!("Failed to retrieve external database: {}", e))
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
pub async fn toggle_database_active(
    id: String,
    is_active: bool,
    app_state: State<'_, AppState>,
) -> Result<(), String> {
    let database_service = app_state.database_service.lock().await;

    database_service
        .toggle_external_database_active(&id, is_active)
        .await
        .map_err(|e| format!("Failed to toggle database active status: {}", e))
}

#[tauri::command]
pub async fn test_external_connection(
    id: String,
    app_state: State<'_, AppState>,
) -> Result<String, String> {
    let database_service = app_state.database_service.lock().await;

    let config = database_service
        .find_external_database_by_id(&id)
        .await
        .map_err(|e| format!("Failed to find database config: {}", e))?
        .ok_or_else(|| "Database configuration not found".to_string())?;

    let master_password_manager = database_service.get_master_password_manager_arc();
    let encryptor = ExternalDbEncryptor::new(master_password_manager);

    let _connection_details = encryptor
        .decrypt_connection_details(&config.connection_details_encrypted)
        .await
        .map_err(|e| format!("Failed to decrypt connection details: {}", e))?;

    match config.db_type {
        DatabaseType::MySQL => Err("MySQL connection testing not yet implemented".to_string()),
        DatabaseType::PostgreSQL => {
            Err("PostgreSQL connection testing not yet implemented".to_string())
        }
        DatabaseType::MongoDB => Err("MongoDB connection testing not yet implemented".to_string()),
    }
}

#[tauri::command]
pub async fn decrypt_connection_details(
    id: String,
    app_state: State<'_, AppState>,
) -> Result<ConnectionDetails, String> {
    let database_service = app_state.database_service.lock().await;

    let config = database_service
        .find_external_database_by_id(&id)
        .await
        .map_err(|e| format!("Failed to find database config: {}", e))?
        .ok_or_else(|| "Database configuration not found".to_string())?;

    let master_password_manager = database_service.get_master_password_manager_arc();
    let encryptor = ExternalDbEncryptor::new(master_password_manager);

    encryptor
        .decrypt_connection_details(&config.connection_details_encrypted)
        .await
        .map_err(|e| format!("Failed to decrypt connection details: {}", e))
}
