// Copyright (C) 2026 Bùi Thanh Xuân (klpod221)
// SPDX-License-Identifier: GPL-3.0-or-later

use crate::models::ssh::profile::{ProxyConfig, ProxyType};
use russh_config::Stream;

/// Error types for proxy connections
#[derive(Debug, thiserror::Error)]
pub enum ProxyError {
    #[error("Failed to connect to proxy server: {0}")]
    ConnectionFailed(String),
    #[error("Invalid proxy configuration: {0}")]
    InvalidConfig(String),
    #[error("IO error: {0}")]
    Io(#[from] std::io::Error),
}

fn validate_host_or_target(val: &str) -> Result<(), ProxyError> {
    if val.is_empty()
        || val.len() > 255
        || !val
            .chars()
            .all(|c| c.is_ascii_alphanumeric() || c == '.' || c == '-' || c == ':')
    {
        return Err(ProxyError::InvalidConfig(format!("Invalid hostname or IP: {}", val)));
    }
    Ok(())
}

fn validate_safe_credential(val: &str) -> Result<(), ProxyError> {
    if val.chars().any(|c| c == ';' || c == '&' || c == '|' || c == '`' || c == '$' || c == '\n' || c == '\r' || c == '"' || c == '\'') {
        return Err(ProxyError::InvalidConfig("Proxy credentials contain forbidden shell characters".to_string()));
    }
    Ok(())
}

/// Validate proxy configuration
pub fn validate_proxy_config(proxy_config: &ProxyConfig) -> Result<(), ProxyError> {
    validate_host_or_target(&proxy_config.host)?;

    if proxy_config.port == 0 {
        return Err(ProxyError::InvalidConfig(
            "Proxy port must be greater than 0".to_string(),
        ));
    }

    if let Some(username) = &proxy_config.username {
        validate_safe_credential(username)?;
    }
    if let Some(password) = &proxy_config.password {
        validate_safe_credential(password)?;
    }

    Ok(())
}

/// Create a proxy command string for russh-config
pub fn create_proxy_command(
    proxy_config: &ProxyConfig,
    target_host: &str,
    target_port: u16,
) -> Result<String, ProxyError> {
    validate_proxy_config(proxy_config)?;
    validate_host_or_target(target_host)?;

    match proxy_config.proxy_type {
        ProxyType::Http => create_http_proxy_command(proxy_config, target_host, target_port),
        ProxyType::Socks4 => create_socks4_proxy_command(proxy_config, target_host, target_port),
        ProxyType::Socks5 => create_socks5_proxy_command(proxy_config, target_host, target_port),
    }
}

/// Create a Stream for proxy connection using russh-config
pub async fn create_proxy_stream(
    proxy_config: &ProxyConfig,
    target_host: &str,
    target_port: u16,
) -> Result<Stream, ProxyError> {
    let command = create_proxy_command(proxy_config, target_host, target_port)?;

    Stream::proxy_command(&command, &[])
        .await
        .map_err(|e| ProxyError::ConnectionFailed(format!("Proxy command failed: {}", e)))
}

/// Create HTTP CONNECT proxy command
fn create_http_proxy_command(
    proxy_config: &ProxyConfig,
    target_host: &str,
    target_port: u16,
) -> Result<String, ProxyError> {
    if let (Some(username), Some(password)) = (&proxy_config.username, &proxy_config.password) {
        Ok(format!(
            "socat - PROXY:{}:{},proxyport={},proxyauth={}:{}",
            proxy_config.host, target_host, proxy_config.port, username, password
        ))
    } else {
        Ok(format!(
            "nc -X connect -x {}:{} {} {}",
            proxy_config.host, proxy_config.port, target_host, target_port
        ))
    }
}

/// Create SOCKS4 proxy command
fn create_socks4_proxy_command(
    proxy_config: &ProxyConfig,
    target_host: &str,
    target_port: u16,
) -> Result<String, ProxyError> {
    Ok(format!(
        "nc -X 4 -x {}:{} {} {}",
        proxy_config.host, proxy_config.port, target_host, target_port
    ))
}

/// Create SOCKS5 proxy command
fn create_socks5_proxy_command(
    proxy_config: &ProxyConfig,
    target_host: &str,
    target_port: u16,
) -> Result<String, ProxyError> {
    Ok(format!(
        "nc -X 5 -x {}:{} {} {}",
        proxy_config.host, proxy_config.port, target_host, target_port
    ))
}
