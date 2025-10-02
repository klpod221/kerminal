use crate::models::ssh::{AuthData, SSHProfile};
use crate::error::AppError;
use crate::models::terminal::{TerminalConfig, TerminalState};
use russh::client::{Handle, Handler};
use russh::{ChannelId, Disconnect, Channel, client::Msg};
use russh::keys::PrivateKeyWithHashAlg;
use std::sync::Arc;
use std::time::Duration;
use tokio::sync::{mpsc, Mutex};
use std::path::Path;

/// SSH client handler implementation
#[derive(Clone)]
struct ClientHandler {
    output_sender: Arc<Mutex<Option<mpsc::UnboundedSender<Vec<u8>>>>>,
}

impl ClientHandler {
    fn new() -> Self {
        Self {
            output_sender: Arc::new(Mutex::new(None)),
        }
    }

    async fn set_output_sender(&self, sender: mpsc::UnboundedSender<Vec<u8>>) {
        let mut guard = self.output_sender.lock().await;
        *guard = Some(sender);
    }
}

impl Handler for ClientHandler {
    type Error = russh::Error;

    async fn check_server_key(
        &mut self,
        _server_public_key: &russh::keys::PublicKey,
    ) -> Result<bool, Self::Error> {
        Ok(true)
    }

    async fn data(
        &mut self,
        _channel: ChannelId,
        data: &[u8],
        _session: &mut russh::client::Session,
    ) -> Result<(), Self::Error> {
        let sender = self.output_sender.lock().await;
        if let Some(sender) = sender.as_ref() {
            let _ = sender.send(data.to_vec());
        }
        Ok(())
    }

    async fn extended_data(
        &mut self,
        _channel: ChannelId,
        _code: u32,
        data: &[u8],
        _session: &mut russh::client::Session,
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
        _session: &mut russh::client::Session,
    ) -> Result<(), Self::Error> {
        let sender = self.output_sender.lock().await;
        if let Some(sender) = sender.as_ref() {
            let eof_msg = b"[SSH: Connection closed by remote host]\r\n";
            let _ = sender.send(eof_msg.to_vec());
        }
        Ok(())
    }

    async fn channel_close(
        &mut self,
        _channel: ChannelId,
        _session: &mut russh::client::Session,
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
}

impl SSHTerminal {
    /// Create a new SSH terminal instance
    pub fn new(
        id: String,
        config: TerminalConfig,
        ssh_profile: SSHProfile,
    ) -> Result<Self, AppError> {
        let handler = Arc::new(ClientHandler::new());
        Ok(SSHTerminal {
            id,
            config,
            ssh_profile,
            state: TerminalState::Disconnected,
            session: None,
            channel: None,
            handler,
        })
    }

    /// Connect to the SSH terminal
    pub async fn connect(&mut self) -> Result<(), AppError> {
        self.state = TerminalState::Connecting;

        let connection_timeout = Duration::from_secs(
            self.ssh_profile.timeout.map(|t| t as u64).unwrap_or(30)
        );

        let result = tokio::time::timeout(connection_timeout, async {
            if self.ssh_profile.proxy.is_some() {
                return Err(AppError::connection_failed(
                    "Proxy connections not yet implemented with russh".to_string(),
                ));
            }

            let config = Arc::new(russh::client::Config {
                inactivity_timeout: Some(Duration::from_secs(300)),
                ..<russh::client::Config as Default>::default()
            });

            let address = format!("{}:{}", self.ssh_profile.host, self.ssh_profile.port);
            let mut session = russh::client::connect(config, &address, (*self.handler).clone())
                .await
                .map_err(|e| {
                    AppError::connection_failed(format!("SSH handshake failed: {}", e))
                })?;

            self.authenticate(&mut session).await?;

            let channel = session
                .channel_open_session()
                .await
                .map_err(|e| {
                    AppError::connection_failed(format!("Failed to create SSH channel: {}", e))
                })?;

            channel
                .request_pty(false, "xterm-256color", 120, 30, 0, 0, &[])
                .await
                .map_err(|e| {
                    AppError::terminal_error(format!("Failed to request PTY: {}", e))
                })?;

            channel
                .request_shell(false)
                .await
                .map_err(|e| {
                    AppError::terminal_error(format!("Failed to start shell: {}", e))
                })?;

            self.session = Some(session);
            self.channel = Some(channel);
            self.state = TerminalState::Connected;

            Ok::<(), AppError>(())
        }).await;

        match result {
            Ok(Ok(())) => Ok(()),
            Ok(Err(e)) => {
                self.state = TerminalState::Disconnected;
                Err(e)
            }
            Err(_) => {
                self.state = TerminalState::Disconnected;
                Err(AppError::connection_failed(format!(
                    "SSH connection to {}:{} timed out after {} seconds",
                    self.ssh_profile.host,
                    self.ssh_profile.port,
                    connection_timeout.as_secs()
                )))
            }
        }
    }

