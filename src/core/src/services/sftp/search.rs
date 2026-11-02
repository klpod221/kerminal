// Copyright (C) 2026 Bùi Thanh Xuân (klpod221)
// SPDX-License-Identifier: GPL-3.0-or-later

use std::sync::Arc;
use std::time::Duration;

use tokio::io::AsyncReadExt;

use crate::models::sftp::search::SearchResult;
use crate::services::sftp::errors::SftpError;
use crate::services::sftp::connection::SFTPClientHandler;

/// Maximum number of search results to return
pub const MAX_SEARCH_RESULTS: usize = 500;

/// Maximum raw output bytes to read from remote grep before truncating
pub const MAX_SEARCH_OUTPUT_BYTES: usize = 2 * 1024 * 1024; // 2MB

/// Search timeout
pub const SEARCH_TIMEOUT: Duration = Duration::from_secs(30);

/// Escape a string for safe inclusion in a POSIX single-quoted shell argument
pub fn escape_shell_arg(arg: &str) -> String {
    format!("'{}'", arg.replace('\'', "'\\''"))
}

/// Execute remote search using grep with safety bounds (timeout, max matches, max bytes)
pub async fn execute_search(
    client: &Arc<russh::client::Handle<SFTPClientHandler>>,
    path: &str,
    query: &str,
    max_results: Option<usize>,
) -> Result<Vec<SearchResult>, SftpError> {
    if query.trim().is_empty() {
        return Ok(Vec::new());
    }

    let limit = max_results.unwrap_or(MAX_SEARCH_RESULTS).max(1);

    // Open a new SSH channel for the search command
    let mut channel = client
        .channel_open_session()
        .await
        .map_err(|e| SftpError::Other {
            message: format!("Failed to open channel for search: {}", e),
        })?;

    let escaped_query = escape_shell_arg(query);
    let escaped_path = escape_shell_arg(path);

    // Use grep with line buffering, recursive, binary-skip (-I), line number (-n), filename (-H)
    let command = format!(
        "grep -rInH -m {} -e {} {}",
        limit, escaped_query, escaped_path
    );

    channel
        .exec(true, command)
        .await
        .map_err(|e| SftpError::Other {
            message: format!("Failed to execute search command: {}", e),
        })?;

    // Read stdout with bounded buffer and timeout
    let output_future = async {
        let mut stdout = channel.make_reader();
        let mut buffer = Vec::new();
        let mut chunk = [0u8; 8192];

        loop {
            let n = stdout.read(&mut chunk).await.map_err(|e| SftpError::Other {
                message: format!("Failed to read search output: {}", e),
            })?;

            if n == 0 {
                break;
            }

            let to_take = n.min(MAX_SEARCH_OUTPUT_BYTES.saturating_sub(buffer.len()));
            buffer.extend_from_slice(&chunk[..to_take]);

            if buffer.len() >= MAX_SEARCH_OUTPUT_BYTES {
                break;
            }
        }

        Ok::<Vec<u8>, SftpError>(buffer)
    };

    let buffer = tokio::time::timeout(SEARCH_TIMEOUT, output_future)
        .await
        .map_err(|_| SftpError::Other {
            message: "Search timed out after 30 seconds".to_string(),
        })??;

    let text = String::from_utf8_lossy(&buffer);
    let mut results = Vec::new();

    for line in text.lines() {
        if results.len() >= limit {
            break;
        }

        // Parse line formatted as: filepath:line:content
        let parts: Vec<&str> = line.splitn(3, ':').collect();
        if parts.len() >= 3 {
            let file_path = parts[0].to_string();
            if let Ok(line_number) = parts[1].parse::<u64>() {
                let content = parts[2].to_string();
                results.push(SearchResult {
                    file_path,
                    line_number,
                    content,
                });
            }
        }
    }

    Ok(results)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_escape_shell_arg() {
        assert_eq!(escape_shell_arg("hello"), "'hello'");
        assert_eq!(escape_shell_arg("don't fail"), "'don'\\''t fail'");
        assert_eq!(escape_shell_arg("$(whoami)`id`"), "'$(whoami)`id`'");
    }
}
