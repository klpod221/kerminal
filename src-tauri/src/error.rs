use serde::Serialize;

#[derive(Debug, thiserror::Error)]
pub enum Error {

    // IO Error
    #[error(transparent)]
    Io(#[from] std::io::Error),

    // Database Error
    #[error("Database error: {0}")]
    Database(String),
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
