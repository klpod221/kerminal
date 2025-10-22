use crate::core::auth_session_manager::SessionLockReason;
use crate::state::AppState;
use chrono::Utc;
use serde_json::json;
use tauri::State;

/// Manually trigger auth session unlock notification (called after successful unlock)
#[tauri::command]
pub async fn notify_session_unlocked(state: State<'_, AppState>) -> Result<(), String> {
    let auth_session_manager = state.auth_session_manager.lock().await;
    auth_session_manager
        .on_session_unlocked()
        .await
        .map_err(|e| e.to_string())?;

    Ok(())
}

/// Manually trigger auth session lock notification (called after manual lock)
#[tauri::command]
pub async fn notify_session_locked(
    state: State<'_, AppState>,
    reason: Option<String>,
) -> Result<(), String> {
    let lock_reason = match reason.as_deref() {
        Some("manual") => SessionLockReason::Manual,
        Some("timeout") => SessionLockReason::Timeout,
        Some(other) => SessionLockReason::Error(other.to_string()),
        None => SessionLockReason::Manual,
    };

    let auth_session_manager = state.auth_session_manager.lock().await;
    auth_session_manager
        .on_session_locked(lock_reason)
        .await
        .map_err(|e| e.to_string())?;

    Ok(())
}

/// Get current auth session status
#[tauri::command]
pub async fn get_auth_session_status(
    state: State<'_, AppState>,
) -> Result<serde_json::Value, String> {
    let database_service = state.database_service.lock().await;
    let status = database_service
        .get_master_password_status()
        .await
        .map_err(|e| e.to_string())?;

    Ok(json!({
        "isSetup": status.is_setup,
        "isUnlocked": status.is_unlocked,
        "autoUnlockEnabled": status.auto_unlock_enabled,
        "keychainAvailable": status.keychain_available,
        "sessionActive": status.session_active,
        "sessionExpiresAt": status.session_expires_at,
        "loadedDeviceCount": status.loaded_device_count,
        "timestamp": Utc::now()
    }))
}
