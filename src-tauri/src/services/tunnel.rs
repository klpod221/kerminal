use anyhow::Result;
use async_trait::async_trait;
use russh::client::{Config, Handle};
use russh_keys::key;
use std::collections::HashMap;
use std::sync::Arc;
use tokio::net::TcpListener;
use tokio::sync::{Mutex, RwLock};
use tokio_util::sync::CancellationToken;

use crate::database::{error::DatabaseResult, service::DatabaseService};
use crate::models::ssh::{
    AuthData, CreateSSHTunnelRequest, SSHProfile, SSHTunnel, TunnelStatus, TunnelType,
    TunnelWithStatus, UpdateSSHTunnelRequest,
};

/// SSH Tunnel service for managing port forwarding and SOCKS proxy
#[derive(Clone)]
pub struct TunnelService {
    database_service: Arc<Mutex<DatabaseService>>,
    active_tunnels: Arc<RwLock<HashMap<String, TunnelHandle>>>,
    ssh_sessions: Arc<RwLock<HashMap<String, Arc<Mutex<Handle<SSHClientHandler>>>>>>,
}

/// Handle for an active tunnel
#[derive(Debug)]
struct TunnelHandle {
    cancel_token: CancellationToken,
    status: Arc<RwLock<TunnelStatus>>,
    error_message: Arc<RwLock<Option<String>>>,
}

/// SSH Client Handler for russh
#[derive(Clone)]
pub struct SSHClientHandler;

#[async_trait]
impl russh::client::Handler for SSHClientHandler {
    type Error = russh::Error;

    async fn check_server_key(
        &mut self,
        _server_public_key: &key::PublicKey,
    ) -> Result<bool, Self::Error> {
        // For now, accept all server keys
        // In production, this should implement proper host key verification
        Ok(true)
    }
}

impl TunnelService {
    /// Create new TunnelService instance
    pub fn new(database_service: Arc<Mutex<DatabaseService>>) -> Self {
        Self {
            database_service,
            active_tunnels: Arc::new(RwLock::new(HashMap::new())),
            ssh_sessions: Arc::new(RwLock::new(HashMap::new())),
        }
    }

    /// Create new TunnelService instance and start auto-start tunnels
    pub async fn new_with_auto_start(database_service: Arc<Mutex<DatabaseService>>) -> Self {
        let service = Self::new(database_service);

        // Start auto-start tunnels in the background
        let service_clone = service.clone();
        tokio::spawn(async move {
            // Wait a moment for the app to fully initialize
            tokio::time::sleep(tokio::time::Duration::from_secs(2)).await;

            if let Err(e) = service_clone.start_auto_start_tunnels().await {
                eprintln!("Failed to start auto-start tunnels: {}", e);
            }
        });

        service
    }

    /// Start tunnels that have auto_start enabled
    pub async fn create_tunnel(
        &self,
        request: CreateSSHTunnelRequest,
    ) -> DatabaseResult<SSHTunnel> {
        let db_service = self.database_service.lock().await;
        db_service.create_ssh_tunnel(request).await
    }

    /// Get SSH tunnel by ID
    pub async fn get_tunnel(&self, id: &str) -> DatabaseResult<SSHTunnel> {
        let db_service = self.database_service.lock().await;
        db_service.get_ssh_tunnel(id).await
    }

    /// Get all SSH tunnels
    pub async fn get_all_tunnels(&self) -> DatabaseResult<Vec<SSHTunnel>> {
        let db_service = self.database_service.lock().await;
        db_service.get_ssh_tunnels().await
    }

    /// Get all SSH tunnels with their current status
    pub async fn get_all_tunnels_with_status(&self) -> DatabaseResult<Vec<TunnelWithStatus>> {
        let tunnels = self.get_all_tunnels().await?;
        let mut tunnels_with_status = Vec::new();

        for tunnel in tunnels {
            let tunnel_with_status = self.get_tunnel_with_status(&tunnel.base.id).await?;
            tunnels_with_status.push(tunnel_with_status);
        }

        Ok(tunnels_with_status)
    }

