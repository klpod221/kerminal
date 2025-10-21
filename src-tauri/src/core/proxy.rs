
use crate::models::ssh::profile::{ProxyConfig, ProxyType};
use russh_config::Stream;

/// Error types for proxy connections
#[derive(Debug, thiserror::Error)]
pub enum ProxyError {
    #[error("Failed to connect to proxy server: {0}")]
    ConnectionFailed(String),
    #[error("IO error: {0}")]
    Io(#[from] std::io::Error),
}

/// Create a proxy command string for russh-config
pub fn create_proxy_command(
    proxy_config: &ProxyConfig,
    target_host: &str,
    target_port: u16,
) -> Result<String, ProxyError> {
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
    // Use nc (netcat) with HTTP CONNECT support
    if let (Some(_username), Some(_password)) = (&proxy_config.username, &proxy_config.password) {
        // Note: nc doesn't directly support HTTP auth, we'd need a more sophisticated approach
        // For now, use a simple connect command and log the auth requirement
        println!("Warning: HTTP proxy authentication not fully supported in netcat command");
        Ok(format!(
            "nc -X connect -x {}:{} {} {}",
            proxy_config.host, proxy_config.port, target_host, target_port
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
