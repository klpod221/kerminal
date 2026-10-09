// Copyright (C) 2026 Bùi Thanh Xuân (klpod221)
// SPDX-License-Identifier: GPL-3.0-or-later

use russh_sftp::protocol::StatusCode;
use serde::{Deserialize, Serialize};
use thiserror::Error;

/// Classified SFTP error
#[derive(Debug, Error, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub enum SftpError {
    /// File or directory not found
    #[error("File not found: {path}")]
    FileNotFound { path: String },

    /// Permission denied on the given path
    #[error("Permission denied: {path}")]
    PermissionDenied { path: String },

    /// Destination already exists
    #[error("File already exists: {path}")]
    FileExists { path: String },

    /// Path is invalid
    #[error("Invalid path: {path}")]
    InvalidPath { path: String },

    /// SFTP session not found
    #[error("SFTP session not found: {session_id}")]
    SessionNotFound { session_id: String },

    /// Failed to establish or initialize SFTP session
    #[error("Failed to establish SFTP session: {message}")]
    SessionFailed { message: String },

    /// Connection to remote host was lost or timed out
    #[error("Connection lost: {message}")]
    ConnectionLost { message: String },

    /// Conflict during update or edit
    #[error("Conflict: {message}")]
    Conflict { message: String },

    /// Operation was cancelled
    #[error("Operation cancelled")]
    Cancelled,

    /// Transfer task not found
    #[error("Transfer not found: {transfer_id}")]
    TransferNotFound { transfer_id: String },

    /// Transfer is not in a state that can be resumed
    #[error("Transfer {transfer_id} is not in a resumable state")]
    TransferNotResumable { transfer_id: String },

    /// Local filesystem or I/O error
    #[error("I/O error: {message}")]
    IoError { message: String },

    /// Remote server error
    #[error("Remote server error: {message}")]
    RemoteError { message: String },

    /// Operation unsupported by server
    #[error("Unsupported operation: {message}")]
    Unsupported { message: String },

    /// Generic / uncategorized SFTP error
    #[error("SFTP error: {message}")]
    Other { message: String },
}

impl SftpError {
    /// Classify error from a russh_sftp status packet
    pub fn from_status(status_code: StatusCode, message: &str, path: Option<&str>) -> Self {
        let p = path.unwrap_or("").to_string();
        let msg_lower = message.to_lowercase();

        match status_code {
            StatusCode::NoSuchFile => SftpError::FileNotFound { path: p },
            StatusCode::PermissionDenied => SftpError::PermissionDenied { path: p },
            StatusCode::Failure => {
                if msg_lower.contains("file exists")
                    || msg_lower.contains("already exists")
                    || msg_lower.contains("directory not empty")
                {
                    SftpError::FileExists { path: p }
                } else if msg_lower.contains("permission denied") {
                    SftpError::PermissionDenied { path: p }
                } else if msg_lower.contains("no such file") || msg_lower.contains("not found") {
                    SftpError::FileNotFound { path: p }
                } else {
                    SftpError::Other {
                        message: if message.is_empty() {
                            format!("Remote failure on path: {}", p)
                        } else {
                            message.to_string()
                        },
                    }
                }
            }
            StatusCode::NoConnection | StatusCode::ConnectionLost => SftpError::ConnectionLost {
                message: if message.is_empty() {
                    "Connection to SFTP server lost".to_string()
                } else {
                    message.to_string()
                },
            },
            StatusCode::OpUnsupported => SftpError::Unsupported {
                message: if message.is_empty() {
                    "Operation unsupported by server".to_string()
                } else {
                    message.to_string()
                },
            },
            _ => SftpError::Other {
                message: format!("{}: {}", status_code, message),
            },
        }
    }

    /// Classify error from russh_sftp client error
    pub fn from_russh_sftp(err: russh_sftp::client::error::Error, path: Option<&str>) -> Self {
        match err {
            russh_sftp::client::error::Error::Status(status) => {
                Self::from_status(status.status_code, &status.error_message, path)
            }
            russh_sftp::client::error::Error::IO(msg) => SftpError::IoError { message: msg },
            russh_sftp::client::error::Error::Timeout => SftpError::ConnectionLost {
                message: "Request timed out".to_string(),
            },
            russh_sftp::client::error::Error::Limited(msg) => SftpError::Other {
                message: format!("Server limit exceeded: {}", msg),
            },
            russh_sftp::client::error::Error::UnexpectedBehavior(msg) => {
                let msg_lower = msg.to_lowercase();
                if msg_lower.contains("closed") || msg_lower.contains("connection") {
                    SftpError::ConnectionLost { message: msg }
                } else {
                    SftpError::Other { message: msg }
                }
            }
            russh_sftp::client::error::Error::UnexpectedPacket => SftpError::Other {
                message: "Unexpected packet received".to_string(),
            },
        }
    }

    /// Returns true if this error represents a missing file or directory
    pub fn is_not_found(&self) -> bool {
        matches!(self, SftpError::FileNotFound { .. })
    }

    /// Returns true if this error represents an already-existing destination
    pub fn is_already_exists(&self) -> bool {
        matches!(self, SftpError::FileExists { .. })
    }

    /// Returns true if this error represents a permission denial
    pub fn is_permission_denied(&self) -> bool {
        matches!(self, SftpError::PermissionDenied { .. })
    }

    /// Returns true if the connection was lost
    pub fn is_connection_lost(&self) -> bool {
        matches!(self, SftpError::ConnectionLost { .. })
    }
}

impl From<std::io::Error> for SftpError {
    fn from(err: std::io::Error) -> Self {
        SftpError::IoError {
            message: err.to_string(),
        }
    }
}

impl From<russh::Error> for SftpError {
    fn from(err: russh::Error) -> Self {
        SftpError::ConnectionLost {
            message: err.to_string(),
        }
    }
}

impl From<anyhow::Error> for SftpError {
    fn from(err: anyhow::Error) -> Self {
        SftpError::Other {
            message: err.to_string(),
        }
    }
}

impl From<SftpError> for String {
    fn from(err: SftpError) -> Self {
        err.to_string()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_error_classification_no_such_file() {
        let err = SftpError::from_status(StatusCode::NoSuchFile, "No such file", Some("/etc/shadow"));
        assert!(err.is_not_found());
        assert_eq!(
            err,
            SftpError::FileNotFound {
                path: "/etc/shadow".to_string()
            }
        );
    }

    #[test]
    fn test_error_classification_permission_denied() {
        let err = SftpError::from_status(StatusCode::PermissionDenied, "Permission denied", Some("/root"));
        assert!(err.is_permission_denied());
        assert_eq!(
            err,
            SftpError::PermissionDenied {
                path: "/root".to_string()
            }
        );
    }

    #[test]
    fn test_error_classification_failure_already_exists() {
        let err = SftpError::from_status(StatusCode::Failure, "Failure: file exists", Some("/tmp/dest"));
        assert!(err.is_already_exists());
        assert_eq!(
            err,
            SftpError::FileExists {
                path: "/tmp/dest".to_string()
            }
        );
    }

    #[test]
    fn test_error_classification_connection_lost() {
        let err = SftpError::from_status(StatusCode::ConnectionLost, "Connection dropped", None);
        assert!(err.is_connection_lost());
    }
}