    /// Get tunnel with current status
    pub async fn get_tunnel_with_status(&self, id: &str) -> DatabaseResult<TunnelWithStatus> {
        let tunnel = self.get_tunnel(id).await?;

        // Get current status
        let status = {
            let active_tunnels = self.active_tunnels.read().await;
            if let Some(handle) = active_tunnels.get(id) {
                let status = handle.status.read().await;
                status.clone()
            } else {
                TunnelStatus::Stopped
            }
        };

        // Get error message if any
        let error_message = {
            let active_tunnels = self.active_tunnels.read().await;
            if let Some(handle) = active_tunnels.get(id) {
                let error = handle.error_message.read().await;
                error.clone()
            } else {
                None
            }
        };

        let mut tunnel_with_status = TunnelWithStatus {
            tunnel,
            status,
            error_message: error_message.clone(),
        };

        // Update status in tunnel
        tunnel_with_status.error_message = error_message;

        Ok(tunnel_with_status)
    }

    /// Update SSH tunnel
    pub async fn update_tunnel(
        &self,
        id: &str,
        request: UpdateSSHTunnelRequest,
    ) -> DatabaseResult<SSHTunnel> {
        let db_service = self.database_service.lock().await;
        db_service.update_ssh_tunnel(id, request).await
    }

    /// Delete SSH tunnel
    pub async fn delete_tunnel(&self, id: &str) -> DatabaseResult<()> {
        // Stop tunnel if it's running
        if let Err(e) = self.stop_tunnel(id.to_string()).await {
            eprintln!("Failed to stop tunnel before deletion: {}", e);
        }

        // Delete from database
        let db_service = self.database_service.lock().await;
        db_service.delete_ssh_tunnel(id).await
    }

    /// Start SSH tunnel
    pub async fn start_tunnel(&self, tunnel_id: String) -> Result<(), String> {
        // Check if tunnel is already running
        {
            let active_tunnels = self.active_tunnels.read().await;
            if active_tunnels.contains_key(&tunnel_id) {
                return Err("Tunnel is already running".to_string());
            }
        }

        // Get tunnel configuration
        let tunnel = {
            let db_service = self.database_service.lock().await;
            db_service
                .get_ssh_tunnel(&tunnel_id)
                .await
                .map_err(|e| format!("Failed to get tunnel: {}", e))?
        };

        // Get SSH profile
        let profile = {
            let db_service = self.database_service.lock().await;
            db_service
                .get_ssh_profile(&tunnel.profile_id)
                .await
                .map_err(|e| format!("Failed to get SSH profile: {}", e))?
        };

        // Create tunnel handle
        let cancel_token = CancellationToken::new();
        let status = Arc::new(RwLock::new(TunnelStatus::Starting));
        let error_message = Arc::new(RwLock::new(None));

        let handle = TunnelHandle {
            cancel_token: cancel_token.clone(),
            status: status.clone(),
            error_message: error_message.clone(),
        };

        // Insert handle before starting
        {
            let mut active_tunnels = self.active_tunnels.write().await;
            active_tunnels.insert(tunnel_id.clone(), handle);
        }

        // Clone necessary data for the async task
        let tunnel_clone = tunnel.clone();
        let profile_clone = profile.clone();
        let sessions_arc = self.ssh_sessions.clone();
        let tunnel_id_clone = tunnel_id.clone();

        tokio::spawn(async move {
            if let Err(e) = Self::run_tunnel(
                tunnel_clone,
                profile_clone,
                cancel_token,
                status,
                error_message,
                sessions_arc,
            )
            .await
            {
                eprintln!("Tunnel {} failed: {}", tunnel_id_clone, e);
            }
        });

        Ok(())
    }

    /// Stop SSH tunnel
    pub async fn stop_tunnel(&self, tunnel_id: String) -> Result<(), String> {
        let mut active_tunnels = self.active_tunnels.write().await;

        if let Some(handle) = active_tunnels.remove(&tunnel_id) {
            // Signal cancellation
            handle.cancel_token.cancel();

            // Update status
            {
                let mut status = handle.status.write().await;
                *status = TunnelStatus::Stopped;
            }

            Ok(())
        } else {
            Err("Tunnel is not running".to_string())
        }
    }

