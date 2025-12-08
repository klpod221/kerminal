use serde::{Deserialize, Serialize};

/// Result of a file search
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct SearchResult {
    pub file_path: String,
    pub line_number: u64,
    pub content: String,
}
