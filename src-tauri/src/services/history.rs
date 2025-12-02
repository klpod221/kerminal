use crate::core::proxy::create_proxy_stream;
use crate::error::AppError;
use crate::models::history::{
    CommandHistoryEntry, ExportHistoryRequest, GetTerminalHistoryRequest, SearchHistoryRequest,
    SearchHistoryResponse,
};
use crate::models::ssh::{AuthData, SSHProfile};
use crate::models::terminal::TerminalType;
use crate::services::ssh::SSHService;
use crate::services::terminal::TerminalManager;
use chrono::{DateTime, Utc};
use russh::client::{Handler, Session};
use russh_keys::key::PublicKey;
use std::collections::HashMap;
use std::path::PathBuf;
use std::sync::Arc;
use tokio::sync::RwLock;

/// History manager for terminal command history
pub struct HistoryManager {
    /// Cached history per terminal ID
    history_cache: Arc<RwLock<HashMap<String, Vec<CommandHistoryEntry>>>>,
    /// Terminal manager reference to get terminal info
    terminal_manager: Arc<TerminalManager>,
    /// SSH service for accessing SSH profiles
    ssh_service: Arc<SSHService>,
}

impl HistoryManager {
    /// Create a new history manager
    pub fn new(terminal_manager: Arc<TerminalManager>, ssh_service: Arc<SSHService>) -> Self {
        Self {
            history_cache: Arc::new(RwLock::new(HashMap::new())),
            terminal_manager,
            ssh_service,
        }
    }

    /// Load history for a terminal from shell history file
    /// For local terminals, this reads from local history files
    /// For SSH terminals, this reads from remote history files
    pub async fn load_terminal_history(
        &self,
        terminal_id: &str,
    ) -> Result<Vec<CommandHistoryEntry>, AppError> {
        // Check cache first
        {
            let cache = self.history_cache.read().await;
            if let Some(history) = cache.get(terminal_id) {
                return Ok(history.clone());
            }
        }

        // Get terminal info to determine type
        let terminal_info = self
            .terminal_manager
            .get_terminal_info(terminal_id.to_string())
            .await?;

        let history = match terminal_info.config.terminal_type {
            TerminalType::Local => {
                // Load from local history file
                // Note: All local terminals share the same history file
                self.load_local_history().await?
            }
            TerminalType::SSH | TerminalType::SSHConfig => {
                // Load from remote SSH server by executing command
                self.load_remote_history(terminal_id).await?
            }
        };

        // Cache the history per terminal ID
        // This ensures each terminal (local or SSH) has its own cache entry
        // Even though local terminals share the same source file, they have separate cache entries
        {
            let mut cache = self.history_cache.write().await;
            cache.insert(terminal_id.to_string(), history.clone());
        }

        Ok(history)
    }

    /// Load history from local shell history files
    async fn load_local_history(&self) -> Result<Vec<CommandHistoryEntry>, AppError> {
        let home_dir = dirs::home_dir()
            .ok_or_else(|| AppError::config_error("Could not determine home directory"))?;

        // Try zsh history first (more common on macOS)
        let zsh_history_path = home_dir.join(".zsh_history");
        if zsh_history_path.exists() {
            return self.parse_zsh_history(&zsh_history_path).await;
        }

        // Fall back to bash history
        let bash_history_path = home_dir.join(".bash_history");
        if bash_history_path.exists() {
            return self.parse_bash_history(&bash_history_path).await;
        }

        // No history file found
        Ok(Vec::new())
    }

    /// Load history from remote SSH server
    async fn load_remote_history(
        &self,
        terminal_id: &str,
    ) -> Result<Vec<CommandHistoryEntry>, AppError> {
        // Get terminal info to get SSH profile
        let terminal_info = self
            .terminal_manager
            .get_terminal_info(terminal_id.to_string())
            .await?;

        // Get SSH profile based on terminal type
        match terminal_info.config.terminal_type {
            TerminalType::SSH => {
                let profile_id = terminal_info
                    .config
                    .ssh_profile_id
                    .as_ref()
                    .ok_or_else(|| {
                        AppError::invalid_config("SSH profile ID not found".to_string())
                    })?;
                self.load_remote_history_from_profile_id(profile_id).await
            }
            TerminalType::SSHConfig => {
                let ssh_config_host =
                    terminal_info
                        .config
                        .ssh_config_host
                        .as_ref()
                        .ok_or_else(|| {
                            AppError::invalid_config("SSH config host not found".to_string())
                        })?;

                let password = terminal_info.config.ssh_config_password.clone();
                let temp_profile = ssh_config_host
                    .to_temporary_profile(password)
                    .map_err(|e| AppError::Config(format!("Failed to create profile: {}", e)))?;

                self.load_remote_history_from_profile(&temp_profile).await
            }
            _ => Err(AppError::invalid_config(
                "Terminal is not an SSH terminal".to_string(),
            )),
        }
    }

