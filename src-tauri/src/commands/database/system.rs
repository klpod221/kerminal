use crate::database::service::DatabaseStats;
use crate::state::AppState;
use tauri::State;

use super::common::app_result;

// === System Commands ===

/// Get database statistics
#[tauri::command]
pub async fn get_database_stats(state: State<'_, AppState>) -> Result<DatabaseStats, String> {
    let db_service = state.database_service.lock().await;
    app_result!(db_service.get_database_stats().await)
}
