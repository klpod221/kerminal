use std::sync::Arc;
use tokio::sync::Mutex;

use crate::database::{
    error::DatabaseResult,
    service::{DatabaseService, DatabaseStats},
};

/// Sync service for handling database synchronization and statistics
pub struct SyncService {
    database_service: Arc<Mutex<DatabaseService>>,
}

impl SyncService {
    /// Create new SyncService instance
    pub fn new(database_service: Arc<Mutex<DatabaseService>>) -> Self {
        Self { database_service }
    }

    /// Get database statistics
    pub async fn get_database_stats(&self) -> DatabaseResult<DatabaseStats> {
        let db_service = self.database_service.lock().await;
        db_service.get_database_stats().await
    }

    // TODO: Add more sync-related methods when implementing sync functionality
    // /// Start sync process
    // pub async fn start_sync(&self) -> DatabaseResult<()> {
    //     let db_service = self.database_service.lock().await;
    //     // Implementation for sync logic
    // }

    // /// Stop sync process
    // pub async fn stop_sync(&self) -> DatabaseResult<()> {
    //     let db_service = self.database_service.lock().await;
    //     // Implementation for sync logic
    // }

    // /// Get sync status
    // pub async fn get_sync_status(&self) -> DatabaseResult<SyncStatus> {
    //     let db_service = self.database_service.lock().await;
    //     // Implementation for sync status
    // }
}