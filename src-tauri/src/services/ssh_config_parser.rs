use crate::error::AppError;
use crate::models::ssh::SSHConfigHost;
use std::path::PathBuf;

/**
 * Parse SSH config file and extract host configurations
 *
 * @param config_path - Path to SSH config file (defaults to ~/.ssh/config)
 * @returns Vector of SSHConfigHost entries
 */
pub async fn parse_ssh_config(config_path: Option<PathBuf>) -> Result<Vec<SSHConfigHost>, AppError> {
    let path = match config_path {
        Some(p) => p,
        None => {
            let home = dirs::home_dir().ok_or_else(|| {
                AppError::config_error("Could not determine home directory")
            })?;
            home.join(".ssh").join("config")
        }
    };

    if !path.exists() {
        return Ok(Vec::new());
    }

    let config_content = tokio::fs::read_to_string(&path)
        .await
        .map_err(|e| AppError::config_error(format!("Failed to read SSH config: {}", e)))?;

    parse_config_content(&config_content)
}

/**
 * Parse SSH config content string
 *
 * @param content - SSH config file content
 * @returns Vector of SSHConfigHost entries
 */
fn parse_config_content(content: &str) -> Result<Vec<SSHConfigHost>, AppError> {
    let mut hosts = Vec::new();
    let mut current_host: Option<SSHConfigHost> = None;

    for line in content.lines() {
        let trimmed = line.trim();

        if trimmed.is_empty() || trimmed.starts_with('#') {
            continue;
        }

        let parts: Vec<&str> = trimmed.splitn(2, char::is_whitespace).collect();
        if parts.len() < 2 {
            continue;
        }

        let keyword = parts[0].to_lowercase();
        let value = parts[1].trim();

        match keyword.as_str() {
            "host" => {
                if let Some(host) = current_host.take() {
                    hosts.push(host);
                }

                for pattern in value.split_whitespace() {
                    if !pattern.contains('*') && !pattern.contains('?') {
                        current_host = Some(SSHConfigHost {
                            name: pattern.to_string(),
                            hostname: pattern.to_string(),
                            port: 22,
                            user: None,
                            identity_file: None,
                            proxy_jump: None,
                            proxy_command: None,
                            forward_agent: None,
                            other_options: None,
                        });
                        break;
                    }
                }
            }
            "hostname" => {
                if let Some(ref mut host) = current_host {
                    host.hostname = value.to_string();
                }
            }
            "port" => {
                if let Some(ref mut host) = current_host {
                    if let Ok(port) = value.parse::<u16>() {
                        host.port = port;
                    }
                }
            }
            "user" => {
                if let Some(ref mut host) = current_host {
                    host.user = Some(value.to_string());
                }
            }
            "identityfile" => {
                if let Some(ref mut host) = current_host {
                    let expanded_path = expand_tilde(value);
                    host.identity_file = Some(expanded_path);
                }
            }
            "proxyjump" => {
                if let Some(ref mut host) = current_host {
                    host.proxy_jump = Some(value.to_string());
                }
            }
            "proxycommand" => {
                if let Some(ref mut host) = current_host {
                    host.proxy_command = Some(value.to_string());
                }
            }
            "forwardagent" => {
                if let Some(ref mut host) = current_host {
                    host.forward_agent = Some(value.to_lowercase() == "yes");
                }
            }
            _ => {}
        }
    }

    if let Some(host) = current_host {
        hosts.push(host);
    }

    Ok(hosts)
}

/**
 * Expand tilde (~) in file paths to home directory
 *
 * @param path - Path that may contain tilde
 * @returns Expanded path
 */
fn expand_tilde(path: &str) -> String {
    if path.starts_with("~/") {
        if let Some(home) = dirs::home_dir() {
            return path.replacen("~", &home.to_string_lossy(), 1);
        }
    }
    path.to_string()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_basic_config() {
        let config = r#"
# Test config
Host myserver
    HostName example.com
    Port 2222
    User admin

Host another
    HostName 192.168.1.100
    User root
    IdentityFile ~/.ssh/id_rsa
"#;

        let result = parse_config_content(config).unwrap();
        assert_eq!(result.len(), 2);

        assert_eq!(result[0].name, "myserver");
        assert_eq!(result[0].hostname, "example.com");
        assert_eq!(result[0].port, 2222);
        assert_eq!(result[0].user, Some("admin".to_string()));

        assert_eq!(result[1].name, "another");
        assert_eq!(result[1].hostname, "192.168.1.100");
        assert_eq!(result[1].user, Some("root".to_string()));
    }

    #[test]
    fn test_parse_with_wildcards() {
        let config = r#"
Host *.example.com
    User admin

Host specific
    HostName server.com
"#;

        let result = parse_config_content(config).unwrap();
        assert_eq!(result.len(), 1);
        assert_eq!(result[0].name, "specific");
    }
}
