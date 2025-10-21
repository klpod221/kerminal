
mod engine;
mod manager;
mod resolver;
mod scheduler;

pub use engine::SyncEngine;
pub use manager::SyncManager;
pub use scheduler::SyncScheduler;

use std::sync::Arc;
use tokio::sync::RwLock;

use crate::database::{
    error::{DatabaseError, DatabaseResult},
    service::DatabaseService,
};
use crate::models::sync::{log::SyncLog, SyncDirection};

/// High-level sync service that orchestrates all sync operations
pub struct SyncService {
    database_service: Arc<RwLock<DatabaseService>>,
    sync_manager: Arc<SyncManager>,
    sync_engine: Arc<SyncEngine>,
    sync_scheduler: Arc<SyncScheduler>,
}

impl SyncService {
    pub fn new(database_service: Arc<RwLock<DatabaseService>>) -> Self {
        let sync_manager = Arc::new(SyncManager::new(database_service.clone()));
        let sync_engine = Arc::new(SyncEngine::new(
            database_service.clone(),
            sync_manager.clone(),
        ));
        let sync_scheduler = Arc::new(SyncScheduler::new(
            database_service.clone(),
            sync_engine.clone(),
        ));

        Self {
            database_service,
            sync_manager,
            sync_engine,
            sync_scheduler,
        }
    }

    /// Initialize sync service (start scheduler, load enabled databases)
    pub async fn initialize(&self) -> DatabaseResult<()> {
        // Load all external database configs
        let db_service = self.database_service.read().await;
        let local_db = db_service.get_local_database();
        let configs = local_db.read().await.get_all_external_databases().await?;

        // Enable auto-sync for databases that have it configured
        for config in configs {
            let sync_settings = config.parse_sync_settings().ok();
            if let Some(settings) = sync_settings {
                if settings.auto_sync {
                    self.sync_scheduler
                        .enable_database(config.base.id.clone())
                        .await?;
                }
            }
        }

        // Start the scheduler
        self.sync_scheduler.start().await?;

        Ok(())
    }

    /// Shutdown sync service
    #[allow(dead_code)]
    pub async fn shutdown(&self) -> DatabaseResult<()> {
        // Stop scheduler
        self.sync_scheduler.stop().await?;

        // Disconnect all active connections
        self.sync_manager.disconnect_all().await?;

        Ok(())
    }

    /// Connect to an external database
    pub async fn connect(&self, database_id: &str) -> DatabaseResult<()> {
        let db_service = self.database_service.read().await;
        let local_db = db_service.get_local_database();
        let config = local_db
            .read()
            .await
            .find_external_database_by_id(database_id)
            .await?
            .ok_or_else(|| {
                crate::database::error::DatabaseError::NotFound(format!(
                    "External database not found: {}",
                    database_id
                ))
            })?;

        self.sync_manager.connect(&config).await
    }

    /// Disconnect from an external database
    pub async fn disconnect(&self, database_id: &str) -> DatabaseResult<()> {
        self.sync_manager.disconnect(database_id).await
    }

    /// Test connection to an external database
    pub async fn test_connection(&self, database_id: &str) -> DatabaseResult<bool> {
        let db_service = self.database_service.read().await;
        let local_db = db_service.get_local_database();
        let config = local_db
            .read()
            .await
            .find_external_database_by_id(database_id)
            .await?
            .ok_or_else(|| {
                crate::database::error::DatabaseError::NotFound(format!(
                    "External database not found: {}",
                    database_id
                ))
            })?;

        match self.sync_manager.connect(&config).await {
            Ok(_) => {
                self.sync_manager.disconnect(database_id).await?;
                Ok(true)
            }
            Err(_) => Ok(false),
        }
    }

    /// Perform a sync operation
    pub async fn sync(
        &self,
        database_id: &str,
        direction: SyncDirection,
    ) -> DatabaseResult<SyncLog> {
        let db_service = self.database_service.read().await;
        let local_db = db_service.get_local_database();
        let config = local_db
            .read()
            .await
            .find_external_database_by_id(database_id)
            .await?
            .ok_or_else(|| {
                crate::database::error::DatabaseError::NotFound(format!(
                    "External database not found: {}",
                    database_id
                ))
            })?;

        match direction {
            SyncDirection::Push => self.sync_engine.push(&config).await,
            SyncDirection::Pull => self.sync_engine.pull(&config).await,
            SyncDirection::Bidirectional => self.sync_engine.sync(&config).await,
        }
    }

