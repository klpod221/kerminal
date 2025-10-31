use crate::models::ssh::{
    CreateSSHTunnelRequest, SSHTunnel, TunnelStatus, TunnelWithStatus, UpdateSSHTunnelRequest,
};
use crate::state::AppState;
use tauri::{Emitter, State};

use crate::commands::database::common::app_result;

/// Macro for tunnel service results that return String errors
macro_rules! tunnel_result {
    ($expr:expr) => {
        $expr
    };
}

/// Create new SSH tunnel
#[tauri::command]
pub async fn create_tunnel(
    state: State<'_, AppState>,
    request: CreateSSHTunnelRequest,
    app_handle: tauri::AppHandle,
) -> Result<SSHTunnel, String> {
    let tunnel = app_result!(state.tunnel_service.create_tunnel(request).await)?;
    let tunnel_with_status = app_result!(state.tunnel_service.get_tunnel_with_status(&tunnel.base.id).await)?;
    let _ = app_handle.emit("tunnel_created", &tunnel_with_status);
    Ok(tunnel)
}

/// Get all SSH tunnels with status
#[tauri::command]
pub async fn get_tunnels(state: State<'_, AppState>) -> Result<Vec<TunnelWithStatus>, String> {
    app_result!(state.tunnel_service.get_all_tunnels_with_status().await)
}

/// Get SSH tunnel by ID with status
#[tauri::command]
pub async fn get_tunnel(
    state: State<'_, AppState>,
    id: String,
) -> Result<TunnelWithStatus, String> {
    app_result!(state.tunnel_service.get_tunnel_with_status(&id).await)
}

/// Update SSH tunnel
#[tauri::command]
pub async fn update_tunnel(
    state: State<'_, AppState>,
    id: String,
    request: UpdateSSHTunnelRequest,
    app_handle: tauri::AppHandle,
) -> Result<SSHTunnel, String> {
    let tunnel = app_result!(state.tunnel_service.update_tunnel(&id, request).await)?;
    let tunnel_with_status = app_result!(state.tunnel_service.get_tunnel_with_status(&tunnel.base.id).await)?;
    let _ = app_handle.emit("tunnel_updated", &tunnel_with_status);
    Ok(tunnel)
}

/// Delete SSH tunnel
#[tauri::command]
pub async fn delete_tunnel(
    state: State<'_, AppState>,
    id: String,
    app_handle: tauri::AppHandle,
) -> Result<(), String> {
    app_result!(state.tunnel_service.delete_tunnel(&id).await)?;
    let _ = app_handle.emit("tunnel_deleted", &serde_json::json!({ "id": id }));
    Ok(())
}

/// Start SSH tunnel
#[tauri::command]
pub async fn start_tunnel(
    state: State<'_, AppState>,
    id: String,
    app_handle: tauri::AppHandle,
) -> Result<(), String> {
    tunnel_result!(state.tunnel_service.start_tunnel(id.clone()).await)?;
    let tunnel_with_status = app_result!(state.tunnel_service.get_tunnel_with_status(&id).await)?;
    let _ = app_handle.emit("tunnel_started", &tunnel_with_status);
    Ok(())
}

/// Stop SSH tunnel
#[tauri::command]
pub async fn stop_tunnel(
    state: State<'_, AppState>,
    id: String,
    app_handle: tauri::AppHandle,
) -> Result<(), String> {
    tunnel_result!(state.tunnel_service.stop_tunnel(id.clone()).await)?;
    let tunnel_with_status = app_result!(state.tunnel_service.get_tunnel_with_status(&id).await)?;
    let _ = app_handle.emit("tunnel_stopped", &tunnel_with_status);
    Ok(())
}

/// Get tunnel status
#[tauri::command]
pub async fn get_tunnel_status(
    state: State<'_, AppState>,
    id: String,
) -> Result<TunnelStatus, String> {
    tunnel_result!(state.tunnel_service.get_tunnel_status(id).await)
}
