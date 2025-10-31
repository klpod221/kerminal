use crate::models::sftp::error::SFTPError;
use crate::models::sftp::file_entry::FileEntry;
use crate::models::sftp::requests::{
    CancelTransferRequest, CompareDirectoriesRequest, ConnectSFTPRequest,
    CreateDirectoryRequest, CreateSymlinkRequest, DeleteRequest,
    DisconnectSFTPRequest, DownloadFileRequest, GetTransferProgressRequest,
    ListDirectoryRequest, ReadSymlinkRequest, RenameRequest,
    ResumeTransferRequest, SetPermissionsRequest, StatRequest,
    SyncDirectoriesRequest, UploadFileRequest,
};
use crate::models::sftp::sync::DiffEntry;
use crate::models::sftp::transfer::TransferProgress;
use crate::state::AppState;
use tauri::State;

/// Convert SFTPError to String for Tauri compatibility
impl From<SFTPError> for String {
    fn from(error: SFTPError) -> Self {
        error.to_string()
    }
}

/// Error conversion macro for SFTP operations
macro_rules! sftp_result {
    ($expr:expr) => {
        $expr.map_err(|e| -> String { e.to_string() })
    };
}

/// Connect to SFTP server using SSH profile
#[tauri::command]
pub async fn sftp_connect(
    state: State<'_, AppState>,
    request: ConnectSFTPRequest,
) -> Result<String, String> {
    sftp_result!(state.sftp_service.connect(request.profile_id).await)
}

/// Disconnect SFTP session
#[tauri::command]
pub async fn sftp_disconnect(
    state: State<'_, AppState>,
    request: DisconnectSFTPRequest,
) -> Result<(), String> {
    sftp_result!(state.sftp_service.disconnect(request.session_id).await)
}

/// List directory contents
#[tauri::command]
pub async fn sftp_list_directory(
    state: State<'_, AppState>,
    request: ListDirectoryRequest,
) -> Result<Vec<FileEntry>, String> {
    sftp_result!(state.sftp_service.list_directory(request.session_id, request.path).await)
}

/// Get file attributes (stat)
#[tauri::command]
pub async fn sftp_stat(
    state: State<'_, AppState>,
    request: StatRequest,
) -> Result<FileEntry, String> {
    sftp_result!(state.sftp_service.stat(request.session_id, request.path).await)
}

/// Create directory
#[tauri::command]
pub async fn sftp_create_directory(
    state: State<'_, AppState>,
    request: CreateDirectoryRequest,
) -> Result<(), String> {
    sftp_result!(state.sftp_service.create_directory(request.session_id, request.path).await)
}

/// Rename/move file or directory
#[tauri::command]
pub async fn sftp_rename(
    state: State<'_, AppState>,
    request: RenameRequest,
) -> Result<(), String> {
    sftp_result!(state.sftp_service.rename(request.session_id, request.old_path, request.new_path).await)
}

/// Delete file or directory
#[tauri::command]
pub async fn sftp_delete(
    state: State<'_, AppState>,
    request: DeleteRequest,
) -> Result<(), String> {
    sftp_result!(state.sftp_service.delete(request.session_id, request.path, request.recursive).await)
}

/// Set file permissions (chmod)
#[tauri::command]
pub async fn sftp_set_permissions(
    state: State<'_, AppState>,
    request: SetPermissionsRequest,
) -> Result<(), String> {
    sftp_result!(state.sftp_service.set_permissions(request.session_id, request.path, request.mode).await)
}

/// Create symlink
#[tauri::command]
pub async fn sftp_create_symlink(
    state: State<'_, AppState>,
    request: CreateSymlinkRequest,
) -> Result<(), String> {
    sftp_result!(state.sftp_service.create_symlink(request.session_id, request.target, request.link_path).await)
}

/// Read symlink target
#[tauri::command]
pub async fn sftp_read_symlink(
    state: State<'_, AppState>,
    request: ReadSymlinkRequest,
) -> Result<String, String> {
    sftp_result!(state.sftp_service.read_symlink(request.session_id, request.path).await)
}

/// Upload file from local to remote
#[tauri::command]
pub async fn sftp_upload_file(
    state: State<'_, AppState>,
    request: UploadFileRequest,
    app_handle: tauri::AppHandle,
) -> Result<String, String> {
    sftp_result!(
        state
            .sftp_transfer_manager
            .upload_file(request.session_id, request.local_path, request.remote_path, app_handle)
            .await
    )
}

/// Download file from remote to local
#[tauri::command]
pub async fn sftp_download_file(
    state: State<'_, AppState>,
    request: DownloadFileRequest,
    app_handle: tauri::AppHandle,
) -> Result<String, String> {
    sftp_result!(
        state
            .sftp_transfer_manager
            .download_file(request.session_id, request.remote_path, request.local_path, app_handle)
            .await
    )
}

/// Get transfer progress
#[tauri::command]
pub async fn sftp_get_transfer_progress(
    state: State<'_, AppState>,
    request: GetTransferProgressRequest,
) -> Result<TransferProgress, String> {
    sftp_result!(state.sftp_transfer_manager.get_progress(request.transfer_id).await)
}

/// Cancel transfer
#[tauri::command]
pub async fn sftp_cancel_transfer(
    state: State<'_, AppState>,
    request: CancelTransferRequest,
) -> Result<(), String> {
    sftp_result!(state.sftp_transfer_manager.cancel_transfer(request.transfer_id).await)
}

/// Resume interrupted transfer
#[tauri::command]
pub async fn sftp_resume_transfer(
    state: State<'_, AppState>,
    request: ResumeTransferRequest,
    app_handle: tauri::AppHandle,
) -> Result<(), String> {
    sftp_result!(
        state
            .sftp_transfer_manager
            .resume_transfer(request.transfer_id, app_handle)
            .await
    )
}

/// Compare local and remote directories
#[tauri::command]
pub async fn sftp_compare_directories(
    state: State<'_, AppState>,
    request: CompareDirectoriesRequest,
) -> Result<Vec<DiffEntry>, String> {
    sftp_result!(
        state
            .sftp_sync_service
            .compare_directories(request.session_id, request.local_path, request.remote_path)
            .await
    )
}

/// Synchronize directories
#[tauri::command]
pub async fn sftp_sync_directory(
    state: State<'_, AppState>,
    request: SyncDirectoriesRequest,
) -> Result<(), String> {
    sftp_result!(
        state
            .sftp_sync_service
            .sync_directories(request.session_id, request.operation)
            .await
    )
}

