use serde::{Serialize, Deserialize};

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct SystemInfo {
  pub platform: String,
  pub release: String,
  pub cpu_arch: String,
  pub hostname: String,
  pub uptime: u64,
  pub total_memory: u64,
  pub free_memory: u64,
  pub load_average: (f64, f64, f64),
  pub cpus: Vec<CPUInfo>,
  pub os_version: Option<String>,
  pub cpu_info: Option<String>,
  pub memory_info: Option<String>,
  pub gpu_info: Option<String>,
  pub resolution: Option<(u32, u32)>,
  pub network_interfaces: Option<Vec<NetworkInterface>>,
  pub disks_info: Option<Vec<DiskInfo>>,
  pub components_info: Option<Vec<ComponentInfo>>,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct CPUInfo {
  pub model: String,
  pub speed: u64,
  pub usage: f32,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct DiskInfo {
  pub name: String,
  pub total_space: u64,
  pub available_space: u64,
  pub file_system: String,
  pub mount_point: String,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct ComponentInfo {
  pub label: String,
  pub temperature: f32,
  pub max: f32,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct NetworkInterface {
  pub name: String,
  pub address: String,
  pub mac: String,
  pub status: String,
}
