// Copyright (C) 2026 Bùi Thanh Xuân (klpod221)
// SPDX-License-Identifier: GPL-3.0-or-later

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use std::sync::Arc;
use tokio::sync::{broadcast, Mutex};
use tokio::time::{interval, Duration};

use crate::database::{error::DatabaseResult, service::DatabaseService};
use crate::events::CoreEventEmitter;

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
    emitter: Option<Arc<dyn CoreEventEmitter>>,
}

impl AuthSessionManager {
    pub fn new(database_service: Arc<Mutex<DatabaseService>>) -> Self {
        let (event_sender, _) = broadcast::channel(100);

        Self {
            database_service,
            event_sender,
            session_check_handle: None,
            emitter: None,
        }
    }

    pub fn set_emitter(&mut self, emitter: Arc<dyn CoreEventEmitter>) {
        self.emitter = Some(emitter);
    }

    fn emit_event(&self, event_name: &str, payload: &impl serde::Serialize) {
        if let Some(emitter) = &self.emitter {
            if let Ok(val) = serde_json::to_value(payload) {
                emitter.emit_event(event_name, val);
            }
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
                    let event = AuthEvent::SessionUnlocked {
                        timestamp: Utc::now(),
                        via_auto_unlock: true,
                    };
                    let _ = self.event_sender.send(event.clone());
                    self.emit_event("auth_session_unlocked", &event);
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
        let emitter = self.emitter.clone();

        let session_check_task = crate::utils::spawn(async move {
            let mut check_interval = interval(Duration::from_secs(30));

            loop {
                check_interval.tick().await;

                let db_guard = db_service.lock().await;

                match db_guard.is_session_valid().await {
                    Ok(is_valid) => {
                        if !is_valid {
                            // Lock session first
                            db_guard.lock_session().await;

                            let event = AuthEvent::SessionLocked {
                                timestamp: Utc::now(),
                                reason: SessionLockReason::Timeout,
                            };
                            let _ = event_sender.send(event.clone());

                            if let Some(emitter) = &emitter {
                                if let Ok(val) = serde_json::to_value(&event) {
                                    emitter.emit_event("auth_session_locked", val);
                                }
                                emitter.emit_event(
                                    "auth_session_updated",
                                    serde_json::json!({
                                        "sessionActive": false,
                                        "sessionExpiresAt": serde_json::Value::Null,
                                    }),
                                );
                            }
                        }
                    }
                    Err(_) => {
                        let event = AuthEvent::SessionLocked {
                            timestamp: Utc::now(),
                            reason: SessionLockReason::Error("Session check failed".to_string()),
                        };
                        let _ = event_sender.send(event.clone());

                        if let Some(emitter) = &emitter {
                            if let Ok(val) = serde_json::to_value(&event) {
                                emitter.emit_event("auth_session_locked", val);
                            }
                        }

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
        let event = AuthEvent::SessionUnlocked {
            timestamp: Utc::now(),
            via_auto_unlock: false,
        };
        let _ = self.event_sender.send(event.clone());
        self.emit_event("auth_session_unlocked", &event);
        Ok(())
    }

    /// Handle manual lock
    pub async fn on_session_locked(&self, reason: SessionLockReason) -> DatabaseResult<()> {
        let event = AuthEvent::SessionLocked {
            timestamp: Utc::now(),
            reason,
        };
        let _ = self.event_sender.send(event.clone());
        self.emit_event("auth_session_locked", &event);
        Ok(())
    }
}

impl Drop for AuthSessionManager {
    fn drop(&mut self) {
        if let Some(handle) = self.session_check_handle.take() {
            handle.abort();
        }
    }
}
