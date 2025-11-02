use serde::{Deserialize, Serialize};

use crate::models::sftp::sync::SyncOperation;

/// Request for connecting to SFTP server
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ConnectSFTPRequest {
    pub profile_id: String,
}

/// Request for disconnecting SFTP session
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct DisconnectSFTPRequest {
    pub session_id: String,
}

/// Request for listing directory
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ListDirectoryRequest {
    pub session_id: String,
    pub path: String,
}

/// Request for getting file stat
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct StatRequest {
    pub session_id: String,
    pub path: String,
}

/// Request for creating directory
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct CreateDirectoryRequest {
    pub session_id: String,
    pub path: String,
}

/// Request for renaming file/directory
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct RenameRequest {
    pub session_id: String,
    pub old_path: String,
    pub new_path: String,
}

/// Request for deleting file/directory
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct DeleteRequest {
    pub session_id: String,
    pub path: String,
    pub recursive: bool,
}

/// Request for setting permissions
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct SetPermissionsRequest {
    pub session_id: String,
    pub path: String,
    pub mode: u32,
}

/// Request for creating symlink
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct CreateSymlinkRequest {
    pub session_id: String,
    pub target: String,
    pub link_path: String,
}

/// Request for reading symlink
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ReadSymlinkRequest {
    pub session_id: String,
    pub path: String,
}

/// Request for uploading file
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct UploadFileRequest {
    pub session_id: String,
    pub local_path: String,
    pub remote_path: String,
}

/// Request for downloading file
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct DownloadFileRequest {
    pub session_id: String,
    pub remote_path: String,
    pub local_path: String,
}

/// Request for getting transfer progress
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct GetTransferProgressRequest {
    pub transfer_id: String,
}

/// Request for canceling transfer
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct CancelTransferRequest {
    pub transfer_id: String,
}

/// Request for pausing transfer
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct PauseTransferRequest {
    pub transfer_id: String,
}

/// Request for resuming transfer
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ResumeTransferRequest {
    pub transfer_id: String,
}

/// Request for comparing directories
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct CompareDirectoriesRequest {
    pub session_id: String,
    pub local_path: String,
    pub remote_path: String,
}

/// Request for syncing directories
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct SyncDirectoriesRequest {
    pub session_id: String,
    pub operation: SyncOperation,
}

/// Request for reading file content
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ReadFileRequest {
    pub session_id: String,
    pub path: String,
}

/// Request for writing file content
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct WriteFileRequest {
    pub session_id: String,
    pub path: String,
    pub content: String,
}


