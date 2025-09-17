pub mod config;
pub mod encryption;
pub mod error;
pub mod migrations;
pub mod models;
pub mod providers;
pub mod service;
pub mod sync;
pub mod traits;

// Re-exports for convenience
pub use service::{DatabaseService, DatabaseServiceConfig};
pub use traits::Database;
// pub use error::DatabaseError;
