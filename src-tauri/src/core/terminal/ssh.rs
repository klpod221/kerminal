use crate::error::AppError;
use crate::models::terminal::{SSHConfig, TerminalConfig, TerminalState};
use ssh2::{Channel, Session};
use std::io::{Read, Write};
use std::net::TcpStream;
use std::sync::Arc;
use std::thread;
use std::time::Duration;
use tokio::sync::{mpsc, Mutex};

/// SSH terminal implementation using ssh2
pub struct SSHTerminal {
    id: String,
    config: TerminalConfig,
    ssh_config: SSHConfig,
    state: TerminalState,
    session: Option<Arc<Mutex<Session>>>,
    channel: Option<Arc<Mutex<Channel>>>,
}

impl SSHTerminal {
    /// Create a new SSH terminal instance
    pub fn new(
        id: String,
        config: TerminalConfig,
        ssh_config: SSHConfig,
    ) -> Result<Self, AppError> {
        Ok(SSHTerminal {
            id,
            config,
            ssh_config,
            state: TerminalState::Disconnected,
            session: None,
            channel: None,
        })
    }

    /// Connect to the SSH terminal
    pub async fn connect(&mut self) -> Result<(), AppError> {
        self.state = TerminalState::Connecting;

        // Create TCP connection
        let tcp = TcpStream::connect(format!("{}:{}", self.ssh_config.host, self.ssh_config.port))
            .map_err(|e| AppError::ConnectionFailed(format!("Failed to connect to {}:{}: {}",
                self.ssh_config.host, self.ssh_config.port, e)))?;

        // Create SSH session
        let mut session = Session::new()
            .map_err(|e| AppError::ConnectionFailed(format!("Failed to create SSH session: {}", e)))?;

        session.set_tcp_stream(tcp);
        session.handshake()
            .map_err(|e| AppError::ConnectionFailed(format!("SSH handshake failed: {}", e)))?;

        // Authenticate
        self.authenticate(&mut session).await?;

        // Create channel
        let mut channel = session.channel_session()
            .map_err(|e| AppError::ConnectionFailed(format!("Failed to create SSH channel: {}", e)))?;

        // Request a pseudo-terminal
        channel.request_pty("xterm", None, Some((80, 24, 0, 0)))
            .map_err(|e| AppError::TerminalError(format!("Failed to request PTY: {}", e)))?;

        // Start shell
        channel.shell()
            .map_err(|e| AppError::TerminalError(format!("Failed to start shell: {}", e)))?;

        self.session = Some(Arc::new(Mutex::new(session)));
        self.channel = Some(Arc::new(Mutex::new(channel)));
        self.state = TerminalState::Connected;

        Ok(())
    }

    /// Authenticate with the SSH server
    async fn authenticate(&self, session: &mut Session) -> Result<(), AppError> {
        // Try public key authentication first if private key is provided
        if let Some(private_key_path) = &self.ssh_config.private_key_path {
            let passphrase = self.ssh_config.private_key_passphrase.as_deref();
            match session.userauth_pubkey_file(
                &self.ssh_config.username,
                None, // public key path (optional)
                std::path::Path::new(private_key_path),
                passphrase,
            ) {
                Ok(_) => return Ok(()),
                Err(e) => {
                    eprintln!("Public key authentication failed: {}", e);
                }
            }
        }

        // Try password authentication
        if let Some(password) = &self.ssh_config.password {
            session.userauth_password(&self.ssh_config.username, password)
                .map_err(|e| AppError::AuthenticationFailed(format!("Password authentication failed: {}", e)))?;
        } else {
            return Err(AppError::AuthenticationFailed("No authentication method available".to_string()));
        }

        Ok(())
    }

    /// Disconnect from the SSH terminal
    pub async fn disconnect(&mut self) -> Result<(), AppError> {
        if let Some(channel) = self.channel.take() {
            if let Ok(mut channel_guard) = channel.try_lock() {
                let _ = channel_guard.close();
                let _ = channel_guard.wait_close();
            }
        }

        if let Some(session) = self.session.take() {
            if let Ok(session_guard) = session.try_lock() {
                let _ = session_guard.disconnect(None, "User requested disconnection", None);
            }
        }

        self.state = TerminalState::Disconnected;
        Ok(())
    }

    /// Write data to the SSH terminal
    pub async fn write(&mut self, data: &[u8]) -> Result<(), AppError> {
        if let Some(channel) = &self.channel {
            let mut channel_guard = channel.lock().await;
            channel_guard.write_all(data)
                .map_err(|e| AppError::TerminalError(format!("Failed to write to SSH channel: {}", e)))?;
            channel_guard.flush()
                .map_err(|e| AppError::TerminalError(format!("Failed to flush SSH channel: {}", e)))?;
            Ok(())
        } else {
            Err(AppError::TerminalError("SSH terminal not connected".to_string()))
        }
    }

    /// Resize the SSH terminal
    pub async fn resize(&mut self, cols: u16, rows: u16) -> Result<(), AppError> {
        if let Some(channel) = &self.channel {
            let mut channel_guard = channel.lock().await;
            channel_guard.request_pty_size(cols as u32, rows as u32, None, None)
                .map_err(|e| AppError::TerminalError(format!("Failed to resize SSH terminal: {}", e)))?;
            Ok(())
        } else {
            Err(AppError::TerminalError("SSH terminal not connected".to_string()))
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
        matches!(self.state, TerminalState::Connected) &&
        self.channel.is_some() &&
        self.session.is_some()
    }

    /// Start reading from SSH terminal and send output to the provided sender
    pub async fn start_read_loop(&mut self, sender: mpsc::UnboundedSender<Vec<u8>>) -> Result<(), AppError> {
        if let Some(channel) = self.channel.clone() {
            // Spawn a blocking task to read from SSH channel
            tokio::task::spawn_blocking(move || {
                let mut buffer = [0u8; 8192];
                loop {
                    if let Ok(mut channel_guard) = channel.try_lock() {
                        match channel_guard.read(&mut buffer) {
                            Ok(0) => {
                                // EOF reached, SSH channel has closed
                                break;
                            }
                            Ok(n) => {
                                let data = buffer[..n].to_vec();
                                if sender.send(data).is_err() {
                                    // Channel closed, stop reading
                                    break;
                                }
                            }
                            Err(e) => {
                                if e.kind() == std::io::ErrorKind::WouldBlock {
                                    // No data available, continue
                                    thread::sleep(Duration::from_millis(10));
                                    continue;
                                }

                                eprintln!("Failed to read from SSH channel: {}", e);
                                // Send error through channel if possible
                                let error_msg = format!("SSH read error: {}", e).into_bytes();
                                let _ = sender.send(error_msg);
                                break;
                            }
                        }
                    } else {
                        // Could not acquire lock, wait a bit and try again
                        thread::sleep(Duration::from_millis(10));
                        continue;
                    }

                    // Small delay to prevent overwhelming the channel
                    thread::sleep(Duration::from_millis(1));
                }
            });

            Ok(())
        } else {
            Err(AppError::TerminalError("SSH terminal not connected".to_string()))
        }
    }
}
