use crate::error::AppError;
use crate::models::ssh::key::ResolvedSSHKey;
use crate::models::ssh::{AuthData, SSHProfile};
use crate::models::terminal::{TerminalConfig, TerminalState};
use async_trait::async_trait;
use russh::client::{Handle, Handler, Session};
use russh::{client::Msg, Channel, ChannelId, Disconnect};
use russh_keys::key::PublicKey;
use std::path::Path;
use std::sync::Arc;
use tokio::sync::{mpsc, Mutex};

/// SSH client handler implementation
#[derive(Clone)]
pub struct ClientHandler {
    output_sender: Arc<Mutex<Option<mpsc::UnboundedSender<Vec<u8>>>>>,
    exit_sender: Arc<Mutex<Option<mpsc::UnboundedSender<crate::models::terminal::TerminalExited>>>>,
    terminal_id: Arc<Mutex<String>>,
}

impl ClientHandler {
    fn new(terminal_id: String) -> Self {
        Self {
            output_sender: Arc::new(Mutex::new(None)),
            exit_sender: Arc::new(Mutex::new(None)),
            terminal_id: Arc::new(Mutex::new(terminal_id)),
        }
    }

    async fn set_output_sender(&self, sender: mpsc::UnboundedSender<Vec<u8>>) {
        *self.output_sender.lock().await = Some(sender);
    }

    async fn set_exit_sender(
        &self,
        sender: mpsc::UnboundedSender<crate::models::terminal::TerminalExited>,
    ) {
        *self.exit_sender.lock().await = Some(sender);
    }
}

#[async_trait]
impl Handler for ClientHandler {
    type Error = russh::Error;

    async fn check_server_key(
        &mut self,
        _server_public_key: &PublicKey,
    ) -> Result<bool, Self::Error> {
        Ok(true)
    }

    async fn data(
        &mut self,
        _channel: ChannelId,
        data: &[u8],
        _session: &mut Session,
    ) -> Result<(), Self::Error> {
        let sender = self.output_sender.lock().await;
        if let Some(ref sender) = *sender {
            let _ = sender.send(data.to_vec());
        }
        Ok(())
    }

    async fn extended_data(
        &mut self,
        _channel: ChannelId,
        _code: u32,
        data: &[u8],
        _session: &mut Session,
    ) -> Result<(), Self::Error> {
        let sender = self.output_sender.lock().await;
        if let Some(sender) = sender.as_ref() {
            let _ = sender.send(data.to_vec());
        }
        Ok(())
    }

    async fn channel_eof(
        &mut self,
        _channel: ChannelId,
        _session: &mut Session,
    ) -> Result<(), Self::Error> {
        let sender = self.output_sender.lock().await;
        if let Some(sender) = sender.as_ref() {
            let eof_msg = b"[SSH: Connection closed by remote host]\r\n";
            let _ = sender.send(eof_msg.to_vec());
        }

        // Emit exit event - this is a graceful close (user typed exit or server closed normally)
        let exit_sender = self.exit_sender.lock().await;
        if let Some(ref sender) = *exit_sender {
            let terminal_id = self.terminal_id.lock().await;
            let exit_event = crate::models::terminal::TerminalExited {
                terminal_id: terminal_id.clone(),
                exit_code: Some(0), // Graceful exit
                reason: Some("user-closed".to_string()),
            };
            let _ = sender.send(exit_event);
        }

        Ok(())
    }

    async fn channel_close(
        &mut self,
        _channel: ChannelId,
        _session: &mut Session,
    ) -> Result<(), Self::Error> {
        Ok(())
    }
}

/// SSH terminal implementation using russh
pub struct SSHTerminal {
    id: String,
    config: TerminalConfig,
    ssh_profile: SSHProfile,
    state: TerminalState,
    session: Option<Handle<ClientHandler>>,
    channel: Option<Channel<Msg>>,
    handler: Arc<ClientHandler>,
    database_service: Option<Arc<tokio::sync::Mutex<crate::database::service::DatabaseService>>>,
}