    /// Parse zsh history file
    /// Zsh history format: `: <timestamp>:<duration>;<command>`
    async fn parse_zsh_history(
        &self,
        path: &PathBuf,
    ) -> Result<Vec<CommandHistoryEntry>, AppError> {
        // Read file as bytes first to handle non-UTF8 encoding
        let bytes = tokio::fs::read(path)
            .await
            .map_err(|e| AppError::General(format!("Failed to read zsh history: {}", e)))?;

        // Try to decode as UTF-8, replacing invalid sequences
        let content = String::from_utf8_lossy(&bytes);

        let mut entries = Vec::new();
        let mut index = 0;

        for line in content.lines() {
            // Skip empty lines
            if line.trim().is_empty() {
                continue;
            }

            // Zsh format: `: <timestamp>:<duration>;<command>`
            if line.starts_with(": ") {
                let parts: Vec<&str> = line[2..].splitn(2, ';').collect();
                if parts.len() == 2 {
                    let timestamp_part = parts[0];
                    let command = parts[1].trim();

                    // Skip lines with null bytes or other invalid characters
                    if command.contains('\0') {
                        continue;
                    }

                    // Parse timestamp (epoch seconds)
                    let timestamp = timestamp_part
                        .split(':')
                        .next()
                        .and_then(|ts| ts.parse::<i64>().ok())
                        .map(|ts| DateTime::from_timestamp(ts, 0).unwrap_or_else(Utc::now));

                    if !command.is_empty() {
                        entries.push(CommandHistoryEntry {
                            command: command.to_string(),
                            timestamp,
                            index,
                        });
                        index += 1;
                    }
                }
            } else {
                // Fallback: treat as simple command (for older zsh or non-extended format)
                let command = line.trim();

                // Skip lines with null bytes or other invalid characters
                if command.contains('\0') {
                    continue;
                }

                if !command.is_empty() {
                    entries.push(CommandHistoryEntry {
                        command: command.to_string(),
                        timestamp: None,
                        index,
                    });
                    index += 1;
                }
            }
        }

        // Reverse to get most recent first
        entries.reverse();
        Ok(entries)
    }

    /// Parse bash history file
    /// Bash history format: one command per line (simple format)
    async fn parse_bash_history(
        &self,
        path: &PathBuf,
    ) -> Result<Vec<CommandHistoryEntry>, AppError> {
        // Read file as bytes first to handle non-UTF8 encoding
        let bytes = tokio::fs::read(path)
            .await
            .map_err(|e| AppError::General(format!("Failed to read bash history: {}", e)))?;

        // Try to decode as UTF-8, replacing invalid sequences
        let content = String::from_utf8_lossy(&bytes);

        let mut entries = Vec::new();
        let mut index = 0;

        for line in content.lines() {
            let command = line.trim();

            // Skip lines with null bytes or other invalid characters
            if command.contains('\0') {
                continue;
            }

            if !command.is_empty() {
                entries.push(CommandHistoryEntry {
                    command: command.to_string(),
                    timestamp: None, // Bash doesn't store timestamps by default
                    index,
                });
                index += 1;
            }
        }

        // Reverse to get most recent first
        entries.reverse();
        Ok(entries)
    }

    /// Get history for a terminal
    pub async fn get_history(
        &self,
        request: GetTerminalHistoryRequest,
    ) -> Result<Vec<CommandHistoryEntry>, AppError> {
        let mut history = self.load_terminal_history(&request.terminal_id).await?;

        // Apply limit if specified
        if let Some(limit) = request.limit {
            if limit > 0 && limit < history.len() {
                history.truncate(limit);
            }
        }

        Ok(history)
    }

