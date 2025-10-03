use crate::models::ssh::{AuthData, SSHProfile};
use crate::error::AppError;
use crate::models::terminal::{TerminalConfig, TerminalState};
use russh::client::{Handle, Handler, Session};
use russh::{ChannelId, Disconnect, Channel, client::Msg};
use russh_keys::key::PublicKey;
use std::sync::Arc;
use std::time::{Duration, Instant};
use tokio::sync::{mpsc, Mutex};
use std::path::Path;
use async_trait::async_trait;

/// SSH client handler implementation
#[derive(Clone)]
struct ClientHandler {
    output_sender: Arc<Mutex<Option<mpsc::UnboundedSender<Vec<u8>>>>>,
    last_data_time: Arc<Mutex<Option<Instant>>>,
}

impl ClientHandler {
    fn new() -> Self {
        Self {
            output_sender: Arc::new(Mutex::new(None)),
            last_data_time: Arc::new(Mutex::new(None)),
        }
    }

    async fn set_output_sender(&self, sender: mpsc::UnboundedSender<Vec<u8>>) {
        println!("[DEBUG][SSH] Setting output sender for ClientHandler");
        let mut guard = self.output_sender.lock().await;
        let had_sender = guard.is_some();
        *guard = Some(sender);
        println!("[DEBUG][SSH] Output sender set (replaced existing: {})", had_sender);
    }
}

#[async_trait]
impl Handler for ClientHandler {
    type Error = russh::Error;

    async fn check_server_key(
        &mut self,
        _server_public_key: &PublicKey,
    ) -> Result<bool, Self::Error> {
        println!("[SSH Handler] check_server_key called");
        Ok(true)
    }

    async fn data(
        &mut self,
        _channel: ChannelId,
        data: &[u8],
        _session: &mut Session,
    ) -> Result<(), Self::Error> {
        println!(
            "[SSH Handler] data() received {} bytes: {}",
            data.len(),
            String::from_utf8_lossy(data)
        );

        let sender = self.output_sender.lock().await;
        if let Some(ref sender) = *sender {
            println!("[SSH Handler] Forwarding data to output channel");
            if sender.send(data.to_vec()).is_err() {
                println!("[SSH Handler] ERROR: Failed to forward data - receiver dropped");
            }
        } else {
            println!("[SSH Handler] WARNING: No output sender available");
        }

        // Update last data time
        *self.last_data_time.lock().await = Some(Instant::now());

        Ok(())
    }

    async fn extended_data(
        &mut self,
        channel: ChannelId,
        code: u32,
        data: &[u8],
        _session: &mut Session,
    ) -> Result<(), Self::Error> {
        let data_len = data.len();
        println!("[DEBUG][SSH] Received {} bytes extended data (code: {}) on channel {}", data_len, code, channel);

        let sender = self.output_sender.lock().await;
        if let Some(sender) = sender.as_ref() {
            match sender.send(data.to_vec()) {
                Ok(_) => {
                    println!("[DEBUG][SSH] Successfully forwarded {} bytes extended data to output", data_len);
                }
                Err(e) => {
                    println!("[ERROR][SSH] Failed to forward {} bytes extended data to output: {}", data_len, e);
                }
            }
        } else {
            println!("[WARN][SSH] No output sender available for {} bytes extended data", data_len);
        }
        Ok(())
    }

    async fn channel_eof(
        &mut self,
        channel: ChannelId,
        _session: &mut Session,
    ) -> Result<(), Self::Error> {
        println!("[DEBUG][SSH] Channel {} received EOF - connection closing", channel);
        let sender = self.output_sender.lock().await;
        if let Some(sender) = sender.as_ref() {
            let eof_msg = b"[SSH: Connection closed by remote host]\r\n";
            let _ = sender.send(eof_msg.to_vec());
        }
        Ok(())
    }