    /// Get tunnel status
    pub async fn get_tunnel_status(&self, tunnel_id: String) -> Result<TunnelStatus, String> {
        let active_tunnels = self.active_tunnels.read().await;

        if let Some(handle) = active_tunnels.get(&tunnel_id) {
            let status = handle.status.read().await;
            Ok(status.clone())
        } else {
            Ok(TunnelStatus::Stopped)
        }
    }

    /// Start auto-start tunnels on application launch
    pub async fn start_auto_start_tunnels(&self) -> Result<(), String> {
        let tunnels = {
            let db_service = self.database_service.lock().await;
            db_service
                .get_auto_start_ssh_tunnels()
                .await
                .map_err(|e| format!("Failed to get auto-start tunnels: {}", e))?
        };

        for tunnel in tunnels {
            if let Err(e) = self.start_tunnel(tunnel.base.id.clone()).await {
                eprintln!("Failed to auto-start tunnel {}: {}", tunnel.name, e);
            }
        }

        Ok(())
    }

    /// Run tunnel implementation
    async fn run_tunnel(
        tunnel: SSHTunnel,
        profile: SSHProfile,
        cancel_token: CancellationToken,
        status: Arc<RwLock<TunnelStatus>>,
        error_message: Arc<RwLock<Option<String>>>,
        sessions: Arc<RwLock<HashMap<String, Arc<Mutex<Handle<SSHClientHandler>>>>>>,
    ) -> Result<()> {
        // Get or create SSH session
        let session = match Self::get_or_create_ssh_session(&profile, sessions).await {
            Ok(s) => s,
            Err(e) => {
                let mut error_msg = error_message.write().await;
                *error_msg = Some(format!("Failed to create SSH session: {}", e));
                let mut status_guard = status.write().await;
                *status_guard = TunnelStatus::Error;
                return Err(e);
            }
        };

        // Update status to connected
        {
            let mut status_guard = status.write().await;
            *status_guard = TunnelStatus::Running;
        }

        // Start appropriate tunnel type
        let result = match &tunnel.tunnel_type {
            TunnelType::Local => {
                Self::start_local_forward(
                    tunnel.local_host.clone(),
                    tunnel.local_port,
                    tunnel.remote_host.clone().unwrap_or_default(),
                    tunnel.remote_port.unwrap_or(22),
                    session,
                    cancel_token,
                )
                .await
            }
            TunnelType::Remote => {
                Self::start_remote_forward(
                    tunnel.local_host.clone(),
                    tunnel.local_port,
                    tunnel.remote_host.clone().unwrap_or_default(),
                    tunnel.remote_port.unwrap_or(22),
                    session,
                    cancel_token,
                )
                .await
            }
            TunnelType::Dynamic => {
                Self::start_dynamic_forward(
                    tunnel.local_host.clone(),
                    tunnel.local_port,
                    session,
                    cancel_token,
                )
                .await
            }
        };

        // Handle result and update status/error message
        if let Err(e) = result {
            let mut error_msg = error_message.write().await;
            *error_msg = Some(format!("Tunnel error: {}", e));
            let mut status_guard = status.write().await;
            *status_guard = TunnelStatus::Error;
            return Err(e);
        }

        // Update status to stopped when tunnel ends
        {
            let mut status_guard = status.write().await;
            *status_guard = TunnelStatus::Stopped;
        }

        Ok(())
    }

    /// Get or create SSH session for a profile
    async fn get_or_create_ssh_session(
        profile: &SSHProfile,
        sessions: Arc<RwLock<HashMap<String, Arc<Mutex<Handle<SSHClientHandler>>>>>>,
    ) -> Result<Arc<Mutex<Handle<SSHClientHandler>>>> {
        let session_key = format!("{}:{}@{}", profile.username, profile.port, profile.host);

        // Check if session already exists
        {
            let sessions_guard = sessions.read().await;
            if let Some(session) = sessions_guard.get(&session_key) {
                return Ok(session.clone());
            }
        }

        // Create new SSH session
        let config = Arc::new(Config::default());
        let handler = SSHClientHandler;
        let mut session =
            russh::client::connect(config, (&profile.host as &str, profile.port), handler).await?;

        let authenticated = match &profile.auth_data {
            AuthData::Password { password } => {
                match session
                    .authenticate_password(&profile.username, password)
                    .await
                {
                    Ok(auth_result) => auth_result,
                    Err(e) => {
                        return Err(anyhow::anyhow!("Password authentication failed: {}", e));
                    }
                }
            }
            AuthData::KeyReference { .. } => {
                return Err(anyhow::anyhow!(
                    "Key-based authentication not supported for tunnels"
                ));
            }
            AuthData::Certificate { .. } => {
                return Err(anyhow::anyhow!(
                    "Certificate-based authentication not supported for tunnels"
                ));
            }
        };

        if !authenticated {
            return Err(anyhow::anyhow!(
                "Authentication failed: Invalid credentials"
            ));
        }

        // Store session wrapped in Mutex
        let session_arc = Arc::new(Mutex::new(session));
        {
            let mut sessions_guard = sessions.write().await;
            sessions_guard.insert(session_key, session_arc.clone());
        }

        Ok(session_arc)
    }

