use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

/// Device information for tracking and encryption
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Device {
    pub device_id: String,
    pub device_name: String,
    pub device_type: DeviceType,
    pub os_info: OsInfo,
    pub app_version: String,
    pub created_at: DateTime<Utc>,
    pub last_seen: DateTime<Utc>,
    pub is_current: bool, // True for current device
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum DeviceType {
    Desktop,
    Laptop,
    Mobile,
    Server,
    Unknown,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct OsInfo {
    pub os_type: String,    // "linux", "windows", "macos", etc.
    pub os_version: String, // OS version
    pub arch: String,       // "x86_64", "arm64", etc.
    pub hostname: String,   // System hostname
}

impl Device {
    /// Create new device with current system info
    pub fn new_current(device_name: String) -> Self {
        let now = Utc::now();

        Self {
            device_id: Uuid::new_v4().to_string(),
            device_name,
            device_type: Self::detect_device_type(),
            os_info: Self::get_current_os_info(),
            app_version: env!("CARGO_PKG_VERSION").to_string(),
            created_at: now,
            last_seen: now,
            is_current: true,
        }
    }

    /// Update last seen timestamp
    pub fn update_last_seen(&mut self) {
        self.last_seen = Utc::now();
    }

    /// Detect device type from system info
    fn detect_device_type() -> DeviceType {
        match std::env::consts::OS {
            "linux" => {
                if std::env::var("DISPLAY").is_err() && std::env::var("WAYLAND_DISPLAY").is_err() {
                    DeviceType::Server
                } else {
                    DeviceType::Desktop
                }
            }
            "windows" | "macos" => DeviceType::Desktop,
            _ => DeviceType::Unknown,
        }
    }

    /// Get current OS information
    fn get_current_os_info() -> OsInfo {
        OsInfo {
            os_type: std::env::consts::OS.to_string(),
            os_version: Self::get_os_version(),
            arch: std::env::consts::ARCH.to_string(),
            hostname: gethostname::gethostname().to_string_lossy().to_string(),
        }
    }

    /// Get OS version (simplified)
    fn get_os_version() -> String {
        match std::env::consts::OS {
            "linux" => std::fs::read_to_string("/etc/os-release")
                .ok()
                .and_then(|content| {
                    content
                        .lines()
                        .find(|line| line.starts_with("VERSION="))
                        .map(|line| {
                            line.trim_start_matches("VERSION=")
                                .trim_matches('"')
                                .to_string()
                        })
                })
                .unwrap_or_else(|| "Unknown".to_string()),
            _ => "Unknown".to_string(),
        }
    }
}

impl Default for DeviceType {
    fn default() -> Self {
        Self::Unknown
    }
}

impl std::fmt::Display for DeviceType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            DeviceType::Desktop => write!(f, "Desktop"),
            DeviceType::Laptop => write!(f, "Laptop"),
            DeviceType::Mobile => write!(f, "Mobile"),
            DeviceType::Server => write!(f, "Server"),
            DeviceType::Unknown => write!(f, "Unknown"),
        }
    }
}
