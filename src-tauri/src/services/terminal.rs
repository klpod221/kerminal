use crate::core::terminal::{TerminalFactory, TerminalWrapper};
use crate::database::service::DatabaseService;
use crate::error::AppError;
use crate::models::terminal::{
    CreateTerminalRequest, CreateTerminalResponse, ResizeTerminalRequest, TerminalData,
    TerminalExited, TerminalInfo, TerminalLatency, TerminalTitleChanged, WriteTerminalRequest,
};
use crate::services::buffer_manager::TerminalBufferManager;
use crate::services::recording::SessionRecorder;
use crate::services::ssh::SSHKeyService;
use std::collections::HashMap;
use std::sync::Arc;
use tauri::{AppHandle, Emitter};
use tokio::sync::{mpsc, Mutex, RwLock};
use uuid::Uuid;

pub struct TerminalManager {
    terminals: Arc<RwLock<HashMap<String, Arc<Mutex<TerminalWrapper>>>>>,
    output_senders: Arc<RwLock<HashMap<String, mpsc::UnboundedSender<Vec<u8>>>>>,
    #[allow(dead_code)]
    output_receiver: Arc<Mutex<Option<mpsc::UnboundedReceiver<TerminalData>>>>,
    output_sender: mpsc::UnboundedSender<TerminalData>,
    buffer_manager: Arc<TerminalBufferManager>,
    database_service: Arc<Mutex<DatabaseService>>,
    ssh_key_service: Option<Arc<Mutex<SSHKeyService>>>,
    pub recorders: Arc<RwLock<HashMap<String, Arc<SessionRecorder>>>>,
    titles: Arc<RwLock<HashMap<String, String>>>,
}

impl TerminalManager {
    pub fn new_with_ssh_key_service(
        database_service: Arc<Mutex<DatabaseService>>,
        ssh_key_service: Arc<Mutex<SSHKeyService>>,
    ) -> Self {
        let (output_sender, output_receiver) = mpsc::unbounded_channel();

        Self {
            terminals: Arc::new(RwLock::new(HashMap::new())),
            output_senders: Arc::new(RwLock::new(HashMap::new())),
            output_receiver: Arc::new(Mutex::new(Some(output_receiver))),
            output_sender,
            buffer_manager: Arc::new(TerminalBufferManager::default()),
            database_service,
            ssh_key_service: Some(ssh_key_service),
            recorders: Arc::new(RwLock::new(HashMap::new())),
            titles: Arc::new(RwLock::new(HashMap::new())),
        }
    }

