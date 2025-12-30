use base64::{engine::general_purpose::STANDARD as BASE64, Engine};
use std::collections::HashMap;

/// Parse Kitty graphics protocol escape sequences
/// Format: ESC _G<control_data>;<payload>ESC \
#[derive(Debug, Clone)]
pub struct KittyGraphicsParser {
    pub action: Option<String>,
    pub format: Option<u32>,
    pub transmission_medium: Option<String>,
    pub image_id: Option<u32>,
    pub width: Option<u32>,
    pub height: Option<u32>,
    pub compression: Option<u32>,
    pub payload: Vec<u8>,
}

impl KittyGraphicsParser {
    pub fn new() -> Self {
        Self {
            action: None,
            format: None,
            transmission_medium: None,
            image_id: None,
            width: None,
            height: None,
            compression: None,
            payload: Vec::new(),
        }
    }

    /// Parse Kitty graphics escape sequence
    /// Returns parsed data if valid, None otherwise
    pub fn parse(data: &str) -> Option<Self> {
        // Check for Kitty graphics protocol start
        if !data.contains("\x1b_G") {
            return None;
        }

        // Extract between ESC _G and ESC \
        let start = data.find("\x1b_G")?;
        let end = data[start..].find("\x1b\\")?;
        let sequence = &data[start + 3..start + end];

        // Split control data and payload
        let parts: Vec<&str> = sequence.splitn(2, ';').collect();
        let control_data = parts.get(0)?;
        let payload_str = parts.get(1).unwrap_or(&"");

        // Parse control data (key=value pairs)
        let mut parser = Self::new();
        let params: HashMap<&str, &str> = control_data
            .split(',')
            .filter_map(|pair| {
                let mut parts = pair.splitn(2, '=');
                Some((parts.next()?, parts.next()?))
            })
            .collect();

        // Extract known parameters
        parser.action = params.get("a").map(|s| s.to_string());
        parser.format = params.get("f").and_then(|s| s.parse().ok());
        parser.transmission_medium = params.get("t").map(|s| s.to_string());
        parser.image_id = params
            .get("i")
            .or_else(|| params.get("I"))
            .and_then(|s| s.parse().ok());
        parser.width = params.get("s").and_then(|s| s.parse().ok());
        parser.height = params.get("v").and_then(|s| s.parse().ok());
        parser.compression = params.get("o").and_then(|s| s.parse().ok());

        // Decode base64 payload
        if !payload_str.is_empty() {
            if let Ok(decoded) = BASE64.decode(payload_str) {
                parser.payload = decoded;
            }
        }

        Some(parser)
    }

    /// Check if this is a PNG image (format=100)
    pub fn is_png(&self) -> bool {
        self.format == Some(100)
    }

    /// Check if this is RGB data (format=24)
    pub fn is_rgb(&self) -> bool {
        self.format == Some(24)
    }

    /// Check if this is RGBA data (format=32)
    pub fn is_rgba(&self) -> bool {
        self.format == Some(32)
    }
}

impl Default for KittyGraphicsParser {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_kitty_png() {
        let data = "\x1b_Gf=100,a=T,t=d;aGVsbG8=\x1b\\";
        let parser = KittyGraphicsParser::parse(data).unwrap();

        assert_eq!(parser.format, Some(100));
        assert_eq!(parser.action, Some("T".to_string()));
        assert_eq!(parser.transmission_medium, Some("d".to_string()));
        assert!(parser.is_png());
        assert_eq!(parser.payload, b"hello");
    }

    #[test]
    fn test_parse_kitty_with_dimensions() {
        let data = "\x1b_Gf=24,s=100,v=200,i=5;\x1b\\";
        let parser = KittyGraphicsParser::parse(data).unwrap();

        assert_eq!(parser.format, Some(24));
        assert_eq!(parser.width, Some(100));
        assert_eq!(parser.height, Some(200));
        assert_eq!(parser.image_id, Some(5));
        assert!(parser.is_rgb());
    }

    #[test]
    fn test_parse_invalid() {
        let data = "not a kitty sequence";
        assert!(KittyGraphicsParser::parse(data).is_none());
    }
}