    /// Start local port forwarding
    async fn start_local_forward(
        local_host: String,
        local_port: u16,
        remote_host: String,
        remote_port: u16,
        session: Arc<Mutex<Handle<SSHClientHandler>>>,
        cancel_token: CancellationToken,
    ) -> Result<()> {
        let listener = TcpListener::bind(format!("{}:{}", local_host, local_port)).await?;

        loop {
            tokio::select! {
                _ = cancel_token.cancelled() => {
                    break;
                }
                result = listener.accept() => {
                    match result {
                        Ok((stream, _)) => {
                            let channel = {
                                let session_guard = session.lock().await;
                                session_guard.channel_open_direct_tcpip(
                                    &remote_host,
                                    remote_port as u32,
                                    &local_host,
                                    local_port as u32,
                                ).await?
                            };

                            tokio::spawn(Self::proxy_connection(stream, session.clone(), channel, cancel_token.clone()));
                        }
                        Err(e) => {
                            eprintln!("Failed to accept connection: {}", e);
                        }
                    }
                }
            }
        }

        Ok(())
    }

    /// Start remote port forwarding
    async fn start_remote_forward(
        _local_host: String,
        _local_port: u16,
        remote_host: String,
        remote_port: u16,
        session: Arc<Mutex<Handle<SSHClientHandler>>>,
        cancel_token: CancellationToken,
    ) -> Result<()> {
        let forwarded_port = {
            let mut session_guard = session.lock().await;
            match session_guard
                .tcpip_forward(&remote_host, remote_port as u32)
                .await
            {
                Ok(port) => port,
                Err(e) => {
                    return Err(anyhow::anyhow!(
                        "Failed to request remote port forwarding: {}",
                        e
                    ));
                }
            }
        };

        let actual_port = if remote_port == 0 {
            forwarded_port as u16
        } else {
            remote_port
        };

        let _connection_count = 0u32;
        let mut heartbeat_counter = 0u32;
        let heartbeat_interval = 120; // 60 seconds (500ms * 120)

        loop {
            tokio::select! {
                _ = cancel_token.cancelled() => {
                    let cancel_result = {
                        let session_guard = session.lock().await;
                        session_guard.cancel_tcpip_forward(&remote_host, actual_port as u32).await
                    };

                    match cancel_result {
                        Ok(_) => {},
                        Err(e) => {
                            eprintln!("❌ Failed to cancel remote port forwarding: {}", e);
                        }
                    }
                    break;
                }
                _ = tokio::time::sleep(tokio::time::Duration::from_millis(500)) => {
                    heartbeat_counter += 1;

                    if heartbeat_counter.is_multiple_of(heartbeat_interval * 5) {
                        let health_check_result = {
                            let session_guard = session.lock().await;
                            tokio::time::timeout(
                                tokio::time::Duration::from_secs(5),
                                session_guard.channel_open_session()
                            ).await
                        };

                        match health_check_result {
                            Ok(Ok(test_channel)) => {
                                let _ = test_channel.close().await;
                            }
                            Ok(Err(e)) => {
                                eprintln!("❌ SSH session health check failed: {}", e);
                                return Err(anyhow::anyhow!("SSH session unhealthy: {}", e));
                            }
                            Err(_) => {
                                eprintln!("❌ SSH session health check timed out");
                                return Err(anyhow::anyhow!("SSH session timeout"));
                            }
                        }
                    }
                }
            }
        }

        Ok(())
    }

