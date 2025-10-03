use crate::core::terminal::{TerminalFactory, TerminalWrapper};
use crate::database::service::DatabaseService;
use crate::error::AppError;
use crate::models::terminal::{
    CreateTerminalRequest, CreateTerminalResponse, ResizeTerminalRequest, TerminalData,
    TerminalExited, TerminalInfo, TerminalTitleChanged, WriteTerminalRequest,
};
use crate::services::buffer_manager::TerminalBufferManager;
use crate::services::ssh::SSHKeyService;
use std::collections::HashMap;
use std::sync::Arc;
use tauri::{AppHandle, Emitter};
use tokio::sync::{mpsc, Mutex, RwLock};
use uuid::Uuid;

pub struct TerminalManager {
    terminals: Arc<RwLock<HashMap<String, Arc<Mutex<TerminalWrapper>>>>>,
    output_senders: Arc<RwLock<HashMap<String, mpsc::UnboundedSender<Vec<u8>>>>>,
    output_receiver: Arc<Mutex<Option<mpsc::UnboundedReceiver<TerminalData>>>>,
    output_sender: mpsc::UnboundedSender<TerminalData>,
    buffer_manager: Arc<TerminalBufferManager>,
    database_service: Arc<Mutex<DatabaseService>>,
    ssh_key_service: Option<Arc<Mutex<SSHKeyService>>>,
}

impl TerminalManager {
    pub fn new(database_service: Arc<Mutex<DatabaseService>>) -> Self {
        let (output_sender, output_receiver) = mpsc::unbounded_channel();

        Self {
            terminals: Arc::new(RwLock::new(HashMap::new())),
            output_senders: Arc::new(RwLock::new(HashMap::new())),
            output_receiver: Arc::new(Mutex::new(Some(output_receiver))),
            output_sender,
            buffer_manager: Arc::new(TerminalBufferManager::default()),
            database_service,
            ssh_key_service: None,
        }
    }

    pub fn new_with_ssh_key_service(
        database_service: Arc<Mutex<DatabaseService>>,
        ssh_key_service: Arc<Mutex<SSHKeyService>>,
    ) -> Self {
        let (output_sender, output_receiver) = mpsc::unbounded_channel();

        Self {
            terminals: Arc::new(RwLock::new(HashMap::new())),
            output_senders: Arc::new(RwLock::new(HashMap::new())),
            output_receiver: Arc::new(Mutex::new(Some(output_receiver))),
            output_sender,
            buffer_manager: Arc::new(TerminalBufferManager::default()),
            database_service,
            ssh_key_service: Some(ssh_key_service),
        }
    }

    pub async fn create_terminal(
        &self,
        request: CreateTerminalRequest,
        app_handle: Option<AppHandle>,
    ) -> Result<CreateTerminalResponse, AppError> {
        let terminal_id = Uuid::new_v4().to_string();

        let mut terminal = TerminalFactory::create_terminal(
            terminal_id.clone(),
            request.config.clone(),
            Some(self.database_service.clone()),
        )
        .await?;

        // Resolve SSH key if needed
        let resolved_key = if matches!(
            request.config.terminal_type,
            crate::models::terminal::TerminalType::SSH
        ) {
            if let Some(ssh_key_service) = &self.ssh_key_service {
                if let Some(ssh_profile_id) = &request.config.ssh_profile_id {
                    // Get SSH profile to check auth data
                    let ssh_profile = {
                        let db_service = self.database_service.lock().await;
                        db_service
                            .get_ssh_profile(ssh_profile_id)
                            .await
                            .map_err(|e| AppError::Database(e.to_string()))?
                    };

                    // Resolve key if it's a KeyReference
                    if let crate::models::ssh::profile::AuthData::KeyReference { key_id } =
                        &ssh_profile.auth_data
                    {
                        let key_service = ssh_key_service.lock().await;
                        Some(
                            key_service
                                .resolve_key_for_auth(key_id)
                                .await
                                .map_err(|e| AppError::Database(e.to_string()))?,
                        )
                    } else {
                        None
                    }
                } else {
                    None
                }
            } else {
                None
            }
        } else {
            None
        };

        // Attempt connection with error handling
        let connect_result = if let Some(resolved_key) = resolved_key {
            terminal
                .connect_with_resolved_data(Some(resolved_key))
                .await
        } else {
            terminal.connect().await
        };

        if let Err(e) = connect_result {
            // Send error event to frontend if we have an app handle
            if let Some(handle) = &app_handle {
                let error_event = TerminalExited {
                    terminal_id: terminal_id.clone(),
                    exit_code: Some(1),
                    reason: Some("error".to_string()),
                };
                let _ = handle.emit("terminal-exited", &error_event);
            }

            return Err(e);
        } else {
            // Connection successful - emit success event for SSH terminals
            if matches!(
                request.config.terminal_type,
                crate::models::terminal::TerminalType::SSH
            ) {
                if let Some(handle) = &app_handle {
                    let success_event = serde_json::json!({
                        "terminalId": terminal_id
                    });
                    let _ = handle.emit("ssh-connected", &success_event);
                }
            }
        }
        let (tx, mut rx) = mpsc::unbounded_channel::<Vec<u8>>();
        let (title_tx, mut title_rx) = mpsc::unbounded_channel::<String>();
        let (exit_tx, mut exit_rx) = mpsc::unbounded_channel::<TerminalExited>();

        {
            let mut senders = self.output_senders.write().await;
            senders.insert(terminal_id.clone(), tx.clone());
        }

        terminal
            .start_read_loop(tx, Some(title_tx), Some(exit_tx))
            .await?;

        let terminal_id_clone = terminal_id.clone();
        let output_sender = self.output_sender.clone();
        let app_handle_clone = app_handle.clone();
        let buffer_manager_clone = self.buffer_manager.clone();
        tokio::spawn(async move {
            while let Some(data) = rx.recv().await {
                let terminal_data = TerminalData {
                    terminal_id: terminal_id_clone.clone(),
                    data: data.clone(),
                };

                if let Ok(data_str) = String::from_utf8(data) {
                    buffer_manager_clone
                        .save_data(&terminal_id_clone, &data_str)
                        .await;
                }

                if let Some(handle) = &app_handle_clone {
                    let _ = handle.emit("terminal-output", &terminal_data);
                }

                if output_sender.send(terminal_data).is_err() {
                    break;
                }
            }
        });

        let terminal_id_clone = terminal_id.clone();
        let app_handle_clone = app_handle.clone();
        tokio::spawn(async move {
            while let Some(new_title) = title_rx.recv().await {
                let title_event = TerminalTitleChanged {
                    terminal_id: terminal_id_clone.clone(),
                    title: new_title,
                };

                if let Some(handle) = &app_handle_clone {
                    let _ = handle.emit("terminal-title-changed", &title_event);
                }
            }
        });

        let app_handle_clone = app_handle.clone();
        tokio::spawn(async move {
            while let Some(exit_event) = exit_rx.recv().await {
                if let Some(handle) = &app_handle_clone {
                    let _ = handle.emit("terminal-exited", &exit_event);
                }
            }
        });
        let terminal_info = TerminalInfo {
            id: terminal_id.clone(),
            config: request.config,
            state: terminal.get_state(),
            created_at: chrono::Utc::now(),
            title: request.title,
        };

        {
            let mut terminals = self.terminals.write().await;
            terminals.insert(terminal_id.clone(), Arc::new(Mutex::new(terminal)));
        }

        Ok(CreateTerminalResponse {
            terminal_id,
            info: terminal_info,
        })
    }

