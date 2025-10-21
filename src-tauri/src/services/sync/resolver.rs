
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use std::cmp::Ordering;

use crate::database::error::DatabaseResult;
use crate::models::sync::conflict::ConflictResolutionStrategy;

/// Represents a conflict between local and remote data
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct DataConflict<T> {
    pub entity_type: String,
    pub entity_id: String,
    pub local_data: T,
    pub local_updated_at: DateTime<Utc>,
    pub remote_data: T,
    pub remote_updated_at: DateTime<Utc>,
}

/// Result of conflict resolution
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub enum ConflictResolution<T> {
    UseLocal(T),
    UseRemote(T),
    UseMerged(T),
    RequiresManual(DataConflict<T>),
}

/// Conflict resolver with multiple strategies
pub struct ConflictResolver;

impl ConflictResolver {
    pub fn new() -> Self {
        Self
    }

    /// Resolve a conflict using the specified strategy
    pub fn resolve<T: Clone>(
        &self,
        conflict: DataConflict<T>,
        strategy: ConflictResolutionStrategy,
    ) -> DatabaseResult<ConflictResolution<T>> {
        match strategy {
            ConflictResolutionStrategy::LastWriteWins => Ok(self.resolve_last_write_wins(conflict)),
            ConflictResolutionStrategy::FirstWriteWins => {
                Ok(self.resolve_first_write_wins(conflict))
            }
            ConflictResolutionStrategy::LocalWins => Ok(self.resolve_local_wins(conflict)),
            ConflictResolutionStrategy::RemoteWins => Ok(self.resolve_remote_wins(conflict)),
            ConflictResolutionStrategy::Manual => Ok(self.resolve_manual(conflict)),
        }
    }

    /// Resolve multiple conflicts
    pub fn resolve_batch<T: Clone>(
        &self,
        conflicts: Vec<DataConflict<T>>,
        strategy: ConflictResolutionStrategy,
    ) -> DatabaseResult<Vec<ConflictResolution<T>>> {
        conflicts
            .into_iter()
            .map(|conflict| self.resolve(conflict, strategy))
            .collect()
    }

    /// Last write wins strategy - use the most recently updated data
    fn resolve_last_write_wins<T: Clone>(
        &self,
        conflict: DataConflict<T>,
    ) -> ConflictResolution<T> {
        match conflict.local_updated_at.cmp(&conflict.remote_updated_at) {
            Ordering::Greater => ConflictResolution::UseLocal(conflict.local_data),
            Ordering::Less => ConflictResolution::UseRemote(conflict.remote_data),
            Ordering::Equal => {
                // If timestamps are equal, prefer local
                ConflictResolution::UseLocal(conflict.local_data)
            }
        }
    }

    /// First write wins strategy - use the oldest data
    fn resolve_first_write_wins<T: Clone>(
        &self,
        conflict: DataConflict<T>,
    ) -> ConflictResolution<T> {
        match conflict.local_updated_at.cmp(&conflict.remote_updated_at) {
            Ordering::Less => ConflictResolution::UseLocal(conflict.local_data),
            Ordering::Greater => ConflictResolution::UseRemote(conflict.remote_data),
            Ordering::Equal => {
                // If timestamps are equal, prefer local
                ConflictResolution::UseLocal(conflict.local_data)
            }
        }
    }

    /// Local wins strategy - always prefer local data
    fn resolve_local_wins<T: Clone>(&self, conflict: DataConflict<T>) -> ConflictResolution<T> {
        ConflictResolution::UseLocal(conflict.local_data)
    }

    /// Remote wins strategy - always prefer remote data
    fn resolve_remote_wins<T: Clone>(&self, conflict: DataConflict<T>) -> ConflictResolution<T> {
        ConflictResolution::UseRemote(conflict.remote_data)
    }

    /// Manual resolution - return conflict for user decision
    fn resolve_manual<T: Clone>(&self, conflict: DataConflict<T>) -> ConflictResolution<T> {
        ConflictResolution::RequiresManual(conflict)
    }