    /// Get sync status for a database
    pub async fn get_status(&self, database_id: &str) -> DatabaseResult<SyncServiceStatus> {
        let is_connected = self.sync_manager.is_connected(database_id).await;

        let db_service = self.database_service.read().await;
        let local_db = db_service.get_local_database();
        let last_sync_log = local_db
            .read()
            .await
            .get_sync_logs(database_id, Some(1))
            .await?
            .into_iter()
            .next();

        let scheduler_enabled = self
            .sync_scheduler
            .get_enabled_databases()
            .await
            .contains(&database_id.to_string());

        Ok(SyncServiceStatus {
            is_connected,
            last_sync: last_sync_log,
            scheduler_enabled,
        })
    }

    /// Enable auto-sync for a database
    pub async fn enable_auto_sync(&self, database_id: &str) -> DatabaseResult<()> {
        // Update config in database
        let db_service = self.database_service.read().await;
        let local_db = db_service.get_local_database();
        let mut config = local_db
            .read()
            .await
            .find_external_database_by_id(database_id)
            .await?
            .ok_or_else(|| {
                crate::database::error::DatabaseError::NotFound(format!(
                    "External database not found: {}",
                    database_id
                ))
            })?;

        let mut sync_settings = config
            .parse_sync_settings()
            .map_err(DatabaseError::SerializationError)?;
        sync_settings.auto_sync = true;
        config.sync_settings =
            serde_json::to_string(&sync_settings).map_err(DatabaseError::SerializationError)?;

        local_db
            .write()
            .await
            .save_external_database(&config)
            .await?;

        // Enable in scheduler
        self.sync_scheduler
            .enable_database(database_id.to_string())
            .await
    }

    /// Disable auto-sync for a database
    pub async fn disable_auto_sync(&self, database_id: &str) -> DatabaseResult<()> {
        // Update config in database
        let db_service = self.database_service.read().await;
        let local_db = db_service.get_local_database();
        let mut config = local_db
            .read()
            .await
            .find_external_database_by_id(database_id)
            .await?
            .ok_or_else(|| {
                crate::database::error::DatabaseError::NotFound(format!(
                    "External database not found: {}",
                    database_id
                ))
            })?;

        let mut sync_settings = config
            .parse_sync_settings()
            .map_err(DatabaseError::SerializationError)?;
        sync_settings.auto_sync = false;
        config.sync_settings =
            serde_json::to_string(&sync_settings).map_err(DatabaseError::SerializationError)?;

        local_db
            .write()
            .await
            .save_external_database(&config)
            .await?;

        // Disable in scheduler
        self.sync_scheduler.disable_database(database_id).await
    }

    /// Get overall service statistics
    pub async fn get_statistics(&self) -> SyncServiceStatistics {
        let connection_stats = self.sync_manager.get_connection_stats().await;
        let scheduler_stats = self.sync_scheduler.get_stats().await;

        SyncServiceStatistics {
            active_connections: connection_stats.total_connections,
            scheduler_running: scheduler_stats.is_running,
            auto_sync_enabled_count: scheduler_stats.enabled_databases_count,
        }
    }
}

/// Sync service status
#[derive(Debug, Clone, serde::Serialize)]
#[serde(rename_all = "camelCase")]
pub struct SyncServiceStatus {
    pub is_connected: bool,
    pub last_sync: Option<SyncLog>,
    pub scheduler_enabled: bool,
}

/// Service-wide statistics
#[derive(Debug, Clone, serde::Serialize)]
#[serde(rename_all = "camelCase")]
pub struct SyncServiceStatistics {
    pub active_connections: usize,
    pub scheduler_running: bool,
    pub auto_sync_enabled_count: usize,
}

#[cfg(test)]
mod tests {
    #[test]
    fn test_sync_service_creation() {
        // Test implementation requires mock DatabaseService
    }
}