    /// Authenticate with the SSH server
    async fn authenticate(&mut self, session: &mut Handle<ClientHandler>) -> Result<(), AppError> {
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

                if !result.success() {
                    return Err(AppError::authentication_failed(format!(
                        "Password authentication failed for user '{}'",
                        username
                    )));
                }
            }
            AuthData::PrivateKey { private_key, .. } => {
                let key = if Path::new(private_key).exists() {
                    russh::keys::load_secret_key(private_key, None)
                        .map_err(|e| {
                            AppError::authentication_failed(format!(
                                "Failed to load private key from '{}': {}",
                                private_key, e
                            ))
                        })?
                } else {
                    russh::keys::decode_secret_key(private_key, None)
                        .map_err(|e| {
                            AppError::authentication_failed(format!(
                                "Failed to parse private key: {}",
                                e
                            ))
                        })?
                };

                let private_key_with_hash = PrivateKeyWithHashAlg::new(Arc::new(key), Some(russh::keys::HashAlg::Sha256));
                let result = session
                    .authenticate_publickey(username, private_key_with_hash)
                    .await
                    .map_err(|e| {
                        AppError::authentication_failed(format!(
                            "Private key authentication error for user '{}': {}",
                            username, e
                        ))
                    })?;

                if !result.success() {
                    return Err(AppError::authentication_failed(format!(
                        "Private key authentication failed for user '{}'",
                        username
                    )));
                }
            }
            AuthData::PrivateKeyWithPassphrase {
                private_key,
                passphrase,
                ..
            } => {
                let key = if Path::new(private_key).exists() {
                    russh::keys::load_secret_key(private_key, Some(passphrase))
                        .map_err(|e| {
                            AppError::authentication_failed(format!(
                                "Failed to load private key from '{}' with passphrase: {}",
                                private_key, e
                            ))
                        })?
                } else {
                    russh::keys::decode_secret_key(private_key, Some(passphrase))
                        .map_err(|e| {
                            AppError::authentication_failed(format!(
                                "Failed to parse private key with passphrase: {}",
                                e
                            ))
                        })?
                };

                let private_key_with_hash = PrivateKeyWithHashAlg::new(Arc::new(key), Some(russh::keys::HashAlg::Sha256));
                let result = session
                    .authenticate_publickey(username, private_key_with_hash)
                    .await
                    .map_err(|e| {
                        AppError::authentication_failed(format!(
                            "Private key with passphrase authentication error for user '{}': {}",
                            username, e
                        ))
                    })?;

                if !result.success() {
                    return Err(AppError::authentication_failed(format!(
                        "Private key with passphrase authentication failed for user '{}'",
                        username
                    )));
                }
            }
            AuthData::Agent { .. } => {
                return Err(AppError::authentication_failed(
                    "SSH agent authentication not yet fully implemented with russh".to_string(),
                ));
            }
            _ => {
                return Err(AppError::authentication_failed(format!(
                    "Unsupported authentication method for user '{}'",
                    username
                )));
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
            let _ = session.disconnect(Disconnect::ByApplication, "", "en").await;
        }

        self.state = TerminalState::Disconnected;
        Ok(())
    }

    /// Write data to the SSH terminal
    pub async fn write(&mut self, data: &[u8]) -> Result<(), AppError> {
        if let Some(channel) = &mut self.channel {
            channel
                .data(data)
                .await
                .map_err(|e| {
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
    ) -> Result<(), AppError> {
        self.handler.set_output_sender(sender).await;
        Ok(())
    }
}
