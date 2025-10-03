use crate::database::encryption::master_password::SetupMasterPasswordRequest;
use crate::models::auth::{VerifyMasterPasswordRequest, ChangeMasterPasswordRequest};
use crate::state::AppState;
use tauri::{State, AppHandle};

use super::common::app_result;

/// Setup master password for first time
#[tauri::command]
pub async fn setup_master_password(
    app: AppHandle,
    state: State<'_, AppState>,
    request: SetupMasterPasswordRequest,
) -> Result<(), String> {
    let auth_service = &state.auth_service;

    // Setup master password first
    let result = auth_service.setup_master_password(request).await;

    match result {
        Ok(()) => {
            // If setup successful, restart the app after a small delay
            let app_clone = app.clone();
            tokio::spawn(async move {
                tokio::time::sleep(tokio::time::Duration::from_millis(500)).await;
                let _ = app_clone.restart();
            });
            Ok(())
        }
        Err(e) => Err(e.to_string())
    }
}

/// Verify master password
#[tauri::command]
pub async fn verify_master_password(
    state: State<'_, AppState>,
    request: serde_json::Value,
) -> Result<bool, String> {
    let req: VerifyMasterPasswordRequest = serde_json::from_value(request.get("request").unwrap_or(&request).clone())
        .map_err(|e| format!("Invalid request format: {}", e))?;

    match state.auth_service.verify_master_password(req.password).await {
        Ok(()) => Ok(true),
        Err(e) => Err(e.to_string()),
    }
}

/// Try auto unlock with keychain
#[tauri::command]
pub async fn try_auto_unlock(state: State<'_, AppState>) -> Result<bool, String> {
    match state.auth_service.try_auto_unlock().await {
        Ok(success) => Ok(success),
        Err(e) => Err(e.to_string())
    }
}/// Lock current session
#[tauri::command]
pub async fn lock_session(state: State<'_, AppState>) -> Result<(), String> {
    state.auth_service.lock_session().await;
    Ok(())
}

/// Get master password status
#[tauri::command]
pub async fn get_master_password_status(
    state: State<'_, AppState>,
) -> Result<serde_json::Value, String> {
    app_result!(state.auth_service.get_master_password_status().await)
}

/// Check if session is valid
#[tauri::command]
pub async fn is_session_valid(state: State<'_, AppState>) -> Result<bool, String> {
    let database_service = state.database_service.clone();
    let db_guard = database_service.lock().await;
    app_result!(db_guard.is_session_valid().await)
}

/// Get master password configuration
#[tauri::command]
pub async fn get_master_password_config(
    state: State<'_, AppState>,
) -> Result<serde_json::Value, String> {
    app_result!(state.auth_service.get_master_password_config().await)
}

/// Get current device information
#[tauri::command]
pub async fn get_current_device(
    state: State<'_, AppState>,
) -> Result<serde_json::Value, String> {
    app_result!(state.auth_service.get_current_device().await)
}

/// Change master password
#[tauri::command]
pub async fn change_master_password(
    state: State<'_, AppState>,
    request: serde_json::Value,
) -> Result<(), String> {
    let req: ChangeMasterPasswordRequest = serde_json::from_value(request.get("request").unwrap_or(&request).clone())
        .map_err(|e| format!("Invalid request format: {}", e))?;

    match state.auth_service.change_master_password(req.old_password, req.new_password).await {
        Ok(()) => Ok(()),
        Err(e) => {
            match e {
                crate::database::error::DatabaseError::AuthenticationFailed(_) => {
                    Err("Current password is incorrect".to_string())
                }
                crate::database::error::DatabaseError::MasterPasswordRequired => {
                    Err("Master password is not set up".to_string())
                }
                crate::database::error::DatabaseError::ValidationError(msg) => {
                    Err(msg)
                }
                _ => Err(format!("Failed to change password: {}", e))
            }
        }
    }
}

/// Reset master password (dangerous operation)
#[tauri::command]
pub async fn reset_master_password(state: State<'_, AppState>) -> Result<(), String> {
    match state.auth_service.reset_master_password().await {
        Ok(()) => Ok(()),
        Err(e) => {
            match e {
                crate::database::error::DatabaseError::MasterPasswordRequired => {
                    Err("Master password is not set up".to_string())
                }
                _ => Err(format!("Failed to reset master password: {}", e))
            }
        }
    }
}

/// Update master password configuration
#[tauri::command]
pub async fn update_master_password_config(
    state: State<'_, AppState>,
    request: serde_json::Value,
) -> Result<(), String> {
    let config = request.get("request").unwrap_or(&request);

    let auto_unlock = config
        .get("autoUnlock")
        .and_then(|v| v.as_bool())
        .unwrap_or(false);

    let auto_lock_timeout = config
        .get("autoLockTimeout")
        .and_then(|v| v.as_u64())
        .map(|v| v as u32);

    let password = config
        .get("password")
        .and_then(|v| v.as_str())
        .map(|s| s.to_string());

    // If password is provided, use the keychain-aware method
    if password.is_some() {
        app_result!(state.auth_service.update_master_password_config_with_keychain(auto_unlock, auto_lock_timeout, password).await)
    } else {
        app_result!(state.auth_service.update_master_password_config(auto_unlock, auto_lock_timeout).await)
    }
}
