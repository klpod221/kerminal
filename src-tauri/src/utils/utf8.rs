// Copyright (C) 2026 Bùi Thanh Xuân (klpod221)
// SPDX-License-Identifier: GPL-3.0-or-later

/// Incremental UTF-8 decoder that buffers incomplete characters
pub struct IncrementalUtf8Decoder {
    buffer: Vec<u8>,
}

impl IncrementalUtf8Decoder {
    pub fn new() -> Self {
        Self { buffer: Vec::new() }
    }

    /// Decode the input bytes, returning a valid UTF-8 string and buffering any incomplete sequence.
    pub fn decode(&mut self, data: &[u8]) -> String {
        if self.buffer.is_empty() {
            match std::str::from_utf8(data) {
                Ok(s) => return s.to_string(),
                Err(e) => {
                    let valid_len = e.valid_up_to();
                    if let Some(_error_len) = e.error_len() {
                        // Error inside the string, not at the end. We'll just use lossy conversion
                        // for the whole thing rather than dealing with complex inner errors,
                        // but actually we should just try String::from_utf8_lossy and buffer the end.
                        // For simplicity, let's just append to buffer and process.
                    } else {
                        // Incomplete char at the end
                        let valid = unsafe { std::str::from_utf8_unchecked(&data[..valid_len]) };
                        let result = valid.to_string();
                        self.buffer.extend_from_slice(&data[valid_len..]);
                        return result;
                    }
                }
            }
        }

        self.buffer.extend_from_slice(data);
        
        let (valid_len, to_keep) = match std::str::from_utf8(&self.buffer) {
            Ok(_) => (self.buffer.len(), 0),
            Err(e) => {
                let valid_up_to = e.valid_up_to();
                if e.error_len().is_none() {
                    // Incomplete sequence at the end
                    (valid_up_to, self.buffer.len() - valid_up_to)
                } else {
                    // Invalid sequence somewhere. Let's just use lossy and clear buffer
                    let lossy = String::from_utf8_lossy(&self.buffer).into_owned();
                    self.buffer.clear();
                    return lossy;
                }
            }
        };

        if valid_len > 0 {
            let valid = unsafe { std::str::from_utf8_unchecked(&self.buffer[..valid_len]) }.to_string();
            if to_keep > 0 {
                let tail = self.buffer[valid_len..].to_vec();
                self.buffer = tail;
            } else {
                self.buffer.clear();
            }
            valid
        } else {
            String::new()
        }
    }
}
