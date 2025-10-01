use crate::models::system::{CPUInfo, ComponentInfo, DiskInfo, NetworkInterface, SystemInfo};
use sysinfo::{Components, Disks, Networks, System};

#[tauri::command]
pub fn get_system_info() -> SystemInfo {
    // Gather system information using sysinfo crate
    let mut sys = System::new_all();
    sys.refresh_all();

    let cpus = sys
        .cpus()
        .iter()
        .map(|cpu| CPUInfo {
            model: cpu.brand().to_string(),
            speed: cpu.frequency(),
            usage: cpu.cpu_usage(),
        })
        .collect();

    // 2. Gather network information
    let networks = Networks::new_with_refreshed_list();
    let network_interfaces = networks
        .iter()
        .map(|(interface_name, data)| NetworkInterface {
            name: interface_name.clone(),
            address: data
                .ip_networks()
                .iter()
                .map(|ip| ip.to_string())
                .collect::<Vec<_>>()
                .join(", "),
            mac: data.mac_address().to_string(),
            status: if data.mac_address().to_string().is_empty() {
                "down".to_string()
            } else {
                "up".to_string()
            },
        })
        .collect();

    // 3. Get disk information
    let disks = Disks::new_with_refreshed_list();
    let disks_info: Vec<DiskInfo> = disks
        .iter()
        .map(|disk| DiskInfo {
            name: disk.name().to_string_lossy().to_string(),
            total_space: disk.total_space(),
            available_space: disk.available_space(),
            file_system: disk.file_system().to_string_lossy().to_string(),
            mount_point: disk.mount_point().to_string_lossy().to_string(),
        })
        .collect();

    // 4. Gather temperature information
    let components = Components::new_with_refreshed_list();
    let components_info: Vec<ComponentInfo> = components
        .iter()
        .map(|component| ComponentInfo {
            label: component.label().to_string(),
            temperature: component.temperature().unwrap_or(0.0),
            max: component.max().unwrap_or(0.0),
        })
        .collect();

    // 5. Aggregate and return
    let load_avg = System::load_average();

    SystemInfo {
        platform: System::name().unwrap_or_else(|| "N/A".to_string()),
        release: System::os_version().unwrap_or_else(|| "N/A".to_string()),
        cpu_arch: System::cpu_arch().to_string(),
        hostname: System::host_name().unwrap_or_else(|| "N/A".to_string()),
        uptime: System::uptime(),
        total_memory: sys.total_memory(),
        free_memory: sys.free_memory(),
        load_average: (load_avg.one, load_avg.five, load_avg.fifteen),
        cpus,
        os_version: Some(System::long_os_version().unwrap_or_else(|| "N/A".to_string())),
        cpu_info: Some(format!(
            "{} Cores / {} Threads",
            System::physical_core_count().unwrap_or(0),
            sys.cpus().len()
        )),
        memory_info: Some(format!(
            "Used: {} MB / Total: {} MB",
            (sys.used_memory() / 1024 / 1024),
            (sys.total_memory() / 1024 / 1024)
        )),
        gpu_info: None,
        resolution: None,
        network_interfaces: Some(network_interfaces),
        disks_info: Some(disks_info),
        components_info: Some(components_info),
    }
}
