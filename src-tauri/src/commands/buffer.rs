use crate::error::AppError;
use crate::models::buffer::{
    CleanupTerminalBuffersRequest, GetTerminalBufferChunkRequest, GetTerminalBufferRequest,
    HasTerminalBufferRequest, TerminalBufferChunk,
};
use crate::services::buffer_manager::BufferStats;
use crate::state::AppState;
use tauri::State;

/// Get buffer as string for a terminal
#[tauri::command]
pub async fn get_terminal_buffer(
    request: GetTerminalBufferRequest,
    app_state: State<'_, AppState>,
) -> Result<String, AppError> {
    let buffer_manager = app_state.terminal_manager.get_buffer_manager();
    let buffer_string = buffer_manager.get_buffer_string(&request.terminal_id).await;
    Ok(buffer_string.unwrap_or_default())
}

/// Get buffer chunk for a terminal
#[tauri::command]
pub async fn get_terminal_buffer_chunk(
    request: GetTerminalBufferChunkRequest,
    app_state: State<'_, AppState>,
) -> Result<TerminalBufferChunk, AppError> {
    let buffer_manager = app_state.terminal_manager.get_buffer_manager();
    let chunk = buffer_manager
        .get_buffer_chunk(&request.terminal_id, request.start_line, request.chunk_size)
        .await;

    Ok(chunk)
}

/// Check if terminal has buffer
#[tauri::command]
pub async fn has_terminal_buffer(
    request: HasTerminalBufferRequest,
    app_state: State<'_, AppState>,
) -> Result<bool, AppError> {
    let buffer_manager = app_state.terminal_manager.get_buffer_manager();
    let has_buffer = buffer_manager.has_buffer(&request.terminal_id).await;
    Ok(has_buffer)
}

/// Get buffer statistics
#[tauri::command]
pub async fn get_buffer_stats(app_state: State<'_, AppState>) -> Result<BufferStats, AppError> {
    let buffer_manager = app_state.terminal_manager.get_buffer_manager();
    let stats = buffer_manager.get_stats().await;
    Ok(stats)
}

/// Cleanup orphaned buffers
#[tauri::command]
pub async fn cleanup_terminal_buffers(
    request: CleanupTerminalBuffersRequest,
    app_state: State<'_, AppState>,
) -> Result<(), AppError> {
    let buffer_manager = app_state.terminal_manager.get_buffer_manager();
    buffer_manager
        .cleanup_orphaned_buffers(&request.active_terminal_ids)
        .await;
    Ok(())
}
