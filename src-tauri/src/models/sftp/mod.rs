pub mod error;
pub mod file_entry;
pub mod requests;
pub mod search;
pub mod sync;
pub mod transfer;

// Re-export FileType which is commonly used
pub use file_entry::FileType;

use serde::{Deserialize, Serialize};

/// Response returned by sftp_connect.
/// Bundles session ID together with the server-resolved home directory so the
/// frontend can navigate there immediately without a second round-trip.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ConnectResponse {
    pub session_id: String,
    /// Absolute path of the remote user's home directory, resolved by the
    /// server via SSH_FXP_REALPATH. Falls back to "/" if the server does not
    /// support the request.
    pub home_dir: String,
}
