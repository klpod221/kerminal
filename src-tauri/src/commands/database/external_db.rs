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

async fn test_external_connection(
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
        DatabaseType::MySQL => {
            let conn_str = format!(
                "mysql://{}:{}@{}:{}/{}",
                _connection_details.username,
                _connection_details.password,
                _connection_details.host,
                _connection_details.port,
                _connection_details.database
            );
            match sqlx::MySqlPool::connect(&conn_str).await {
                Ok(_) => Ok("Connection successful".to_string()),
                Err(e) => Err(format!("MySQL connection failed: {}", e)),
            }
        }
        DatabaseType::PostgreSQL => {
            let conn_str = format!(
                "postgresql://{}:{}@{}:{}/{}",
                _connection_details.username,
                _connection_details.password,
                _connection_details.host,
                _connection_details.port,
                _connection_details.database
            );
            match sqlx::PgPool::connect(&conn_str).await {
                Ok(_) => Ok("Connection successful".to_string()),
                Err(e) => Err(format!("PostgreSQL connection failed: {}", e)),
            }
        }
        DatabaseType::MongoDB => {
            let conn_str = format!(
                "mongodb://{}:{}@{}:{}/{}",
                _connection_details.username,
                _connection_details.password,
                _connection_details.host,
                _connection_details.port,
                _connection_details.database
            );
            match mongodb::Client::with_uri_str(&conn_str).await {
                Ok(client) => {
                    match client.list_database_names(None, None).await {
                        Ok(_) => Ok("Connection successful".to_string()),
                        Err(e) => Err(format!("MongoDB connection failed: {}", e)),
                    }
                }
                Err(e) => Err(format!("MongoDB connection failed: {}", e)),
            }
        }
    }
}

#[tauri::command]
pub async fn test_external_database_connection(
    id: String,
    app_state: State<'_, AppState>,
) -> Result<String, String> {
    test_external_connection(id, app_state).await
}

#[tauri::command]
pub async fn connect_to_database(
    database_id: String,
    app_state: State<'_, AppState>,
) -> Result<(), String> {
    let database_service = app_state.database_service.lock().await;

    database_service
        .toggle_external_database_active(&database_id, true)
        .await
        .map_err(|e| format!("Failed to connect to database: {}", e))
}

#[tauri::command]
pub async fn disconnect_from_database(
    database_id: String,
    app_state: State<'_, AppState>,
) -> Result<(), String> {
    let database_service = app_state.database_service.lock().await;

    database_service
        .toggle_external_database_active(&database_id, false)
        .await
        .map_err(|e| format!("Failed to disconnect from database: {}", e))
}
