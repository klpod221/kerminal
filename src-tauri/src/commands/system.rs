use std::collections::HashSet;
use std::env;
use std::process::Command;

/// Get current user and hostname for terminal title
#[tauri::command]
pub async fn get_user_hostname() -> Result<String, String> {
    let username = env::var("USER")
        .or_else(|_| env::var("USERNAME"))
        .unwrap_or_else(|_| "user".to_string());

    let hostname = gethostname::gethostname().to_string_lossy().to_string();

    Ok(format!("{}@{}", username, hostname))
}

/// Get list of available system fonts
#[tauri::command]
pub fn get_system_fonts() -> Vec<String> {
    let mut fonts = HashSet::new();

    // Try to get fonts using fc-list command (Linux/Unix)
    #[cfg(target_os = "linux")]
    {
        if let Ok(output) = Command::new("fc-list").arg(":").arg("family").output() {
            if let Ok(result) = String::from_utf8(output.stdout) {
                for line in result.lines() {
                    // fc-list returns fonts like "Font Name,Font Name Style"
                    let font_name = line.split(',').next().unwrap_or("").trim();
                    if !font_name.is_empty() {
                        fonts.insert(font_name.to_string());
                    }
                }
            }
        }
    }

    // Try to get fonts on macOS
    #[cfg(target_os = "macos")]
    {
        // Try fc-list first (if fontconfig is installed)
        if let Ok(output) = Command::new("fc-list").arg(":").arg("family").output() {
            if let Ok(result) = String::from_utf8(output.stdout) {
                for line in result.lines() {
                    let font_name = line.split(',').next().unwrap_or("").trim();
                    if !font_name.is_empty() {
                        fonts.insert(font_name.to_string());
                    }
                }
            }
        }
    }

    // Try to get fonts on Windows via Registry (returns proper display names)
    #[cfg(target_os = "windows")]
    {
        use winreg::enums::{HKEY_CURRENT_USER, HKEY_LOCAL_MACHINE};
        use winreg::RegKey;

        const FONTS_REG_PATH: &str =
            "SOFTWARE\\Microsoft\\Windows NT\\CurrentVersion\\Fonts";

        // Suffixes to strip from registry key names, e.g. " (TrueType)"
        let suffixes = [
            " (TrueType)",
            " (OpenType)",
            " (Type 1)",
        ];

        let strip_suffix = |name: &str| -> String {
            let mut s = name.to_string();
            for suffix in &suffixes {
                if s.ends_with(suffix) {
                    s.truncate(s.len() - suffix.len());
                    break;
                }
            }
            s.trim().to_string()
        };

        // System-wide fonts (HKEY_LOCAL_MACHINE)
        if let Ok(hklm) = RegKey::predef(HKEY_LOCAL_MACHINE).open_subkey(FONTS_REG_PATH) {
            for (name, _value) in hklm.enum_values().flatten() {
                let font_name = strip_suffix(&name);
                if !font_name.is_empty() {
                    fonts.insert(font_name);
                }
            }
        }

        // User-installed fonts (HKEY_CURRENT_USER) — e.g. fonts installed without admin
        if let Ok(hkcu) = RegKey::predef(HKEY_CURRENT_USER).open_subkey(FONTS_REG_PATH) {
            for (name, _value) in hkcu.enum_values().flatten() {
                let font_name = strip_suffix(&name);
                if !font_name.is_empty() {
                    fonts.insert(font_name);
                }
            }
        }
    }

    // Always ensure FiraCode Nerd Font is available (bundled with app)
    fonts.insert("FiraCode Nerd Font".to_string());

    let mut font_list: Vec<String> = fonts.into_iter().collect();
    font_list.sort();

    // Move FiraCode Nerd Font to the top of the list
    if let Some(pos) = font_list.iter().position(|f| f == "FiraCode Nerd Font") {
        let firacode = font_list.remove(pos);
        font_list.insert(0, firacode);
    }

    font_list
}
