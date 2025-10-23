use chrono::{DateTime, Utc};
use std::sync::Arc;
use tokio::sync::{Mutex, RwLock};

use crate::database::{
    error::{DatabaseError, DatabaseResult},
    service::DatabaseService,
    traits::Database,
};
use crate::models::sync::{
    external_db::ExternalDatabaseConfig,
    log::{SyncDirection, SyncLog, SyncStatus},
    ConflictResolutionStrategy,
};
use crate::services::sync::{
    manager::SyncManager,
    resolver::{ConflictResolver, DataConflict},
};

/// Sync engine for managing data synchronization
pub struct SyncEngine {
    database_service: Arc<Mutex<DatabaseService>>,
    sync_manager: Arc<SyncManager>,
    #[allow(dead_code)]
    conflict_resolver: Arc<ConflictResolver>,
}

impl SyncEngine {
    pub fn new(
        database_service: Arc<Mutex<DatabaseService>>,
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
                sync_log.records_synced = stats.total_synced as i32;
                sync_log.conflicts_resolved = stats.conflicts_resolved as i32;
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
                sync_log.records_synced = stats.total_synced as i32;
                sync_log.conflicts_resolved = stats.conflicts_resolved as i32;
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

    /// Full bidirectional sync with conflict resolution
    pub async fn sync(&self, config: &ExternalDatabaseConfig) -> DatabaseResult<SyncLog> {
        let mut sync_log = self
            .create_sync_log(config, SyncDirection::Bidirectional)
            .await?;

        match self.sync_internal(config).await {
            Ok(stats) => {
                sync_log.status = SyncStatus::Completed;
                sync_log.records_synced = stats.total_synced as i32;
                sync_log.conflicts_resolved = stats.conflicts_resolved as i32;
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

    /// Internal push implementation - Push local data to remote
    async fn push_internal(&self, config: &ExternalDatabaseConfig) -> DatabaseResult<SyncStats> {
        use crate::services::sync::SyncSerializable;

        self.sync_manager.ensure_connection(config).await?;
        let remote = self.sync_manager.get_provider(&config.base.id).await?;

        let mut stats = SyncStats::default();

        let db_service = self.database_service.lock().await;
        let local = db_service.get_local_database();
        let local_guard = local.read().await;

        let profiles = local_guard.find_all_ssh_profiles().await?;
        let json_profiles: Vec<_> = profiles.iter().filter_map(|p| p.to_json().ok()).collect();
        if !json_profiles.is_empty() {
            let count = remote.push_records("ssh_profiles", json_profiles).await?;
            stats.total_synced += count;
        }

        let groups = local_guard.find_all_ssh_groups().await?;
        let json_groups: Vec<_> = groups.iter().filter_map(|g| g.to_json().ok()).collect();
        if !json_groups.is_empty() {
            let count = remote.push_records("ssh_groups", json_groups).await?;
            stats.total_synced += count;
        }

        let keys = local_guard.find_all_ssh_keys().await?;
        let json_keys: Vec<_> = keys.iter().filter_map(|k| k.to_json().ok()).collect();
        if !json_keys.is_empty() {
            let count = remote.push_records("ssh_keys", json_keys).await?;
            stats.total_synced += count;
        }

        let cmd_groups = local_guard.find_all_saved_command_groups().await?;
        let json_cmd_groups: Vec<_> = cmd_groups.iter().filter_map(|g| g.to_json().ok()).collect();
        if !json_cmd_groups.is_empty() {
            let count = remote
                .push_records("saved_command_groups", json_cmd_groups)
                .await?;
            stats.total_synced += count;
        }

        let commands = local_guard.find_all_saved_commands().await?;
        let json_commands: Vec<_> = commands.iter().filter_map(|c| c.to_json().ok()).collect();
        if !json_commands.is_empty() {
            let count = remote.push_records("saved_commands", json_commands).await?;
            stats.total_synced += count;
        }

        Ok(stats)
    }

    /// Internal pull implementation - Pull remote data to local
    async fn pull_internal(&self, config: &ExternalDatabaseConfig) -> DatabaseResult<SyncStats> {
        use crate::services::sync::SyncSerializable;

        self.sync_manager.ensure_connection(config).await?;
        let remote = self.sync_manager.get_provider(&config.base.id).await?;

        let mut stats = SyncStats::default();

        let last_sync = self.get_last_sync_time(&config.base.id).await?;

        let db_service = self.database_service.lock().await;
        let local = db_service.get_local_database();

        let local_guard = local.write().await;

        let json_profiles = remote.pull_records("ssh_profiles", last_sync).await?;
        for json in json_profiles {
            if let Ok(profile) = crate::models::ssh::SSHProfile::from_json(&json) {
                local_guard.save_ssh_profile(&profile).await?;
                stats.total_synced += 1;
            }
        }

        let json_groups = remote.pull_records("ssh_groups", last_sync).await?;
        for json in json_groups {
            if let Ok(group) = crate::models::ssh::SSHGroup::from_json(&json) {
                local_guard.save_ssh_group(&group).await?;
                stats.total_synced += 1;
            }
        }

        let json_keys = remote.pull_records("ssh_keys", last_sync).await?;
        for json in json_keys {
            if let Ok(key) = crate::models::ssh::SSHKey::from_json(&json) {
                local_guard.save_ssh_key(&key).await?;
                stats.total_synced += 1;
            }
        }

        let json_cmd_groups = remote
            .pull_records("saved_command_groups", last_sync)
            .await?;
        for json in json_cmd_groups {
            if let Ok(group) = crate::models::saved_command::SavedCommandGroup::from_json(&json) {
                local_guard.save_saved_command_group(&group).await?;
                stats.total_synced += 1;
            }
        }

        let json_commands = remote.pull_records("saved_commands", last_sync).await?;
        for json in json_commands {
            if let Ok(command) = crate::models::saved_command::SavedCommand::from_json(&json) {
                local_guard.save_saved_command(&command).await?;
                stats.total_synced += 1;
            }
        }

        Ok(stats)
    }

    /// Internal bidirectional sync with conflict resolution
    async fn sync_internal(&self, config: &ExternalDatabaseConfig) -> DatabaseResult<SyncStats> {
        use crate::models::sync::ConflictResolutionStrategy;

        self.sync_manager.ensure_connection(config).await?;
        let remote = self.sync_manager.get_provider(&config.base.id).await?;

        let mut stats = SyncStats::default();

        let last_sync = self.get_last_sync_time(&config.base.id).await?;

        let strategy = {
            let db_service = self.database_service.lock().await;
            let local = db_service.get_local_database();
            let sync_settings = {
                let guard = local.read().await;
                guard.get_global_sync_settings().await?
            };
            drop(db_service); // Explicitly drop lock before proceeding
            sync_settings
                .map(|s| s.conflict_strategy)
                .unwrap_or(ConflictResolutionStrategy::Manual)
        };

        let profile_stats = self
            .sync_table_bidirectional(&remote, "ssh_profiles", last_sync, strategy)
            .await?;
        stats.merge(profile_stats);

        let group_stats = self
            .sync_table_bidirectional(&remote, "ssh_groups", last_sync, strategy)
            .await?;
        stats.merge(group_stats);

        let key_stats = self
            .sync_table_bidirectional(&remote, "ssh_keys", last_sync, strategy)
            .await?;
        stats.merge(key_stats);

        let cmd_group_stats = self
            .sync_table_bidirectional(&remote, "saved_command_groups", last_sync, strategy)
            .await?;
        stats.merge(cmd_group_stats);

        let cmd_stats = self
            .sync_table_bidirectional(&remote, "saved_commands", last_sync, strategy)
            .await?;
        stats.merge(cmd_stats);

        Ok(stats)
    }

    /// Sync a single table bidirectionally with conflict resolution
    async fn sync_table_bidirectional(
        &self,
        remote: &Arc<dyn crate::database::traits_sync::SyncTarget>,
        table: &str,
        last_sync: Option<DateTime<Utc>>,
        strategy: ConflictResolutionStrategy,
    ) -> DatabaseResult<SyncStats> {
        use crate::services::sync::SyncSerializable;
        use chrono::{DateTime, Utc};
        use serde_json::Value;
        use std::collections::HashMap;

        let mut stats = SyncStats::default();

        let db_service = self.database_service.lock().await;
        let local = db_service.get_local_database();
        drop(db_service);

        let local_records = {
            let local_guard = local.read().await;
            match table {
                "ssh_profiles" => {
                    let profiles = local_guard.find_all_ssh_profiles().await?;
                    profiles
                        .iter()
                        .filter_map(|p| p.to_json().ok())
                        .collect::<Vec<_>>()
                }
                "ssh_groups" => {
                    let groups = local_guard.find_all_ssh_groups().await?;
                    groups
                        .iter()
                        .filter_map(|g| g.to_json().ok())
                        .collect::<Vec<_>>()
                }
                "ssh_keys" => {
                    let keys = local_guard.find_all_ssh_keys().await?;
                    keys.iter()
                        .filter_map(|k| k.to_json().ok())
                        .collect::<Vec<_>>()
                }
                "saved_command_groups" => {
                    let groups = local_guard.find_all_saved_command_groups().await?;
                    groups
                        .iter()
                        .filter_map(|g| g.to_json().ok())
                        .collect::<Vec<_>>()
                }
                "saved_commands" => {
                    let commands = local_guard.find_all_saved_commands().await?;
                    commands
                        .iter()
                        .filter_map(|c| c.to_json().ok())
                        .collect::<Vec<_>>()
                }
                _ => vec![],
            }
        };

        let remote_records = remote.pull_records(table, last_sync).await?;

        let local_ids: Vec<String> = local_records
            .iter()
            .filter_map(|r| r.get("id").and_then(|v| v.as_str()).map(|s| s.to_string()))
            .collect();

        let remote_versions = remote.get_record_versions(table, local_ids.clone()).await?;

        let local_versions: HashMap<String, u64> = local_records
            .iter()
            .filter_map(|r| {
                let id = r.get("id")?.as_str()?.to_string();
                let version = r.get("version")?.as_u64()?;
                Some((id, version))
            })
            .collect();

        let mut records_to_push = Vec::new();
        let mut _records_to_pull: Vec<Value> = Vec::new();

        for local_record in local_records {
            if let Some(id) = local_record.get("id").and_then(|v| v.as_str()) {
                if let Some(&remote_version) = remote_versions.get(id) {
                    let local_version = local_versions.get(id).copied().unwrap_or(0);

                    if local_version > remote_version {
                        records_to_push.push(local_record);
                    } else if remote_version > local_version {
                        match strategy {
                            ConflictResolutionStrategy::LocalWins => {
                                records_to_push.push(local_record);
                                stats.conflicts_resolved += 1;
                            }
                            ConflictResolutionStrategy::RemoteWins => {
                                stats.conflicts_resolved += 1;
                            }
                            ConflictResolutionStrategy::LastWriteWins => {
                                let local_updated = local_record
                                    .get("updated_at")
                                    .and_then(|v| v.as_str())
                                    .and_then(|s| DateTime::parse_from_rfc3339(s).ok())
                                    .map(|dt| dt.with_timezone(&Utc));

                                if local_updated.is_some() {
                                    records_to_push.push(local_record);
                                }
                                stats.conflicts_resolved += 1;
                            }
                            ConflictResolutionStrategy::FirstWriteWins => {
                                let local_created = local_record
                                    .get("created_at")
                                    .and_then(|v| v.as_str())
                                    .and_then(|s| DateTime::parse_from_rfc3339(s).ok())
                                    .map(|dt| dt.with_timezone(&Utc));

                                if local_created.is_some() {
                                    records_to_push.push(local_record);
                                }
                                stats.conflicts_resolved += 1;
                            }
                            ConflictResolutionStrategy::Manual => {
                                continue;
                            }
                        }
                    }
                } else {
                    records_to_push.push(local_record);
                }
            }
        }

        if !records_to_push.is_empty() {
            let count = remote.push_records(table, records_to_push).await?;
            stats.total_synced += count;
        }

        for remote_record in remote_records {
            if let Some(id) = remote_record.get("id").and_then(|v| v.as_str()) {
                let remote_version = remote_record
                    .get("version")
                    .and_then(|v| v.as_u64())
                    .unwrap_or(0);
                let local_version = local_versions.get(id).copied().unwrap_or(0);

                if remote_version >= local_version {
                    let local_guard = local.write().await;

                    match table {
                        "ssh_profiles" => {
                            if let Ok(profile) =
                                crate::models::ssh::SSHProfile::from_json(&remote_record)
                            {
                                local_guard.save_ssh_profile(&profile).await?;
                                stats.total_synced += 1;
                            }
                        }
                        "ssh_groups" => {
                            if let Ok(group) =
                                crate::models::ssh::SSHGroup::from_json(&remote_record)
                            {
                                local_guard.save_ssh_group(&group).await?;
                                stats.total_synced += 1;
                            }
                        }
                        "ssh_keys" => {
                            if let Ok(key) = crate::models::ssh::SSHKey::from_json(&remote_record) {
                                local_guard.save_ssh_key(&key).await?;
                                stats.total_synced += 1;
                            }
                        }
                        _ => {}
                    }
                }
            }
        }

        Ok(stats)
    }

    /// Create a new sync log entry
    async fn create_sync_log(
        &self,
        config: &ExternalDatabaseConfig,
        direction: SyncDirection,
    ) -> DatabaseResult<SyncLog> {
        let db_service = self.database_service.lock().await;
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
        let db_service = self.database_service.lock().await;
        let local = db_service.get_local_database();
        let guard = local.write().await;
        guard.save_sync_log(log).await
    }

    /// Get last sync time for a database
    async fn get_last_sync_time(&self, database_id: &str) -> DatabaseResult<Option<DateTime<Utc>>> {
        let db_service = self.database_service.lock().await;
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
        let db_service = self.database_service.lock().await;
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

        let guard = local.write().await;
        guard.save_conflict_resolution(&conflict_resolution).await?;

        eprintln!(
            "[INFO] Saved conflict for manual resolution: {} ({})",
            conflict_resolution.entity_type, conflict_resolution.entity_id
        );

        Ok(())
    }

    /// Sync SSH Profiles with conflict detection
    #[allow(dead_code)]
    async fn sync_ssh_profiles(
        &self,
        local: &Arc<RwLock<crate::database::providers::sqlite::SQLiteProvider>>,
        remote: &Arc<dyn crate::database::traits::Database>,
        last_sync: Option<DateTime<Utc>>,
        strategy: ConflictResolutionStrategy,
    ) -> DatabaseResult<SyncStats> {
        let mut stats = SyncStats::default();

        let local_guard = local.read().await;
        let local_profiles = local_guard.find_all_ssh_profiles().await?;
        drop(local_guard);

        let remote_profiles = remote.find_all_ssh_profiles().await?;

        let local_data: Vec<_> = local_profiles
            .iter()
            .map(|p| (p.base.id.clone(), p.clone(), p.base.updated_at))
            .collect();

        let remote_data: Vec<_> = remote_profiles
            .iter()
            .map(|p| (p.base.id.clone(), p.clone(), p.base.updated_at))
            .collect();

        let conflicts = self.conflict_resolver.detect_conflicts(
            local_data,
            remote_data,
            last_sync,
            "SSHProfile".to_string(),
        );

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

        for profile in local_profiles {
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
    #[allow(dead_code)]
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
    #[allow(dead_code)]
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
    total_synced: usize,
    conflicts_resolved: usize,
    manual_conflicts: usize,
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
