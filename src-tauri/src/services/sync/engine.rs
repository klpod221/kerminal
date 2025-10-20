use chrono::{DateTime, Utc};
use std::sync::Arc;
use tokio::sync::RwLock;

use crate::database::{
    error::{DatabaseError, DatabaseResult},
    service::DatabaseService,
    traits::Database,
};
use crate::models::{
    ssh::{SSHGroup, SSHKey, SSHProfile},
    sync::{
        external_db::ExternalDatabaseConfig,
        log::{SyncDirection, SyncLog, SyncStatus},
        ConflictResolutionStrategy,
    },
};
use crate::services::sync::{
    manager::SyncManager,
    resolver::{ConflictResolution, ConflictResolver, DataConflict},
};

/// Sync engine for managing data synchronization
pub struct SyncEngine {
    database_service: Arc<RwLock<DatabaseService>>,
    sync_manager: Arc<SyncManager>,
    conflict_resolver: Arc<ConflictResolver>,
}

impl SyncEngine {
    pub fn new(
        database_service: Arc<RwLock<DatabaseService>>,
        sync_manager: Arc<SyncManager>,
    ) -> Self {
        Self {
            database_service,
            sync_manager,
            conflict_resolver: Arc::new(ConflictResolver::new()),
        }
    }

    /// Push local data to remote database
    pub async fn push(&self, config: &ExternalDatabaseConfig) -> DatabaseResult<SyncLog> {
        let mut sync_log = self.create_sync_log(config, SyncDirection::Push).await?;

        match self.push_internal(config).await {
            Ok(stats) => {
                sync_log.status = SyncStatus::Completed;
                sync_log.records_synced = stats.total_synced;
                sync_log.conflicts_resolved = stats.conflicts_resolved;
                sync_log.completed_at = Some(Utc::now());
            }
            Err(e) => {
                sync_log.status = SyncStatus::Failed;
                sync_log.error_message = Some(e.to_string());
                sync_log.completed_at = Some(Utc::now());
            }
        }

        self.save_sync_log(&sync_log).await?;
        Ok(sync_log)
    }

    /// Pull remote data to local database
    pub async fn pull(&self, config: &ExternalDatabaseConfig) -> DatabaseResult<SyncLog> {
        let mut sync_log = self.create_sync_log(config, SyncDirection::Pull).await?;

        match self.pull_internal(config).await {
            Ok(stats) => {
                sync_log.status = SyncStatus::Completed;
                sync_log.records_synced = stats.total_synced;
                sync_log.conflicts_resolved = stats.conflicts_resolved;
                sync_log.completed_at = Some(Utc::now());
            }
            Err(e) => {
                sync_log.status = SyncStatus::Failed;
                sync_log.error_message = Some(e.to_string());
                sync_log.completed_at = Some(Utc::now());
            }
        }

        self.save_sync_log(&sync_log).await?;
        Ok(sync_log)
    }

    /// Bidirectional sync (pull then push with conflict resolution)
    pub async fn sync(&self, config: &ExternalDatabaseConfig) -> DatabaseResult<SyncLog> {
        let mut sync_log = self
            .create_sync_log(config, SyncDirection::Bidirectional)
            .await?;

        match self.sync_internal(config).await {
            Ok(stats) => {
                sync_log.status = SyncStatus::Completed;
                sync_log.records_synced = stats.total_synced;
                sync_log.conflicts_resolved = stats.conflicts_resolved;
                sync_log.completed_at = Some(Utc::now());
            }
            Err(e) => {
                sync_log.status = SyncStatus::Failed;
                sync_log.error_message = Some(e.to_string());
                sync_log.completed_at = Some(Utc::now());
            }
        }

        self.save_sync_log(&sync_log).await?;
        Ok(sync_log)
    }