    /// Check if two entities have a conflict based on timestamps
    pub fn has_conflict(
        local_updated_at: DateTime<Utc>,
        remote_updated_at: DateTime<Utc>,
        last_sync_at: Option<DateTime<Utc>>,
    ) -> bool {
        // If never synced, consider any difference as conflict
        let last_sync = match last_sync_at {
            Some(sync_time) => sync_time,
            None => return local_updated_at != remote_updated_at,
        };

        // Conflict exists if both were modified after last sync
        local_updated_at > last_sync && remote_updated_at > last_sync
    }

    /// Detect conflicts between local and remote data lists
    pub fn detect_conflicts<T>(
        &self,
        local_items: Vec<(String, T, DateTime<Utc>)>,
        remote_items: Vec<(String, T, DateTime<Utc>)>,
        last_sync_at: Option<DateTime<Utc>>,
        entity_type: String,
    ) -> Vec<DataConflict<T>>
    where
        T: Clone,
    {
        let mut conflicts = Vec::new();

        // Create lookup maps
        let local_map: std::collections::HashMap<_, _> = local_items
            .into_iter()
            .map(|(id, data, updated)| (id, (data, updated)))
            .collect();

        let remote_map: std::collections::HashMap<_, _> = remote_items
            .into_iter()
            .map(|(id, data, updated)| (id, (data, updated)))
            .collect();

        // Check for conflicts
        for (id, (local_data, local_updated)) in local_map.iter() {
            if let Some((remote_data, remote_updated)) = remote_map.get(id) {
                if Self::has_conflict(*local_updated, *remote_updated, last_sync_at) {
                    conflicts.push(DataConflict {
                        entity_type: entity_type.clone(),
                        entity_id: id.clone(),
                        local_data: local_data.clone(),
                        local_updated_at: *local_updated,
                        remote_data: remote_data.clone(),
                        remote_updated_at: *remote_updated,
                    });
                }
            }
        }

        conflicts
    }
}

impl Default for ConflictResolver {
    fn default() -> Self {
        Self::new()
    }
}

/// Helper for manual conflict resolution
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ManualResolutionChoice {
    pub conflict_id: String,
    pub choice: ManualChoice,
}

#[allow(dead_code)]
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub enum ManualChoice {
    UseLocal,
    UseRemote,
    #[serde(rename_all = "camelCase")]
    UseMerged {
        merged_data: serde_json::Value,
    },
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_last_write_wins() {
        let resolver = ConflictResolver::new();
        let now = Utc::now();
        let earlier = now - chrono::Duration::hours(1);

        let conflict = DataConflict {
            entity_type: "test".to_string(),
            entity_id: "1".to_string(),
            local_data: "local".to_string(),
            local_updated_at: now,
            remote_data: "remote".to_string(),
            remote_updated_at: earlier,
        };

        let result = resolver.resolve(conflict, ConflictResolutionStrategy::LastWriteWins);
        assert!(matches!(result, Ok(ConflictResolution::UseLocal(_))));
    }

    #[test]
    fn test_first_write_wins() {
        let resolver = ConflictResolver::new();
        let now = Utc::now();
        let earlier = now - chrono::Duration::hours(1);

        let conflict = DataConflict {
            entity_type: "test".to_string(),
            entity_id: "1".to_string(),
            local_data: "local".to_string(),
            local_updated_at: now,
            remote_data: "remote".to_string(),
            remote_updated_at: earlier,
        };

        let result = resolver.resolve(conflict, ConflictResolutionStrategy::FirstWriteWins);
        assert!(matches!(result, Ok(ConflictResolution::UseRemote(_))));
    }

    #[test]
    fn test_has_conflict() {
        let now = Utc::now();
        let before_sync = now - chrono::Duration::hours(2);
        let after_sync = now - chrono::Duration::minutes(30);
        let last_sync = now - chrono::Duration::hours(1);

        // Both modified after sync = conflict
        assert!(ConflictResolver::has_conflict(
            after_sync,
            after_sync,
            Some(last_sync)
        ));

        // Only one modified after sync = no conflict
        assert!(!ConflictResolver::has_conflict(
            before_sync,
            after_sync,
            Some(last_sync)
        ));

        // Never synced, different times = conflict
        assert!(ConflictResolver::has_conflict(now, before_sync, None));
    }
}
