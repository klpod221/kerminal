use crate::core::proxy::create_proxy_stream;
use crate::error::AppError;
use crate::models::ssh::key::ResolvedSSHKey;
use crate::models::ssh::{AuthData, SSHProfile};
use crate::models::terminal::{TerminalConfig, TerminalState};
use async_trait::async_trait;
use russh::client::{DisconnectReason, Handle, Handler, Session};
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
        if let Some(sender) = self.output_sender.lock().await.as_ref() {
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
        if let Some(sender) = self.output_sender.lock().await.as_ref() {
            let _ = sender.send(data.to_vec());
        }
        Ok(())
    }

    async fn channel_eof(
        &mut self,
        _channel: ChannelId,
        _session: &mut Session,
    ) -> Result<(), Self::Error> {
        if let Some(sender) = self.output_sender.lock().await.as_ref() {
            let eof_msg = b"[SSH: Connection closed by remote host]\r\n";
            let _ = sender.send(eof_msg.to_vec());
        }

        if let Some(sender) = self.exit_sender.lock().await.as_ref() {
            let terminal_id = self.terminal_id.lock().await;
            let exit_event = crate::models::terminal::TerminalExited {
                terminal_id: terminal_id.clone(),
                exit_code: Some(0),
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

    async fn disconnected(
        &mut self,
        reason: DisconnectReason<Self::Error>,
    ) -> Result<(), Self::Error> {
        let (message, exit_code, reason_str) = match &reason {
            DisconnectReason::ReceivedDisconnect(disconnect) => {
                let msg = if disconnect.message.is_empty() {
                    "[SSH: Connection disconnected by server]\r\n".to_string()
                } else {
                    format!(
                        "[SSH: Connection disconnected - {}]\r\n",
                        disconnect.message
                    )
                };
                (msg, Some(1), "server-disconnect")
            }
            DisconnectReason::Error(e) => {
                let msg = if format!("{:?}", e).contains("timeout")
                    || format!("{:?}", e).contains("Timeout")
                {
                    "[SSH: Connection timeout - No response from server]\r\n".to_string()
                } else {
                    format!("[SSH: Connection error - {}]\r\n", e)
                };
                (msg, Some(1), "connection-error")
            }
        };

        let output_sender = self.output_sender.lock().await;
        if let Some(sender) = output_sender.as_ref() {
            let _ = sender.send(message.as_bytes().to_vec());
        }

        let exit_sender = self.exit_sender.lock().await;
        if let Some(sender) = exit_sender.as_ref() {
            let terminal_id = self.terminal_id.lock().await;
            let exit_event = crate::models::terminal::TerminalExited {
                terminal_id: terminal_id.clone(),
                exit_code,
                reason: Some(reason_str.to_string()),
            };
            let _ = sender.send(exit_event);
        }

        match reason {
            DisconnectReason::ReceivedDisconnect(_) => Ok(()),
            DisconnectReason::Error(e) => Err(e),
        }
    }
}

/// SSH terminal implementation using russh
pub struct SSHTerminal {
    config: TerminalConfig,
    ssh_profile: SSHProfile,
    state: TerminalState,
    session: Option<Arc<Handle<ClientHandler>>>,
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
        database_service: Option<
            Arc<tokio::sync::Mutex<crate::database::service::DatabaseService>>,
        >,
    ) -> Result<Self, AppError> {
        let handler = Arc::new(ClientHandler::new(id.clone()));
        Ok(SSHTerminal {
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
        self.state = TerminalState::Connecting;

        let keepalive_interval = if self.ssh_profile.keep_alive {
            Some(std::time::Duration::from_secs(15))
        } else {
            None
        };

        let inactivity_timeout = self
            .ssh_profile
            .timeout
            .map(|t| std::time::Duration::from_secs(t as u64));

        let mut config = russh::client::Config {
            inactivity_timeout,
            keepalive_interval,
            keepalive_max: 10,
            ..<russh::client::Config as Default>::default()
        };

        config.window_size = 2097152;
        config.maximum_packet_size = 32768;

        let config = Arc::new(config);

        let handler = (*self.handler).clone();

        // Clone jump_hosts to avoid borrow checker issues
        let jump_hosts_cloned = self.ssh_profile.jump_hosts.clone();

        // Check if we need to connect via jump hosts
        let mut session = if let Some(jump_hosts) = &jump_hosts_cloned {
            if !jump_hosts.is_empty() {
                self.connect_via_jump_hosts(
                    jump_hosts,
                    config.clone(),
                    handler,
                    resolved_key.clone(),
                )
                .await?
            } else {
                self.connect_direct(config, handler).await?
            }
        } else {
            self.connect_direct(config, handler).await?
        };

        // Authenticate with the final host (only if we didn't go through jump hosts,
        // as jump host path handles its own authentication)
        if jump_hosts_cloned.is_none()
            || jump_hosts_cloned
                .as_ref()
                .map(|jh| jh.is_empty())
                .unwrap_or(true)
        {
            self.authenticate_with_resolved_data(&mut session, resolved_key)
                .await?;
        }

        let channel = session.channel_open_session().await.map_err(|e| {
            self.state = TerminalState::Disconnected;
            AppError::terminal_error(format!("Failed to open SSH channel: {}", e))
        })?;

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

        // Handle command, working directory, and environment variables
        let mut command_parts: Vec<String> = Vec::new();

        // Inject environment variables via export commands
        if let Some(env) = &self.ssh_profile.env {
            for (key, value) in env {
                // Escape single quotes in value
                let escaped_value = value.replace("'", "'\\''");
                command_parts.push(format!("export {}='{}'", key, escaped_value));
            }
        }

        if let Some(wd) = &self.ssh_profile.working_dir {
            if !wd.is_empty() {
                command_parts.push(format!("cd \"{}\"", wd));
            }
        }

        if let Some(cmd) = &self.ssh_profile.command {
            if !cmd.is_empty() {
                command_parts.push(cmd.clone());
            }
        }

        if !command_parts.is_empty() {
            // Join parts with && to ensure sequence
            let mut full_command = command_parts.join(" && ");
            // Append shell execution to keep session open
            full_command.push_str("; exec ${SHELL:-bash} -l");

            let _ = channel.exec(false, full_command.as_bytes()).await;
        } else {
            let _ = channel.request_shell(false).await;
        }

        self.session = Some(Arc::new(session));
        self.channel = Some(channel);
        self.state = TerminalState::Connected;

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

                if let Some(db_service) = &self.database_service {
                    let db = db_service.lock().await;
                    if let Err(e) = db.mark_key_used(key_id).await {
                        eprintln!("Warning: Failed to mark SSH key {} as used: {}", key_id, e);
                    }
                }
            }
            AuthData::Certificate {
                certificate,
                private_key,
                key_type: _,
                validity_period: _,
            } => {
                let key = if Path::new(private_key).exists() {
                    russh_keys::load_secret_key(private_key, None).map_err(|e| {
                        AppError::authentication_failed(format!(
                            "Failed to load private key: {}",
                            e
                        ))
                    })?
                } else {
                    russh_keys::decode_secret_key(private_key, None).map_err(|e| {
                        AppError::authentication_failed(format!(
                            "Failed to parse private key: {}",
                            e
                        ))
                    })?
                };

                let _cert = if Path::new(certificate).exists() {
                    russh_keys::load_public_key(certificate).map_err(|e| {
                        AppError::authentication_failed(format!(
                            "Failed to load certificate: {}",
                            e
                        ))
                    })?
                } else {
                    russh_keys::parse_public_key_base64(certificate).map_err(|e| {
                        AppError::authentication_failed(format!(
                            "Failed to parse certificate: {}",
                            e
                        ))
                    })?
                };

                let result = session
                    .authenticate_publickey(username, Arc::new(key))
                    .await
                    .map_err(|e| {
                        AppError::authentication_failed(format!(
                            "Certificate authentication error for user '{}': {}",
                            username, e
                        ))
                    })?;

                if !result {
                    return Err(AppError::authentication_failed(format!(
                        "Certificate authentication failed for user '{}'",
                        username
                    )));
                }
            }
        }

        Ok(())
    }

    /// Connect directly to the target (with optional proxy)
    async fn connect_direct(
        &mut self,
        config: Arc<russh::client::Config>,
        handler: ClientHandler,
    ) -> Result<Handle<ClientHandler>, AppError> {
        if let Some(proxy_config) = &self.ssh_profile.proxy {
            let stream =
                create_proxy_stream(proxy_config, &self.ssh_profile.host, self.ssh_profile.port)
                    .await
                    .map_err(|e| {
                        self.state = TerminalState::Disconnected;
                        AppError::connection_failed(format!(
                            "Failed to create proxy connection: {}",
                            e
                        ))
                    })?;

            russh::client::connect_stream(config, stream, handler)
                .await
                .map_err(|e| {
                    self.state = TerminalState::Disconnected;
                    AppError::connection_failed(format!(
                        "Failed to connect to SSH server {}:{}: {}",
                        self.ssh_profile.host, self.ssh_profile.port, e
                    ))
                })
        } else {
            russh::client::connect(
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
            })
        }
    }

    /// Connect to the final target via jump hosts
    async fn connect_via_jump_hosts(
        &mut self,
        jump_hosts: &[crate::models::ssh::profile::JumpHostConfig],
        config: Arc<russh::client::Config>,
        handler: ClientHandler,
        _resolved_key: Option<ResolvedSSHKey>,
    ) -> Result<Handle<ClientHandler>, AppError> {
        // We need to chain connections through jump hosts
        // Support profile_id references and inline config

        let db_service = self.database_service.as_ref().ok_or_else(|| {
            AppError::Config("Database service required for jump host resolution".to_string())
        })?;

        // Collect all jump profiles we need
        let mut jump_profiles = Vec::new();
        for jh in jump_hosts {
            if let Some(profile_id) = &jh.profile_id {
                // Get the profile with decrypted password
                let profile = {
                    let db = db_service.lock().await;
                    db.get_ssh_profile(profile_id).await.map_err(|e| {
                        AppError::Config(format!(
                            "Failed to get jump host profile '{}': {}",
                            profile_id, e
                        ))
                    })?
                }; // Lock released here
                jump_profiles.push(profile);
            } else if let (Some(host), Some(port), Some(username)) =
                (&jh.host, jh.port, &jh.username)
            {
                // Create a temporary profile from inline config
                let mut temp_profile = crate::models::ssh::SSHProfile::new(
                    "temp".to_string(),
                    format!("jump-{}", host),
                    host.clone(),
                    port,
                    username.clone(),
                );
                if let Some(auth_method) = &jh.auth_method {
                    temp_profile.auth_method = auth_method.clone();
                }
                if let Some(auth_data) = &jh.auth_data {
                    temp_profile.auth_data = auth_data.clone();
                }
                jump_profiles.push(temp_profile);
            } else {
                return Err(AppError::Config(
                    "Invalid jump host config: need either profile_id or inline config".to_string(),
                ));
            }
        }

        if jump_profiles.is_empty() {
            return self.connect_direct(config, handler).await;
        }

        // Connect to the first jump host
        let first_jump = &jump_profiles[0];
        let jump_handler = ClientHandler::new(format!("jump-{}", first_jump.host));

        let mut current_session: Handle<ClientHandler> = russh::client::connect(
            config.clone(),
            (first_jump.host.as_str(), first_jump.port),
            jump_handler,
        )
        .await
        .map_err(|e| {
            self.state = TerminalState::Disconnected;
            AppError::connection_failed(format!(
                "Failed to connect to first jump host {}:{}: {}",
                first_jump.host, first_jump.port, e
            ))
        })?;

        // Authenticate with first jump host
        self.authenticate_jump_host(&mut current_session, first_jump)
            .await?;

        // Chain through remaining jump hosts
        for i in 1..jump_profiles.len() {
            let next_jump = &jump_profiles[i];
            let jump_handler = ClientHandler::new(format!("jump-{}", next_jump.host));

            // Open a direct TCP/IP channel to the next jump host
            let channel = current_session
                .channel_open_direct_tcpip(&next_jump.host, next_jump.port as u32, "127.0.0.1", 0)
                .await
                .map_err(|e| {
                    self.state = TerminalState::Disconnected;
                    AppError::connection_failed(format!(
                        "Failed to forward to jump host {}:{}: {}",
                        next_jump.host, next_jump.port, e
                    ))
                })?;

            // Connect through the forwarded channel
            current_session =
                russh::client::connect_stream(config.clone(), channel.into_stream(), jump_handler)
                    .await
                    .map_err(|e| {
                        self.state = TerminalState::Disconnected;
                        AppError::connection_failed(format!(
                            "Failed to establish SSH through channel to {}:{}: {}",
                            next_jump.host, next_jump.port, e
                        ))
                    })?;

            // Authenticate with this jump host
            self.authenticate_jump_host(&mut current_session, next_jump)
                .await?;
        }

        // Now forward to the final target
        let channel = current_session
            .channel_open_direct_tcpip(
                &self.ssh_profile.host,
                self.ssh_profile.port as u32,
                "127.0.0.1",
                0,
            )
            .await
            .map_err(|e| {
                self.state = TerminalState::Disconnected;
                AppError::connection_failed(format!(
                    "Failed to forward to final target {}:{}: {}",
                    self.ssh_profile.host, self.ssh_profile.port, e
                ))
            })?;

        // Connect to final target through the channel
        let mut final_session =
            russh::client::connect_stream(config, channel.into_stream(), handler)
                .await
                .map_err(|e| {
                    self.state = TerminalState::Disconnected;
                    AppError::connection_failed(format!(
                        "Failed to establish SSH to final target {}:{}: {}",
                        self.ssh_profile.host, self.ssh_profile.port, e
                    ))
                })?;

        // Authenticate with the final target
        self.authenticate_with_resolved_data(&mut final_session, None)
            .await?;

        Ok(final_session)
    }

    /// Authenticate with a jump host
    async fn authenticate_jump_host(
        &self,
        session: &mut Handle<ClientHandler>,
        profile: &crate::models::ssh::SSHProfile,
    ) -> Result<(), AppError> {
        use crate::models::ssh::AuthData;

        let username = &profile.username;

        match &profile.auth_data {
            AuthData::Password { password } => {
                let result = session
                    .authenticate_password(username, password)
                    .await
                    .map_err(|e| {
                        AppError::authentication_failed(format!(
                            "Jump host password auth error for '{}@{}': {}",
                            username, profile.host, e
                        ))
                    })?;

                if !result {
                    return Err(AppError::authentication_failed(format!(
                        "Jump host password auth failed for '{}@{}'",
                        username, profile.host
                    )));
                }
            }
            AuthData::KeyReference { key_id } => {
                // Resolve the key from database
                if let Some(db_service) = &self.database_service {
                    let db = db_service.lock().await;
                    let key = db.get_ssh_key(key_id).await.map_err(|e| {
                        AppError::authentication_failed(format!(
                            "Failed to get jump host key: {}",
                            e
                        ))
                    })?;
                    let secret_key =
                        russh_keys::decode_secret_key(&key.private_key, key.passphrase.as_deref())
                            .map_err(|e| {
                                AppError::authentication_failed(format!(
                                    "Failed to parse jump host key: {}",
                                    e
                                ))
                            })?;

                    let result = session
                        .authenticate_publickey(username, Arc::new(secret_key))
                        .await
                        .map_err(|e| {
                            AppError::authentication_failed(format!(
                                "Jump host key auth error for '{}@{}': {}",
                                username, profile.host, e
                            ))
                        })?;

                    if !result {
                        return Err(AppError::authentication_failed(format!(
                            "Jump host key auth failed for '{}@{}'",
                            username, profile.host
                        )));
                    }
                } else {
                    return Err(AppError::Config(
                        "Database service required for key auth".to_string(),
                    ));
                }
            }
            AuthData::Certificate { private_key, .. } => {
                let secret_key = russh_keys::decode_secret_key(private_key, None).map_err(|e| {
                    AppError::authentication_failed(format!(
                        "Failed to parse jump host certificate key: {}",
                        e
                    ))
                })?;

                let result = session
                    .authenticate_publickey(username, Arc::new(secret_key))
                    .await
                    .map_err(|e| {
                        AppError::authentication_failed(format!(
                            "Jump host certificate auth error for '{}@{}': {}",
                            username, profile.host, e
                        ))
                    })?;

                if !result {
                    return Err(AppError::authentication_failed(format!(
                        "Jump host certificate auth failed for '{}@{}'",
                        username, profile.host
                    )));
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

    /// Start reading from SSH terminal and send output to the provided sender
    pub async fn start_read_loop(
        &mut self,
        sender: mpsc::UnboundedSender<Vec<u8>>,
        _title_sender: Option<mpsc::UnboundedSender<String>>,
        exit_sender: Option<mpsc::UnboundedSender<crate::models::terminal::TerminalExited>>,
        latency_sender: Option<mpsc::UnboundedSender<crate::models::terminal::TerminalLatency>>,
    ) -> Result<(), AppError> {
        self.handler.set_output_sender(sender).await;
        if let Some(exit_sender) = exit_sender {
            self.handler.set_exit_sender(exit_sender).await;
        }

        // Spawn latency measurement task
        if let Some(latency_sender) = latency_sender {
            if let Some(session) = &self.session {
                let session_handle = session.clone();
                let terminal_id = self.handler.terminal_id.lock().await.clone();

                tokio::spawn(async move {
                    let mut interval = tokio::time::interval(std::time::Duration::from_secs(5));
                    loop {
                        interval.tick().await;

                        let start = std::time::Instant::now();
                        // Use channel_open_session as a ping mechanism
                        // It involves a round-trip to the server
                        match session_handle.channel_open_session().await {
                            Ok(channel) => {
                                let latency = start.elapsed().as_millis() as u64;
                                // Close the channel immediately
                                let _ = channel.close().await;

                                let event = crate::models::terminal::TerminalLatency {
                                    terminal_id: terminal_id.clone(),
                                    latency_ms: latency,
                                };
                                if latency_sender.send(event).is_err() {
                                    break;
                                }
                            }
                            Err(_) => {}
                        }
                    }
                });
            }
        }

        Ok(())
    }
}
