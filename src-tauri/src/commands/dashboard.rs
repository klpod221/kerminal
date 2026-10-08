use crate::models::system::{CPUInfo, ComponentInfo, DiskInfo, NetworkInterface, SystemInfo};
use crate::state::AppState;
use log::info;
use serde::Serialize;
use sysinfo::{Components, Disks, Networks, ProcessesToUpdate, System};
use tauri::State;

#[derive(Serialize)]
pub struct SystemIntegrityStatus {
    pub integrity_check: bool,
    pub active_nodes: Vec<String>,
    pub memory_pressure: u64,
    pub process_count: usize,
}

#[tauri::command]
pub fn verify_system_integrity() -> SystemIntegrityStatus {
    info!("Starting system integrity verification sequence...");

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

// STUB for Docker Management
#[derive(Serialize)]
pub struct DockerContainer {
    pub id: String,
    pub name: String,
    pub image: String,
    pub state: String,
    pub status: String,
    pub ports: String,
}

async fn execute_ssh_command(
    profile_id: &str,
    command: &str,
    app_state: &State<'_, AppState>,
) -> Result<String, String> {
    let session_key = app_state
        .sftp_service
        .connect(profile_id.to_string())
        .await
        .map_err(|e| format!("Connection failed: {:?}", e))?;

    let session_data = app_state
        .sftp_service
        .get_session(&session_key)
        .await
        .map_err(|e| format!("Session error: {:?}", e))?;

    let client = session_data.lock().await.client.clone();

    let mut channel = client
        .channel_open_session()
        .await
        .map_err(|e| format!("Channel error: {:?}", e))?;

    channel
        .exec(true, command)
        .await
        .map_err(|e| format!("Exec error: {:?}", e))?;

    let mut stdout = channel.make_reader();
    let mut buffer = Vec::new();
    use tokio::io::AsyncReadExt;
    stdout
        .read_to_end(&mut buffer)
        .await
        .map_err(|e| format!("Read error: {:?}", e))?;

    Ok(String::from_utf8_lossy(&buffer).to_string())
}

#[tauri::command]
pub async fn execute_remote_docker_command(
    profile_id: String,
    action: String,
    container_id: Option<String>,
    app_state: State<'_, AppState>,
) -> Result<Vec<DockerContainer>, String> {
    log::info!("Executing docker action '{}' on profile {}", action, profile_id);
    
    if action == "ps" {
        let output = execute_ssh_command(&profile_id, "docker ps -a --format '{{json .}}'", &app_state).await?;
        let mut containers = Vec::new();
        for line in output.lines() {
            if let Ok(parsed) = serde_json::from_str::<serde_json::Value>(line) {
                containers.push(DockerContainer {
                    id: parsed["ID"].as_str().unwrap_or("").to_string(),
                    name: parsed["Names"].as_str().unwrap_or("").to_string(),
                    image: parsed["Image"].as_str().unwrap_or("").to_string(),
                    state: parsed["State"].as_str().unwrap_or("").to_string(),
                    status: parsed["Status"].as_str().unwrap_or("").to_string(),
                    ports: parsed["Ports"].as_str().unwrap_or("").to_string(),
                });
            }
        }
        return Ok(containers);
    } else if action == "start" || action == "stop" {
        if let Some(id) = container_id {
            let cmd = format!("docker {} {}", action, id);
            execute_ssh_command(&profile_id, &cmd, &app_state).await?;
        }
        return Ok(vec![]);
    }
    
    Err("Unknown action".into())
}

// STUB for Server Monitor
#[derive(Serialize)]
pub struct ServerMetrics {
    pub cpu: u8,
    pub ram: u8,
    pub ram_used: String,
    pub ram_total: String,
    pub has_gpu: bool,
    pub gpu: u8,
    pub vram_used: String,
    pub vram_total: String,
    pub disk: u8,
    pub gpu_name: String,
    pub cpu_name: String,
}

#[tauri::command]
pub async fn get_remote_server_metrics(
    profile_id: String,
    app_state: State<'_, AppState>,
) -> Result<ServerMetrics, String> {
    log::info!("Fetching remote metrics for profile {}", profile_id);
    
    // CPU: run top -bn1 | grep 'Cpu(s)' 
    let cpu_out = execute_ssh_command(&profile_id, "top -bn1 | grep 'Cpu(s)' | awk '{print $2 + $4}'", &app_state).await?;
    let cpu: u8 = cpu_out.trim().parse::<f32>().unwrap_or(0.0) as u8;

    // CPU Name:
    let cpu_name_out = execute_ssh_command(&profile_id, "cat /proc/cpuinfo | grep 'model name' | head -n 1 | awk -F: '{print $2}'", &app_state).await?;
    let mut cpu_name = cpu_name_out.trim().to_string();
    if cpu_name.is_empty() {
        cpu_name = "CPU".to_string();
    }

    // RAM: run free -m
    let ram_out = execute_ssh_command(&profile_id, "free -m | grep Mem | awk '{print $3\",\"$2}'", &app_state).await?;
    let mut ram_used_mb = 0.0;
    let mut ram_total_mb = 0.0;
    if let Some((u, t)) = ram_out.trim().split_once(',') {
        ram_used_mb = u.parse::<f32>().unwrap_or(0.0);
        ram_total_mb = t.parse::<f32>().unwrap_or(0.0);
    }
    let ram: u8 = if ram_total_mb > 0.0 { ((ram_used_mb / ram_total_mb) * 100.0) as u8 } else { 0 };
    let ram_used = format!("{:.1}", ram_used_mb / 1024.0);
    let ram_total = format!("{:.1}", ram_total_mb / 1024.0);

    // Disk: run df -h /
    let disk_out = execute_ssh_command(&profile_id, "df -h / | tail -n 1 | awk '{print $5}' | sed 's/%//'", &app_state).await?;
    let disk: u8 = disk_out.trim().parse().unwrap_or(0);

    // GPU: nvidia-smi (NVIDIA) or rocm-smi (AMD)
    let mut has_gpu = false;
    let mut gpu = 0;
    let mut vram_used = "0.0".to_string();
    let mut vram_total = "0.0".to_string();
    let mut gpu_name = "GPU Compute".to_string();
    
    // Check NVIDIA first
    if let Ok(gpu_out) = execute_ssh_command(&profile_id, "nvidia-smi --query-gpu=name,utilization.gpu,memory.used,memory.total --format=csv,noheader,nounits", &app_state).await {
        if !gpu_out.trim().is_empty() && !gpu_out.contains("command not found") && !gpu_out.contains("not found") {
            has_gpu = true;
            if let Some(first_line) = gpu_out.lines().next() {
                let parts: Vec<&str> = first_line.split(',').map(|s| s.trim()).collect();
                if parts.len() >= 4 {
                    gpu_name = parts[0].to_string();
                    gpu = parts[1].parse().unwrap_or(0);
                    let vu = parts[2].parse::<f32>().unwrap_or(0.0) / 1024.0;
                    let vt = parts[3].parse::<f32>().unwrap_or(0.0) / 1024.0;
                    vram_used = format!("{:.1}", vu);
                    vram_total = format!("{:.1}", vt);
                }
            }
        }
    }

    // If no NVIDIA, check AMD (rocm-smi)
    if !has_gpu {
        if let Ok(amd_out) = execute_ssh_command(&profile_id, "rocm-smi --showuse --showmeminfo vram --csv", &app_state).await {
            if !amd_out.trim().is_empty() && !amd_out.contains("command not found") && !amd_out.contains("not found") {
                has_gpu = true;
                // Parse CSV from rocm-smi
                // Example output: device, GPU use (%), vram total, vram used
                let mut found_data = false;
                for line in amd_out.lines().skip(1) { // Skip header
                    let parts: Vec<&str> = line.split(',').collect();
                    if parts.len() >= 4 {
                        gpu = parts[1].trim().parse().unwrap_or(0);
                        let vt_bytes: f32 = parts[2].trim().parse().unwrap_or(0.0);
                        let vu_bytes: f32 = parts[3].trim().parse().unwrap_or(0.0);
                        vram_total = format!("{:.1}", vt_bytes / 1024.0 / 1024.0 / 1024.0);
                        vram_used = format!("{:.1}", vu_bytes / 1024.0 / 1024.0 / 1024.0);
                        found_data = true;
                        break;
                    }
                }
                if !found_data {
                    has_gpu = false; // Failed to parse
                }
            }
        }
    }

    Ok(ServerMetrics {
        cpu,
        ram,
        ram_used,
        ram_total,
        has_gpu,
        gpu,
        vram_used,
        vram_total,
        disk,
        gpu_name,
        cpu_name,
    })
}
