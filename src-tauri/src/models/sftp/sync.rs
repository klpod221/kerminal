use serde::{Deserialize, Serialize};

use crate::models::sftp::file_entry::FileEntry;

/// Synchronization operation parameters
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct SyncOperation {
    /// Synchronization direction
    pub direction: SyncDirection,
    /// Local directory path
    pub local_path: String,
    /// Remote directory path
    pub remote_path: String,
    /// Whether to delete files that exist in target but not in source
    pub delete_extra_files: bool,
    /// Whether to preserve symlinks
    pub preserve_symlinks: bool,
    /// Whether to preserve permissions
    pub preserve_permissions: bool,
    /// Maximum file size to sync (None = no limit)
    pub max_file_size: Option<u64>,
    /// File patterns to exclude (glob patterns)
    pub exclude_patterns: Vec<String>,
}

/// Synchronization direction
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "lowercase")]
pub enum SyncDirection {
    /// Sync from local to remote (upload)
    LocalToRemote,
    /// Sync from remote to local (download)
    RemoteToLocal,
    /// Bidirectional sync (merge both ways)
    Bidirectional,
}

/// Represents a difference between local and remote files
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct DiffEntry {
    /// File path relative to sync root
    pub path: String,
    /// Difference type
    pub diff_type: DiffType,
    /// Local file entry (if exists)
    pub local_entry: Option<FileEntry>,
    /// Remote file entry (if exists)
    pub remote_entry: Option<FileEntry>,
}

/// Type of difference between local and remote
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub enum DiffType {
    /// File exists only on local
    OnlyLocal,
    /// File exists only on remote
    OnlyRemote,
    /// File exists on both but sizes differ
    SizeDiffers,
    /// File exists on both but modification times differ
    TimeDiffers,
    /// File exists on both and appears identical
    Identical,
    /// File exists on both but permissions differ
    PermissionsDiffer,
}

