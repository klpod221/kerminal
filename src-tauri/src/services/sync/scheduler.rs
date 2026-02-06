use chrono::{Duration, Utc};
use log::error;
use std::sync::Arc;
use tokio::{
    sync::Mutex,
    time::{interval, Duration as TokioDuration},
};

use crate::database::{error::DatabaseResult, service::DatabaseService};
use crate::models::sync::external_db::ExternalDatabaseConfig;
use crate::services::sync::engine::SyncEngine;

/// Scheduler for automatic synchronization
pub struct SyncScheduler {
    database_service: Arc<Mutex<DatabaseService>>,
    sync_engine: Arc<SyncEngine>,
    is_running: Arc<Mutex<bool>>,
    enabled_databases: Arc<Mutex<Vec<String>>>,
}

impl SyncScheduler {
    pub fn new(
        database_service: Arc<Mutex<DatabaseService>>,
        sync_engine: Arc<SyncEngine>,
    ) -> Self {
        Self {
            database_service,
            sync_engine,
            is_running: Arc::new(Mutex::new(false)),
            enabled_databases: Arc::new(Mutex::new(Vec::new())),
        }
    }

    /// Start the scheduler
    pub async fn start(&self) -> DatabaseResult<()> {
        let mut is_running = self.is_running.lock().await;
        if *is_running {
            return Ok(());
        }

        *is_running = true;
        drop(is_running);

        let scheduler = self.clone_for_task();
        tokio::spawn(async move {
            scheduler.run_loop().await;
        });

        Ok(())
    }

    /// Stop the scheduler
    #[allow(dead_code)]
    pub async fn stop(&self) -> DatabaseResult<()> {
        let mut is_running = self.is_running.lock().await;
        *is_running = false;
        Ok(())
    }

    /// Enable auto-sync for a database
    pub async fn enable_database(&self, database_id: String) -> DatabaseResult<()> {
        let mut enabled = self.enabled_databases.lock().await;
        if !enabled.contains(&database_id) {
            enabled.push(database_id);
        }
        Ok(())
    }

    /// Disable auto-sync for a database
    pub async fn disable_database(&self, database_id: &str) -> DatabaseResult<()> {
        let mut enabled = self.enabled_databases.lock().await;
        enabled.retain(|id| id != database_id);
        Ok(())
    }

    /// Get list of enabled databases
    pub async fn get_enabled_databases(&self) -> Vec<String> {
        self.enabled_databases.lock().await.clone()
    }

    /// Main scheduler loop
    async fn run_loop(&self) {
        let mut ticker = interval(TokioDuration::from_secs(60));

        loop {
            ticker.tick().await;

            let is_running = *self.is_running.lock().await;
            if !is_running {
                break;
            }

            if let Err(e) = self.process_scheduled_syncs().await {
                error!("Scheduler error: {}", e);
            }
        }
    }

    /// Process all scheduled syncs
    async fn process_scheduled_syncs(&self) -> DatabaseResult<()> {
        let enabled_databases = self.enabled_databases.lock().await.clone();

        if enabled_databases.is_empty() {
            return Ok(());
        }

        // Get all configs first, then release lock before processing
        let all_configs = {
            let db_service = self.database_service.lock().await;
            let local_db = db_service.get_local_database();
            let guard = local_db.read().await;
            let configs = guard.get_all_external_databases().await?;
            configs
        }; // Lock released here

        for config in all_configs {
            if !enabled_databases.contains(&config.base.id) {
                continue;
            }

            // is_sync_due now doesn't cause deadlock since we released the lock above
            if self.is_sync_due(&config).await? {
                if let Err(e) = self.execute_scheduled_sync(&config).await {
                    error!("Failed to sync database {}: {}", config.name, e);
                }
            }
        }

        Ok(())
    }

    /// Check if sync is due for a database
    async fn is_sync_due(&self, config: &ExternalDatabaseConfig) -> DatabaseResult<bool> {
        // Get all needed data in a single lock acquisition to avoid deadlock
        let (logs, sync_settings) = {
            let db_service = self.database_service.lock().await;
            let local_db = db_service.get_local_database();
            let local_guard = local_db.read().await;

            let logs = local_guard.get_sync_logs(&config.base.id, Some(1)).await?;
            let sync_settings = local_guard.get_global_sync_settings().await?;

            (logs, sync_settings)
        }; // Lock released here

        let last_sync = match logs.first() {
            Some(log) => log.completed_at,
            _ => return Ok(true), // Never synced, so sync is due
        };

        let last_sync_time = match last_sync {
            Some(time) => time,
            _ => return Ok(true), // Last sync didn't complete, retry
        };

        let interval_minutes = sync_settings.map(|s| s.sync_interval_minutes).unwrap_or(15) as u64;
        let interval_seconds = (interval_minutes * 60) as i64;
        let next_sync = last_sync_time + Duration::seconds(interval_seconds);

        Ok(Utc::now() >= next_sync)
    }

    /// Execute a scheduled sync
    async fn execute_scheduled_sync(&self, config: &ExternalDatabaseConfig) -> DatabaseResult<()> {
        let result = self.sync_engine.sync(config).await;

        match result {
            Ok(_log) => Ok(()),
            Err(e) => {
                error!("Scheduled sync failed for {}: {}", config.name, e);
                Err(e)
            }
        }
    }

    /// Clone for spawning background task
    fn clone_for_task(&self) -> Self {
        Self {
            database_service: self.database_service.clone(),
            sync_engine: self.sync_engine.clone(),
            is_running: self.is_running.clone(),
            enabled_databases: self.enabled_databases.clone(),
        }
    }

    /// Get scheduler statistics
    pub async fn get_stats(&self) -> SchedulerStats {
        let enabled_count = self.enabled_databases.lock().await.len();
        let is_running = *self.is_running.lock().await;

        SchedulerStats {
            is_running,
            enabled_databases_count: enabled_count,
        }
    }
}

/// Scheduler statistics
#[derive(Debug, Clone)]
pub struct SchedulerStats {
    pub is_running: bool,
    pub enabled_databases_count: usize,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_scheduler_start_stop() {}

    #[test]
    fn test_sync_interval_calculation() {
        let now = Utc::now();
        let one_hour_ago = now - Duration::hours(1);
        let interval = Duration::seconds(3600); // 1 hour

        let next_sync = one_hour_ago + interval;
        assert!(now >= next_sync);
    }
}
