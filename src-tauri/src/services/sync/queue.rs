use std::collections::VecDeque;
use std::sync::Arc;
use tokio::sync::{Mutex, Semaphore};

/// A request to sync a database
#[allow(dead_code)]
#[derive(Debug, Clone)]
pub struct SyncRequest {
    pub database_id: String,
    pub direction: crate::models::sync::log::SyncDirection,
}

/// Sync queue to manage and serialize sync operations
/// Prevents concurrent syncs to the same database
pub struct SyncQueue {
    /// Queue of pending sync requests
    pending: Arc<Mutex<VecDeque<SyncRequest>>>,
    /// Semaphore to limit max concurrent syncs (default: 1)
    semaphore: Arc<Semaphore>,
    /// Track currently syncing database IDs
    in_progress: Arc<Mutex<Vec<String>>>,
}

impl SyncQueue {
    /// Create a new sync queue with specified max concurrent syncs
    pub fn new(max_concurrent: usize) -> Self {
        Self {
            pending: Arc::new(Mutex::new(VecDeque::new())),
            semaphore: Arc::new(Semaphore::new(max_concurrent)),
            in_progress: Arc::new(Mutex::new(Vec::new())),
        }
    }

    /// Enqueue a sync request
    /// Returns true if enqueued, false if already in queue or in progress
    #[allow(dead_code)]
    pub async fn enqueue(&self, request: SyncRequest) -> bool {
        let in_progress = self.in_progress.lock().await;

        // Check if already syncing this database
        if in_progress.contains(&request.database_id) {
            return false;
        }

        let mut pending = self.pending.lock().await;

        // Check if already in queue
        if pending.iter().any(|r| r.database_id == request.database_id) {
            return false;
        }

        pending.push_back(request);
        true
    }

    /// Try to acquire a sync slot for the given database
    /// Returns a guard that should be held while syncing
    pub async fn acquire(&self, database_id: &str) -> Option<SyncGuard> {
        // Try to acquire semaphore permit
        let permit = self.semaphore.clone().try_acquire_owned().ok()?;

        // Mark as in progress
        let mut in_progress = self.in_progress.lock().await;
        if in_progress.contains(&database_id.to_string()) {
            // Already in progress, release permit
            drop(permit);
            return None;
        }
        in_progress.push(database_id.to_string());

        // Remove from pending queue
        let mut pending = self.pending.lock().await;
        pending.retain(|r| r.database_id != database_id);

        Some(SyncGuard {
            database_id: database_id.to_string(),
            in_progress: self.in_progress.clone(),
            _permit: permit,
        })
    }

    /// Check if a database is currently syncing
    #[allow(dead_code)]
    pub async fn is_syncing(&self, database_id: &str) -> bool {
        let in_progress = self.in_progress.lock().await;
        in_progress.contains(&database_id.to_string())
    }

    /// Get the number of pending sync requests
    #[allow(dead_code)]
    pub async fn pending_count(&self) -> usize {
        let pending = self.pending.lock().await;
        pending.len()
    }

    /// Get the number of currently active syncs
    #[allow(dead_code)]
    pub async fn active_count(&self) -> usize {
        let in_progress = self.in_progress.lock().await;
        in_progress.len()
    }
}

/// Guard that represents an active sync operation
/// Automatically releases the sync slot when dropped
pub struct SyncGuard {
    database_id: String,
    in_progress: Arc<Mutex<Vec<String>>>,
    #[allow(dead_code)]
    _permit: tokio::sync::OwnedSemaphorePermit,
}

impl Drop for SyncGuard {
    fn drop(&mut self) {
        let database_id = self.database_id.clone();
        let in_progress = self.in_progress.clone();

        // Use block_on for synchronous drop
        // This is safe because we're just updating a small mutex
        tokio::spawn(async move {
            let mut guard = in_progress.lock().await;
            guard.retain(|id| id != &database_id);
        });
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::models::sync::log::SyncDirection;

    #[tokio::test]
    async fn test_sync_queue_enqueue() {
        let queue = SyncQueue::new(1);

        let request = SyncRequest {
            database_id: "test-db".to_string(),
            direction: SyncDirection::Bidirectional,
        };

        // First enqueue should succeed
        assert!(queue.enqueue(request.clone()).await);

        // Second enqueue of same database should fail
        assert!(!queue.enqueue(request.clone()).await);

        assert_eq!(queue.pending_count().await, 1);
    }

    #[tokio::test]
    async fn test_sync_queue_acquire() {
        let queue = SyncQueue::new(1);

        // Acquire should succeed
        let guard1 = queue.acquire("db1").await;
        assert!(guard1.is_some());

        // Second acquire should fail (max concurrent = 1)
        let guard2 = queue.acquire("db2").await;
        assert!(guard2.is_none());

        // After dropping guard1, should be able to acquire again
        drop(guard1);

        // Need to wait for async cleanup
        tokio::time::sleep(tokio::time::Duration::from_millis(10)).await;

        let guard3 = queue.acquire("db2").await;
        assert!(guard3.is_some());
    }
}
