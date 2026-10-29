// Copyright (C) 2026 Bùi Thanh Xuân (klpod221)
// SPDX-License-Identifier: GPL-3.0-or-later

use regex::Regex;
use std::sync::LazyLock;

static KV_SECRET_RE: LazyLock<Regex> = LazyLock::new(|| {
    Regex::new(r"(?i)(password|passwd|pwd|secret|token|api_key|apikey|access_token|auth_token)(\s*[=:]\s*)([^\s;]+)").unwrap()
});
static FLAG_SECRET_RE: LazyLock<Regex> = LazyLock::new(|| {
    Regex::new(r"(?i)(-p|--password)(\s+)([^\s]+)").unwrap()
});
static BEARER_SECRET_RE: LazyLock<Regex> = LazyLock::new(|| {
    Regex::new(r"(?i)(bearer)(\s+)([^\s]+)").unwrap()
});

pub fn redact_secrets(input: &str) -> String {
    let res = KV_SECRET_RE.replace_all(input, "${1}${2}***REDACTED***");
    let res = FLAG_SECRET_RE.replace_all(&res, "${1}${2}***REDACTED***");
    BEARER_SECRET_RE.replace_all(&res, "${1}${2}***REDACTED***").into_owned()
}