impl SSHTerminal {
    /// Create a new SSH terminal instance
    pub fn new(
        id: String,
        config: TerminalConfig,
        ssh_profile: SSHProfile,
        database_service: Option<Arc<tokio::sync::Mutex<crate::database::service::DatabaseService>>>,
    ) -> Result<Self, AppError> {
        let handler = Arc::new(ClientHandler::new(id.clone()));
        Ok(SSHTerminal {
            id,
            config,
            ssh_profile,
            state: TerminalState::Disconnected,
            session: None,
            channel: None,
            handler,
            database_service,
        })
    }

    /// Connect to the SSH server
    pub async fn connect(&mut self) -> Result<(), AppError> {
        self.connect_with_resolved_data(None).await
    }

    /// Connect to the SSH server with optionally resolved key data
    pub async fn connect_with_resolved_data(
        &mut self,
        resolved_key: Option<crate::models::ssh::key::ResolvedSSHKey>,
    ) -> Result<(), AppError> {
        println!(
            "Connecting to SSH server {}:{}",
            self.ssh_profile.host, self.ssh_profile.port
        );

        self.state = TerminalState::Connecting;

        // Create SSH configuration
        let config = Arc::new(russh::client::Config {
            inactivity_timeout: Some(std::time::Duration::from_secs(60)),
            ..<russh::client::Config as Default>::default()
        });

        // Connect to the server
        let handler = (*self.handler).clone();
        let mut session = russh::client::connect(
            config,
            (self.ssh_profile.host.as_str(), self.ssh_profile.port),
            handler,
        )
        .await
        .map_err(|e| {
            self.state = TerminalState::Disconnected;
            AppError::connection_failed(format!(
                "Failed to connect to SSH server {}:{}: {}",
                self.ssh_profile.host, self.ssh_profile.port, e
            ))
        })?;

        // Authenticate with resolved data
        self.authenticate_with_resolved_data(&mut session, resolved_key)
            .await?;

        // Create a channel
        let channel = session.channel_open_session().await.map_err(|e| {
            self.state = TerminalState::Disconnected;
            AppError::terminal_error(format!("Failed to open SSH channel: {}", e))
        })?;

        // Request PTY
        let _ = channel
            .request_pty(
                false,
                "xterm-256color",
                80,
                24,
                0,
                0,
                &[
                    (russh::Pty::TTY_OP_ISPEED, 38400),
                    (russh::Pty::TTY_OP_OSPEED, 38400),
                ],
            )
            .await;

        // Start shell
        let _ = channel.request_shell(false).await;

        self.session = Some(session);
        self.channel = Some(channel);
        self.state = TerminalState::Connected;

        println!("Successfully connected to SSH server");
        Ok(())
    }

    /// Authenticate with the SSH server
    pub async fn authenticate_with_resolved_data(
        &mut self,
        session: &mut Handle<ClientHandler>,
        resolved_key: Option<ResolvedSSHKey>,
    ) -> Result<(), AppError> {
        let username = &self.ssh_profile.username;

        match &self.ssh_profile.auth_data {
            AuthData::Password { password } => {
                let result = session
                    .authenticate_password(username, password)
                    .await
                    .map_err(|e| {
                        AppError::authentication_failed(format!(
                            "Password authentication error for user '{}': {}",
                            username, e
                        ))
                    })?;

                if !result {
                    return Err(AppError::authentication_failed(format!(
                        "Password authentication failed for user '{}'",
                        username
                    )));
                }
            }
            AuthData::KeyReference { key_id } => {
                let key_data = resolved_key.ok_or_else(|| {
                    AppError::authentication_failed(format!(
                        "No resolved key data provided for KeyReference {}",
                        key_id
                    ))
                })?;

                let key = if Path::new(&key_data.private_key).exists() {
                    russh_keys::load_secret_key(
                        &key_data.private_key,
                        key_data.passphrase.as_deref(),
                    )
                    .map_err(|e| {
                        AppError::authentication_failed(format!("Failed to load SSH key: {}", e))
                    })?
                } else {
                    russh_keys::decode_secret_key(
                        &key_data.private_key,
                        key_data.passphrase.as_deref(),
                    )
                    .map_err(|e| {
                        AppError::authentication_failed(format!("Failed to parse SSH key: {}", e))
                    })?
                };

                let result = session
                    .authenticate_publickey(username, Arc::new(key))
                    .await
                    .map_err(|e| {
                        AppError::authentication_failed(format!(
                            "SSH key authentication error for user '{}': {}",
                            username, e
                        ))
                    })?;

                if !result {
                    return Err(AppError::authentication_failed(format!(
                        "SSH key authentication failed for user '{}'",
                        username
                    )));
                }

                // Mark SSH key as used after successful authentication
                if let Some(db_service) = &self.database_service {
                    let db = db_service.lock().await;
                    if let Err(e) = db.mark_key_used(key_id).await {
                        // Log error but don't fail the connection
                        eprintln!("Warning: Failed to mark SSH key {} as used: {}", key_id, e);
                    }
                }
            }
        }

        Ok(())
    }

