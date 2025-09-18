use crate::error::AppError;
use crate::models::terminal::{
    CreateTerminalRequest, CreateTerminalResponse, LocalConfig, ResizeTerminalRequest,
    TerminalConfig, TerminalInfo, TerminalType, WriteTerminalRequest,
};
use crate::state::AppState;
use tauri::{AppHandle, State};

/// Create a new local terminal
#[tauri::command]
pub async fn create_terminal(
    shell: Option<String>,
    working_dir: Option<String>,
    title: Option<String>,
    app_state: State<'_, AppState>,
    app_handle: AppHandle,
) -> Result<CreateTerminalResponse, AppError> {
    let config = TerminalConfig {
        terminal_type: TerminalType::Local,
        local_config: Some(LocalConfig {
            shell,
            working_dir,
            env_vars: None,
        }),
        ssh_profile_id: None,
    };

    let request = CreateTerminalRequest { config, title };
    app_state.terminal_manager
        .create_terminal(request, Some(app_handle))
        .await
}

/// Create a new SSH terminal using profile ID
#[tauri::command]
pub async fn create_ssh_terminal(
    profile_id: String,
    app_state: State<'_, AppState>,
    app_handle: AppHandle,
) -> Result<CreateTerminalResponse, AppError> {
    // Get SSH profile from database
    let ssh_profile = app_state.ssh_service
        .get_ssh_profile(&profile_id)
        .await
        .map_err(|e| AppError::Database(e.to_string()))?;

    // Create terminal config with SSH profile ID
    let config = TerminalConfig {
        terminal_type: TerminalType::SSH,
        local_config: None,
        ssh_profile_id: Some(profile_id),
    };

    let request = CreateTerminalRequest {
        config,
        title: Some(ssh_profile.display_name()),
    };

    app_state.terminal_manager
        .create_terminal(request, Some(app_handle))
        .await
}

/// Write data to a terminal
#[tauri::command]
pub async fn write_to_terminal(
    request: WriteTerminalRequest,
    app_state: State<'_, AppState>,
) -> Result<(), AppError> {
    app_state.terminal_manager.write_to_terminal(request).await
}

/// Resize a terminal
#[tauri::command]
pub async fn resize_terminal(
    request: ResizeTerminalRequest,
    app_state: State<'_, AppState>,
) -> Result<(), AppError> {
    app_state.terminal_manager.resize_terminal(request).await
}

/// Close a specific terminal
#[tauri::command]
pub async fn close_terminal(
    terminal_id: String,
    app_state: State<'_, AppState>,
) -> Result<(), AppError> {
    app_state.terminal_manager.close_terminal(terminal_id).await
}

/// Get information about a specific terminal
#[tauri::command]
pub async fn get_terminal_info(
    terminal_id: String,
    app_state: State<'_, AppState>,
) -> Result<TerminalInfo, AppError> {
    app_state.terminal_manager.get_terminal_info(terminal_id).await
}

/// List all active terminals
#[tauri::command]
pub async fn list_terminals(
    app_state: State<'_, AppState>,
) -> Result<Vec<TerminalInfo>, AppError> {
    app_state.terminal_manager.list_terminals().await
}

/// Close all terminals
#[tauri::command]
pub async fn close_all_terminals(
    app_state: State<'_, AppState>,
) -> Result<(), AppError> {
    app_state.terminal_manager.close_all_terminals().await
}
