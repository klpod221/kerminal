

mod engine;
mod manager;
mod resolver;
mod scheduler;
mod serializer;

pub use engine::SyncEngine;
pub use manager::SyncManager;
pub use scheduler::SyncScheduler;
pub use serializer::SyncSerializable;

use std::sync::Arc;
use tokio::sync::Mutex;

use crate::database::{error::DatabaseResult, service::DatabaseService};
use crate::models::sync::{log::SyncLog, SyncDirection};

/// High-level sync service that orchestrates all sync operations
pub struct SyncService {
    database_service: Arc<Mutex<DatabaseService>>,
    sync_manager: Arc<SyncManager>,
    sync_engine: Arc<SyncEngine>,
    sync_scheduler: Arc<SyncScheduler>,
}

impl SyncService {
    pub fn new(database_service: Arc<Mutex<DatabaseService>>) -> Self {
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

        // Check if master password is unlocked
        let is_unlocked = {
            let db_service = self.database_service.lock().await;
            let manager = db_service.get_master_password_manager_arc();
            let manager_guard = manager.read().await;
            let status = manager_guard.get_status().await;
            status.is_unlocked
        };

        if !is_unlocked {
            return Ok(());
        }

        // Load all external database configs
        let configs = {
            let db_service = self.database_service.lock().await;
            let local_db = db_service.get_local_database();
            let local_guard = local_db.read().await;
            local_guard.get_all_external_databases().await?
        }; // Drop locks


        // Auto-connect based on sync_settings
        let db_service = self.database_service.lock().await;
        let local_db = db_service.get_local_database();
        let sync_settings = {
            let guard = local_db.read().await;
            guard.get_global_sync_settings().await?
        };

        let auto_sync_enabled = sync_settings
            .as_ref()
            .map(|s| s.auto_sync_enabled)
            .unwrap_or(false);

        if auto_sync_enabled {
            for config in configs {
                if let Err(e) = self.sync_manager.connect(&config).await {
                    eprintln!("Failed to auto-connect to {}: {}", config.name, e);
                } else {
                    // Enable auto-sync in scheduler
                    self.sync_scheduler.enable_database(config.base.id.clone()).await?;
                }
            }
        }

        // Start the scheduler
        self.sync_scheduler.start().await?;

        Ok(())
    }    /// Shutdown sync service
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
        let config = {
            let db_service = self.database_service.lock().await;
            let local_db = db_service.get_local_database();
            let local_db_guard = local_db.read().await;

            local_db_guard
                .find_external_database_by_id(database_id)
                .await?
                .ok_or_else(|| {
                    crate::database::error::DatabaseError::NotFound(format!(
                        "External database not found: {}",
                        database_id
                    ))
                })?
                .clone() // Clone to move out of the lock
        }; // Drop locks here

        self.sync_manager.connect(&config).await
    }

    /// Disconnect from an external database
    pub async fn disconnect(&self, database_id: &str) -> DatabaseResult<()> {
        self.sync_manager.disconnect(database_id).await
    }

    /// Check if database is connected
    #[allow(dead_code)]
    pub async fn is_connected(&self, database_id: &str) -> bool {
        self.sync_manager.is_connected(database_id).await
    }

    /// Perform a sync operation
    pub async fn sync(
        &self,
        database_id: &str,
        direction: SyncDirection,
    ) -> DatabaseResult<SyncLog> {
        let config = {
            let db_service = self.database_service.lock().await;
            let local_db = db_service.get_local_database();
            let local_db_guard = local_db.read().await;

            local_db_guard
                .find_external_database_by_id(database_id)
                .await?
                .ok_or_else(|| {
                    crate::database::error::DatabaseError::NotFound(format!(
                        "External database not found: {}",
                        database_id
                    ))
                })?
                .clone()
        }; // Drop locks here

        let result = match direction {
            SyncDirection::Push => {
                println!("SyncService::sync: Executing PUSH");
                self.sync_engine.push(&config).await
            },
            SyncDirection::Pull => {
                println!("SyncService::sync: Executing PULL");
                self.sync_engine.pull(&config).await
            },
            SyncDirection::Bidirectional => {
                println!("SyncService::sync: Executing BIDIRECTIONAL");
                self.sync_engine.sync(&config).await
            },
        };

        match &result {
            Ok(log) => println!("SyncService::sync: Sync completed successfully. Records synced: {}", log.records_synced),
            Err(e) => eprintln!("SyncService::sync: Sync failed: {}", e),
        }

        result
    }

    /// Get sync status for a database
    pub async fn get_status(&self, database_id: &str) -> DatabaseResult<SyncServiceStatus> {
        let is_connected = self.sync_manager.is_connected(database_id).await;

        let db_service = self.database_service.lock().await;
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
        let db_service = self.database_service.lock().await;
        let master_password_manager = db_service.get_master_password_manager_arc();
        drop(db_service);

        let _encryptor = crate::database::encryption::ExternalDbEncryptor::new(master_password_manager);

        let db_service = self.database_service.lock().await;
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

        // Update global sync_settings to enable auto-sync
        let update_request = crate::models::sync::UpdateSyncSettingsRequest {
            is_active: None,
            auto_sync_enabled: Some(true),
            sync_interval_minutes: None,
            conflict_strategy: None,
            sync_direction: None,
            selected_database_id: None,
        };

        local_db
            .write()
            .await
            .update_sync_settings(&update_request)
            .await?;

        // Enable in scheduler
        self.sync_scheduler
            .enable_database(config.base.id.clone())
            .await
    }

    /// Disable auto-sync for a database
    pub async fn disable_auto_sync(&self, database_id: &str) -> DatabaseResult<()> {
        // Update config in database
        let db_service = self.database_service.lock().await;
        let master_password_manager = db_service.get_master_password_manager_arc();
        drop(db_service);

        let _encryptor = crate::database::encryption::ExternalDbEncryptor::new(master_password_manager);

        let db_service = self.database_service.lock().await;
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

        // Update global sync_settings to disable auto-sync
        let update_request = crate::models::sync::UpdateSyncSettingsRequest {
            is_active: None,
            auto_sync_enabled: Some(false),
            sync_interval_minutes: None,
            conflict_strategy: None,
            sync_direction: None,
            selected_database_id: None,
        };

        local_db
            .write()
            .await
            .update_sync_settings(&update_request)
            .await?;

        let _ = config; // Suppress unused warning

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
