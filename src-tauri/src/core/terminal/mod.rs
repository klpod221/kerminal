pub mod local;
pub mod ssh;

use crate::database::service::DatabaseService;
use crate::error::AppError;
use crate::models::terminal::{TerminalConfig, TerminalExited, TerminalState, TerminalType};
use std::sync::Arc;
use tokio::sync::{mpsc, Mutex};

/// Unified terminal wrapper that can handle both local and SSH terminals
pub enum TerminalWrapper {
    Local(local::LocalTerminal),
    SSH(ssh::SSHTerminal),
}

impl TerminalWrapper {
    /// Connect to the terminal
    pub async fn connect(&mut self) -> Result<(), AppError> {
        match self {
            TerminalWrapper::Local(terminal) => terminal.connect().await,
            TerminalWrapper::SSH(terminal) => terminal.connect().await,
        }
    }

    /// Disconnect from the terminal
    pub async fn disconnect(&mut self) -> Result<(), AppError> {
        match self {
            TerminalWrapper::Local(terminal) => terminal.disconnect().await,
            TerminalWrapper::SSH(terminal) => terminal.disconnect().await,
        }
    }

    /// Write data to the terminal
    pub async fn write(&mut self, data: &[u8]) -> Result<(), AppError> {
        match self {
            TerminalWrapper::Local(terminal) => terminal.write(data).await,
            TerminalWrapper::SSH(terminal) => terminal.write(data).await,
        }
    }

    /// Resize the terminal
    pub async fn resize(&mut self, cols: u16, rows: u16) -> Result<(), AppError> {
        match self {
            TerminalWrapper::Local(terminal) => terminal.resize(cols, rows).await,
            TerminalWrapper::SSH(terminal) => terminal.resize(cols, rows).await,
        }
    }

    /// Get current state of the terminal
    pub fn get_state(&self) -> TerminalState {
        match self {
            TerminalWrapper::Local(terminal) => terminal.get_state(),
            TerminalWrapper::SSH(terminal) => terminal.get_state(),
        }
    }

    /// Get terminal configuration
    pub fn get_config(&self) -> &TerminalConfig {
        match self {
            TerminalWrapper::Local(terminal) => terminal.get_config(),
            TerminalWrapper::SSH(terminal) => terminal.get_config(),
        }
    }

    /// Get terminal ID
    pub fn get_id(&self) -> &str {
        match self {
            TerminalWrapper::Local(terminal) => terminal.get_id(),
            TerminalWrapper::SSH(terminal) => terminal.get_id(),
        }
    }

    /// Check if terminal is alive/connected
    pub fn is_alive(&self) -> bool {
        match self {
            TerminalWrapper::Local(terminal) => terminal.is_alive(),
            TerminalWrapper::SSH(terminal) => terminal.is_alive(),
        }
    }

    /// Start reading from terminal and send output to the provided sender
    pub async fn start_read_loop(
        &mut self,
        sender: mpsc::UnboundedSender<Vec<u8>>,
        title_sender: Option<mpsc::UnboundedSender<String>>,
        exit_sender: Option<mpsc::UnboundedSender<TerminalExited>>,
    ) -> Result<(), AppError> {
        match self {
            TerminalWrapper::Local(terminal) => {
                terminal
                    .start_read_loop(sender, title_sender, exit_sender)
                    .await
            }
            TerminalWrapper::SSH(terminal) => {
                // SSH doesn't support title detection and exit events yet
                terminal.start_read_loop(sender).await
            }
        }
    }
}

/// Factory for creating terminal instances
pub struct TerminalFactory;

impl TerminalFactory {
    /// Create a new terminal instance based on configuration
    pub async fn create_terminal(
        id: String,
        config: TerminalConfig,
        database_service: Option<Arc<Mutex<DatabaseService>>>,
    ) -> Result<TerminalWrapper, AppError> {
        match config.terminal_type {
            TerminalType::Local => {
                let local_config = config.local_config.clone().unwrap_or_default();
                Ok(TerminalWrapper::Local(local::LocalTerminal::new(
                    id,
                    config,
                    local_config,
                )?))
            }
            TerminalType::SSH => {
                let ssh_profile_id = config.ssh_profile_id.clone().ok_or_else(|| {
                    AppError::invalid_config(
                        "SSH profile ID is required for SSH terminal".to_string(),
                    )
                })?;

                let database_service = database_service.ok_or_else(|| {
                    AppError::invalid_config(
                        "Database service is required for SSH terminal".to_string(),
                    )
                })?;

                let ssh_profile = {
                    let db_service = database_service.lock().await;
                    db_service
                        .get_ssh_profile(&ssh_profile_id)
                        .await
                        .map_err(|e| {
                            AppError::Database(e.to_string())
                        })?
                };

                Ok(TerminalWrapper::SSH(ssh::SSHTerminal::new(
                    id,
                    config,
                    ssh_profile,
                )?))
            }
        }
    }
}
