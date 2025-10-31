use serde::{Deserialize, Serialize};
use thiserror::Error;

/// SFTP-specific errors
#[derive(Debug, Error, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub enum SFTPError {
    /// Session not found
    #[error("SFTP session not found: {session_id}")]
    SessionNotFound { session_id: String },
    
    /// Failed to establish SFTP session
    #[error("Failed to establish SFTP session: {message}")]
    SessionFailed { message: String },
    
    /// File or directory not found
    #[error("File not found: {path}")]
    FileNotFound { path: String },
    
    /// Permission denied
    #[error("Permission denied: {path}")]
    PermissionDenied { path: String },
    
    /// File already exists
    #[error("File already exists: {path}")]
    FileExists { path: String },
    
    /// Invalid path
    #[error("Invalid path: {path}")]
    InvalidPath { path: String },
    
    /// Transfer not found
    #[error("Transfer not found: {transfer_id}")]
    TransferNotFound { transfer_id: String },
    
    /// Transfer already in progress or completed
    #[error("Transfer {transfer_id} is not in a resumable state")]
    TransferNotResumable { transfer_id: String },
    
    /// I/O error
    #[error("I/O error: {message}")]
    IoError { message: String },
    
    /// Remote server error
    #[error("Remote server error: {message}")]
    RemoteError { message: String },
    
    /// Connection lost
    #[error("Connection lost: {message}")]
    ConnectionLost { message: String },
    
    /// Generic error
    #[error("SFTP error: {message}")]
    Other { message: String },
}

impl From<anyhow::Error> for SFTPError {
    fn from(err: anyhow::Error) -> Self {
        SFTPError::Other {
            message: err.to_string(),
        }
    }
}

impl From<std::io::Error> for SFTPError {
    fn from(err: std::io::Error) -> Self {
        SFTPError::IoError {
            message: err.to_string(),
        }
    }
}

