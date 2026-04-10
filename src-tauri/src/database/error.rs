/*
 * Kerminal - Modern Terminal Emulator & SSH Manager
 * Copyright (C) 2026 Bùi Thanh Xuân (klpod221)
 *
 * This program is free software: you can redistribute it and/or modify
 * it under the terms of the GNU General Public License as published by
 * the Free Software Foundation, either version 3 of the License, or
 * (at your option) any later version.
 *
 * This program is distributed in the hope that it will be useful,
 * but WITHOUT ANY WARRANTY; without even the implied warranty of
 * MERCHANTABILITY or FITNESS FOR A PARTICULAR PURPOSE.  See the
 * GNU General Public License for more details.
 *
 * You should have received a copy of the GNU General Public License
 * along with this program.  If not, see <https://www.gnu.org/licenses/>.
 */

use thiserror::Error;

/// Database operation errors
#[derive(Error, Debug)]
pub enum DatabaseError {
    #[error("Connection failed: {0}")]
    ConnectionFailed(String),

    #[error("Authentication failed: {0}")]
    AuthenticationFailed(String),

    #[error("Query execution failed: {0}")]
    QueryFailed(String),

    #[error("Transaction failed: {0}")]
    TransactionFailed(String),

    #[error("Record not found: {0}")]
    NotFound(String),

    #[error("Validation error: {0}")]
    ValidationError(String),

    #[error("Not implemented: {0}")]
    NotImplemented(String),

    #[error("Parse error: {0}")]
    ParseError(String),

    #[error("Serialization error: {0}")]
    SerializationError(#[from] serde_json::Error),

    #[error("Encryption error: {0}")]
    EncryptionError(#[from] EncryptionError),

    #[error("Sync error: {0}")]
    SyncError(String),

    #[error("Configuration error: {0}")]
    ConfigError(String),

    #[error("Migration error: {0}")]
    MigrationError(String),

    #[error("Conflict resolution required")]
    ConflictResolutionRequired,

    #[error("Resource conflict: {0}")]
    Conflict(String),

    #[error("Master password required")]
    MasterPasswordRequired,

    #[error("Database provider not supported: {0}")]
    UnsupportedProvider(String),

    #[error("Internal error: {0}")]
    Internal(#[from] anyhow::Error),
}

/// Encryption-specific errors
#[derive(Error, Debug)]
pub enum EncryptionError {
    #[error("Encryption failed: {0}")]
    EncryptionFailed(String),

    #[error("Decryption failed: {0}")]
    DecryptionFailed(String),

    #[error("Invalid key: {0}")]
    InvalidKey(String),

    #[error("Key derivation failed: {0}")]
    KeyDerivationFailed(String),

    #[error("Master password verification failed")]
    MasterPasswordVerificationFailed,

    #[error("Unknown device encryption key: {0}")]
    UnknownDeviceKey(String),

    #[error("Keychain error: {0}")]
    KeychainError(String),

    #[error("Invalid encryption format")]
    InvalidFormat,
}

/// SSH connection errors
#[derive(Error, Debug)]
pub enum SSHError {
    #[error("Connection failed: {0}")]
    ConnectionFailed(String),

    #[error("Authentication failed")]
    AuthenticationFailed,

    #[error("Auth method mismatch")]
    AuthMethodMismatch,

    #[error("Private key error: {0}")]
    PrivateKeyError(String),

    #[error("Session error: {0}")]
    SessionError(String),

    #[error("Channel error: {0}")]
    ChannelError(String),

    #[error("Timeout")]
    Timeout,
}

/// Convenient Result type for database operations
pub type DatabaseResult<T> = Result<T, DatabaseError>;

/// Convenient Result type for encryption operations
pub type EncryptionResult<T> = Result<T, EncryptionError>;
