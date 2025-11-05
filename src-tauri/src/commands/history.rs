use crate::error::AppError;
use crate::models::history::{
    CommandHistoryEntry, GetTerminalHistoryRequest, SearchHistoryRequest,
    SearchHistoryResponse, ExportHistoryRequest,
};
use crate::state::AppState;
use tauri::State;

/// Get history for a terminal
#[tauri::command]
pub async fn get_terminal_history(
    request: GetTerminalHistoryRequest,
    app_state: State<'_, AppState>,
) -> Result<Vec<CommandHistoryEntry>, AppError> {
    app_state
        .history_manager
        .get_history(request)
        .await
}

/// Search history for a terminal
#[tauri::command]
pub async fn search_history(
    request: SearchHistoryRequest,
    app_state: State<'_, AppState>,
) -> Result<SearchHistoryResponse, AppError> {
    app_state
        .history_manager
        .search_history(request)
        .await
}

/// Export history to file
#[tauri::command]
pub async fn export_history(
    request: ExportHistoryRequest,
    app_state: State<'_, AppState>,
) -> Result<String, AppError> {
    app_state
        .history_manager
        .export_history(request)
        .await
}