    /// Start dynamic port forwarding (SOCKS proxy)
    async fn start_dynamic_forward(
        local_host: String,
        local_port: u16,
        session: Arc<Mutex<Handle<SSHClientHandler>>>,
        cancel_token: CancellationToken,
    ) -> Result<()> {
        let listener = TcpListener::bind(format!("{}:{}", local_host, local_port)).await?;

        loop {
            tokio::select! {
                _ = cancel_token.cancelled() => {
                    break;
                }
                result = listener.accept() => {
                    match result {
                        Ok((stream, _)) => {
                            tokio::spawn(Self::handle_socks_connection(stream, session.clone(), cancel_token.clone()));
                        }
                        Err(e) => {
                            eprintln!("Failed to accept SOCKS connection: {}", e);
                        }
                    }
                }
            }
        }

        Ok(())
    }

    /// Proxy connection between local and remote
    async fn proxy_connection(
        mut local_stream: tokio::net::TcpStream,
        _session: Arc<Mutex<Handle<SSHClientHandler>>>,
        mut channel: russh::Channel<russh::client::Msg>,
        cancel_token: CancellationToken,
    ) -> Result<()> {
        use russh::ChannelMsg;
        use tokio::io::{AsyncReadExt, AsyncWriteExt};

        let (mut local_reader, mut local_writer) = local_stream.split();
        let mut buffer = [0u8; 8192];

        loop {
            tokio::select! {
                _ = cancel_token.cancelled() => {
                    break;
                }
                // Read from local and write to remote
                result = local_reader.read(&mut buffer) => {
                    match result {
                        Ok(0) => break, // EOF
                        Ok(n) => {
                            if let Err(e) = channel.data(&buffer[..n]).await {
                                eprintln!("Failed to send data to remote: {}", e);
                                break;
                            }
                        }
                        Err(e) => {
                            eprintln!("Failed to read from local stream: {}", e);
                            break;
                        }
                    }
                }
                // Read from remote and write to local
                msg = channel.wait() => {
                    match msg {
                        Some(ChannelMsg::Data { ref data }) => {
                            if let Err(e) = local_writer.write_all(data).await {
                                eprintln!("Failed to write to local stream: {}", e);
                                break;
                            }
                            if let Err(e) = local_writer.flush().await {
                                eprintln!("Failed to flush local stream: {}", e);
                                break;
                            }
                        }
                        Some(ChannelMsg::Eof) | None => {
                            break;
                        }
                        _ => {}
                    }
                }
            }
        }

        // Close the channel gracefully
        let _ = channel.eof().await;
        let _ = channel.close().await;

        Ok(())
    }

