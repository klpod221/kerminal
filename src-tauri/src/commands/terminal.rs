use crate::error::AppError;
use crate::models::terminal::{
    CloseTerminalRequest, CreateLocalTerminalRequest, CreateSshTerminalRequest,
    CreateTerminalRequest, CreateTerminalResponse, GetTerminalInfoRequest, LocalConfig,
    ResizeTerminalRequest, TerminalConfig, TerminalInfo, TerminalType, WriteBatchTerminalRequest,
    WriteTerminalRequest,
};
use crate::state::AppState;
use tauri::{AppHandle, State};

/// Create a new local terminal
#[tauri::command]
pub async fn create_terminal(
    request: CreateLocalTerminalRequest,
    app_state: State<'_, AppState>,
    app_handle: AppHandle,
) -> Result<CreateTerminalResponse, AppError> {
    let config = TerminalConfig {
        terminal_type: TerminalType::Local,
        local_config: Some(LocalConfig {
            shell: request.shell,
            working_dir: request.working_dir,
            env_vars: None,
        }),
        ssh_profile_id: None,
    };

    let create_request = CreateTerminalRequest {
        config,
        title: request.title,
    };
    app_state
        .terminal_manager
        .create_terminal(create_request, Some(app_handle))
        .await
}

/// Create a new SSH terminal using profile ID
#[tauri::command]
pub async fn create_ssh_terminal(
    request: CreateSshTerminalRequest,
    app_state: State<'_, AppState>,
    app_handle: AppHandle,
) -> Result<CreateTerminalResponse, AppError> {
    let ssh_profile = app_state
        .ssh_service
        .get_ssh_profile(&request.profile_id)
        .await
        .map_err(|e| AppError::Database(e.to_string()))?;

    let config = TerminalConfig {
        terminal_type: TerminalType::SSH,
        local_config: None,
        ssh_profile_id: Some(request.profile_id),
    };

    let terminal_request = CreateTerminalRequest {
        config,
        title: Some(ssh_profile.display_name()),
    };

    app_state
        .terminal_manager
        .create_terminal(terminal_request, Some(app_handle))
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

/// Write data to multiple terminals in batch
#[tauri::command]
pub async fn write_batch_to_terminal(
    request: WriteBatchTerminalRequest,
    app_state: State<'_, AppState>,
) -> Result<(), AppError> {
    for write_request in request.requests {
        app_state
            .terminal_manager
            .write_to_terminal(write_request)
            .await?;
    }
    Ok(())
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
    request: CloseTerminalRequest,
    app_state: State<'_, AppState>,
) -> Result<(), AppError> {
    app_state
        .terminal_manager
        .close_terminal(request.terminal_id)
        .await
}

/// Get information about a specific terminal
#[tauri::command]
pub async fn get_terminal_info(
    request: GetTerminalInfoRequest,
    app_state: State<'_, AppState>,
) -> Result<TerminalInfo, AppError> {
    app_state
        .terminal_manager
        .get_terminal_info(request.terminal_id)
        .await
}

/// List all active terminals
#[tauri::command]
pub async fn list_terminals(app_state: State<'_, AppState>) -> Result<Vec<TerminalInfo>, AppError> {
    app_state.terminal_manager.list_terminals().await
}