    /// Disconnect from the SSH terminal
    pub async fn disconnect(&mut self) -> Result<(), AppError> {
        if let Some(channel) = self.channel.take() {
            let _ = channel.eof().await;
            let _ = channel.close().await;
        }

        if let Some(session) = self.session.take() {
            let _ = session
                .disconnect(Disconnect::ByApplication, "", "en")
                .await;
        }

        self.state = TerminalState::Disconnected;
        Ok(())
    }

    /// Write data to the SSH terminal
    pub async fn write(&mut self, data: &[u8]) -> Result<(), AppError> {
        // Check if session is still valid
        if let Some(session) = &self.session {
            if session.is_closed() {
                self.state = TerminalState::Disconnected;
                return Err(AppError::terminal_error(
                    "SSH session is closed".to_string(),
                ));
            }
        }

        if let Some(channel) = &mut self.channel {
            channel.data(data).await.map_err(|e| {
                self.state = TerminalState::Disconnected;
                AppError::terminal_error(format!("Failed to write to SSH channel: {}", e))
            })?;
            Ok(())
        } else {
            Err(AppError::terminal_error(
                "SSH terminal not connected".to_string(),
            ))
        }
    }
    /// Resize the SSH terminal
    pub async fn resize(&mut self, cols: u16, rows: u16) -> Result<(), AppError> {
        if let Some(channel) = &mut self.channel {
            channel
                .window_change(cols as u32, rows as u32, 0, 0)
                .await
                .map_err(|e| {
                    self.state = TerminalState::Disconnected;
                    AppError::terminal_error(format!("Failed to resize SSH terminal: {}", e))
                })?;
            Ok(())
        } else {
            Err(AppError::terminal_error(
                "SSH terminal not connected".to_string(),
            ))
        }
    }

    /// Get current state of the terminal
    pub fn get_state(&self) -> TerminalState {
        self.state.clone()
    }

    /// Get terminal configuration
    pub fn get_config(&self) -> &TerminalConfig {
        &self.config
    }

    /// Get terminal ID
    pub fn get_id(&self) -> &str {
        &self.id
    }

    /// Check if terminal is alive/connected
    pub fn is_alive(&self) -> bool {
        matches!(self.state, TerminalState::Connected)
            && self.session.is_some()
            && self.channel.is_some()
    }

    /// Start reading from SSH terminal and send output to the provided sender
    pub async fn start_read_loop(
        &mut self,
        sender: mpsc::UnboundedSender<Vec<u8>>,
        _title_sender: Option<mpsc::UnboundedSender<String>>,
        exit_sender: Option<mpsc::UnboundedSender<crate::models::terminal::TerminalExited>>,
    ) -> Result<(), AppError> {
        self.handler.set_output_sender(sender).await;
        if let Some(exit_sender) = exit_sender {
            self.handler.set_exit_sender(exit_sender).await;
        }
        Ok(())
    }
}