    /// Internal push implementation
    async fn push_internal(&self, config: &ExternalDatabaseConfig) -> DatabaseResult<SyncStats> {
        // Ensure connection
        self.sync_manager.ensure_connection(config).await?;
        let remote = self.sync_manager.get_provider(&config.base.id).await?;

        let db_service = self.database_service.read().await;
        let local = db_service.get_local_database();
        let local_guard = local.read().await;

        let mut stats = SyncStats::default();

        // Sync SSH Profiles
        let profiles = local_guard.find_all_ssh_profiles().await?;
        for profile in profiles {
            remote.save_ssh_profile(&profile).await?;
            stats.total_synced += 1;
        }

        // Sync SSH Groups
        let groups = local_guard.find_all_ssh_groups().await?;
        for group in groups {
            remote.save_ssh_group(&group).await?;
            stats.total_synced += 1;
        }

        // Sync SSH Keys
        let keys = local_guard.find_all_ssh_keys().await?;
        for key in keys {
            remote.save_ssh_key(&key).await?;
            stats.total_synced += 1;
        }

        Ok(stats)
    }

    /// Internal pull implementation
    async fn pull_internal(&self, config: &ExternalDatabaseConfig) -> DatabaseResult<SyncStats> {
        // Ensure connection
        self.sync_manager.ensure_connection(config).await?;
        let remote = self.sync_manager.get_provider(&config.base.id).await?;

        let db_service = self.database_service.read().await;
        let local = db_service.get_local_database();
        let local_guard = local.write().await;

        let mut stats = SyncStats::default();

        // Sync SSH Profiles
        let profiles = remote.find_all_ssh_profiles().await?;
        for profile in profiles {
            local_guard.save_ssh_profile(&profile).await?;
            stats.total_synced += 1;
        }

        // Sync SSH Groups
        let groups = remote.find_all_ssh_groups().await?;
        for group in groups {
            local_guard.save_ssh_group(&group).await?;
            stats.total_synced += 1;
        }

        // Sync SSH Keys
        let keys = remote.find_all_ssh_keys().await?;
        for key in keys {
            local_guard.save_ssh_key(&key).await?;
            stats.total_synced += 1;
        }

        Ok(stats)
    }

    /// Internal bidirectional sync with conflict resolution
    async fn sync_internal(&self, config: &ExternalDatabaseConfig) -> DatabaseResult<SyncStats> {
        // Ensure connection
        self.sync_manager.ensure_connection(config).await?;
        let remote = self.sync_manager.get_provider(&config.base.id).await?;

        let db_service = self.database_service.read().await;
        let local = db_service.get_local_database();

        let mut stats = SyncStats::default();

        // Get last sync time
        let last_sync = self.get_last_sync_time(&config.base.id).await?;

        // Parse sync settings to get conflict resolution strategy
        let sync_settings = config
            .parse_sync_settings()
            .map_err(DatabaseError::SerializationError)?;
        let strategy = sync_settings.conflict_resolution_strategy;

        // Sync SSH Profiles with conflict detection
        let profile_stats = self
            .sync_ssh_profiles(&local, &remote, last_sync, strategy)
            .await?;
        stats.merge(profile_stats);

        // Sync SSH Groups with conflict detection
        let group_stats = self
            .sync_ssh_groups(&local, &remote, last_sync, strategy)
            .await?;
        stats.merge(group_stats);

        // Sync SSH Keys with conflict detection
        let key_stats = self
            .sync_ssh_keys(&local, &remote, last_sync, strategy)
            .await?;
        stats.merge(key_stats);

        Ok(stats)
    }

    /// Create a new sync log entry
    async fn create_sync_log(
        &self,
        config: &ExternalDatabaseConfig,
        direction: SyncDirection,
    ) -> DatabaseResult<SyncLog> {
        let db_service = self.database_service.read().await;
        let local = db_service.get_local_database();
        let local_guard = local.read().await;
        let device = local_guard
            .get_current_device()
            .await?
            .ok_or_else(|| DatabaseError::NotFound("Current device not found".to_string()))?;

        Ok(SyncLog {
            id: uuid::Uuid::new_v4().to_string(),
            database_id: config.base.id.clone(),
            device_id: device.device_id.clone(),
            direction,
            status: SyncStatus::InProgress,
            started_at: Utc::now(),
            completed_at: None,
            records_synced: 0,
            conflicts_resolved: 0,
            error_message: None,
        })
    }