    async fn channel_close(
        &mut self,
        channel: ChannelId,
        _session: &mut Session,
    ) -> Result<(), Self::Error> {
        println!("[DEBUG][SSH] Channel {} closed by remote host", channel);
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
    last_data_received: Arc<Mutex<std::time::Instant>>,
    last_write_time: Arc<Mutex<std::time::Instant>>,
}

impl SSHTerminal {
    /// Create a new SSH terminal instance
    pub fn new(
        id: String,
        config: TerminalConfig,
        ssh_profile: SSHProfile,
    ) -> Result<Self, AppError> {
        println!("[DEBUG][SSH] Creating new SSH terminal instance [{}] for {}:{}",
                 id, ssh_profile.host, ssh_profile.port);
        let handler = Arc::new(ClientHandler::new());
        let now = std::time::Instant::now();
        Ok(SSHTerminal {
            id,
            config,
            ssh_profile,
            state: TerminalState::Disconnected,
            session: None,
            channel: None,
            handler,
            last_data_received: Arc::new(Mutex::new(now)),
            last_write_time: Arc::new(Mutex::new(now)),
        })
    }

    /// Connect to the SSH terminal
    pub async fn connect(&mut self) -> Result<(), AppError> {
        println!("[DEBUG][SSH] Starting connection to {}:{} for user {} [terminal_id: {}]",
                   self.ssh_profile.host, self.ssh_profile.port, self.ssh_profile.username, self.id);

        // Check if already connected
        if matches!(self.state, TerminalState::Connected) {
            println!("[WARN][SSH] Terminal [{}] already connected to {}:{}, disconnecting first",
                     self.id, self.ssh_profile.host, self.ssh_profile.port);
            let _ = self.disconnect().await;
        }

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

            println!("[DEBUG][SSH] Successfully connected to {}:{}",
                       self.ssh_profile.host, self.ssh_profile.port);

            Ok::<(), AppError>(())
        }).await;

