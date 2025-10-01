use std::sync::Arc;
use tokio::sync::RwLock;
use tokio::time::Duration;

#[derive(Debug)]
#[allow(dead_code)]
pub struct SyncScheduler {
    interval_minutes: u32,
    is_running: Arc<RwLock<bool>>,
}

#[allow(dead_code)]
impl SyncScheduler {
    pub fn new(interval_minutes: u32) -> Self {
        Self {
            interval_minutes,
            is_running: Arc::new(RwLock::new(false)),
        }
    }

    pub async fn start(&self) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
        let mut is_running = self.is_running.write().await;
        if *is_running {
            return Ok(()); // Already running
        }
        *is_running = true;

        let duration = Duration::from_secs((self.interval_minutes * 60) as u64);

        let _is_running_clone = Arc::clone(&self.is_running);
        tokio::spawn(async move {
            let mut sync_interval = tokio::time::interval(duration);
            loop {
                sync_interval.tick().await;
            }
        });

        Ok(())
    }

    pub async fn stop(&self) {
        let mut is_running = self.is_running.write().await;
        *is_running = false;
    }

    pub async fn is_running(&self) -> bool {
        let is_running = self.is_running.read().await;
        *is_running
    }
}

impl Default for SyncScheduler {
    fn default() -> Self {
        Self::new(15) // Default to 15 minutes
    }
}