    /// Save sync log to database
    async fn save_sync_log(&self, log: &SyncLog) -> DatabaseResult<()> {
        let db_service = self.database_service.read().await;
        let local = db_service.get_local_database();
        let mut guard = local.write().await;
        guard.save_sync_log(log).await
    }

    /// Get last sync time for a database
    async fn get_last_sync_time(&self, database_id: &str) -> DatabaseResult<Option<DateTime<Utc>>> {
        let db_service = self.database_service.read().await;
        let local = db_service.get_local_database();
        let guard = local.read().await;
        let logs = guard.get_sync_logs(database_id, Some(1)).await?;

        Ok(logs.first().and_then(|log| log.completed_at))
    }

    /// Save conflict for manual resolution
    async fn save_conflict_for_manual_resolution<T>(
        &self,
        conflict: DataConflict<T>,
    ) -> DatabaseResult<()>
    where
        T: serde::Serialize,
    {
        let db_service = self.database_service.read().await;
        let local = db_service.get_local_database();

        let conflict_resolution = crate::models::sync::conflict::ConflictResolution {
            id: uuid::Uuid::new_v4().to_string(),
            entity_type: conflict.entity_type,
            entity_id: conflict.entity_id,
            local_data: serde_json::to_value(&conflict.local_data)
                .map_err(DatabaseError::SerializationError)?,
            remote_data: serde_json::to_value(&conflict.remote_data)
                .map_err(DatabaseError::SerializationError)?,
            resolution_strategy: None,
            resolved_at: None,
            created_at: Utc::now(),
        };

        let mut guard = local.write().await;
        guard.save_conflict_resolution(&conflict_resolution).await
    }

    /// Sync SSH Profiles with conflict detection
    async fn sync_ssh_profiles(
        &self,
        local: &Arc<RwLock<crate::database::providers::sqlite::SQLiteProvider>>,
        remote: &Arc<dyn crate::database::traits::Database>,
        last_sync: Option<DateTime<Utc>>,
        strategy: ConflictResolutionStrategy,
    ) -> DatabaseResult<SyncStats> {
        let mut stats = SyncStats::default();

        // Get all local and remote profiles
        let local_guard = local.read().await;
        let local_profiles = local_guard.find_all_ssh_profiles().await?;
        drop(local_guard);

        let remote_profiles = remote.find_all_ssh_profiles().await?;

        // Build maps for conflict detection
        let local_data: Vec<_> = local_profiles
            .iter()
            .map(|p| (p.base.id.clone(), p.clone(), p.base.updated_at))
            .collect();

        let remote_data: Vec<_> = remote_profiles
            .iter()
            .map(|p| (p.base.id.clone(), p.clone(), p.base.updated_at))
            .collect();

        // Detect conflicts
        let conflicts = self.conflict_resolver.detect_conflicts(
            local_data,
            remote_data,
            last_sync,
            "SSHProfile".to_string(),
        );

        // Resolve each conflict
        for conflict in &conflicts {
            match self.conflict_resolver.resolve(conflict.clone(), strategy)? {
                crate::services::sync::resolver::ConflictResolution::UseLocal(item) => {
                    remote.save_ssh_profile(&item).await?;
                    stats.total_synced += 1;
                    stats.conflicts_resolved += 1;
                }
                crate::services::sync::resolver::ConflictResolution::UseRemote(item) => {
                    let local_guard = local.write().await;
                    local_guard.save_ssh_profile(&item).await?;
                    drop(local_guard);
                    stats.total_synced += 1;
                    stats.conflicts_resolved += 1;
                }
                crate::services::sync::resolver::ConflictResolution::UseMerged(item) => {
                    // Save merged to both
                    remote.save_ssh_profile(&item).await?;
                    let local_guard = local.write().await;
                    local_guard.save_ssh_profile(&item).await?;
                    drop(local_guard);
                    stats.total_synced += 1;
                    stats.conflicts_resolved += 1;
                }
                crate::services::sync::resolver::ConflictResolution::RequiresManual(c) => {
                    self.save_conflict_for_manual_resolution(c).await?;
                    stats.manual_conflicts += 1;
                }
            }
        }

        // Sync non-conflicting items
        for profile in local_profiles {
            // Check if this was already handled in conflict resolution
            if conflicts.iter().any(|c| c.entity_id == profile.base.id) {
                continue;
            }
            remote.save_ssh_profile(&profile).await?;
            stats.total_synced += 1;
        }

        for profile in remote_profiles {
            if conflicts.iter().any(|c| c.entity_id == profile.base.id) {
                continue;
            }
            let local_guard = local.write().await;
            local_guard.save_ssh_profile(&profile).await?;
            drop(local_guard);
            stats.total_synced += 1;
        }

        Ok(stats)
    }

