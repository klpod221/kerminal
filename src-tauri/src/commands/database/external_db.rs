use tauri::State;

use crate::database::encryption::ExternalDbEncryptor;
use crate::database::providers::{MongoDBProvider, MySQLProvider, PostgreSQLProvider};
use crate::database::traits::Database;
use crate::models::sync::{
    AddExternalDatabaseRequest, ConflictResolutionStrategy, DatabaseType,
    ExternalDatabaseConfig, ExternalDatabaseWithDetails, SyncSettings, TestConnectionRequest,
    UpdateExternalDatabaseRequest,
};
use crate::state::AppState;

#[tauri::command]
pub async fn add_external_database(
    request: AddExternalDatabaseRequest,
    app_state: State<'_, AppState>,
) -> Result<ExternalDatabaseConfig, String> {
    let database_service = app_state.database_service.lock().await;
    let device_id = database_service.get_device_id().to_string();

    let master_password_manager = database_service.get_master_password_manager_arc();
    let encryptor = ExternalDbEncryptor::new(master_password_manager);

    let encrypted_connection_details = encryptor
        .encrypt_connection_details(&request.connection_details)
        .await
        .map_err(|e| format!("Failed to encrypt connection details: {}", e))?;

    let sync_settings = crate::models::sync::SyncSettings {
        auto_sync: request.auto_sync,
        sync_interval_minutes: request.sync_interval_minutes,
        conflict_resolution_strategy: request
            .conflict_resolution_strategy
            .parse()
            .map_err(|e| format!("Invalid conflict resolution strategy: {}", e))?,
    };

    let encrypted_sync_settings = encryptor
        .encrypt_sync_settings(&sync_settings)
        .await
        .map_err(|e| format!("Failed to encrypt sync settings: {}", e))?;

    let config = ExternalDatabaseConfig::new(
        device_id,
        request.name,
        request.db_type,
        encrypted_connection_details,
        encrypted_sync_settings,
        request.auto_sync,
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

    let mut databases = database_service
        .find_all_external_databases()
        .await
        .map_err(|e| format!("Failed to retrieve external databases: {}", e))?;

    // Update isActive status based on actual connections
    for db in &mut databases {
        db.is_active = app_state.sync_service.is_connected(&db.base.id).await;
    }

    Ok(databases)
}

#[tauri::command]
pub async fn get_external_database_with_details(
    id: String,
    app_state: State<'_, AppState>,
) -> Result<ExternalDatabaseWithDetails, String> {
    let database_service = app_state.database_service.lock().await;

    let config = database_service
        .find_external_database_by_id(&id)
        .await
        .map_err(|e| format!("Failed to get external database: {}", e))?
        .ok_or_else(|| "External database not found".to_string())?;

    let master_password_manager = database_service.get_master_password_manager_arc();
    let encryptor = ExternalDbEncryptor::new(master_password_manager);

    let connection_details = encryptor
        .decrypt_connection_details(&config.connection_details_encrypted)
        .await
        .map_err(|e| format!("Failed to decrypt connection details: {}", e))?;

    Ok(ExternalDatabaseWithDetails {
        config,
        connection_details,
    })
}

#[tauri::command]
pub async fn update_external_database(
    request: UpdateExternalDatabaseRequest,
    app_state: State<'_, AppState>,
) -> Result<ExternalDatabaseConfig, String> {
    let database_service = app_state.database_service.lock().await;

    let mut config = database_service
        .find_external_database_by_id(&request.id)
        .await
        .map_err(|e| format!("Failed to find external database: {}", e))?
        .ok_or_else(|| "External database not found".to_string())?;

    if let Some(name) = request.name {
        config.name = name;
    }

    if request.connection_details.is_some()
        || request.auto_sync.is_some()
        || request.sync_interval_minutes.is_some()
        || request.conflict_resolution_strategy.is_some()
    {
        let master_password_manager = database_service.get_master_password_manager_arc();
        let encryptor = ExternalDbEncryptor::new(master_password_manager);

        if let Some(connection_details) = request.connection_details {
            let encrypted = encryptor
                .encrypt_connection_details(&connection_details)
                .await
                .map_err(|e| format!("Failed to encrypt connection details: {}", e))?;
            config.connection_details_encrypted = encrypted;
        }

        // Try to parse existing settings, or use defaults if empty/invalid
        let mut sync_settings = config.parse_sync_settings().unwrap_or_else(|_| {
            SyncSettings {
                auto_sync: false,
                sync_interval_minutes: 15,
                conflict_resolution_strategy: ConflictResolutionStrategy::LastWriteWins,
            }
        });

        if let Some(auto_sync) = request.auto_sync {
            sync_settings.auto_sync = auto_sync;
        }
        if let Some(interval) = request.sync_interval_minutes {
            sync_settings.sync_interval_minutes = interval;
        }
        if let Some(strategy) = request.conflict_resolution_strategy {
            sync_settings.conflict_resolution_strategy = strategy
                .parse()
                .map_err(|e| format!("Invalid conflict resolution strategy: {}", e))?;
        }

        let encrypted_settings = encryptor
            .encrypt_sync_settings(&sync_settings)
            .await
            .map_err(|e| format!("Failed to encrypt sync settings: {}", e))?;
        config.sync_settings = encrypted_settings;

        // Update denormalized field for fast queries
        config.auto_sync_enabled = sync_settings.auto_sync;
    }

    config.base.touch();

    database_service
        .save_external_database(&config)
        .await
        .map_err(|e| format!("Failed to update external database: {}", e))?;

    Ok(config)
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
    request: TestConnectionRequest,
    app_state: State<'_, AppState>,
) -> Result<bool, String> {
    let database_service = app_state.database_service.lock().await;

    // If password is empty and database_id is provided, fetch existing credentials
    let connection_details = if request.connection_details.password.is_empty()
        && request.database_id.is_some()
    {
        let db_id = request.database_id.as_ref().unwrap();
        let config = database_service
            .find_external_database_by_id(db_id)
            .await
            .map_err(|e| format!("Failed to get existing database: {}", e))?
            .ok_or_else(|| format!("Database not found: {}", db_id))?;

        let master_password_manager = database_service.get_master_password_manager_arc();
        let encryptor = ExternalDbEncryptor::new(master_password_manager);

        encryptor
            .decrypt_connection_details(&config.connection_details_encrypted)
            .await
            .map_err(|e| format!("Failed to decrypt existing connection details: {}", e))?
    } else {
        request.connection_details
    };

    let connection_string = connection_details.to_connection_string(&request.db_type);

    let result = match request.db_type {
        DatabaseType::MySQL => {
            let mut provider = MySQLProvider::new(connection_string);
            provider
                .connect()
                .await
                .map_err(|e| format!("MySQL connection failed: {}", e))?;
            provider
                .test_connection()
                .await
                .map_err(|e| format!("MySQL test failed: {}", e))?;
            Ok(())
        }
        DatabaseType::PostgreSQL => {
            let mut provider = PostgreSQLProvider::new(connection_string);
            provider
                .connect()
                .await
                .map_err(|e| format!("PostgreSQL connection failed: {}", e))?;
            provider
                .test_connection()
                .await
                .map_err(|e| format!("PostgreSQL test failed: {}", e))?;
            Ok(())
        }
        DatabaseType::MongoDB => {
            let database_name = connection_details.database.clone();
            let mut provider = MongoDBProvider::new(connection_string, database_name);
            provider
                .connect()
                .await
                .map_err(|e| format!("MongoDB connection failed: {}", e))?;
            provider
                .test_connection()
                .await
                .map_err(|e| format!("MongoDB test failed: {}", e))?;
            Ok(())
        }
    };

    result.map(|_| true)
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
