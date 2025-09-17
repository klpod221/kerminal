use crate::database::models::ssh_profile::{AuthData, ProxyConfig, ProxyType, SSHProfile};
use crate::error::AppError;
use crate::models::terminal::{TerminalConfig, TerminalState};
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
    ssh_profile: SSHProfile,
    state: TerminalState,
    session: Option<Arc<Mutex<Session>>>,
    channel: Option<Arc<Mutex<Channel>>>,
}

impl SSHTerminal {
    /// Create a new SSH terminal instance
    pub fn new(
        id: String,
        config: TerminalConfig,
        ssh_profile: SSHProfile,
    ) -> Result<Self, AppError> {
        Ok(SSHTerminal {
            id,
            config,
            ssh_profile,
            state: TerminalState::Disconnected,
            session: None,
            channel: None,
        })
    }

    /// Connect to the SSH terminal
    pub async fn connect(&mut self) -> Result<(), AppError> {
        self.state = TerminalState::Connecting;

        // Create TCP connection (with proxy support if configured)
        let tcp = if let Some(proxy) = &self.ssh_profile.proxy {
            self.connect_via_proxy(proxy).await?
        } else {
            TcpStream::connect(format!(
                "{}:{}",
                self.ssh_profile.host, self.ssh_profile.port
            ))
            .map_err(|e| {
                AppError::ConnectionFailed(format!(
                    "Failed to connect to {}:{}: {}",
                    self.ssh_profile.host, self.ssh_profile.port, e
                ))
            })?
        };

        // Create SSH session
        let mut session = Session::new().map_err(|e| {
            AppError::ConnectionFailed(format!("Failed to create SSH session: {}", e))
        })?;

        // Set connection timeout if specified
        if let Some(timeout) = self.ssh_profile.timeout {
            session.set_timeout(timeout * 1000); // Convert to milliseconds
        }

        // Enable compression if configured
        if self.ssh_profile.compression {
            session.set_compress(true);
        }

        session.set_tcp_stream(tcp);
        session
            .handshake()
            .map_err(|e| AppError::ConnectionFailed(format!("SSH handshake failed: {}", e)))?;

        // Authenticate
        self.authenticate(&mut session).await?;

        // Create channel
        let mut channel = session.channel_session().map_err(|e| {
            AppError::ConnectionFailed(format!("Failed to create SSH channel: {}", e))
        })?;

        // Request a pseudo-terminal
        channel
            .request_pty("xterm", None, Some((80, 24, 0, 0)))
            .map_err(|e| AppError::TerminalError(format!("Failed to request PTY: {}", e)))?;

        // Start shell
        channel
            .shell()
            .map_err(|e| AppError::TerminalError(format!("Failed to start shell: {}", e)))?;

        self.session = Some(Arc::new(Mutex::new(session)));
        self.channel = Some(Arc::new(Mutex::new(channel)));
        self.state = TerminalState::Connected;

        Ok(())
    }

    /// Authenticate with the SSH server
    async fn authenticate(&self, session: &mut Session) -> Result<(), AppError> {
        match &self.ssh_profile.auth_data {
            AuthData::Password { password } => {
                session
                    .userauth_password(&self.ssh_profile.username, password)
                    .map_err(|e| {
                        AppError::AuthenticationFailed(format!(
                            "Password authentication failed: {}",
                            e
                        ))
                    })?;
            }
            AuthData::PrivateKey { private_key, .. } => {
                // For now, assume private_key is a file path
                // TODO: Handle in-memory private keys
                match session.userauth_pubkey_file(
                    &self.ssh_profile.username,
                    None,
                    std::path::Path::new(private_key),
                    None,
                ) {
                    Ok(_) => {}
                    Err(e) => {
                        return Err(AppError::AuthenticationFailed(format!(
                            "Private key authentication failed: {}",
                            e
                        )))
                    }
                }
            }
            AuthData::PrivateKeyWithPassphrase {
                private_key,
                passphrase,
                ..
            } => {
                match session.userauth_pubkey_file(
                    &self.ssh_profile.username,
                    None,
                    std::path::Path::new(private_key),
                    Some(passphrase.as_str()),
                ) {
                    Ok(_) => {}
                    Err(e) => {
                        return Err(AppError::AuthenticationFailed(format!(
                            "Private key with passphrase authentication failed: {}",
                            e
                        )))
                    }
                }
            }
            AuthData::Agent { .. } => {
                // Try SSH agent authentication
                session
                    .userauth_agent(&self.ssh_profile.username)
                    .map_err(|e| {
                        AppError::AuthenticationFailed(format!(
                            "SSH agent authentication failed: {}",
                            e
                        ))
                    })?;
            }
            _ => {
                return Err(AppError::AuthenticationFailed(
                    "Unsupported authentication method".to_string(),
                ));
            }
        }

        if !session.authenticated() {
            return Err(AppError::AuthenticationFailed(
                "Authentication failed".to_string(),
            ));
        }

        Ok(())
    }

