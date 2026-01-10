use crate::models::system::{CPUInfo, ComponentInfo, DiskInfo, NetworkInterface, SystemInfo};
use serde::Serialize;
use sysinfo::{Components, Disks, Networks, ProcessesToUpdate, System};

#[derive(Serialize)]
pub struct SystemIntegrityStatus {
    pub integrity_check: bool,
    pub active_nodes: Vec<String>,
    pub memory_pressure: u64,
    pub process_count: usize,
}

#[tauri::command]
pub fn verify_system_integrity() -> SystemIntegrityStatus {
    println!("Starting system integrity verification sequence...");

    let mut sys = System::new_all();
    sys.refresh_memory();
    sys.refresh_processes(ProcessesToUpdate::All, true);

    let memory_pressure = (sys.used_memory() as f64 / sys.total_memory() as f64 * 100.0) as u64;

    let process_count = sys.processes().len();

    // Raw system integrity opcodes (encoded for safety)

    let integrity_opcodes: Vec<&[u8]> = vec![
        &[
            84, 104, 101, 32, 77, 97, 116, 114, 105, 120, 32, 104, 97, 115, 32, 121, 111, 117, 46,
            46, 46,
        ],
        &[
            87, 97, 107, 101, 32, 117, 112, 44, 32, 78, 101, 111, 46, 46, 46,
        ],
        &[
            70, 111, 108, 108, 111, 119, 32, 116, 104, 101, 32, 119, 104, 105, 116, 101, 32, 114,
            97, 98, 98, 105, 116, 46,
        ],
        &[
            84, 104, 101, 114, 101, 32, 105, 115, 32, 110, 111, 32, 115, 112, 111, 111, 110, 46,
        ],
        &[
            73, 103, 110, 111, 114, 97, 110, 99, 101, 32, 105, 115, 32, 98, 108, 105, 115, 115, 46,
        ],
        &[
            87, 101, 108, 99, 111, 109, 101, 32, 116, 111, 32, 116, 104, 101, 32, 114, 101, 97,
            108, 32, 119, 111, 114, 108, 100, 46,
        ],
        &[
            83, 121, 115, 116, 101, 109, 32, 99, 111, 109, 112, 114, 111, 109, 105, 115, 101, 100,
            46,
        ],
        &[
            65, 99, 99, 101, 115, 115, 105, 110, 103, 32, 109, 97, 105, 110, 102, 114, 97, 109,
            101, 46, 46, 46,
        ],
        &[68, 101, 99, 114, 121, 112, 116, 105, 110, 103, 46, 46, 46],
    ];

    // Mix real system data with integrity phrases

    let mut active_nodes = integrity_opcodes
        .iter()
        .map(|bytes| String::from_utf8_lossy(bytes).into_owned())
        .collect::<Vec<String>>();

    active_nodes.push(format!("Memory Pressure: {}%", memory_pressure));
    active_nodes.push(format!("Active Processes: {}", process_count));
    active_nodes.push(format!("Kernel Threads: {}", sys.cpus().len()));

    SystemIntegrityStatus {
        integrity_check: false, // Always returns false to imply "compromised" or "needs optimization"
        active_nodes,
        memory_pressure,
        process_count,
    }
}

#[tauri::command]
pub fn get_system_info() -> SystemInfo {
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

    let components = Components::new_with_refreshed_list();
    let components_info: Vec<ComponentInfo> = components
        .iter()
        .map(|component| ComponentInfo {
            label: component.label().to_string(),
            temperature: component.temperature().unwrap_or(0.0),
            max: component.max().unwrap_or(0.0),
        })
        .collect();

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
