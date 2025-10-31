use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

/// Represents a file or directory entry on the remote system
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct FileEntry {
    /// File or directory name
    pub name: String,
    /// Full path
    pub path: String,
    /// File type
    pub file_type: FileType,
    /// File size in bytes (None for directories)
    pub size: Option<u64>,
    /// Permissions (Unix-style, e.g., 0o755)
    pub permissions: u32,
    /// Last modified time
    pub modified: DateTime<Utc>,
    /// Last accessed time
    pub accessed: Option<DateTime<Utc>>,
    /// Symlink target (if this is a symlink)
    pub symlink_target: Option<String>,
    /// Owner user ID
    pub uid: Option<u32>,
    /// Owner group ID
    pub gid: Option<u32>,
}

/// File type enumeration
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "lowercase")]
pub enum FileType {
    /// Regular file
    File,
    /// Directory
    Directory,
    /// Symbolic link
    Symlink,
    /// Unknown type
    Unknown,
}

impl FileEntry {
    /// Check if this entry is a directory
    pub fn is_directory(&self) -> bool {
        matches!(self.file_type, FileType::Directory)
    }
}

