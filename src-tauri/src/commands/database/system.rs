use crate::database::service::DatabaseStats;
use crate::state::AppState;
use tauri::State;

use super::common::app_result;

// === System Commands ===

/// Get database statistics
#[tauri::command]
pub async fn get_database_stats(state: State<'_, AppState>) -> Result<DatabaseStats, String> {
    app_result!(state.sync_service.get_database_stats().await)
}