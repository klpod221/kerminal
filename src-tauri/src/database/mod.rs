pub mod config;
pub mod encryption;
pub mod error;
pub mod providers;
pub mod service;
pub mod traits;
pub mod traits_sync;

// Re-exports for convenience
pub use service::{DatabaseService, DatabaseServiceConfig};
