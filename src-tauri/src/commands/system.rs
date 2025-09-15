use std::env;

/// Get current user and hostname for terminal title
#[tauri::command]
pub async fn get_user_hostname() -> Result<String, String> {
    let username = env::var("USER")
        .or_else(|_| env::var("USERNAME"))
        .unwrap_or_else(|_| "user".to_string());

    let hostname = gethostname::gethostname()
        .to_string_lossy()
        .to_string();

    Ok(format!("{}@{}", username, hostname))
}
