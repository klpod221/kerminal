use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

/// Device information cho tracking và encryption
#[derive(Debug, Clone, Serialize, Deserialize)]
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
pub struct OsInfo {
    pub os_type: String,    // "linux", "windows", "macos", etc.
    pub os_version: String, // OS version
    pub arch: String,       // "x86_64", "arm64", etc.
    pub hostname: String,   // System hostname
}

impl Device {
    /// Create new device với current system info
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

    /// Create device from sync data
    pub fn new_remote(
        device_id: String,
        device_name: String,
        device_type: DeviceType,
        os_info: OsInfo,
        app_version: String,
        created_at: DateTime<Utc>,
        last_seen: DateTime<Utc>,
    ) -> Self {
        Self {
            device_id,
            device_name,
            device_type,
            os_info,
            app_version,
            created_at,
            last_seen,
            is_current: false,
        }
    }

    /// Update last seen timestamp
    pub fn update_last_seen(&mut self) {
        self.last_seen = Utc::now();
    }

    /// Check if device was seen recently
    pub fn is_online(&self, threshold_minutes: i64) -> bool {
        let threshold = chrono::Duration::minutes(threshold_minutes);
        (Utc::now() - self.last_seen) < threshold
    }

    /// Detect device type từ system info
    fn detect_device_type() -> DeviceType {
        // Simple detection logic - có thể cải thiện
        match std::env::consts::OS {
            "linux" => {
                // Check if running on server (no GUI)
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
        // This is a simplified version - có thể sử dụng sysinfo crate cho chi tiết hơn
        match std::env::consts::OS {
            "linux" => {
                // Try to read from /etc/os-release
                std::fs::read_to_string("/etc/os-release")
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
                    .unwrap_or_else(|| "Unknown".to_string())
            }
            _ => "Unknown".to_string(),
        }
    }

    /// Get display name for UI
    pub fn display_name(&self) -> String {
        if self.is_current {
            format!("{} (Current)", self.device_name)
        } else {
            self.device_name.clone()
        }
    }

    /// Get short device info
    pub fn short_info(&self) -> String {
        format!(
            "{} - {} {}",
            self.device_name, self.os_info.os_type, self.os_info.arch
        )
    }
}

/// Device information for API responses
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DeviceInfo {
    pub device_id: String,
    pub device_name: String,
    pub device_type: DeviceType,
    pub os_info: OsInfo,
    pub app_version: String,
    pub created_at: DateTime<Utc>,
    pub last_seen: DateTime<Utc>,
    pub is_current: bool,
    pub is_online: bool,
}

impl From<Device> for DeviceInfo {
    fn from(device: Device) -> Self {
        let is_online = device.is_online(5); // Calculate before any moves
        Self {
            device_id: device.device_id,
            device_name: device.device_name,
            device_type: device.device_type,
            os_info: device.os_info,
            app_version: device.app_version,
            created_at: device.created_at,
            last_seen: device.last_seen,
            is_current: device.is_current,
            is_online,
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