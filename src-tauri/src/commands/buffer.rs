use crate::error::AppError;
use crate::services::buffer_manager::BufferStats;
use crate::services::terminal::TerminalManager;
use tauri::State;

/// Get buffer as string for a terminal
#[tauri::command]
pub async fn get_terminal_buffer(
    terminal_id: String,
    terminal_manager: State<'_, TerminalManager>,
) -> Result<String, AppError> {
    let buffer_manager = terminal_manager.get_buffer_manager();
    let buffer_string = buffer_manager.get_buffer_string(&terminal_id).await;
    Ok(buffer_string.unwrap_or_default())
}

/// Check if terminal has buffer
#[tauri::command]
pub async fn has_terminal_buffer(
    terminal_id: String,
    terminal_manager: State<'_, TerminalManager>,
) -> Result<bool, AppError> {
    let buffer_manager = terminal_manager.get_buffer_manager();
    let has_buffer = buffer_manager.has_buffer(&terminal_id).await;
    Ok(has_buffer)
}

/// Get buffer statistics
#[tauri::command]
pub async fn get_buffer_stats(
    terminal_manager: State<'_, TerminalManager>,
) -> Result<BufferStats, AppError> {
    let buffer_manager = terminal_manager.get_buffer_manager();
    let stats = buffer_manager.get_stats().await;
    Ok(stats)
}

/// Cleanup orphaned buffers
#[tauri::command]
pub async fn cleanup_terminal_buffers(
    active_terminal_ids: Vec<String>,
    terminal_manager: State<'_, TerminalManager>,
) -> Result<(), AppError> {
    let buffer_manager = terminal_manager.get_buffer_manager();
    buffer_manager
        .cleanup_orphaned_buffers(&active_terminal_ids)
        .await;
    Ok(())
}
