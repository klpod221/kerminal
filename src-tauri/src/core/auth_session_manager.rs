use chrono::{DateTime, Utc};
use std::sync::Arc;
use tokio::sync::{Mutex, broadcast};
use tokio::time::{interval, Duration};
use serde::{Deserialize, Serialize};

use crate::database::{
    service::DatabaseService,
    error::DatabaseResult,
};

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "type")]
pub enum AuthEvent {
    SessionUnlocked {
        timestamp: DateTime<Utc>,
        via_auto_unlock: bool,
    },
    SessionLocked {
        timestamp: DateTime<Utc>,
        reason: SessionLockReason,
    },
    AutoUnlockAttempted {
        timestamp: DateTime<Utc>,
        success: bool,
        error: Option<String>,
    },
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum SessionLockReason {
    Manual,
    Timeout,
    Error(String),
}

pub struct AuthSessionManager {
    database_service: Arc<Mutex<DatabaseService>>,
    event_sender: broadcast::Sender<AuthEvent>,
    session_check_handle: Option<tokio::task::JoinHandle<()>>,
}

impl AuthSessionManager {
    pub fn new(database_service: Arc<Mutex<DatabaseService>>) -> Self {
        let (event_sender, _) = broadcast::channel(100);

        Self {
            database_service,
            event_sender,
            session_check_handle: None,
        }
    }

    pub async fn initialize(&mut self) -> DatabaseResult<()> {
        self.attempt_auto_unlock().await?;
        self.start_session_monitoring().await?;
        Ok(())
    }

    pub async fn attempt_auto_unlock(&self) -> DatabaseResult<bool> {
        let db_guard = self.database_service.lock().await;

        let config = db_guard.get_master_password_config().await?;
        if !config.auto_unlock {
            return Ok(false);
        }

        let success = match db_guard.try_auto_unlock().await {
            Ok(success) => {
                let _ = self.event_sender.send(AuthEvent::AutoUnlockAttempted {
                    timestamp: Utc::now(),
                    success,
                    error: None,
                });

                if success {
                    let _ = self.event_sender.send(AuthEvent::SessionUnlocked {
                        timestamp: Utc::now(),
                        via_auto_unlock: true,
                    });
                }

                success
            }
            Err(e) => {
                let _ = self.event_sender.send(AuthEvent::AutoUnlockAttempted {
                    timestamp: Utc::now(),
                    success: false,
                    error: Some(e.to_string()),
                });
                false
            }
        };

        Ok(success)
    }

    pub async fn start_session_monitoring(&mut self) -> DatabaseResult<()> {
        self.stop_session_monitoring().await;

        let db_service = Arc::clone(&self.database_service);
        let event_sender = self.event_sender.clone();

        let session_check_task = tokio::spawn(async move {
            let mut check_interval = interval(Duration::from_secs(30));

            loop {
                check_interval.tick().await;

                let db_guard = db_service.lock().await;

                match db_guard.is_session_valid().await {
                    Ok(is_valid) => {
                        if !is_valid {
                            // Session was expired and auto-locked by backend
                            let _ = event_sender.send(AuthEvent::SessionLocked {
                                timestamp: Utc::now(),
                                reason: SessionLockReason::Timeout,
                            });
                        }
                    }
                    Err(_) => {
                        // Error checking session - consider it locked
                        let _ = event_sender.send(AuthEvent::SessionLocked {
                            timestamp: Utc::now(),
                            reason: SessionLockReason::Error("Session check failed".to_string()),
                        });
                        break; // Stop monitoring on error
                    }
                }
            }
        });

        self.session_check_handle = Some(session_check_task);
        Ok(())
    }

    /// Stop session monitoring
    pub async fn stop_session_monitoring(&mut self) {
        if let Some(handle) = self.session_check_handle.take() {
            handle.abort();
        }
    }

    /// Handle manual unlock
    pub async fn on_session_unlocked(&self) -> DatabaseResult<()> {
        // Broadcast session unlocked event
        let _ = self.event_sender.send(AuthEvent::SessionUnlocked {
            timestamp: Utc::now(),
            via_auto_unlock: false,
        });

        Ok(())
    }

    /// Handle manual lock
    pub async fn on_session_locked(&self, reason: SessionLockReason) -> DatabaseResult<()> {
        // Broadcast session locked event
        let _ = self.event_sender.send(AuthEvent::SessionLocked {
            timestamp: Utc::now(),
            reason,
        });

        Ok(())
    }


}

impl Drop for AuthSessionManager {
    fn drop(&mut self) {
        // Ensure monitoring is stopped when the manager is dropped
        if let Some(handle) = self.session_check_handle.take() {
            handle.abort();
        }
    }
}
