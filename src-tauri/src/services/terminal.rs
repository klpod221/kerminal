use crate::core::terminal::{TerminalFactory, TerminalWrapper};
use crate::database::service::DatabaseService;
use crate::error::AppError;
use crate::models::terminal::{
    CreateTerminalRequest, CreateTerminalResponse, ResizeTerminalRequest, TerminalData,
    TerminalExited, TerminalInfo, TerminalTitleChanged, WriteTerminalRequest,
};
use crate::services::buffer_manager::TerminalBufferManager;
use std::collections::HashMap;
use std::sync::Arc;
use tauri::{AppHandle, Emitter};
use tokio::sync::{mpsc, Mutex, RwLock};
use uuid::Uuid;

/// Manager for handling multiple terminal instances
pub struct TerminalManager {
    terminals: Arc<RwLock<HashMap<String, Arc<Mutex<TerminalWrapper>>>>>,
    output_senders: Arc<RwLock<HashMap<String, mpsc::UnboundedSender<Vec<u8>>>>>,
    output_receiver: Arc<Mutex<Option<mpsc::UnboundedReceiver<TerminalData>>>>,
    output_sender: mpsc::UnboundedSender<TerminalData>,
    buffer_manager: Arc<TerminalBufferManager>,
    database_service: Arc<Mutex<DatabaseService>>,
}

impl TerminalManager {
    /// Create a new terminal manager
    pub fn new(database_service: Arc<Mutex<DatabaseService>>) -> Self {
        let (output_sender, output_receiver) = mpsc::unbounded_channel();

        Self {
            terminals: Arc::new(RwLock::new(HashMap::new())),
            output_senders: Arc::new(RwLock::new(HashMap::new())),
            output_receiver: Arc::new(Mutex::new(Some(output_receiver))),
            output_sender,
            buffer_manager: Arc::new(TerminalBufferManager::default()),
            database_service,
        }
    }

    /// Create a new terminal and return its ID
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

        // Connect to the terminal
        terminal.connect().await?;

        // Create output channel for this terminal
        let (tx, mut rx) = mpsc::unbounded_channel::<Vec<u8>>();

        // Create title change channel for this terminal
        let (title_tx, mut title_rx) = mpsc::unbounded_channel::<String>();

        // Create exit channel for this terminal
        let (exit_tx, mut exit_rx) = mpsc::unbounded_channel::<TerminalExited>();

        // Store the sender for this terminal
        {
            let mut senders = self.output_senders.write().await;
            senders.insert(terminal_id.clone(), tx.clone());
        }

        // Start the read loop for this terminal
        terminal
            .start_read_loop(tx, Some(title_tx), Some(exit_tx))
            .await?;

        // Spawn task to forward terminal output to the main output channel
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

                // Convert data to string and save to buffer
                if let Ok(data_str) = String::from_utf8(data) {
                    buffer_manager_clone
                        .save_data(&terminal_id_clone, &data_str)
                        .await;
                }

                // Emit event to frontend if app handle is available
                if let Some(handle) = &app_handle_clone {
                    if let Err(e) = handle.emit("terminal-output", &terminal_data) {
                        eprintln!("Failed to emit terminal output event: {}", e);
                    }
                }

                if output_sender.send(terminal_data).is_err() {
                    // Main channel closed, stop forwarding
                    break;
                }
            }
        });

        // Spawn task to handle title changes
        let terminal_id_clone = terminal_id.clone();
        let app_handle_clone = app_handle.clone();
        tokio::spawn(async move {
            while let Some(new_title) = title_rx.recv().await {
                let title_event = TerminalTitleChanged {
                    terminal_id: terminal_id_clone.clone(),
                    title: new_title,
                };

                // Emit title change event to frontend if app handle is available
                if let Some(handle) = &app_handle_clone {
                    if let Err(e) = handle.emit("terminal-title-changed", &title_event) {
                        eprintln!("Failed to emit terminal title change event: {}", e);
                    }
                }
            }
        });

        // Spawn task to handle terminal exits
        let app_handle_clone = app_handle.clone();
        tokio::spawn(async move {
            while let Some(exit_event) = exit_rx.recv().await {
                // Emit exit event to frontend if app handle is available
                if let Some(handle) = &app_handle_clone {
                    if let Err(e) = handle.emit("terminal-exited", &exit_event) {
                        eprintln!("Failed to emit terminal exit event: {}", e);
                    }
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

        // Store the terminal
        {
            let mut terminals = self.terminals.write().await;
            terminals.insert(terminal_id.clone(), Arc::new(Mutex::new(terminal)));
        }

        Ok(CreateTerminalResponse {
            terminal_id,
            info: terminal_info,
        })
    }

    /// Write data to a specific terminal
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

    /// Resize a specific terminal
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

    /// Close a specific terminal
    pub async fn close_terminal(&self, terminal_id: String) -> Result<(), AppError> {
        // Remove from active terminals
        let terminal = {
            let mut terminals = self.terminals.write().await;
            terminals.remove(&terminal_id)
        };

        // Remove output sender
        {
            let mut senders = self.output_senders.write().await;
            senders.remove(&terminal_id);
        }

        // Remove buffer for this terminal
        self.buffer_manager.remove_buffer(&terminal_id).await;

        // Disconnect the terminal if it exists
        if let Some(terminal) = terminal {
            let mut terminal_guard = terminal.lock().await;
            terminal_guard.disconnect().await?;
        }

        Ok(())
    }

    /// Get information about a specific terminal
    pub async fn get_terminal_info(&self, terminal_id: String) -> Result<TerminalInfo, AppError> {
        let terminals = self.terminals.read().await;

        if let Some(terminal) = terminals.get(&terminal_id) {
            let terminal_guard = terminal.lock().await;
            Ok(TerminalInfo {
                id: terminal_id,
                config: terminal_guard.get_config().clone(),
                state: terminal_guard.get_state(),
                created_at: chrono::Utc::now(), // This should be stored properly
                title: None,                    // This should be stored properly
            })
        } else {
            Err(AppError::TerminalNotFound(terminal_id))
        }
    }

    /// List all active terminals
    pub async fn list_terminals(&self) -> Result<Vec<TerminalInfo>, AppError> {
        let terminals = self.terminals.read().await;
        let mut terminal_infos = Vec::new();

        for (terminal_id, terminal) in terminals.iter() {
            let terminal_guard = terminal.lock().await;
            terminal_infos.push(TerminalInfo {
                id: terminal_id.clone(),
                config: terminal_guard.get_config().clone(),
                state: terminal_guard.get_state(),
                created_at: chrono::Utc::now(), // This should be stored properly
                title: None,                    // This should be stored properly
            });
        }

        Ok(terminal_infos)
    }

    /// Close all terminals
    pub async fn close_all_terminals(&self) -> Result<(), AppError> {
        let terminal_ids: Vec<String> = {
            let terminals = self.terminals.read().await;
            terminals.keys().cloned().collect()
        };

        for terminal_id in terminal_ids {
            if let Err(e) = self.close_terminal(terminal_id).await {
                eprintln!("Failed to close terminal: {}", e);
            }
        }

        Ok(())
    }

    /// Get the buffer manager instance
    pub fn get_buffer_manager(&self) -> Arc<TerminalBufferManager> {
        self.buffer_manager.clone()
    }
}
