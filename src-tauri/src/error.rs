use serde::Serialize;

#[derive(Debug, thiserror::Error)]
pub enum Error {
    // IO Error
    #[error(transparent)]
    Io(#[from] std::io::Error),

    // Database Error
    #[error("Database error: {0}")]
    Database(String),

    // Terminal Errors
    #[error("Terminal connection failed: {0}")]
    ConnectionFailed(String),

    #[error("Terminal operation failed: {0}")]
    TerminalError(String),

    #[error("SSH authentication failed: {0}")]
    AuthenticationFailed(String),

    #[error("Invalid configuration: {0}")]
    InvalidConfig(String),

    #[error("Terminal not found: {0}")]
    TerminalNotFound(String),

    #[error("Serialization error: {0}")]
    SerializationError(String),

    #[error("Internal error: {0}")]
    InternalError(String),

    #[error(transparent)]
    SshError(#[from] ssh2::Error),

    #[error("PTY error: {0}")]
    PtyError(String),

    #[error(transparent)]
    JsonError(#[from] serde_json::Error),
}

// Implement Serialize for Error to allow sending it over Tauri commands
impl Serialize for Error {
    fn serialize<S>(&self, serializer: S) -> std::result::Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        serializer.serialize_str(self.to_string().as_ref())
    }
}

pub type Result<T> = std::result::Result<T, Error>;
pub type AppError = Error;

impl From<anyhow::Error> for Error {
    fn from(err: anyhow::Error) -> Self {
        Error::PtyError(err.to_string())
    }
}