    /// Sync SSH Groups with conflict detection
    async fn sync_ssh_groups(
        &self,
        local: &Arc<RwLock<crate::database::providers::sqlite::SQLiteProvider>>,
        remote: &Arc<dyn crate::database::traits::Database>,
        last_sync: Option<DateTime<Utc>>,
        strategy: ConflictResolutionStrategy,
    ) -> DatabaseResult<SyncStats> {
        let mut stats = SyncStats::default();

        let local_guard = local.read().await;
        let local_groups = local_guard.find_all_ssh_groups().await?;
        drop(local_guard);

        let remote_groups = remote.find_all_ssh_groups().await?;

        let local_data: Vec<_> = local_groups
            .iter()
            .map(|g| (g.base.id.clone(), g.clone(), g.base.updated_at))
            .collect();

        let remote_data: Vec<_> = remote_groups
            .iter()
            .map(|g| (g.base.id.clone(), g.clone(), g.base.updated_at))
            .collect();

        let conflicts = self.conflict_resolver.detect_conflicts(
            local_data,
            remote_data,
            last_sync,
            "SSHGroup".to_string(),
        );

        for conflict in &conflicts {
            match self.conflict_resolver.resolve(conflict.clone(), strategy)? {
                crate::services::sync::resolver::ConflictResolution::UseLocal(item) => {
                    remote.save_ssh_group(&item).await?;
                    stats.total_synced += 1;
                    stats.conflicts_resolved += 1;
                }
                crate::services::sync::resolver::ConflictResolution::UseRemote(item) => {
                    let local_guard = local.write().await;
                    local_guard.save_ssh_group(&item).await?;
                    drop(local_guard);
                    stats.total_synced += 1;
                    stats.conflicts_resolved += 1;
                }
                crate::services::sync::resolver::ConflictResolution::UseMerged(item) => {
                    remote.save_ssh_group(&item).await?;
                    let local_guard = local.write().await;
                    local_guard.save_ssh_group(&item).await?;
                    drop(local_guard);
                    stats.total_synced += 1;
                    stats.conflicts_resolved += 1;
                }
                crate::services::sync::resolver::ConflictResolution::RequiresManual(c) => {
                    self.save_conflict_for_manual_resolution(c).await?;
                    stats.manual_conflicts += 1;
                }
            }
        }

        for group in local_groups {
            if conflicts.iter().any(|c| c.entity_id == group.base.id) {
                continue;
            }
            remote.save_ssh_group(&group).await?;
            stats.total_synced += 1;
        }

        for group in remote_groups {
            if conflicts.iter().any(|c| c.entity_id == group.base.id) {
                continue;
            }
            let local_guard = local.write().await;
            local_guard.save_ssh_group(&group).await?;
            drop(local_guard);
            stats.total_synced += 1;
        }

        Ok(stats)
    }