    pub async fn write_to_terminal(&self, request: WriteTerminalRequest) -> Result<(), AppError> {
        let terminals = self.terminals.read().await;

        if let Some(terminal) = terminals.get(&request.terminal_id) {
            let mut terminal_guard = terminal.lock().await;
            terminal_guard.write(request.data.as_bytes()).await?;
            Ok(())
        } else {
            Err(AppError::TerminalNotFound(request.terminal_id))
        }
    }

    pub async fn resize_terminal(&self, request: ResizeTerminalRequest) -> Result<(), AppError> {
        let terminals = self.terminals.read().await;

        if let Some(terminal) = terminals.get(&request.terminal_id) {
            let mut terminal_guard = terminal.lock().await;
            terminal_guard.resize(request.cols, request.rows).await?;
            Ok(())
        } else {
            Err(AppError::TerminalNotFound(request.terminal_id))
        }
    }

    pub async fn close_terminal(&self, terminal_id: String) -> Result<(), AppError> {
        let terminal = {
            let mut terminals = self.terminals.write().await;
            terminals.remove(&terminal_id)
        };

        {
            let mut senders = self.output_senders.write().await;
            senders.remove(&terminal_id);
        }

        self.buffer_manager.remove_buffer(&terminal_id).await;

        if let Some(terminal) = terminal {
            let mut terminal_guard = terminal.lock().await;
            terminal_guard.disconnect().await?;
        }

        Ok(())
    }

    pub async fn get_terminal_info(&self, terminal_id: String) -> Result<TerminalInfo, AppError> {
        let terminals = self.terminals.read().await;

        if let Some(terminal) = terminals.get(&terminal_id) {
            let terminal_guard = terminal.lock().await;
            Ok(TerminalInfo {
                id: terminal_id,
                config: terminal_guard.get_config().clone(),
                state: terminal_guard.get_state(),
                created_at: chrono::Utc::now(),
                title: None,
            })
        } else {
            Err(AppError::TerminalNotFound(terminal_id))
        }
    }

    pub async fn list_terminals(&self) -> Result<Vec<TerminalInfo>, AppError> {
        let terminals = self.terminals.read().await;
        let mut terminal_infos = Vec::new();

        for (terminal_id, terminal) in terminals.iter() {
            let terminal_guard = terminal.lock().await;
            terminal_infos.push(TerminalInfo {
                id: terminal_id.clone(),
                config: terminal_guard.get_config().clone(),
                state: terminal_guard.get_state(),
                created_at: chrono::Utc::now(),
                title: None,
            });
        }

        Ok(terminal_infos)
    }

    pub fn get_buffer_manager(&self) -> Arc<TerminalBufferManager> {
        self.buffer_manager.clone()
    }
}
