use crate::database::error::{DatabaseError, EncryptionError, SSHError};
use serde::{Deserialize, Serialize};
use std::fmt;

/// Unified application error type for frontend communication
#[derive(Debug, Serialize, Deserialize)]
pub enum AppError {
    /// Database-related errors
    Database(String),
    /// Encryption-related errors
    Encryption(String),
    /// SSH-related errors
    SSH(String),
    /// Terminal-related errors
    Terminal(String),
    /// Configuration errors
    Config(String),
    /// Authentication/Authorization errors
    Auth(String),
    /// Network/Connection errors
    Network(String),
    /// Validation errors
    Validation(String),
    /// Not found errors
    NotFound(String),
    /// General application errors
    General(String),

    ConflictResolutionRequired,
    MasterPasswordRequired,
    TerminalNotFound(String),
}

impl fmt::Display for AppError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            AppError::Database(msg) => write!(f, "Database error: {}", msg),
            AppError::Encryption(msg) => write!(f, "Encryption error: {}", msg),
            AppError::SSH(msg) => write!(f, "SSH error: {}", msg),
            AppError::Terminal(msg) => write!(f, "Terminal error: {}", msg),
            AppError::Config(msg) => write!(f, "Configuration error: {}", msg),
            AppError::Auth(msg) => write!(f, "Authentication error: {}", msg),
            AppError::Network(msg) => write!(f, "Network error: {}", msg),
            AppError::Validation(msg) => write!(f, "Validation error: {}", msg),
            AppError::NotFound(msg) => write!(f, "Not found: {}", msg),
            AppError::General(msg) => write!(f, "Application error: {}", msg),
            AppError::ConflictResolutionRequired => write!(f, "Conflict resolution required"),
            AppError::MasterPasswordRequired => write!(f, "Master password required"),
            AppError::TerminalNotFound(id) => write!(f, "Terminal not found: {}", id),
        }
    }
}

impl std::error::Error for AppError {}

impl From<AppError> for String {
    fn from(error: AppError) -> Self {
        error.to_string()
    }
}

impl From<std::io::Error> for AppError {
    fn from(error: std::io::Error) -> Self {
        AppError::General(format!("IO error: {}", error))
    }
}

impl From<DatabaseError> for AppError {
    fn from(error: DatabaseError) -> Self {
        match error {
            DatabaseError::ConnectionFailed(msg) => {
                AppError::Network(format!("Database connection failed: {}", msg))
            }
            DatabaseError::AuthenticationFailed(msg) => {
                AppError::Auth(format!("Database authentication failed: {}", msg))
            }
            DatabaseError::QueryFailed(msg) => AppError::Database(format!("Query failed: {}", msg)),
            DatabaseError::TransactionFailed(msg) => {
                AppError::Database(format!("Transaction failed: {}", msg))
            }
            DatabaseError::NotFound(msg) => AppError::NotFound(msg),
            DatabaseError::ValidationError(msg) => AppError::Validation(msg),
            DatabaseError::ParseError(msg) => AppError::Database(format!("Parse error: {}", msg)),
            DatabaseError::SerializationError(err) => {
                AppError::Database(format!("Serialization error: {}", err))
            }
            DatabaseError::EncryptionError(err) => AppError::Encryption(err.to_string()),
            DatabaseError::SyncError(msg) => AppError::Database(format!("Sync error: {}", msg)),
            DatabaseError::ConfigError(msg) => AppError::Config(msg),
            DatabaseError::MigrationError(msg) => {
                AppError::Database(format!("Migration error: {}", msg))
            }
            DatabaseError::ConflictResolutionRequired => AppError::ConflictResolutionRequired,
            DatabaseError::Conflict(msg) => AppError::Validation(format!("Conflict: {}", msg)),
            DatabaseError::MasterPasswordRequired => AppError::MasterPasswordRequired,
            DatabaseError::UnsupportedProvider(msg) => {
                AppError::Config(format!("Unsupported provider: {}", msg))
            }
            DatabaseError::Internal(err) => {
                AppError::General(format!("Internal database error: {}", err))
            }
            DatabaseError::NotImplemented(msg) => {
                AppError::General(format!("Not implemented: {}", msg))
            }
        }
    }
}

impl From<EncryptionError> for AppError {
    fn from(error: EncryptionError) -> Self {
        match error {
            EncryptionError::EncryptionFailed(msg) => {
                AppError::Encryption(format!("Encryption failed: {}", msg))
            }
            EncryptionError::DecryptionFailed(msg) => {
                AppError::Encryption(format!("Decryption failed: {}", msg))
            }
            EncryptionError::InvalidKey(msg) => {
                AppError::Encryption(format!("Invalid key: {}", msg))
            }
            EncryptionError::KeyDerivationFailed(msg) => {
                AppError::Encryption(format!("Key derivation failed: {}", msg))
            }
            EncryptionError::MasterPasswordVerificationFailed => {
                AppError::Auth("Master password verification failed".to_string())
            }
            EncryptionError::UnknownDeviceKey(msg) => {
                AppError::Encryption(format!("Unknown device key: {}", msg))
            }
            EncryptionError::KeychainError(msg) => {
                AppError::Config(format!("Keychain error: {}", msg))
            }
            EncryptionError::InvalidFormat => {
                AppError::Encryption("Invalid encryption format".to_string())
            }
        }
    }
}