    /// Sync SSH Keys with conflict detection
    async fn sync_ssh_keys(
        &self,
        local: &Arc<RwLock<crate::database::providers::sqlite::SQLiteProvider>>,
        remote: &Arc<dyn crate::database::traits::Database>,
        last_sync: Option<DateTime<Utc>>,
        strategy: ConflictResolutionStrategy,
    ) -> DatabaseResult<SyncStats> {
        let mut stats = SyncStats::default();

        let local_guard = local.read().await;
        let local_keys = local_guard.find_all_ssh_keys().await?;
        drop(local_guard);

        let remote_keys = remote.find_all_ssh_keys().await?;

        let local_data: Vec<_> = local_keys
            .iter()
            .map(|k| (k.base.id.clone(), k.clone(), k.base.updated_at))
            .collect();

        let remote_data: Vec<_> = remote_keys
            .iter()
            .map(|k| (k.base.id.clone(), k.clone(), k.base.updated_at))
            .collect();

        let conflicts = self.conflict_resolver.detect_conflicts(
            local_data,
            remote_data,
            last_sync,
            "SSHKey".to_string(),
        );

        for conflict in &conflicts {
            match self.conflict_resolver.resolve(conflict.clone(), strategy)? {
                crate::services::sync::resolver::ConflictResolution::UseLocal(item) => {
                    remote.save_ssh_key(&item).await?;
                    stats.total_synced += 1;
                    stats.conflicts_resolved += 1;
                }
                crate::services::sync::resolver::ConflictResolution::UseRemote(item) => {
                    let local_guard = local.write().await;
                    local_guard.save_ssh_key(&item).await?;
                    drop(local_guard);
                    stats.total_synced += 1;
                    stats.conflicts_resolved += 1;
                }
                crate::services::sync::resolver::ConflictResolution::UseMerged(item) => {
                    remote.save_ssh_key(&item).await?;
                    let local_guard = local.write().await;
                    local_guard.save_ssh_key(&item).await?;
                    drop(local_guard);
                    stats.total_synced += 1;
                    stats.conflicts_resolved += 1;
                }
                crate::services::sync::resolver::ConflictResolution::RequiresManual(c) => {
                    self.save_conflict_for_manual_resolution(c).await?;
                    stats.manual_conflicts += 1;
                }
            }
        }

        for key in local_keys {
            if conflicts.iter().any(|c| c.entity_id == key.base.id) {
                continue;
            }
            remote.save_ssh_key(&key).await?;
            stats.total_synced += 1;
        }

        for key in remote_keys {
            if conflicts.iter().any(|c| c.entity_id == key.base.id) {
                continue;
            }
            let local_guard = local.write().await;
            local_guard.save_ssh_key(&key).await?;
            drop(local_guard);
            stats.total_synced += 1;
        }

        Ok(stats)
    }
}

/// Sync statistics
#[derive(Debug, Default, Clone)]
struct SyncStats {
    total_synced: i32,
    conflicts_resolved: i32,
    manual_conflicts: i32,
}

impl SyncStats {
    fn merge(&mut self, other: SyncStats) {
        self.total_synced += other.total_synced;
        self.conflicts_resolved += other.conflicts_resolved;
        self.manual_conflicts += other.manual_conflicts;
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_sync_stats_merge() {
        let mut stats1 = SyncStats {
            total_synced: 10,
            conflicts_resolved: 2,
            manual_conflicts: 1,
        };

        let stats2 = SyncStats {
            total_synced: 5,
            conflicts_resolved: 1,
            manual_conflicts: 0,
        };

        stats1.merge(stats2);

        assert_eq!(stats1.total_synced, 15);
        assert_eq!(stats1.conflicts_resolved, 3);
        assert_eq!(stats1.manual_conflicts, 1);
    }
}