    pub async fn create_terminal(
        &self,
        request: CreateTerminalRequest,
        app_handle: Option<AppHandle>,
    ) -> Result<CreateTerminalResponse, AppError> {
        let terminal_id = Uuid::new_v4().to_string();

        let mut config = request.config.clone();

        // Apply terminal profile if specified
        if let Some(profile_id) = &config.terminal_profile_id {
            let db_service = self.database_service.lock().await;
            if let Ok(profile) = db_service.get_terminal_profile(profile_id).await {
                if matches!(
                    config.terminal_type,
                    crate::models::terminal::TerminalType::Local
                ) {
                    let mut local_config = config.local_config.unwrap_or_default();

                    if local_config.shell.is_none() {
                        local_config.shell = Some(profile.shell);
                    }

                    if local_config.working_dir.is_none() && profile.working_dir.is_some() {
                        local_config.working_dir = profile.working_dir;
                    }

                    if let Some(profile_env) = profile.env {
                        let mut env = local_config.env_vars.unwrap_or_default();
                        for (k, v) in profile_env {
                            env.entry(k).or_insert(v);
                        }
                        local_config.env_vars = Some(env);
                    }

                    if local_config.command.is_none() && profile.command.is_some() {
                        local_config.command = profile.command;
                    }

                    config.local_config = Some(local_config);
                }
            }
        }

        let mut terminal = TerminalFactory::create_terminal(
            terminal_id.clone(),
            config.clone(),
            Some(self.database_service.clone()),
        )
        .await?;

        let resolved_key = if matches!(
            request.config.terminal_type,
            crate::models::terminal::TerminalType::SSH
        ) {
            if let Some(ssh_key_service) = &self.ssh_key_service {
                if let Some(ssh_profile_id) = &request.config.ssh_profile_id {
                    let ssh_profile = {
                        let db_service = self.database_service.lock().await;
                        db_service
                            .get_ssh_profile(ssh_profile_id)
                            .await
                            .map_err(|e| AppError::Database(e.to_string()))?
                    };

                    if let crate::models::ssh::profile::AuthData::KeyReference { key_id } =
                        &ssh_profile.auth_data
                    {
                        let key_service = ssh_key_service.lock().await;
                        Some(
                            key_service
                                .resolve_key_for_auth(key_id)
                                .await
                                .map_err(|e| AppError::Database(e.to_string()))?,
                        )
                    } else {
                        None
                    }
                } else {
                    None
                }
            } else {
                None
            }
        } else {
            None
        };

        let connect_result = if let Some(resolved_key) = resolved_key {
            terminal
                .connect_with_resolved_data(Some(resolved_key))
                .await
        } else {
            terminal.connect().await
        };

        if let Err(e) = connect_result {
            if let Some(handle) = &app_handle {
                let error_event = TerminalExited {
                    terminal_id: terminal_id.clone(),
                    exit_code: Some(1),
                    reason: Some("error".to_string()),
                };
                let _ = handle.emit("terminal-exited", &error_event);
            }

            return Err(e);
        } else if matches!(
            request.config.terminal_type,
            crate::models::terminal::TerminalType::SSH
                | crate::models::terminal::TerminalType::SSHConfig
        ) {
            if let Some(handle) = &app_handle {
                let success_event = serde_json::json!({
                    "terminalId": terminal_id
                });
                let _ = handle.emit("ssh-connected", &success_event);
            }
        }
        let (tx, mut rx) = mpsc::unbounded_channel::<Vec<u8>>();
        let (title_tx, mut title_rx) = mpsc::unbounded_channel::<String>();
        let (exit_tx, mut exit_rx) = mpsc::unbounded_channel::<TerminalExited>();
        let (latency_tx, mut latency_rx) = mpsc::unbounded_channel::<TerminalLatency>();

        {
            let mut senders = self.output_senders.write().await;
            senders.insert(terminal_id.clone(), tx.clone());
        }

        terminal
            .start_read_loop(tx, Some(title_tx), Some(exit_tx), Some(latency_tx))
            .await?;

        let terminal_id_clone = terminal_id.clone();
        let output_sender = self.output_sender.clone();
        let app_handle_clone = app_handle.clone();
        let buffer_manager_clone = self.buffer_manager.clone();
        let recorders_clone = self.recorders.clone();

        // Track alt screen state for this terminal
        let alt_screen_filter = Arc::new(Mutex::new(AltScreenFilter::new()));

        tokio::spawn(async move {
            while let Some(data) = rx.recv().await {
                let terminal_data = TerminalData {
                    terminal_id: terminal_id_clone.clone(),
                    data: data.clone(),
                };

                // Record output if recording is active (always record raw output)
                if let Some(recorder) = recorders_clone.read().await.get(&terminal_id_clone) {
                    let _ = recorder.record_output(&data).await;
                }

                // Process buffer saving with smart filtering
                if let Ok(data_str) = String::from_utf8(data.clone()) {
                    let mut filter = alt_screen_filter.lock().await;
                    let filtered_data = filter.process(&data_str);

                    if !filtered_data.is_empty() {
                        buffer_manager_clone
                            .save_data(&terminal_id_clone, &filtered_data)
                            .await;
                    }
                }

                if let Some(handle) = &app_handle_clone {
                    let _ = handle.emit("terminal-output", &terminal_data);
                }

                if output_sender.send(terminal_data).is_err() {
                    break;
                }
            }
        });

        let terminal_id_clone = terminal_id.clone();
        let app_handle_clone = app_handle.clone();
        let titles_clone = self.titles.clone();
        let should_lock_title = request.config.terminal_profile_id.is_some();

        tokio::spawn(async move {
            while let Some(new_title) = title_rx.recv().await {
                if should_lock_title {
                    continue;
                }

                {
                    let mut titles = titles_clone.write().await;
                    titles.insert(terminal_id_clone.clone(), new_title.clone());
                }

                let title_event = TerminalTitleChanged {
                    terminal_id: terminal_id_clone.clone(),
                    title: new_title,
                };

                if let Some(handle) = &app_handle_clone {
                    let _ = handle.emit("terminal-title-changed", &title_event);
                }
            }
        });

        let app_handle_clone = app_handle.clone();
        tokio::spawn(async move {
            while let Some(exit_event) = exit_rx.recv().await {
                if let Some(handle) = &app_handle_clone {
                    let _ = handle.emit("terminal-exited", &exit_event);
                }
            }
        });

        let app_handle_clone = app_handle.clone();
        tokio::spawn(async move {
            while let Some(latency_event) = latency_rx.recv().await {
                if let Some(handle) = &app_handle_clone {
                    let _ = handle.emit("terminal-latency", &latency_event);
                }
            }
        });
        let terminal_info = TerminalInfo {
            id: terminal_id.clone(),
            config: request.config,
            state: terminal.get_state(),
            created_at: chrono::Utc::now(),
            title: request.title,
        };

        {
            let mut terminals = self.terminals.write().await;
            terminals.insert(terminal_id.clone(), Arc::new(Mutex::new(terminal)));
        }

        if let Some(title) = &terminal_info.title {
            let mut titles = self.titles.write().await;
            titles.insert(terminal_id.clone(), title.clone());
        }

        Ok(CreateTerminalResponse {
            terminal_id,
            info: terminal_info,
        })
    }

