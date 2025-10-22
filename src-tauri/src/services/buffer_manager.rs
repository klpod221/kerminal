use crate::models::buffer::TerminalBufferChunk;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::RwLock;

/// Statistics about the buffer manager
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BufferStats {
    pub total_terminals: usize,
    pub total_lines: usize,
    pub memory_usage: usize,
}

/// Terminal buffer data structure
#[derive(Debug, Clone)]
struct TerminalBuffer {
    lines: Vec<String>,
    total_bytes: usize,
}

impl TerminalBuffer {
    fn new() -> Self {
        Self {
            lines: Vec::new(),
            total_bytes: 0,
        }
    }

    fn append_data(&mut self, data: &str, max_lines: usize) {
        let new_lines: Vec<&str> = data.split('\n').collect();

        if !self.lines.is_empty() && !data.starts_with('\n') && !new_lines.is_empty() {
            if let Some(last_line) = self.lines.last_mut() {
                last_line.push_str(new_lines[0]);
                self.total_bytes += new_lines[0].len();
            }

            for line in new_lines.iter().skip(1) {
                self.lines.push(line.to_string());
                self.total_bytes += line.len() + 1; // +1 for newline
            }
        } else {
            for line in new_lines {
                self.lines.push(line.to_string());
                self.total_bytes += line.len() + 1; // +1 for newline
            }
        }

        if self.lines.len() > max_lines {
            let lines_to_remove = self.lines.len() - max_lines;
            for _ in 0..lines_to_remove {
                if !self.lines.is_empty() {
                    let removed_line = self.lines.remove(0);
                    self.total_bytes = self.total_bytes.saturating_sub(removed_line.len() + 1);
                }
            }
        }
    }
    fn get_lines(&self) -> &Vec<String> {
        &self.lines
    }

    fn get_as_string(&self) -> String {
        self.lines.join("\n")
    }

    fn get_memory_usage(&self) -> usize {
        self.total_bytes
    }
}

/// Manager for terminal output buffers
pub struct TerminalBufferManager {
    buffers: Arc<RwLock<HashMap<String, TerminalBuffer>>>,
    max_lines_per_terminal: usize,
}

impl TerminalBufferManager {
    /// Create a new buffer manager
    pub fn new(max_lines_per_terminal: usize) -> Self {
        Self {
            buffers: Arc::new(RwLock::new(HashMap::new())),
            max_lines_per_terminal,
        }
    }

    /// Save data to a terminal's buffer
    pub async fn save_data(&self, terminal_id: &str, data: &str) {
        let mut buffers = self.buffers.write().await;
        let buffer = buffers
            .entry(terminal_id.to_string())
            .or_insert_with(TerminalBuffer::new);
        buffer.append_data(data, self.max_lines_per_terminal);
    }

    /// Get buffer as string for a terminal
    pub async fn get_buffer_string(&self, terminal_id: &str) -> Option<String> {
        let buffers = self.buffers.read().await;
        buffers
            .get(terminal_id)
            .map(|buffer| buffer.get_as_string())
    }

    /// Get buffer chunk for a terminal
    pub async fn get_buffer_chunk(
        &self,
        terminal_id: &str,
        start_line: usize,
        chunk_size: usize,
    ) -> TerminalBufferChunk {
        let buffers = self.buffers.read().await;

        if let Some(buffer) = buffers.get(terminal_id) {
            let lines = buffer.get_lines();
            let total_lines = lines.len();

            let end_line = std::cmp::min(start_line + chunk_size, total_lines);
            let has_more = end_line < total_lines;

            let chunk_lines = if start_line < total_lines {
                &lines[start_line..end_line]
            } else {
                &[]
            };

            let data = chunk_lines.join("\n");

            TerminalBufferChunk {
                terminal_id: terminal_id.to_string(),
                start_line,
                end_line,
                total_lines,
                data,
                has_more,
            }
        } else {
            TerminalBufferChunk {
                terminal_id: terminal_id.to_string(),
                start_line: 0,
                end_line: 0,
                total_lines: 0,
                data: String::new(),
                has_more: false,
            }
        }
    }

    /// Check if a terminal has a buffer
    pub async fn has_buffer(&self, terminal_id: &str) -> bool {
        let buffers = self.buffers.read().await;
        buffers.contains_key(terminal_id)
    }

    /// Remove buffer for a specific terminal
    pub async fn remove_buffer(&self, terminal_id: &str) {
        let mut buffers = self.buffers.write().await;
        buffers.remove(terminal_id);
    }

    /// Get buffer statistics
    pub async fn get_stats(&self) -> BufferStats {
        let buffers = self.buffers.read().await;
        let total_terminals = buffers.len();
        let mut total_lines = 0;
        let mut memory_usage = 0;

        for buffer in buffers.values() {
            total_lines += buffer.get_lines().len();
            memory_usage += buffer.get_memory_usage();
        }

        BufferStats {
            total_terminals,
            total_lines,
            memory_usage,
        }
    }

    /// Cleanup orphaned buffers (call with list of active terminal IDs)
    pub async fn cleanup_orphaned_buffers(&self, active_terminal_ids: &[String]) {
        let mut buffers = self.buffers.write().await;

        let mut to_remove = Vec::new();
        for terminal_id in buffers.keys() {
            if !active_terminal_ids.contains(terminal_id) {
                to_remove.push(terminal_id.clone());
            }
        }

        for terminal_id in to_remove {
            buffers.remove(&terminal_id);
        }
    }
}

impl Default for TerminalBufferManager {
    fn default() -> Self {
        Self::new(1000) // Default to 1000 lines per terminal
    }
}
