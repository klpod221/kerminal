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

        let service_clone = service.clone();
        tokio::spawn(async move {
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

        let status = {
            let active_tunnels = self.active_tunnels.read().await;
            if let Some(handle) = active_tunnels.get(id) {
                let status = handle.status.read().await;
                status.clone()
            } else {
                TunnelStatus::Stopped
            }
        };

        let error_message = {
            let active_tunnels = self.active_tunnels.read().await;
            if let Some(handle) = active_tunnels.get(id) {
                let error = handle.error_message.read().await;
                error.clone()
            } else {
                None
            }
        };

        Ok(TunnelWithStatus {
            tunnel,
            status,
            error_message,
        })
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
        if let Err(e) = self.stop_tunnel(id.to_string()).await {
            eprintln!("Failed to stop tunnel before deletion: {}", e);
        }

        let db_service = self.database_service.lock().await;
        db_service.delete_ssh_tunnel(id).await
    }

    /// Start SSH tunnel
    pub async fn start_tunnel(&self, tunnel_id: String) -> Result<(), String> {
        // Check if tunnel is already running (not in error state)
        {
            let active_tunnels = self.active_tunnels.read().await;
            if let Some(handle) = active_tunnels.get(&tunnel_id) {
                let status = handle.status.read().await;
                match *status {
                    TunnelStatus::Running | TunnelStatus::Starting => {
                        return Err("Tunnel is already running".to_string());
                    }
                    TunnelStatus::Error | TunnelStatus::Stopped => {
                        // Allow restart from error or stopped state
                        drop(status);
                    }
                }
            }
        }

        // Remove existing handle if in error/stopped state
        {
            let mut active_tunnels = self.active_tunnels.write().await;
            active_tunnels.remove(&tunnel_id);
        }

        let tunnel = {
            let db_service = self.database_service.lock().await;
            db_service
                .get_ssh_tunnel(&tunnel_id)
                .await
                .map_err(|e| format!("Failed to get tunnel: {}", e))?
        };

        let profile = {
            let db_service = self.database_service.lock().await;
            db_service
                .get_ssh_profile(&tunnel.profile_id)
                .await
                .map_err(|e| format!("Failed to get SSH profile: {}", e))?
        };

        let cancel_token = CancellationToken::new();
        let status = Arc::new(RwLock::new(TunnelStatus::Starting));
        let error_message = Arc::new(RwLock::new(None));

        let handle = TunnelHandle {
            cancel_token: cancel_token.clone(),
            status: status.clone(),
            error_message: error_message.clone(),
        };

        {
            let mut active_tunnels = self.active_tunnels.write().await;
            active_tunnels.insert(tunnel_id.clone(), handle);
        }

        let tunnel_clone = tunnel.clone();
        let profile_clone = profile.clone();
        let sessions_arc = self.ssh_sessions.clone();
        let tunnel_id_clone = tunnel_id.clone();
        let active_tunnels_arc = self.active_tunnels.clone();

        tokio::spawn(async move {
            match Self::run_tunnel(
                tunnel_clone,
                profile_clone,
                cancel_token,
                status.clone(),
                error_message.clone(),
                sessions_arc,
            )
            .await
            {
                Err(e) => {
                    eprintln!("Tunnel {} failed: {}", tunnel_id_clone, e);

                    // Ensure error status is set
                    {
                        let mut error_msg = error_message.write().await;
                        if error_msg.is_none() {
                            *error_msg = Some(e.to_string());
                        }
                    }

                    {
                        let mut status_guard = status.write().await;
                        *status_guard = TunnelStatus::Error;
                    }

                    // Keep in active tunnels with error status so UI can show the error
                    // User needs to manually stop/restart the tunnel
                }
                Ok(_) => {
                    // Tunnel stopped normally (e.g., cancelled by user)
                    // Remove from active tunnels
                    tokio::time::sleep(tokio::time::Duration::from_millis(100)).await;
                    let mut active_tunnels = active_tunnels_arc.write().await;
                    active_tunnels.remove(&tunnel_id_clone);
                }
            }
        });

        Ok(())
    }

    /// Stop SSH tunnel
    pub async fn stop_tunnel(&self, tunnel_id: String) -> Result<(), String> {
        let mut active_tunnels = self.active_tunnels.write().await;

        if let Some(handle) = active_tunnels.remove(&tunnel_id) {
            handle.cancel_token.cancel();

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

        {
            let mut status_guard = status.write().await;
            *status_guard = TunnelStatus::Running;
        }

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

        if let Err(e) = result {
            let mut error_msg = error_message.write().await;
            *error_msg = Some(format!("Tunnel error: {}", e));
            let mut status_guard = status.write().await;
            *status_guard = TunnelStatus::Error;
            return Err(e);
        }

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

        {
            let sessions_guard = sessions.read().await;
            if let Some(session) = sessions_guard.get(&session_key) {
                return Ok(session.clone());
            }
        }

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
        local_host: String,
        local_port: u16,
        remote_host: String,
        remote_port: u16,
        session: Arc<Mutex<Handle<SSHClientHandler>>>,
        cancel_token: CancellationToken,
    ) -> Result<()> {
        // For remote forwarding, bind address should be empty string or "0.0.0.0"
        // to allow SSH server to bind on appropriate interface
        let bind_address = if remote_host == "localhost" || remote_host == "127.0.0.1" {
            "127.0.0.1"
        } else {
            "" // Empty string lets SSH server decide
        };

        let forwarded_port = {
            let mut session_guard = session.lock().await;
            match session_guard
                .tcpip_forward(bind_address, remote_port as u32)
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

        eprintln!(
            "✅ Remote port forwarding established: {}:{} -> {}:{}",
            bind_address, actual_port, local_host, local_port
        );

        let bind_address_clone = bind_address.to_string();

        let mut heartbeat_counter = 0u32;
        let heartbeat_interval = 120; // 60 seconds (500ms * 120)

        loop {
            tokio::select! {
                _ = cancel_token.cancelled() => {
                    let cancel_result = {
                        let session_guard = session.lock().await;
                        session_guard.cancel_tcpip_forward(&bind_address_clone, actual_port as u32).await
                    };

                    match cancel_result {
                        Ok(_) => {
                            eprintln!("✅ Remote port forwarding cancelled: {}:{}", bind_address_clone, actual_port);
                        },
                        Err(e) => {
                            eprintln!("❌ Failed to cancel remote port forwarding: {}", e);
                        }
                    }
                    break;
                }
                _ = tokio::time::sleep(tokio::time::Duration::from_millis(500)) => {
                    heartbeat_counter += 1;

                    // Health check every 5 minutes
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

        let mut buffer = [0u8; 512];

        let n = local_stream.read(&mut buffer).await?;
        if n < 3 || buffer[0] != 0x05 {
            return Err(anyhow::anyhow!("Invalid SOCKS5 greeting"));
        }

        local_stream.write_all(&[0x05, 0x00]).await?;

        let n = local_stream.read(&mut buffer).await?;
        if n < 4 || buffer[0] != 0x05 || buffer[1] != 0x01 {
            return Err(anyhow::anyhow!("Invalid SOCKS5 request"));
        }

        let (target_host, target_port) = match buffer[3] {
            0x01 => {
                if n < 10 {
                    return Err(anyhow::anyhow!("Invalid IPv4 request"));
                }
                let ip = Ipv4Addr::new(buffer[4], buffer[5], buffer[6], buffer[7]);
                let port = u16::from_be_bytes([buffer[8], buffer[9]]);
                (ip.to_string(), port)
            }
            0x03 => {
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

        let channel_result = {
            let session_guard = session.lock().await;
            session_guard
                .channel_open_direct_tcpip(&target_host, target_port as u32, "127.0.0.1", 0)
                .await
        };

        match channel_result {
            Ok(channel) => {
                let response = [
                    0x05, 0x00, 0x00, 0x01, // SOCKS5, success, reserved, IPv4
                    0x00, 0x00, 0x00, 0x00, // Bind IP (0.0.0.0)
                    0x00, 0x00, // Bind port (0)
                ];
                local_stream.write_all(&response).await?;

                Self::proxy_socks_connection(local_stream, session, channel, cancel_token).await?;
            }
            Err(e) => {
                eprintln!("Failed to establish SSH channel: {}", e);
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

        let _ = channel.eof().await;
        let _ = channel.close().await;

        Ok(())
    }
}
