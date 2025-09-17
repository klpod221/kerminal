use crate::database::{DatabaseService, DatabaseServiceConfig};
use futures::FutureExt;
use std::sync::Arc;
use tokio::sync::Mutex;

// Application state
pub struct AppState {
    pub database_service: Arc<Mutex<DatabaseService>>,
}

impl AppState {
    /// Create new app state with initialized database service
    pub async fn new() -> Result<Self, String> {
        let config = DatabaseServiceConfig::default();
        let database_service = DatabaseService::new(config)
            .await
            .map_err(|e| format!("Failed to initialize database service: {}", e))?;

        Ok(Self {
            database_service: Arc::new(Mutex::new(database_service)),
        })
    }
}

impl Default for AppState {
    fn default() -> Self {
        // For backward compatibility, create empty state
        // Real initialization should use AppState::new()
        Self {
            database_service: Arc::new(Mutex::new(
                DatabaseService::new(DatabaseServiceConfig::default())
                    .now_or_never()
                    .unwrap()
                    .unwrap(),
            )),
        }
    }
}