        match result {
            Ok(Ok(())) => Ok(()),
            Ok(Err(e)) => {
                println!("[ERROR][SSH] Connection failed to {}:{} - {}",
                           self.ssh_profile.host, self.ssh_profile.port, e);
                self.state = TerminalState::Disconnected;
                Err(e)
            }
            Err(_) => {
                println!("[ERROR][SSH] Connection timeout to {}:{} after {} seconds",
                           self.ssh_profile.host, self.ssh_profile.port, connection_timeout.as_secs());
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

                if !result {
                    return Err(AppError::authentication_failed(format!(
                        "Password authentication failed for user '{}'",
                        username
                    )));
                }
            }
            AuthData::PrivateKey { private_key, .. } => {
                let key = if Path::new(private_key).exists() {
                    russh_keys::load_secret_key(private_key, None)
                        .map_err(|e| {
                            AppError::authentication_failed(format!(
                                "Failed to load private key from '{}': {}",
                                private_key, e
                            ))
                        })?
                } else {
                    russh_keys::decode_secret_key(private_key, None)
                        .map_err(|e| {
                            AppError::authentication_failed(format!(
                                "Failed to parse private key: {}",
                                e
                            ))
                        })?
                };

                let result = session
                    .authenticate_publickey(username, Arc::new(key))
                    .await
                    .map_err(|e| {
                        AppError::authentication_failed(format!(
                            "Private key authentication error for user '{}': {}",
                            username, e
                        ))
                    })?;

                if !result {
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
                    russh_keys::load_secret_key(private_key, Some(passphrase))
                        .map_err(|e| {
                            AppError::authentication_failed(format!(
                                "Failed to load private key from '{}' with passphrase: {}",
                                private_key, e
                            ))
                        })?
                } else {
                    russh_keys::decode_secret_key(private_key, Some(passphrase))
                        .map_err(|e| {
                            AppError::authentication_failed(format!(
                                "Failed to parse private key with passphrase: {}",
                                e
                            ))
                        })?
                };

                let result = session
                    .authenticate_publickey(username, Arc::new(key))
                    .await
                    .map_err(|e| {
                        AppError::authentication_failed(format!(
                            "Private key with passphrase authentication error for user '{}': {}",
                            username, e
                        ))
                    })?;

                if !result {
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
        println!("[DEBUG][SSH] Initiating disconnect for {}:{} [terminal_id: {}, current_state: {:?}]",
                   self.ssh_profile.host, self.ssh_profile.port, self.id, self.state);

        if let Some(channel) = self.channel.take() {
            println!("[DEBUG][SSH] Closing SSH channel for [{}]", self.id);
            let eof_result = channel.eof().await;
            let close_result = channel.close().await;
            println!("[DEBUG][SSH] Channel close results - EOF: {:?}, Close: {:?}", eof_result, close_result);
        } else {
            println!("[DEBUG][SSH] No channel to close for [{}]", self.id);
        }

        if let Some(session) = self.session.take() {
            println!("[DEBUG][SSH] Disconnecting SSH session for [{}]", self.id);
            let disconnect_result = session.disconnect(Disconnect::ByApplication, "", "en").await;
            println!("[DEBUG][SSH] Session disconnect result: {:?}", disconnect_result);
        } else {
            println!("[DEBUG][SSH] No session to disconnect for [{}]", self.id);
        }

        self.state = TerminalState::Disconnected;
        println!("[DEBUG][SSH] Successfully disconnected [{}] from {}:{}",
                   self.id, self.ssh_profile.host, self.ssh_profile.port);
        Ok(())
    }

    /// Write data to the SSH terminal
    pub async fn write(&mut self, data: &[u8]) -> Result<(), AppError> {
        let data_str = String::from_utf8_lossy(data);
        let data_len = data.len();

        // Check if we haven't received data for a while after writing
        let last_received = *self.last_data_received.lock().await;
        let last_write = *self.last_write_time.lock().await;
        let time_since_data = last_received.elapsed();
        let time_since_write = last_write.elapsed();

        // If we wrote something but haven't received response in 5 seconds, warn
        if time_since_write < time_since_data && time_since_data.as_secs() > 5 {
            println!("[WARN][SSH] No data received for {} seconds after last write to {}:{}",
                     time_since_data.as_secs(), self.ssh_profile.host, self.ssh_profile.port);
            println!("[WARN][SSH] Connection might be stuck or server is not responding");
        }

        println!("[DEBUG][SSH] Attempting to write {} bytes to {}:{}: {:?}",
                 data_len, self.ssh_profile.host, self.ssh_profile.port,
                 if data_len <= 50 { data_str.as_ref() } else { "<large data>" });

        // Check if channel and session are still valid
        if let Some(session) = &self.session {
            if session.is_closed() {
                println!("[ERROR][SSH] Session is closed for {}:{}",
                         self.ssh_profile.host, self.ssh_profile.port);
                self.state = TerminalState::Disconnected;
                return Err(AppError::terminal_error("SSH session is closed".to_string()));
            }
        }

        if let Some(channel) = &mut self.channel {
            // Check channel state before writing
            println!("[DEBUG][SSH] Channel available for {}:{}, state: {:?}",
                     self.ssh_profile.host, self.ssh_profile.port, self.state);

            // Check if channel is still open by checking if we can get its ID
            let channel_id = channel.id();
            println!("[DEBUG][SSH] Writing to channel ID: {}", channel_id);

            let write_result = channel.data(data).await;
            match write_result {
                Ok(_) => {
                    // Update last write time
                    let mut last_write = self.last_write_time.lock().await;
                    *last_write = std::time::Instant::now();

                    println!("[DEBUG][SSH] Successfully wrote {} bytes to {}:{}",
                             data_len, self.ssh_profile.host, self.ssh_profile.port);
                    Ok(())
                }
                Err(e) => {
                    println!("[ERROR][SSH] Failed to write {} bytes to channel for {}:{} - {}",
                             data_len, self.ssh_profile.host, self.ssh_profile.port, e);
                    // Mark as disconnected if write fails
                    self.state = TerminalState::Disconnected;
                    Err(AppError::terminal_error(format!("Failed to write to SSH channel: {}", e)))
                }
            }
        } else {
            println!("[WARN][SSH] Attempted to write {} bytes to disconnected terminal {}:{}",
                      data_len, self.ssh_profile.host, self.ssh_profile.port);
            Err(AppError::terminal_error(
                "SSH terminal not connected".to_string(),
            ))
        }
    }    /// Resize the SSH terminal
    pub async fn resize(&mut self, cols: u16, rows: u16) -> Result<(), AppError> {
        if let Some(channel) = &mut self.channel {
            channel
                .window_change(cols as u32, rows as u32, 0, 0)
                .await
                .map_err(|e| {
                    println!("[ERROR][SSH] Failed to resize terminal for {}:{} - {}",
                               self.ssh_profile.host, self.ssh_profile.port, e);
                    // Mark as disconnected if resize fails
                    self.state = TerminalState::Disconnected;
                    AppError::terminal_error(format!("Failed to resize SSH terminal: {}", e))
                })?;
            println!("[DEBUG][SSH] Terminal resized to {}x{} for {}:{}",
                       cols, rows, self.ssh_profile.host, self.ssh_profile.port);
            Ok(())
        } else {
            println!("[WARN][SSH] Attempted to resize disconnected terminal {}:{}",
                      self.ssh_profile.host, self.ssh_profile.port);
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
        let state_ok = matches!(self.state, TerminalState::Connected);
        let session_ok = self.session.is_some();
        let channel_ok = self.channel.is_some();
        let is_alive = state_ok && session_ok && channel_ok;

        if !is_alive {
            println!("[DEBUG][SSH] Terminal [{}] alive check - State: {:?} ({}), Session: {}, Channel: {}",
                     self.id, self.state, state_ok, session_ok, channel_ok);
        }

        is_alive
    }

    /// Start reading from SSH terminal and send output to the provided sender
    pub async fn start_read_loop(
        &mut self,
        sender: mpsc::UnboundedSender<Vec<u8>>,
    ) -> Result<(), AppError> {
        self.handler.set_output_sender(sender).await;

        println!("[DEBUG][SSH] Read loop started for [{}] {}:{}",
                 self.id, self.ssh_profile.host, self.ssh_profile.port);
        println!("[DEBUG][SSH] IMPORTANT: russh uses async event model - data comes via Handler callbacks");
        println!("[DEBUG][SSH] If no data received after writes, check russh version or server config");

        // Start a heartbeat monitor task
        let handler_clone = self.handler.clone();
        let last_data_received_clone = self.last_data_received.clone();
        let host = self.ssh_profile.host.clone();
        let port = self.ssh_profile.port;
        let terminal_id = self.id.clone();

        tokio::spawn(async move {
            let mut interval = tokio::time::interval(Duration::from_secs(2));
            loop {
                interval.tick().await;

                // Check if we have received data from handler
                let handler_last_time = handler_clone.last_data_time.lock().await;
                if let Some(handler_time) = *handler_last_time {
                    // Update SSHTerminal's last_data_received
                    let mut terminal_last_time = last_data_received_clone.lock().await;
                    *terminal_last_time = handler_time;
                    drop(terminal_last_time);

                    let elapsed = handler_time.elapsed();
                    if elapsed.as_secs() > 10 {
                        println!("[WARN][SSH] Terminal [{}] No data received from {}:{} for {} seconds",
                                 terminal_id, host, port, elapsed.as_secs());
                        println!("[WARN][SSH] This usually indicates:");
                        println!("[WARN][SSH]   1. Server stopped responding");
                        println!("[WARN][SSH]   2. Network issue or firewall");
                        println!("[WARN][SSH]   3. russh library bug or version incompatibility");
                        println!("[WARN][SSH]   4. SSH channel closed on server side");
                    }
                }
            }
        });

        Ok(())
    }
}
