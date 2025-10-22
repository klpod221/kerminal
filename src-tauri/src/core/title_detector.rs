use regex::Regex;

/// Detects terminal title from various sources
pub struct TitleDetector {
    /// Regex to match ANSI title escape sequences
    ansi_title_regex: Regex,
    /// Regex to match user@hostname from prompts
    user_host_regex: Regex,
    /// Last detected title
    last_title: Option<String>,
}

impl TitleDetector {
    pub fn new() -> Self {
        let ansi_title_regex = Regex::new(r"\x1b\](?:0|2);([^\x07\x1b]*?)(?:\x07|\x1b\\)").unwrap();

        let user_host_regex = Regex::new(r"\b([a-zA-Z0-9_-]+)@([a-zA-Z0-9._-]+)\b").unwrap();

        Self {
            ansi_title_regex,
            user_host_regex,
            last_title: None,
        }
    }

    /// Process terminal output and detect title changes
    pub fn process_output(&mut self, data: &[u8]) -> Option<String> {
        let text = String::from_utf8_lossy(data);

        if let Some(ansi_title) = self.extract_title_from_ansi(&text) {
            if let Some(user_host) = self.extract_user_host_from_text(&ansi_title) {
                if Some(&user_host) != self.last_title.as_ref() {
                    self.last_title = Some(user_host.clone());
                    return Some(user_host);
                }
            }
        }

        None
    }

    /// Extract title from ANSI escape sequences
    fn extract_title_from_ansi(&self, text: &str) -> Option<String> {
        if let Some(captures) = self.ansi_title_regex.captures(text) {
            if let Some(title_match) = captures.get(1) {
                let title = title_match.as_str().trim();
                if !title.is_empty() {
                    return Some(title.to_string());
                }
            }
        }
        None
    }

    /// Extract user@hostname from text
    fn extract_user_host_from_text(&self, text: &str) -> Option<String> {
        if let Some(captures) = self.user_host_regex.captures(text) {
            if let (Some(user), Some(host)) = (captures.get(1), captures.get(2)) {
                let user_str = user.as_str();
                let host_str = host.as_str();

                if user_str.is_empty()
                    || host_str.is_empty()
                    || user_str.len() > 50
                    || host_str.len() > 50
                {
                    return None;
                }

                if user_str.len() < 2 {
                    return None;
                }

                if host_str.chars().all(|c| c.is_ascii_digit() || c == '.') {
                    return None;
                }

                return Some(format!("{}@{}", user_str, host_str));
            }
        }

        None
    }
}

impl Default for TitleDetector {
    fn default() -> Self {
        Self::new()
    }
}
