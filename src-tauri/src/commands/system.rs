use std::env;
use std::collections::HashSet;
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
        if let Ok(output) = Command::new("fc-list")
            .arg(":")
            .arg("family")
            .output()
        {
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
        if let Ok(output) = Command::new("fc-list")
            .arg(":")
            .arg("family")
            .output()
        {
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

    // Try to get fonts on Windows
    #[cfg(target_os = "windows")]
    {
        use std::fs;
        use std::path::Path;

        // Try to read fonts from Windows Fonts directory
        let fonts_dir = Path::new("C:\\Windows\\Fonts");
        if let Ok(entries) = fs::read_dir(fonts_dir) {
            for entry in entries.flatten() {
                if let Some(file_name) = entry.file_name().to_str() {
                    // Extract font name from filename (remove extension and variants)
                    if file_name.ends_with(".ttf") || file_name.ends_with(".otf") {
                        let font_name = file_name
                            .replace(".ttf", "")
                            .replace(".otf", "")
                            .replace("_", " ")
                            .trim()
                            .to_string();

                        // Clean up common suffixes
                        let cleaned = font_name
                            .replace("Bold", "")
                            .replace("Italic", "")
                            .replace("Regular", "")
                            .replace("Light", "")
                            .trim()
                            .to_string();

                        if !cleaned.is_empty() {
                            fonts.insert(cleaned);
                        }
                    }
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