    pub async fn write_to_terminal(&self, request: WriteTerminalRequest) -> Result<(), AppError> {
        let terminals = self.terminals.read().await;

        if let Some(terminal) = terminals.get(&request.terminal_id) {
            let mut terminal_guard = terminal.lock().await;
            terminal_guard.write(request.data.as_bytes()).await?;
            Ok(())
        } else {
            Err(AppError::TerminalNotFound(request.terminal_id))
        }
    }

    pub async fn resize_terminal(&self, request: ResizeTerminalRequest) -> Result<(), AppError> {
        let terminals = self.terminals.read().await;

        if let Some(terminal) = terminals.get(&request.terminal_id) {
            let mut terminal_guard = terminal.lock().await;
            terminal_guard.resize(request.cols, request.rows).await?;
            Ok(())
        } else {
            Err(AppError::TerminalNotFound(request.terminal_id))
        }
    }

    pub async fn close_terminal(&self, terminal_id: String) -> Result<(), AppError> {
        let terminal = {
            let mut terminals = self.terminals.write().await;
            terminals.remove(&terminal_id)
        };

        {
            let mut senders = self.output_senders.write().await;
            senders.remove(&terminal_id);
        }

        {
            let mut titles = self.titles.write().await;
            titles.remove(&terminal_id);
        }

        self.buffer_manager.remove_buffer(&terminal_id).await;

        if let Some(terminal) = terminal {
            let mut terminal_guard = terminal.lock().await;
            terminal_guard.disconnect().await?;
        }

        Ok(())
    }

    pub async fn get_terminal_info(&self, terminal_id: String) -> Result<TerminalInfo, AppError> {
        let terminals = self.terminals.read().await;

        if let Some(terminal) = terminals.get(&terminal_id) {
            let terminal_guard = terminal.lock().await;
            let titles = self.titles.read().await;
            let title = titles.get(&terminal_id).cloned();

            Ok(TerminalInfo {
                id: terminal_id,
                config: terminal_guard.get_config().clone(),
                state: terminal_guard.get_state(),
                created_at: chrono::Utc::now(),
                title,
            })
        } else {
            Err(AppError::TerminalNotFound(terminal_id))
        }
    }

