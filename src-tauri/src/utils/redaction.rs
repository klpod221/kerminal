// Copyright (C) 2026 Bùi Thanh Xuân (klpod221)
// SPDX-License-Identifier: GPL-3.0-or-later

use regex::Regex;

pub fn redact_secrets(input: &str) -> String {
    let patterns = [
        r"(?i)(password|passwd|pwd|secret|token|api_key|apikey|access_token|auth_token)\s*(=|:)\s*([^\s;]+)",
        r"(?i)(-p|--password)\s+([^\s]+)",
        r"(?i)(bearer)\s+([^\s]+)",
    ];

    let mut result = input.to_string();
    for pattern in &patterns {
        if let Ok(re) = Regex::new(pattern) {
            result = re.replace_all(&result, "${1}${2} ***REDACTED***").to_string();
        }
    }
    result
}