    /// Connect via proxy
    async fn connect_via_proxy(&self, proxy: &ProxyConfig) -> Result<TcpStream, AppError> {
        match proxy.proxy_type {
            ProxyType::Http => {
                // TODO: Implement HTTP proxy support
                return Err(AppError::ConnectionFailed(
                    "HTTP proxy not yet implemented".to_string(),
                ));
            }
            ProxyType::Socks5 => {
                // TODO: Implement SOCKS5 proxy support
                return Err(AppError::ConnectionFailed(
                    "SOCKS5 proxy not yet implemented".to_string(),
                ));
            }
            ProxyType::Socks4 => {
                // TODO: Implement SOCKS4 proxy support
                return Err(AppError::ConnectionFailed(
                    "SOCKS4 proxy not yet implemented".to_string(),
                ));
            }
        }
    }

    /// Old authentication method - deprecated - will be removed
    async fn authenticate_old(&self, _session: &mut Session) -> Result<(), AppError> {
        // This method is deprecated and will be removed
        Err(AppError::AuthenticationFailed(
            "Deprecated authentication method".to_string(),
        ))
    }

    /// Disconnect from the SSH terminal
    pub async fn disconnect(&mut self) -> Result<(), AppError> {
        if let Some(channel) = self.channel.take() {
            if let Ok(mut channel_guard) = channel.try_lock() {
                if let Err(e) = channel_guard.close() {
                    eprintln!("Failed to close SSH channel: {}", e);
                }
                if let Err(e) = channel_guard.wait_close() {
                    eprintln!("Failed to wait for SSH channel close: {}", e);
                }
            }
        }

        if let Some(session) = self.session.take() {
            if let Ok(session_guard) = session.try_lock() {
                if let Err(e) = session_guard.disconnect(None, "User requested disconnection", None)
                {
                    eprintln!("Failed to disconnect SSH session: {}", e);
                }
            }
        }

        self.state = TerminalState::Disconnected;
        Ok(())
    }

    /// Write data to the SSH terminal
    pub async fn write(&mut self, data: &[u8]) -> Result<(), AppError> {
        if let Some(channel) = &self.channel {
            let mut channel_guard = channel.lock().await;
            channel_guard.write_all(data).map_err(|e| {
                AppError::TerminalError(format!("Failed to write to SSH channel: {}", e))
            })?;
            channel_guard.flush().map_err(|e| {
                AppError::TerminalError(format!("Failed to flush SSH channel: {}", e))
            })?;
            Ok(())
        } else {
            Err(AppError::TerminalError(
                "SSH terminal not connected".to_string(),
            ))
        }
    }

    /// Resize the SSH terminal
    pub async fn resize(&mut self, cols: u16, rows: u16) -> Result<(), AppError> {
        if let Some(channel) = &self.channel {
            let mut channel_guard = channel.lock().await;
            channel_guard
                .request_pty_size(cols as u32, rows as u32, None, None)
                .map_err(|e| {
                    AppError::TerminalError(format!("Failed to resize SSH terminal: {}", e))
                })?;
            Ok(())
        } else {
            Err(AppError::TerminalError(
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
            && self.channel.is_some()
            && self.session.is_some()
    }

    /// Start reading from SSH terminal and send output to the provided sender
    pub async fn start_read_loop(
        &mut self,
        sender: mpsc::UnboundedSender<Vec<u8>>,
    ) -> Result<(), AppError> {
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
                                if let Err(_) = sender.send(error_msg) {
                                    // Channel closed, receiver has been dropped
                                    eprintln!("Data channel closed for SSH terminal");
                                }
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
            Err(AppError::TerminalError(
                "SSH terminal not connected".to_string(),
            ))
        }
    }
}
