use crate::core::title_detector::TitleDetector;
use crate::error::AppError;
use crate::models::terminal::{LocalConfig, TerminalConfig, TerminalExited, TerminalState};
use portable_pty::{Child, CommandBuilder, MasterPty, PtySize};
use std::io::{Read, Write};
use std::sync::Arc;
use std::thread;
use std::time::Duration;
use tokio::sync::{mpsc, Mutex};

/// Local terminal implementation using portable-pty
pub struct LocalTerminal {
    id: String,
    config: TerminalConfig,
    local_config: LocalConfig,
    state: TerminalState,
    pty_pair: Option<(Box<dyn MasterPty + Send>, Box<dyn Child + Send + Sync>)>,
    writer: Option<Arc<Mutex<Box<dyn Write + Send>>>>,
    title_detector: TitleDetector,
}

impl LocalTerminal {
    /// Create a new local terminal instance
    pub fn new(
        id: String,
        config: TerminalConfig,
        local_config: LocalConfig,
    ) -> Result<Self, AppError> {
        Ok(LocalTerminal {
            id,
            config,
            local_config,
            state: TerminalState::Disconnected,
            pty_pair: None,
            writer: None,
            title_detector: TitleDetector::new(),
        })
    }

    /// Connect to the local terminal by creating a PTY
    pub async fn connect(&mut self) -> Result<(), AppError> {
        self.state = TerminalState::Connecting;

        let pty_system = portable_pty::native_pty_system();
        let pty_pair = pty_system
            .openpty(PtySize {
                rows: 24,
                cols: 80,
                pixel_width: 0,
                pixel_height: 0,
            })
            .map_err(|e| AppError::pty_error(e.to_string()))?;

        // Determine shell to use with better detection
        let shell = self
            .local_config
            .shell
            .clone()
            .or_else(|| std::env::var("SHELL").ok())
            .unwrap_or_else(|| {
                if cfg!(windows) {
                    "cmd.exe".to_string()
                } else {
                    // Try to find available shells in order of preference
                    let preferred_shells =
                        ["/bin/zsh", "/usr/bin/zsh", "/bin/bash", "/usr/bin/bash"];
                    for shell_path in &preferred_shells {
                        if std::path::Path::new(shell_path).exists() {
                            return shell_path.to_string();
                        }
                    }
                    "/bin/sh".to_string()
                }
            });

        let mut cmd = CommandBuilder::new(&shell);

        // For interactive shells, add appropriate flags to load user configuration
        if shell.contains("zsh") {
            // Use login shell to load .zshrc and all configurations
            cmd.arg("-l");
        } else if shell.contains("bash") {
            // Use interactive login shell
            cmd.arg("-l");
        }

        // Set working directory if specified
        if let Some(working_dir) = &self.local_config.working_dir {
            cmd.cwd(working_dir);
        }

        // Set essential environment variables for proper terminal operation
        cmd.env("TERM", "xterm-256color");
        cmd.env("COLORTERM", "truecolor");

        // Inherit ALL environment variables from current process
        // This ensures compatibility with all shell configurations and plugins
        for (key, value) in std::env::vars() {
            cmd.env(&key, &value);
        }

        // Set custom environment variables if specified (these can override inherited ones)
        if let Some(env_vars) = &self.local_config.env_vars {
            for (key, value) in env_vars {
                cmd.env(key, value);
            }
        }

        let child = pty_pair
            .slave
            .spawn_command(cmd)
            .map_err(|e| AppError::pty_error(e.to_string()))?;
        let writer = pty_pair
            .master
            .take_writer()
            .map_err(|e| AppError::pty_error(e.to_string()))?;

        self.writer = Some(Arc::new(Mutex::new(writer)));
        self.pty_pair = Some((pty_pair.master, child));
        self.state = TerminalState::Connected;

        Ok(())
    }