    /// Search history for a terminal
    pub async fn search_history(
        &self,
        request: SearchHistoryRequest,
    ) -> Result<SearchHistoryResponse, AppError> {
        let history = self.load_terminal_history(&request.terminal_id).await?;
        let query_lower = request.query.to_lowercase();

        let mut filtered: Vec<CommandHistoryEntry> = history
            .into_iter()
            .filter(|entry| entry.command.to_lowercase().contains(&query_lower))
            .collect();

        let total_count = filtered.len();

        // Apply limit if specified
        if let Some(limit) = request.limit {
            if limit > 0 && limit < filtered.len() {
                filtered.truncate(limit);
            }
        }

        Ok(SearchHistoryResponse {
            entries: filtered,
            total_count,
        })
    }

    /// Export history to file
    pub async fn export_history(&self, request: ExportHistoryRequest) -> Result<String, AppError> {
        let history = if let Some(query) = &request.query {
            // Filter by query first
            let search_result = self
                .search_history(SearchHistoryRequest {
                    terminal_id: request.terminal_id.clone(),
                    query: query.clone(),
                    limit: None,
                })
                .await?;
            search_result.entries
        } else {
            // Get all history
            self.get_history(GetTerminalHistoryRequest {
                terminal_id: request.terminal_id.clone(),
                limit: None,
            })
            .await?
        };

        let content = match request.format.as_str() {
            "json" => serde_json::to_string_pretty(&history)
                .map_err(|e| AppError::General(format!("Failed to serialize JSON: {}", e)))?,
            "txt" => history
                .iter()
                .map(|entry| {
                    if let Some(ts) = entry.timestamp {
                        format!("[{}] {}\n", ts.format("%Y-%m-%d %H:%M:%S"), entry.command)
                    } else {
                        format!("{}\n", entry.command)
                    }
                })
                .collect::<String>(),
            _ => {
                return Err(AppError::Validation(format!(
                    "Unsupported export format: {}",
                    request.format
                )))
            }
        };

        tokio::fs::write(&request.file_path, content)
            .await
            .map_err(|e| {
                AppError::General(format!(
                    "Failed to write export file {}: {}",
                    request.file_path, e
                ))
            })?;

        Ok(request.file_path)
    }

    /// Load history from remote using SSH profile ID
    async fn load_remote_history_from_profile_id(
        &self,
        profile_id: &str,
    ) -> Result<Vec<CommandHistoryEntry>, AppError> {
        let profile = self
            .ssh_service
            .get_ssh_profile(profile_id)
            .await
            .map_err(|e| AppError::Database(e.to_string()))?;
        self.load_remote_history_from_profile(&profile).await
    }

