// Copyright (C) 2026 Bùi Thanh Xuân (klpod221)
// SPDX-License-Identifier: GPL-3.0-or-later

use crate::error::AppError;
use crate::models::history::{
    CommandHistoryEntry, ExportHistoryRequest, GetTerminalHistoryRequest, SearchHistoryRequest,
    SearchHistoryResponse,
};
use crate::state::AppState;
use tauri::State;

/// Get history for a terminal
#[tauri::command]
pub async fn get_terminal_history(
    request: GetTerminalHistoryRequest,
    app_state: State<'_, AppState>,
    app_handle: tauri::AppHandle,
) -> Result<Vec<CommandHistoryEntry>, AppError> {
    let known_hosts_path = crate::commands::ssh_host_key::get_known_hosts_path(&app_handle);
    app_state.history_manager.get_history(request, known_hosts_path).await
}

/// Search history for a terminal
#[tauri::command]
pub async fn search_history(
    request: SearchHistoryRequest,
    app_state: State<'_, AppState>,
    app_handle: tauri::AppHandle,
) -> Result<SearchHistoryResponse, AppError> {
    let known_hosts_path = crate::commands::ssh_host_key::get_known_hosts_path(&app_handle);
    app_state.history_manager.search_history(request, known_hosts_path).await
}

/// Export history to file
#[tauri::command]
pub async fn export_history(
    request: ExportHistoryRequest,
    app_state: State<'_, AppState>,
    app_handle: tauri::AppHandle,
) -> Result<String, AppError> {
    let known_hosts_path = crate::commands::ssh_host_key::get_known_hosts_path(&app_handle);
    app_state.history_manager.export_history(request, known_hosts_path).await
}
