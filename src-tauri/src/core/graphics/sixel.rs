/*
 * Kerminal - Modern Terminal Emulator & SSH Manager
 * Copyright (C) 2026 Bùi Thanh Xuân (klpod221)
 *
 * This program is free software: you can redistribute it and/or modify
 * it under the terms of the GNU General Public License as published by
 * the Free Software Foundation, either version 3 of the License, or
 * (at your option) any later version.
 *
 * This program is distributed in the hope that it will be useful,
 * but WITHOUT ANY WARRANTY; without even the implied warranty of
 * MERCHANTABILITY or FITNESS FOR A PARTICULAR PURPOSE.  See the
 * GNU General Public License for more details.
 *
 * You should have received a copy of the GNU General Public License
 * along with this program.  If not, see <https://www.gnu.org/licenses/>.
 */

//! Sixel graphics protocol parser
//!
//! Sixel is handled primarily by xterm-addon-image on the frontend,
//! so this module mainly provides passthrough and validation utilities.

#[allow(dead_code)]
#[derive(Debug, Clone)]
pub struct SixelParser;

impl SixelParser {
    /// Check if data contains Sixel escape sequence
    /// Format: ESC P q ... ESC \
    pub fn is_sixel(data: &str) -> bool {
        data.contains("\x1bP") && data.contains("q")
    }

    /// Validate Sixel sequence (basic check)
    pub fn validate(data: &str) -> bool {
        if !Self::is_sixel(data) {
            return false;
        }

        // Check for proper escape sequence boundaries
        if let Some(start) = data.find("\x1bP") {
            if let Some(end) = data[start..].find("\x1b\\") {
                // Ensure 'q' is present in the sequence
                let sequence = &data[start..start + end];
                return sequence.contains('q');
            }
        }

        false
    }

    /// Extract Sixel data from escape sequence
    /// Returns the raw Sixel string including escape codes
    pub fn extract(data: &str) -> Option<String> {
        if !Self::is_sixel(data) {
            return None;
        }

        let start = data.find("\x1bP")?;
        let end_offset = data[start..].find("\x1b\\")?;
        let end = start + end_offset + 2; // +2 for ESC \

        Some(data[start..end].to_string())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_is_sixel() {
        let data = "\x1bPq\"1;1;100;100#0;2;0;0;0#1;2;100;100;0\x1b\\";
        assert!(SixelParser::is_sixel(data));
    }

    #[test]
    fn test_is_not_sixel() {
        let data = "regular text";
        assert!(!SixelParser::is_sixel(data));
    }

    #[test]
    fn test_validate() {
        let data = "\x1bPq\"1;1;100;100\x1b\\";
        assert!(SixelParser::validate(data));
    }

    #[test]
    fn test_extract() {
        let data = "before\x1bPq\"1;1\x1b\\after";
        let extracted = SixelParser::extract(data).unwrap();
        assert_eq!(extracted, "\x1bPq\"1;1\x1b\\");
    }
}
