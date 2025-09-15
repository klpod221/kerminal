use crate::models::terminal::{
    CreateTerminalRequest, CreateTerminalResponse, ResizeTerminalRequest,
    TerminalInfo, WriteTerminalRequest
};
use crate::services::terminal::TerminalManager;
use crate::error::AppError;
use tauri::{State, AppHandle};/// Create a new terminal (local or SSH)
#[tauri::command]
pub async fn create_terminal(
    request: CreateTerminalRequest,
    terminal_manager: State<'_, TerminalManager>,
    app_handle: AppHandle,
) -> Result<CreateTerminalResponse, AppError> {
    terminal_manager.create_terminal(request, Some(app_handle)).await
}

/// Write data to a terminal
#[tauri::command]
pub async fn write_to_terminal(
    request: WriteTerminalRequest,
    terminal_manager: State<'_, TerminalManager>,
) -> Result<(), AppError> {
    terminal_manager.write_to_terminal(request).await
}

/// Resize a terminal
#[tauri::command]
pub async fn resize_terminal(
    request: ResizeTerminalRequest,
    terminal_manager: State<'_, TerminalManager>,
) -> Result<(), AppError> {
    terminal_manager.resize_terminal(request).await
}

/// Close a specific terminal
#[tauri::command]
pub async fn close_terminal(
    terminal_id: String,
    terminal_manager: State<'_, TerminalManager>,
) -> Result<(), AppError> {
    terminal_manager.close_terminal(terminal_id).await
}

/// Get information about a specific terminal
#[tauri::command]
pub async fn get_terminal_info(
    terminal_id: String,
    terminal_manager: State<'_, TerminalManager>,
) -> Result<TerminalInfo, AppError> {
    terminal_manager.get_terminal_info(terminal_id).await
}

/// List all active terminals
#[tauri::command]
pub async fn list_terminals(
    terminal_manager: State<'_, TerminalManager>,
) -> Result<Vec<TerminalInfo>, AppError> {
    terminal_manager.list_terminals().await
}

/// Close all terminals
#[tauri::command]
pub async fn close_all_terminals(
    terminal_manager: State<'_, TerminalManager>,
) -> Result<(), AppError> {
    terminal_manager.close_all_terminals().await
}
