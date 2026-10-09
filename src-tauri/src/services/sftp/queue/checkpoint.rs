// Copyright (C) 2026 Bùi Thanh Xuân (klpod221)
// SPDX-License-Identifier: GPL-3.0-or-later

use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::RwLock;

/// Resume checkpoint info for interrupted transfers
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct CheckpointInfo {
    pub transfer_id: String,
    pub confirmed_offset: u64,
    pub partial_path: String,
    pub total_bytes: u64,
    pub updated_at: chrono::DateTime<chrono::Utc>,
}

/// In-memory and persistent-ready checkpoint store
#[derive(Default, Clone)]
pub struct CheckpointStore {
    checkpoints: Arc<RwLock<HashMap<String, CheckpointInfo>>>,
}

impl CheckpointStore {
    pub fn new() -> Self {
        Self {
            checkpoints: Arc::new(RwLock::new(HashMap::new())),
        }
    }

    /// Record or update checkpoint
    pub async fn update(&self, transfer_id: &str, confirmed_offset: u64, partial_path: &str, total_bytes: u64) {
        let mut map = self.checkpoints.write().await;
        map.insert(
            transfer_id.to_string(),
            CheckpointInfo {
                transfer_id: transfer_id.to_string(),
                confirmed_offset,
                partial_path: partial_path.to_string(),
                total_bytes,
                updated_at: chrono::Utc::now(),
            },
        );
    }

    /// Get checkpoint info
    pub async fn get(&self, transfer_id: &str) -> Option<CheckpointInfo> {
        let map = self.checkpoints.read().await;
        map.get(transfer_id).cloned()
    }

    /// Remove checkpoint on completion or cancellation
    pub async fn remove(&self, transfer_id: &str) {
        let mut map = self.checkpoints.write().await;
        map.remove(transfer_id);
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_checkpoint_store_lifecycle() {
        let store = CheckpointStore::new();
        assert!(store.get("task-1").await.is_none());

        store.update("task-1", 1024, "/tmp/foo.part", 4096).await;
        let cp = store.get("task-1").await.expect("checkpoint exists");
        assert_eq!(cp.confirmed_offset, 1024);
        assert_eq!(cp.total_bytes, 4096);
        assert_eq!(cp.partial_path, "/tmp/foo.part");

        store.remove("task-1").await;
        assert!(store.get("task-1").await.is_none());
    }
}