    /// Load history from remote using SSH profile
    async fn load_remote_history_from_profile(
        &self,
        profile: &SSHProfile,
    ) -> Result<Vec<CommandHistoryEntry>, AppError> {
        // Create temporary SSH session to execute command
        let config = Arc::new(russh::client::Config::default());
        let handler = RemoteCommandHandler::new();

        let mut session = if let Some(proxy_config) = &profile.proxy {
            let stream = create_proxy_stream(proxy_config, &profile.host, profile.port)
                .await
                .map_err(|e| {
                    AppError::connection_failed(format!("Failed to create proxy connection: {}", e))
                })?;
            russh::client::connect_stream(config, stream, handler).await
        } else {
            russh::client::connect(config, (&profile.host as &str, profile.port), handler).await
        }
        .map_err(|e| {
            AppError::connection_failed(format!(
                "Failed to connect to SSH server {}:{}: {}",
                profile.host, profile.port, e
            ))
        })?;

        // Authenticate
        match &profile.auth_data {
            AuthData::Password { password } => {
                let authenticated = session
                    .authenticate_password(&profile.username, password)
                    .await
                    .map_err(|e| {
                        AppError::authentication_failed(format!(
                            "Password authentication failed: {}",
                            e
                        ))
                    })?;
                if !authenticated {
                    let _ = session
                        .disconnect(russh::Disconnect::ByApplication, "", "en")
                        .await;
                    return Err(AppError::authentication_failed(
                        "Password authentication failed".to_string(),
                    ));
                }
            }
            AuthData::KeyReference { .. } => {
                // Key-based auth requires SSH key service which we don't have direct access to
                // For now, return empty - can be enhanced later
                let _ = session
                    .disconnect(russh::Disconnect::ByApplication, "", "en")
                    .await;
                return Ok(Vec::new());
            }
            AuthData::Certificate { .. } => {
                // Certificate auth not implemented yet
                let _ = session
                    .disconnect(russh::Disconnect::ByApplication, "", "en")
                    .await;
                return Ok(Vec::new());
            }
        }

        // Execute command to get history
        // Try to detect shell and read appropriate history file
        let command = "test -f ~/.zsh_history && cat ~/.zsh_history 2>/dev/null || (test -f ~/.bash_history && cat ~/.bash_history 2>/dev/null || echo '')";

        let mut channel = session
            .channel_open_session()
            .await
            .map_err(|e| AppError::terminal_error(format!("Failed to open SSH channel: {}", e)))?;

        channel
            .exec(true, command)
            .await
            .map_err(|e| AppError::terminal_error(format!("Failed to execute command: {}", e)))?;

        // Collect output from channel
        let mut output_bytes = Vec::new();
        loop {
            match channel.wait().await {
                Some(russh::ChannelMsg::Data { data }) => {
                    output_bytes.extend_from_slice(&data);
                }
                Some(russh::ChannelMsg::Eof) => {
                    break;
                }
                Some(russh::ChannelMsg::Close) => {
                    break;
                }
                None => {
                    break;
                }
                _ => {}
            }
        }

        // Close channel and session
        let _ = channel.close().await;
        let _ = session
            .disconnect(russh::Disconnect::ByApplication, "", "en")
            .await;

        // Parse output
        let content = String::from_utf8_lossy(&output_bytes);
        let entries = self.parse_history_content(&content, true).await?;

        Ok(entries)
    }

    /// Parse history content (from string, not file)
    async fn parse_history_content(
        &self,
        content: &str,
        _is_remote: bool,
    ) -> Result<Vec<CommandHistoryEntry>, AppError> {
        let mut entries = Vec::new();
        let mut index = 0;

        for line in content.lines() {
            if line.trim().is_empty() {
                continue;
            }

            // Try zsh format first
            if line.starts_with(": ") {
                let parts: Vec<&str> = line[2..].splitn(2, ';').collect();
                if parts.len() == 2 {
                    let timestamp_part = parts[0];
                    let command = parts[1].trim();

                    if command.contains('\0') {
                        continue;
                    }

                    let timestamp = timestamp_part
                        .split(':')
                        .next()
                        .and_then(|ts| ts.parse::<i64>().ok())
                        .map(|ts| DateTime::from_timestamp(ts, 0).unwrap_or_else(Utc::now));

                    if !command.is_empty() {
                        entries.push(CommandHistoryEntry {
                            command: command.to_string(),
                            timestamp,
                            index,
                        });
                        index += 1;
                    }
                }
            } else {
                // Bash format or simple command
                let command = line.trim();
                if command.contains('\0') {
                    continue;
                }
                if !command.is_empty() {
                    entries.push(CommandHistoryEntry {
                        command: command.to_string(),
                        timestamp: None,
                        index,
                    });
                    index += 1;
                }
            }
        }

        entries.reverse();
        Ok(entries)
    }
}

/// Handler for executing remote commands
#[derive(Clone)]
struct RemoteCommandHandler;

impl RemoteCommandHandler {
    fn new() -> Self {
        Self
    }
}

#[async_trait::async_trait]
impl Handler for RemoteCommandHandler {
    type Error = russh::Error;

    async fn check_server_key(
        &mut self,
        _server_public_key: &PublicKey,
    ) -> Result<bool, Self::Error> {
        Ok(true)
    }

    async fn data(
        &mut self,
        _channel: russh::ChannelId,
        _data: &[u8],
        _session: &mut Session,
    ) -> Result<(), Self::Error> {
        // Data is collected via channel.wait() in the calling code
        Ok(())
    }

    async fn extended_data(
        &mut self,
        _channel: russh::ChannelId,
        _code: u32,
        _data: &[u8],
        _session: &mut Session,
    ) -> Result<(), Self::Error> {
        // Data is collected via channel.wait() in the calling code
        Ok(())
    }
}
