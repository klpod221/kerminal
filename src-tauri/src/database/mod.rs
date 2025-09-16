pub mod traits;
pub mod models;
pub mod providers;
pub mod encryption;
pub mod sync;
pub mod migrations;
pub mod service;
pub mod config;
pub mod error;

// Re-exports for convenience
pub use traits::{Database, Syncable, Encryptable, EncryptionService};
pub use service::{DatabaseService, DatabaseServiceConfig, DatabaseStats};
pub use encryption::MasterPasswordManager;
pub use error::{DatabaseError, DatabaseResult};