    /// Handle SOCKS proxy connection
    async fn handle_socks_connection(
        mut local_stream: tokio::net::TcpStream,
        session: Arc<Mutex<Handle<SSHClientHandler>>>,
        cancel_token: CancellationToken,
    ) -> Result<()> {
        use std::net::{Ipv4Addr, Ipv6Addr};
        use tokio::io::{AsyncReadExt, AsyncWriteExt};

        // SOCKS5 greeting phase
        let mut buffer = [0u8; 512];

        // Read client greeting
        let n = local_stream.read(&mut buffer).await?;
        if n < 3 || buffer[0] != 0x05 {
            return Err(anyhow::anyhow!("Invalid SOCKS5 greeting"));
        }

        // Send greeting response (no authentication required)
        local_stream.write_all(&[0x05, 0x00]).await?;

        // Read connection request
        let n = local_stream.read(&mut buffer).await?;
        if n < 4 || buffer[0] != 0x05 || buffer[1] != 0x01 {
            return Err(anyhow::anyhow!("Invalid SOCKS5 request"));
        }

        // Parse target address
        let (target_host, target_port) = match buffer[3] {
            0x01 => {
                // IPv4
                if n < 10 {
                    return Err(anyhow::anyhow!("Invalid IPv4 request"));
                }
                let ip = Ipv4Addr::new(buffer[4], buffer[5], buffer[6], buffer[7]);
                let port = u16::from_be_bytes([buffer[8], buffer[9]]);
                (ip.to_string(), port)
            }
            0x03 => {
                // Domain name
                if n < 7 {
                    return Err(anyhow::anyhow!("Invalid domain name request"));
                }
                let domain_len = buffer[4] as usize;
                if n < 7 + domain_len {
                    return Err(anyhow::anyhow!("Incomplete domain name request"));
                }
                let domain = String::from_utf8_lossy(&buffer[5..5 + domain_len]).to_string();
                let port = u16::from_be_bytes([buffer[5 + domain_len], buffer[6 + domain_len]]);
                (domain, port)
            }
            0x04 => {
                // IPv6
                if n < 22 {
                    return Err(anyhow::anyhow!("Invalid IPv6 request"));
                }
                let mut ipv6_bytes = [0u8; 16];
                ipv6_bytes.copy_from_slice(&buffer[4..20]);
                let ip = Ipv6Addr::from(ipv6_bytes);
                let port = u16::from_be_bytes([buffer[20], buffer[21]]);
                (ip.to_string(), port)
            }
            _ => {
                return Err(anyhow::anyhow!("Unsupported address type"));
            }
        };

        // Try to establish SSH channel to target
        let channel_result = {
            let session_guard = session.lock().await;
            session_guard
                .channel_open_direct_tcpip(&target_host, target_port as u32, "127.0.0.1", 0)
                .await
        };

        match channel_result {
            Ok(channel) => {
                // Send success response
                let response = [
                    0x05, 0x00, 0x00, 0x01, // SOCKS5, success, reserved, IPv4
                    0x00, 0x00, 0x00, 0x00, // Bind IP (0.0.0.0)
                    0x00, 0x00, // Bind port (0)
                ];
                local_stream.write_all(&response).await?;

                // Start proxying data
                Self::proxy_socks_connection(local_stream, session, channel, cancel_token).await?;
            }
            Err(e) => {
                eprintln!("Failed to establish SSH channel: {}", e);
                // Send connection refused response
                let response = [
                    0x05, 0x05, 0x00, 0x01, // SOCKS5, connection refused, reserved, IPv4
                    0x00, 0x00, 0x00, 0x00, // Bind IP (0.0.0.0)
                    0x00, 0x00, // Bind port (0)
                ];
                let _ = local_stream.write_all(&response).await;
            }
        }

        Ok(())
    }

    /// Proxy SOCKS connection between local stream and SSH channel
    async fn proxy_socks_connection(
        mut local_stream: tokio::net::TcpStream,
        _session: Arc<Mutex<Handle<SSHClientHandler>>>,
        mut channel: russh::Channel<russh::client::Msg>,
        cancel_token: CancellationToken,
    ) -> Result<()> {
        use russh::ChannelMsg;
        use tokio::io::{AsyncReadExt, AsyncWriteExt};

        let (mut local_reader, mut local_writer) = local_stream.split();
        let mut buffer = [0u8; 8192];

        loop {
            tokio::select! {
                _ = cancel_token.cancelled() => {
                    break;
                }
                // Read from local and write to remote
                result = local_reader.read(&mut buffer) => {
                    match result {
                        Ok(0) => break, // EOF
                        Ok(n) => {
                            if let Err(e) = channel.data(&buffer[..n]).await {
                                eprintln!("Failed to send data to remote via SOCKS: {}", e);
                                break;
                            }
                        }
                        Err(e) => {
                            eprintln!("Failed to read from local SOCKS stream: {}", e);
                            break;
                        }
                    }
                }
                // Read from remote and write to local
                msg = channel.wait() => {
                    match msg {
                        Some(ChannelMsg::Data { ref data }) => {
                            if let Err(e) = local_writer.write_all(data).await {
                                eprintln!("Failed to write to local SOCKS stream: {}", e);
                                break;
                            }
                            if let Err(e) = local_writer.flush().await {
                                eprintln!("Failed to flush local SOCKS stream: {}", e);
                                break;
                            }
                        }
                        Some(ChannelMsg::Eof) | None => {
                            break;
                        }
                        _ => {}
                    }
                }
            }
        }

        // Close the channel gracefully
        let _ = channel.eof().await;
        let _ = channel.close().await;

        Ok(())
    }
}
