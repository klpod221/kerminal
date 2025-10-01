use crate::database::error::DatabaseResult;
use crate::database::sync::{
    conflict::{ConflictRecord, ConflictResolver},
    scheduler::SyncScheduler,
    strategies::SyncStrategy,
};
use std::sync::Arc;
use tokio::sync::RwLock;

#[derive(Debug)]
#[allow(dead_code)]
pub struct SyncManager {
    conflict_resolver: ConflictResolver,
    scheduler: SyncScheduler,
    active_conflicts: Arc<RwLock<Vec<ConflictRecord>>>,
}

#[allow(dead_code)]
impl SyncManager {
    pub fn new(strategy: SyncStrategy, sync_interval_minutes: u32) -> Self {
        Self {
            conflict_resolver: ConflictResolver::new(strategy),
            scheduler: SyncScheduler::new(sync_interval_minutes),
            active_conflicts: Arc::new(RwLock::new(Vec::new())),
        }
    }

    pub async fn start_scheduled_sync(&self) -> DatabaseResult<()> {
        self.scheduler
            .start()
            .await
            .map_err(|e| crate::database::error::DatabaseError::SyncError(e.to_string()))?;
        Ok(())
    }

    pub async fn stop_scheduled_sync(&self) {
        self.scheduler.stop().await;
    }

    pub async fn is_sync_running(&self) -> bool {
        self.scheduler.is_running().await
    }

    pub async fn get_active_conflicts(&self) -> Vec<ConflictRecord> {
        let conflicts = self.active_conflicts.read().await;
        conflicts.clone()
    }

    pub async fn add_conflict(&self, conflict: ConflictRecord) {
        let mut conflicts = self.active_conflicts.write().await;
        conflicts.push(conflict);
    }

    pub async fn resolve_conflict(&self, conflict_id: &str) -> DatabaseResult<()> {
        let mut conflicts = self.active_conflicts.write().await;
        if let Some(pos) = conflicts.iter().position(|c| c.id == conflict_id) {
            conflicts.remove(pos);
        }
        Ok(())
    }

    pub async fn sync(&self) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
        Ok(())
    }
}

impl Default for SyncManager {
    fn default() -> Self {
        Self::new(SyncStrategy::default(), 15)
    }
}
