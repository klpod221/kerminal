use crate::models::recording::*;
use crate::services::recording::*;
use crate::state::AppState;
use log::warn;
use std::sync::Arc;
use tauri::{Emitter, Manager, State};

#[derive(Debug, serde::Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct StartRecordingRequest {
    terminal_id: String,
    session_name: Option<String>,
    width: Option<u16>,
    height: Option<u16>,
}

#[tauri::command]
pub async fn start_recording(
    request: StartRecordingRequest,
    state: State<'_, AppState>,
    app_handle: tauri::AppHandle,
) -> Result<String, String> {
    let recording_id = uuid::Uuid::new_v4().to_string();
    let name = request.session_name.unwrap_or_else(|| {
        format!(
            "Session {}",
            chrono::Local::now().format("%Y-%m-%d %H:%M:%S")
        )
    });

    // Create recordings directory
    let app_dir = app_handle
        .path()
        .app_data_dir()
        .map_err(|e| format!("Failed to get app data dir: {}", e))?;
    let recordings_dir = app_dir.join("recordings");
    tokio::fs::create_dir_all(&recordings_dir)
        .await
        .map_err(|e| format!("Failed to create recordings dir: {}", e))?;

    let file_path = recordings_dir.join(format!("{}.cast", recording_id));

    // Use provided dimensions or defaults
    let cols = request.width.unwrap_or(80);
    let rows = request.height.unwrap_or(24);

    // Determine terminal type - use default for now
    // In the future, we can get this from terminal info if needed
    let terminal_type = "Local".to_string();

    // Create recorder
    let recorder = SessionRecorder::new(
        recording_id.clone(),
        file_path.to_string_lossy().to_string(),
        cols,
        rows,
        Some(name.clone()),
        terminal_type,
    )
    .await
    .map_err(|e| e.to_string())?;

    recorder.write_header().await.map_err(|e| e.to_string())?;

    // Store recorder
    state
        .terminal_manager
        .recorders
        .write()
        .await
        .insert(request.terminal_id.clone(), Arc::new(recorder));

    Ok(recording_id)
}

#[derive(Debug, serde::Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct StopRecordingRequest {
    terminal_id: String,
}

#[tauri::command]
pub async fn stop_recording(
    request: StopRecordingRequest,
    state: State<'_, AppState>,
    app_handle: tauri::AppHandle,
) -> Result<SessionRecording, String> {
    let recorder = state
        .terminal_manager
        .recorders
        .write()
        .await
        .remove(&request.terminal_id)
        .ok_or_else(|| "No active recording for this terminal".to_string())?;

    recorder.stop().await.map_err(|e| e.to_string())?;

    // Get file info
    let file_size = tokio::fs::metadata(&recorder.file_path)
        .await
        .map(|m| m.len() as i64)
        .unwrap_or(0);

    let recording = SessionRecording {
        id: recorder.recording_id.clone(),
        terminal_id: request.terminal_id.clone(),
        session_name: recorder.session_name.clone(),
        terminal_type: recorder.terminal_type.clone(),
        started_at: recorder.started_at,
        ended_at: Some(chrono::Utc::now()),
        duration_ms: Some(recorder.get_duration_ms()),
        file_path: recorder.file_path.clone(),
        file_size,
        width: recorder.header.width,
        height: recorder.header.height,
        metadata: None,
        created_at: recorder.started_at,
    };

    // Save to database
    let db = state.database_service.lock().await;
    db.save_session_recording(&recording)
        .await
        .map_err(|e| format!("Failed to save recording to database: {}", e))?;

    // Emit realtime event
    let _ = app_handle.emit("recording_saved", &recording);

    Ok(recording)
}

#[tauri::command]
pub async fn list_recordings(state: State<'_, AppState>) -> Result<Vec<SessionRecording>, String> {
    let db = state.database_service.lock().await;
    db.list_session_recordings()
        .await
        .map_err(|e| format!("Failed to list recordings: {}", e))
}

#[derive(Debug, serde::Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct DeleteRecordingRequest {
    recording_id: String,
}

#[tauri::command]
pub async fn delete_recording(
    request: DeleteRecordingRequest,
    state: State<'_, AppState>,
    app_handle: tauri::AppHandle,
) -> Result<(), String> {
    // Get recording to find file path
    let db = state.database_service.lock().await;
    if let Some(recording) = db
        .get_session_recording(&request.recording_id)
        .await
        .map_err(|e| format!("Failed to get recording: {}", e))?
    {
        // Delete file
        if let Err(e) = tokio::fs::remove_file(&recording.file_path).await {
            warn!("Warning: Failed to delete recording file: {}", e);
        }
    }

    // Delete from database
    db.delete_session_recording(&request.recording_id)
        .await
        .map_err(|e| format!("Failed to delete recording from database: {}", e))?;

    // Emit realtime event
    let _ = app_handle.emit(
        "recording_deleted",
        &serde_json::json!({ "id": request.recording_id }),
    );

    Ok(())
}

#[derive(Debug, serde::Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ExportRecordingRequest {
    recording_id: String,
    export_path: String,
}

#[tauri::command]
pub async fn export_recording(
    request: ExportRecordingRequest,
    state: State<'_, AppState>,
) -> Result<String, String> {
    // Get recording to find source file
    let db = state.database_service.lock().await;
    let recording = db
        .get_session_recording(&request.recording_id)
        .await
        .map_err(|e| format!("Failed to get recording: {}", e))?
        .ok_or_else(|| format!("Recording not found: {}", request.recording_id))?;

    // Copy file to export path
    tokio::fs::copy(&recording.file_path, &request.export_path)
        .await
        .map_err(|e| format!("Failed to copy recording file: {}", e))?;

    Ok(request.export_path)
}

#[derive(Debug, serde::Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ReadCastFileRequest {
    file_path: String,
}

#[tauri::command]
pub async fn read_cast_file(request: ReadCastFileRequest) -> Result<String, String> {
    tokio::fs::read_to_string(&request.file_path)
        .await
        .map_err(|e| format!("Failed to read file: {}", e))
}