impl AppError {
    pub fn connection_failed(msg: impl Into<String>) -> Self {
        AppError::Network(format!("Connection failed: {}", msg.into()))
    }

    pub fn authentication_failed(msg: impl Into<String>) -> Self {
        AppError::Auth(format!("Authentication failed: {}", msg.into()))
    }

    pub fn query_failed(msg: impl Into<String>) -> Self {
        AppError::Database(format!("Query failed: {}", msg.into()))
    }

    pub fn transaction_failed(msg: impl Into<String>) -> Self {
        AppError::Database(format!("Transaction failed: {}", msg.into()))
    }

    pub fn validation_error(msg: impl Into<String>) -> Self {
        AppError::Validation(msg.into())
    }

    pub fn parse_error(msg: impl Into<String>) -> Self {
        AppError::Database(format!("Parse error: {}", msg.into()))
    }

    pub fn serialization_error(msg: impl Into<String>) -> Self {
        AppError::Database(format!("Serialization error: {}", msg.into()))
    }

    pub fn sync_error(msg: impl Into<String>) -> Self {
        AppError::Database(format!("Sync error: {}", msg.into()))
    }

    pub fn config_error(msg: impl Into<String>) -> Self {
        AppError::Config(msg.into())
    }

    pub fn migration_error(msg: impl Into<String>) -> Self {
        AppError::Database(format!("Migration error: {}", msg.into()))
    }

    pub fn unsupported_provider(msg: impl Into<String>) -> Self {
        AppError::Config(format!("Unsupported provider: {}", msg.into()))
    }

    pub fn internal_error(msg: impl Into<String>) -> Self {
        AppError::General(format!("Internal error: {}", msg.into()))
    }

    pub fn not_implemented(msg: impl Into<String>) -> Self {
        AppError::General(format!("Not implemented: {}", msg.into()))
    }

    pub fn pty_error(msg: impl Into<String>) -> Self {
        AppError::Terminal(format!("PTY error: {}", msg.into()))
    }

    pub fn terminal_error(msg: impl Into<String>) -> Self {
        AppError::Terminal(msg.into())
    }

    pub fn invalid_config(msg: impl Into<String>) -> Self {
        AppError::Config(format!("Invalid configuration: {}", msg.into()))
    }

    pub fn ssh_connection_failed(msg: impl Into<String>) -> Self {
        AppError::Network(format!("SSH connection failed: {}", msg.into()))
    }

    pub fn ssh_authentication_failed(msg: impl Into<String>) -> Self {
        AppError::Auth(format!("SSH authentication failed: {}", msg.into()))
    }

    pub fn ssh_channel_failed(msg: impl Into<String>) -> Self {
        AppError::SSH(format!("SSH channel failed: {}", msg.into()))
    }

    pub fn ssh_command_failed(msg: impl Into<String>) -> Self {
        AppError::SSH(format!("SSH command failed: {}", msg.into()))
    }

    pub fn ssh_transfer_failed(msg: impl Into<String>) -> Self {
        AppError::SSH(format!("SSH transfer failed: {}", msg.into()))
    }

    pub fn ssh_config_error(msg: impl Into<String>) -> Self {
        AppError::Config(format!("SSH config error: {}", msg.into()))
    }

    pub fn encryption_failed(msg: impl Into<String>) -> Self {
        AppError::Encryption(format!("Encryption failed: {}", msg.into()))
    }

    pub fn decryption_failed(msg: impl Into<String>) -> Self {
        AppError::Encryption(format!("Decryption failed: {}", msg.into()))
    }

    pub fn key_derivation_failed(msg: impl Into<String>) -> Self {
        AppError::Encryption(format!("Key derivation failed: {}", msg.into()))
    }

    pub fn invalid_key(msg: impl Into<String>) -> Self {
        AppError::Encryption(format!("Invalid key: {}", msg.into()))
    }

    pub fn invalid_data(msg: impl Into<String>) -> Self {
        AppError::Encryption(format!("Invalid data: {}", msg.into()))
    }
}

impl From<SSHError> for AppError {
    fn from(error: SSHError) -> Self {
        match error {
            SSHError::ConnectionFailed(msg) => {
                AppError::Network(format!("SSH connection failed: {}", msg))
            }
            SSHError::AuthenticationFailed => {
                AppError::Auth("SSH authentication failed".to_string())
            }
            SSHError::AuthMethodMismatch => AppError::Auth("SSH auth method mismatch".to_string()),
            SSHError::PrivateKeyError(msg) => {
                AppError::Config(format!("SSH private key error: {}", msg))
            }
            SSHError::SessionError(msg) => AppError::SSH(format!("SSH session error: {}", msg)),
            SSHError::ChannelError(msg) => AppError::SSH(format!("SSH channel error: {}", msg)),
            SSHError::Timeout => AppError::Network("SSH connection timeout".to_string()),
        }
    }
}