    /// Disconnect from the terminal
    pub async fn disconnect(&mut self) -> Result<(), AppError> {
        if let Some((_, mut child)) = self.pty_pair.take() {
            // Try to kill the child process gracefully
            if let Err(e) = child.kill() {
                eprintln!("Failed to kill child process: {}", e);
            }

            // Wait for the process to exit
            match child.wait() {
                Ok(_) => {}
                Err(e) => {
                    eprintln!("Failed to wait for child process: {}", e);
                }
            }
        }

        self.writer = None;
        self.state = TerminalState::Disconnected;
        Ok(())
    }

    /// Write data to the terminal
    pub async fn write(&mut self, data: &[u8]) -> Result<(), AppError> {
        if let Some(writer) = &self.writer {
            let mut writer_guard = writer.lock().await;
            writer_guard.write_all(data)?;
            writer_guard.flush()?;
            Ok(())
        } else {
            Err(AppError::terminal_error(
                "Terminal not connected".to_string(),
            ))
        }
    }

    /// Resize the terminal
    pub async fn resize(&mut self, cols: u16, rows: u16) -> Result<(), AppError> {
        if let Some((pty_master, _)) = &mut self.pty_pair {
            let size = PtySize {
                rows,
                cols,
                pixel_width: 0,
                pixel_height: 0,
            };
            pty_master
                .resize(size)
                .map_err(|e| AppError::pty_error(e.to_string()))?;
            Ok(())
        } else {
            Err(AppError::terminal_error(
                "Terminal not connected".to_string(),
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
        matches!(self.state, TerminalState::Connected) && self.pty_pair.is_some()
    }

    /// Start reading from terminal and send output to the provided sender
    pub async fn start_read_loop(
        &mut self,
        sender: mpsc::UnboundedSender<Vec<u8>>,
        title_sender: Option<mpsc::UnboundedSender<String>>,
        exit_sender: Option<mpsc::UnboundedSender<TerminalExited>>,
    ) -> Result<(), AppError> {
        if let Some((pty_master, _)) = &mut self.pty_pair {
            let mut reader = pty_master
                .try_clone_reader()
                .map_err(|e| AppError::pty_error(e.to_string()))?;

            // Move title detector to the spawned task
            let mut title_detector = std::mem::take(&mut self.title_detector);
            let terminal_id = self.id.clone();

            // Spawn a blocking task to read from PTY
            tokio::task::spawn_blocking(move || {
                let mut buffer = [0u8; 8192];
                loop {
                    match reader.read(&mut buffer) {
                        Ok(0) => {
                            // EOF reached, terminal has closed
                            if let Some(ref exit_sender) = exit_sender {
                                let exit_event = TerminalExited {
                                    terminal_id: terminal_id.clone(),
                                    exit_code: None::<i32>,
                                    reason: Some("user-closed".to_string()),
                                };
                                if exit_sender.send(exit_event).is_err() {
                                    eprintln!(
                                        "Exit event channel closed for terminal {}",
                                        terminal_id
                                    );
                                }
                            }
                            break;
                        }
                        Ok(n) => {
                            let data = buffer[..n].to_vec();

                            // Process output for title detection
                            if let Some(ref title_sender) = title_sender {
                                if let Some(new_title) = title_detector.process_output(&data) {
                                    let _ = title_sender.send(new_title);
                                }
                            }

                            if sender.send(data).is_err() {
                                // Channel closed, stop reading
                                break;
                            }
                        }
                        Err(e) => {
                            eprintln!("Failed to read from PTY: {}", e);
                            let error_msg = format!("PTY read error: {}", e).into_bytes();
                            if sender.send(error_msg).is_err() {
                                eprintln!("Data channel closed for terminal {}", terminal_id);
                            }
                            break;
                        }
                    }

                    // Small delay to prevent overwhelming the channel
                    thread::sleep(Duration::from_millis(1));
                }
            });

            Ok(())
        } else {
            Err(AppError::terminal_error(
                "Terminal not connected".to_string(),
            ))
        }
    }
}
