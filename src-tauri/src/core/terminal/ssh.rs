use crate::models::ssh::{AuthData, ProxyConfig, ProxyType, SSHProfile};
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
        println!("🔌 SSH Terminal {}: Starting connection to {}@{}:{}",
                 self.id, self.ssh_profile.username, self.ssh_profile.host, self.ssh_profile.port);

        self.state = TerminalState::Connecting;

        // Use tokio::time::timeout to enforce a connection timeout
        let connection_timeout = Duration::from_secs(
            self.ssh_profile.timeout.map(|t| t as u64).unwrap_or(30)
        );

        println!("⏰ SSH Terminal {}: Connection timeout set to {} seconds", self.id, connection_timeout.as_secs());

        let result = tokio::time::timeout(connection_timeout, async {
            // Create TCP connection (with proxy support if configured)
            let tcp = if let Some(proxy) = &self.ssh_profile.proxy {
                self.connect_via_proxy(proxy).await?
            } else {
                // Use blocking task for TCP connection to avoid blocking the async runtime
                let host = self.ssh_profile.host.clone();
                let port = self.ssh_profile.port;

                tokio::task::spawn_blocking(move || {
                    TcpStream::connect(format!("{}:{}", host, port))
                }).await
                .map_err(|e| AppError::connection_failed(format!("Task join error: {}", e)))?
                .map_err(|e| {
                    AppError::connection_failed(format!(
                        "Failed to connect to {}:{}: {}",
                        self.ssh_profile.host, self.ssh_profile.port, e
                    ))
                })?
            };

            // All SSH operations need to be done in a blocking task since ssh2 is synchronous
            let ssh_profile = self.ssh_profile.clone();
            let (session, channel) = tokio::task::spawn_blocking(move || -> Result<(Session, Channel), AppError> {
                // Create SSH session
                let mut session = Session::new().map_err(|e| {
                    AppError::connection_failed(format!("Failed to create SSH session: {}", e))
                })?;

                // Set connection timeout if specified
                if let Some(timeout) = ssh_profile.timeout {
                    session.set_timeout(timeout * 1000); // Convert to milliseconds
                } else {
                    session.set_timeout(30000); // Default 30 seconds
                }

                // Enable compression if configured
                if ssh_profile.compression {
                    session.set_compress(true);
                }

                // Set TCP stream and perform handshake
                session.set_tcp_stream(tcp);
                session
                    .handshake()
                    .map_err(|e| AppError::connection_failed(format!("SSH handshake failed: {}", e)))?;

                // Authenticate (we need to pass the session by reference, so we can't move ssh_profile)
                Self::authenticate_sync(&ssh_profile, &mut session)?;

                // Create channel
                let mut channel = session.channel_session().map_err(|e| {
                    AppError::connection_failed(format!("Failed to create SSH channel: {}", e))
                })?;

                // Request a pseudo-terminal with better terminal settings
                channel
                    .request_pty("xterm-256color", None, Some((120, 30, 0, 0)))
                    .map_err(|e| AppError::terminal_error(format!("Failed to request PTY: {}", e)))?;

                // Start shell
                channel
                    .shell()
                    .map_err(|e| AppError::terminal_error(format!("Failed to start shell: {}", e)))?;

                Ok((session, channel))
            }).await
            .map_err(|e| AppError::connection_failed(format!("SSH setup task failed: {}", e)))??;

            self.session = Some(Arc::new(Mutex::new(session)));
            self.channel = Some(Arc::new(Mutex::new(channel)));
            self.state = TerminalState::Connected;

            Ok::<(), AppError>(())
        }).await;

        match result {
            Ok(Ok(())) => {
                println!("SSH connection established successfully for terminal {}", self.id);
                Ok(())
            }
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

    /// Synchronous authentication helper (for use in blocking context)
    fn authenticate_sync(ssh_profile: &SSHProfile, session: &mut Session) -> Result<(), AppError> {
        match &ssh_profile.auth_data {
            AuthData::Password { password } => {
                session
                    .userauth_password(&ssh_profile.username, password)
                    .map_err(|e| {
                        AppError::authentication_failed(format!(
                            "Password authentication failed for user '{}': {}",
                            ssh_profile.username, e
                        ))
                    })?;
            }
            AuthData::PrivateKey { private_key, .. } => {
                // For now, assume private_key is a file path
                // TODO: Handle in-memory private keys
                match session.userauth_pubkey_file(
                    &ssh_profile.username,
                    None,
                    std::path::Path::new(private_key),
                    None,
                ) {
                    Ok(_) => {}
                    Err(e) => {
                        return Err(AppError::authentication_failed(format!(
                            "Private key authentication failed for user '{}' with key '{}': {}",
                            ssh_profile.username, private_key, e
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
                    &ssh_profile.username,
                    None,
                    std::path::Path::new(private_key),
                    Some(passphrase.as_str()),
                ) {
                    Ok(_) => {}
                    Err(e) => {
                        return Err(AppError::authentication_failed(format!(
                            "Private key with passphrase authentication failed for user '{}' with key '{}': {}",
                            ssh_profile.username, private_key, e
                        )))
                    }
                }
            }
            AuthData::Agent { .. } => {
                // Try SSH agent authentication
                session
                    .userauth_agent(&ssh_profile.username)
                    .map_err(|e| {
                        AppError::authentication_failed(format!(
                            "SSH agent authentication failed for user '{}': {}",
                            ssh_profile.username, e
                        ))
                    })?;
            }
            _ => {
                return Err(AppError::authentication_failed(format!(
                    "Unsupported authentication method for user '{}'",
                    ssh_profile.username
                )));
            }
        }

        if !session.authenticated() {
            return Err(AppError::authentication_failed(format!(
                "Authentication verification failed for user '{}'",
                ssh_profile.username
            )));
        }

        Ok(())
    }



    /// Connect via proxy
    async fn connect_via_proxy(&self, proxy: &ProxyConfig) -> Result<TcpStream, AppError> {
        match proxy.proxy_type {
            ProxyType::Http => {
                // TODO: Implement HTTP proxy support
                return Err(AppError::connection_failed(
                    "HTTP proxy not yet implemented".to_string(),
                ));
            }
            ProxyType::Socks5 => {
                // TODO: Implement SOCKS5 proxy support
                return Err(AppError::connection_failed(
                    "SOCKS5 proxy not yet implemented".to_string(),
                ));
            }
            ProxyType::Socks4 => {
                // TODO: Implement SOCKS4 proxy support
                return Err(AppError::connection_failed(
                    "SOCKS4 proxy not yet implemented".to_string(),
                ));
            }
        }
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
                AppError::terminal_error(format!("Failed to write to SSH channel: {}", e))
            })?;
            channel_guard.flush().map_err(|e| {
                AppError::terminal_error(format!("Failed to flush SSH channel: {}", e))
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
        if let Some(channel) = &self.channel {
            let mut channel_guard = channel.lock().await;
            channel_guard
                .request_pty_size(cols as u32, rows as u32, None, None)
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
            && self.channel.is_some()
            && self.session.is_some()
    }

    /// Start reading from SSH terminal and send output to the provided sender
    pub async fn start_read_loop(
        &mut self,
        sender: mpsc::UnboundedSender<Vec<u8>>,
    ) -> Result<(), AppError> {
        if let Some(channel) = self.channel.clone() {
            let terminal_id = self.id.clone();

            // Spawn a blocking task to read from SSH channel
            tokio::task::spawn_blocking(move || {
                let mut buffer = [0u8; 8192];
                let mut consecutive_errors = 0;
                const MAX_CONSECUTIVE_ERRORS: u32 = 10;

                loop {
                    // Check if we should exit due to too many consecutive errors
                    if consecutive_errors >= MAX_CONSECUTIVE_ERRORS {
                        eprintln!("SSH terminal {}: Too many consecutive read errors, stopping", terminal_id);
                        let error_msg = "SSH connection unstable - too many read errors".to_string().into_bytes();
                        let _ = sender.send(error_msg);
                        break;
                    }

                    match channel.try_lock() {
                        Ok(mut channel_guard) => {
                            match channel_guard.read(&mut buffer) {
                                Ok(0) => {
                                    // EOF reached, SSH channel has closed normally
                                    eprintln!("SSH terminal {}: Connection closed by remote host", terminal_id);
                                    break;
                                }
                                Ok(n) => {
                                    let data = buffer[..n].to_vec();
                                    if sender.send(data).is_err() {
                                        // Channel closed, stop reading
                                        eprintln!("SSH terminal {}: Output channel closed", terminal_id);
                                        break;
                                    }
                                    consecutive_errors = 0; // Reset error count on successful read
                                }
                                Err(e) => {
                                    match e.kind() {
                                        std::io::ErrorKind::WouldBlock |
                                    std::io::ErrorKind::TimedOut => {
                                        // No data available or timeout, this is normal
                                        thread::sleep(Duration::from_millis(10));
                                        continue;
                                    }
                                        std::io::ErrorKind::ConnectionAborted |
                                        std::io::ErrorKind::ConnectionReset |
                                        std::io::ErrorKind::BrokenPipe => {
                                            // Connection terminated
                                            eprintln!("SSH terminal {}: Connection terminated: {}", terminal_id, e);
                                            let error_msg = format!("SSH connection terminated: {}", e).into_bytes();
                                            let _ = sender.send(error_msg);
                                            break;
                                        }
                                        _ => {
                                            consecutive_errors += 1;

                                            // Only log non-timeout errors or every 10th timeout
                                            let should_log = !e.to_string().contains("Timed out") || consecutive_errors % 10 == 1;
                                            if should_log {
                                                eprintln!("SSH terminal {}: Read error #{}: {}", terminal_id, consecutive_errors, e);
                                            }

                                            // Send error through channel if it's a significant non-timeout error
                                            if !e.to_string().contains("Timed out") && consecutive_errors % 5 == 1 {
                                                let error_msg = format!("SSH read error: {}", e).into_bytes();
                                                if sender.send(error_msg).is_err() {
                                                    eprintln!("SSH terminal {}: Output channel closed during error reporting", terminal_id);
                                                    break;
                                                }
                                            }

                                            // Shorter delay for timeout errors, longer for others
                                            let delay = if e.to_string().contains("Timed out") {
                                                50 // Short delay for timeouts
                                            } else {
                                                std::cmp::min(consecutive_errors * 100, 2000)
                                            };
                                            thread::sleep(Duration::from_millis(delay as u64));
                                        }
                                    }
                                }
                            }
                        }
                        Err(_) => {
                            // Could not acquire lock, wait a bit and try again
                            thread::sleep(Duration::from_millis(10));
                            continue;
                        }
                    }

                    // Small delay to prevent overwhelming the channel
                    thread::sleep(Duration::from_millis(1));
                }

                eprintln!("SSH terminal {}: Read loop terminated", terminal_id);
            });

            Ok(())
        } else {
            Err(AppError::terminal_error(
                "SSH terminal not connected".to_string(),
            ))
        }
    }
}