    pub async fn list_terminals(&self) -> Result<Vec<TerminalInfo>, AppError> {
        let terminals = self.terminals.read().await;
        let mut terminal_infos = Vec::new();

        for (terminal_id, terminal) in terminals.iter() {
            let terminal_guard = terminal.lock().await;
            let titles = self.titles.read().await;
            let title = titles.get(terminal_id).cloned();

            terminal_infos.push(TerminalInfo {
                id: terminal_id.clone(),
                config: terminal_guard.get_config().clone(),
                state: terminal_guard.get_state(),
                created_at: chrono::Utc::now(),
                title,
            });
        }

        Ok(terminal_infos)
    }

    pub fn get_buffer_manager(&self) -> Arc<TerminalBufferManager> {
        self.buffer_manager.clone()
    }
}

// Helper struct to track alternate screen state
struct AltScreenFilter {
    in_alt_screen: bool,
}

impl AltScreenFilter {
    fn new() -> Self {
        Self {
            in_alt_screen: false,
        }
    }

    fn process(&mut self, data: &str) -> String {
        let mut result = String::new();
        let mut current_pos = 0;
        let enter_seq = "\x1b[?1049h";
        let exit_seq = "\x1b[?1049l";

        while current_pos < data.len() {
            let remaining = &data[current_pos..];

            if self.in_alt_screen {
                // Look for exit sequence
                if let Some(idx) = remaining.find(exit_seq) {
                    self.in_alt_screen = false;
                    // Skip everything strictly before the exit sequence (it's alt screen content)
                    // But we DO want to capture content AFTER the exit sequence
                    current_pos += idx + exit_seq.len();
                } else {
                    // No exit sequence found, everything remaining is alt screen
                    break;
                }
            } else {
                // Look for enter sequence
                if let Some(idx) = remaining.find(enter_seq) {
                    // Capture everything up to the enter sequence
                    result.push_str(&remaining[..idx]);
                    self.in_alt_screen = true;
                    current_pos += idx + enter_seq.len();
                } else {
                    // No enter sequence, everything remaining is normal buffer
                    result.push_str(remaining);
                    break;
                }
            }
        }

        result
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_normal_output() {
        let mut filter = AltScreenFilter::new();
        assert_eq!(filter.process("hello world\n"), "hello world\n");
    }

    #[test]
    fn test_enter_alt_screen() {
        let mut filter = AltScreenFilter::new();
        // "before" saved, sequence triggers state change, "after" dropped
        assert_eq!(filter.process("before\x1b[?1049h after"), "before");
        assert!(filter.in_alt_screen);
        // Subsequent output dropped
        assert_eq!(filter.process("more alt screen content"), "");
    }

    #[test]
    fn test_exit_alt_screen() {
        let mut filter = AltScreenFilter::new();
        filter.in_alt_screen = true;
        // "alt content" dropped, sequence triggers state change, "after" saved
        assert_eq!(filter.process("alt content\x1b[?1049l after"), " after");
        assert!(!filter.in_alt_screen);
    }

    #[test]
    fn test_toggle_in_single_chunk() {
        let mut filter = AltScreenFilter::new();
        let input = "start \x1b[?1049h inside alt \x1b[?1049l end";
        assert_eq!(filter.process(input), "start  end");
        assert!(!filter.in_alt_screen);
    }

    #[test]
    fn test_multiple_toggles() {
        let mut filter = AltScreenFilter::new();
        let input = "1\x1b[?1049h(hide)\x1b[?1049l2\x1b[?1049h(hide again)";
        assert_eq!(filter.process(input), "12");
        assert!(filter.in_alt_screen);
    }
}
