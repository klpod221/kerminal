// Copyright (C) 2026 Bùi Thanh Xuân (klpod221)
// SPDX-License-Identifier: GPL-3.0-or-later

use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use tauri::State;
use crate::state::AppState;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ContainerInfo {
    pub id: String,
    pub name: String,
    pub image: String,
    pub state: String,
    pub status: String,
    pub ports: String,
    pub created: String,
    pub compose_project: Option<String>,
    pub compose_service: Option<String>,
    pub engine: String,
    pub cpu_perc: Option<String>,
    pub mem_usage: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ContainerStatInfo {
    pub id: String,
    pub cpu: String,
    pub memory: String,
}

/// Execute a command locally on the host machine
async fn run_local_command(cmd: &str) -> Result<String, String> {
    #[cfg(target_os = "windows")]
    let output = tokio::process::Command::new("cmd")
        .args(["/C", cmd])
        .output()
        .await
        .map_err(|e| format!("Failed to run local command: {}", e))?;

    #[cfg(not(target_os = "windows"))]
    let output = tokio::process::Command::new("sh")
        .args(["-c", cmd])
        .output()
        .await
        .map_err(|e| format!("Failed to run local command: {}", e))?;

    let stdout = String::from_utf8_lossy(&output.stdout);
    let stderr = String::from_utf8_lossy(&output.stderr);

    if output.status.success() {
        Ok(stdout.to_string())
    } else if !stderr.trim().is_empty() {
        Err(stderr.trim().to_string())
    } else if !stdout.trim().is_empty() {
        Err(stdout.trim().to_string())
    } else {
        Err(format!("Command exited with status code {:?}", output.status.code()))
    }
}

/// Execute a command remotely over an established SSH session
async fn run_remote_command(
    profile_id: &str,
    cmd: &str,
    app_state: &State<'_, AppState>,
    app_handle: &tauri::AppHandle,
) -> Result<String, String> {
    let known_hosts_path = crate::commands::ssh_host_key::get_known_hosts_path(app_handle);
    let session_key = app_state
        .sftp_service
        .connect(profile_id.to_string(), known_hosts_path)
        .await
        .map_err(|e| format!("SSH connection failed: {:?}", e))?;

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

    let full_cmd = format!("sh -c '{}' 2>&1", cmd.replace('\'', "'\\''"));

    channel
        .exec(true, full_cmd.as_str())
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

/// Run command either locally or remotely based on profile_id
async fn run_command(
    profile_id: Option<&str>,
    cmd: &str,
    app_state: &State<'_, AppState>,
    app_handle: &tauri::AppHandle,
) -> Result<String, String> {
    match profile_id {
        Some(pid) if !pid.is_empty() && pid != "local" => {
            run_remote_command(pid, cmd, app_state, app_handle).await
        }
        _ => run_local_command(cmd).await,
    }
}

/// Detect whether Docker or Podman is installed and usable
async fn resolve_engine(
    profile_id: Option<&str>,
    preferred: Option<&str>,
    app_state: &State<'_, AppState>,
    app_handle: &tauri::AppHandle,
) -> Result<String, String> {
    if let Some(pref) = preferred {
        let p = pref.to_lowercase();
        if p == "docker" || p == "podman" {
            return Ok(p);
        }
    }

    if let Ok(out) = run_command(profile_id, "docker --version", app_state, app_handle).await {
        if out.to_lowercase().contains("docker") || out.to_lowercase().contains("version") {
            return Ok("docker".to_string());
        }
    }

    if let Ok(out) = run_command(profile_id, "podman --version", app_state, app_handle).await {
        if out.to_lowercase().contains("podman") || out.to_lowercase().contains("version") {
            return Ok("podman".to_string());
        }
    }

    Err("Neither Docker nor Podman is available on the target environment.".to_string())
}

/// Extract compose project and service labels from labels string or JSON
fn extract_compose_metadata(labels_val: &serde_json::Value) -> (Option<String>, Option<String>) {
    let mut project = None;
    let mut service = None;

    if let Some(map) = labels_val.as_object() {
        for (k, v) in map {
            let v_str = v.as_str().unwrap_or("").to_string();
            if k == "com.docker.compose.project" || k == "io.podman.compose.project" {
                project = Some(v_str);
            } else if k == "com.docker.compose.service" || k == "io.podman.compose.service" {
                service = Some(v_str);
            }
        }
    } else if let Some(raw_str) = labels_val.as_str() {
        for part in raw_str.split(',') {
            if let Some((k, v)) = part.split_once('=') {
                let k_trim = k.trim();
                let v_trim = v.trim().to_string();
                if k_trim == "com.docker.compose.project" || k_trim == "io.podman.compose.project" {
                    project = Some(v_trim);
                } else if k_trim == "com.docker.compose.service" || k_trim == "io.podman.compose.service" {
                    service = Some(v_trim);
                }
            }
        }
    }

    (project, service)
}

#[tauri::command]
pub async fn get_container_list(
    profile_id: Option<String>,
    engine: Option<String>,
    app_state: State<'_, AppState>,
    app_handle: tauri::AppHandle,
) -> Result<Vec<ContainerInfo>, String> {
    let pid = profile_id.as_deref();
    let bin = resolve_engine(pid, engine.as_deref(), &app_state, &app_handle).await?;

    let cmd = format!("{} ps -a --format '{{{{json .}}}}'", bin);
    let output = run_command(pid, &cmd, &app_state, &app_handle).await?;

    let mut containers = Vec::new();
    for line in output.lines() {
        let trimmed = line.trim();
        if trimmed.is_empty() {
            continue;
        }

        if let Ok(parsed) = serde_json::from_str::<serde_json::Value>(trimmed) {
            let id = parsed.get("ID")
                .or_else(|| parsed.get("Id"))
                .and_then(|v| v.as_str())
                .unwrap_or("")
                .to_string();

            let raw_name = parsed.get("Names")
                .or_else(|| parsed.get("Name"))
                .and_then(|v| v.as_str())
                .unwrap_or("")
                .to_string();
            let name = raw_name.trim_start_matches('/').to_string();

            let image = parsed.get("Image")
                .and_then(|v| v.as_str())
                .unwrap_or("")
                .to_string();

            let state = parsed.get("State")
                .and_then(|v| v.as_str())
                .unwrap_or("")
                .to_string();

            let status = parsed.get("Status")
                .and_then(|v| v.as_str())
                .unwrap_or("")
                .to_string();

            let ports = parsed.get("Ports")
                .and_then(|v| v.as_str())
                .unwrap_or("")
                .to_string();

            let created = parsed.get("CreatedAt")
                .or_else(|| parsed.get("Created"))
                .and_then(|v| v.as_str())
                .unwrap_or("")
                .to_string();

            let labels_val = parsed.get("Labels").unwrap_or(&serde_json::Value::Null);
            let (compose_project, compose_service) = extract_compose_metadata(labels_val);

            containers.push(ContainerInfo {
                id,
                name,
                image,
                state,
                status,
                ports,
                created,
                compose_project,
                compose_service,
                engine: bin.clone(),
                cpu_perc: None,
                mem_usage: None,
            });
        }
    }

    Ok(containers)
}

#[tauri::command]
pub async fn execute_container_action(
    profile_id: Option<String>,
    action: String,
    container_id: String,
    engine: Option<String>,
    app_state: State<'_, AppState>,
    app_handle: tauri::AppHandle,
) -> Result<(), String> {
    let pid = profile_id.as_deref();
    let bin = resolve_engine(pid, engine.as_deref(), &app_state, &app_handle).await?;

    // Validate container_id to avoid malicious command injection
    if container_id.contains(';') || container_id.contains('&') || container_id.contains('|') {
        return Err("Invalid container ID format.".to_string());
    }

    let cmd = match action.as_str() {
        "start" => format!("{} start {}", bin, container_id),
        "stop" => format!("{} stop {}", bin, container_id),
        "restart" => format!("{} restart {}", bin, container_id),
        "pause" => format!("{} pause {}", bin, container_id),
        "unpause" => format!("{} unpause {}", bin, container_id),
        "rm" | "delete" => format!("{} rm -f {}", bin, container_id),
        _ => return Err(format!("Unsupported action '{}'", action)),
    };

    run_command(pid, &cmd, &app_state, &app_handle).await?;
    Ok(())
}

#[tauri::command]
pub async fn get_container_logs(
    profile_id: Option<String>,
    container_id: String,
    tail: Option<u32>,
    engine: Option<String>,
    app_state: State<'_, AppState>,
    app_handle: tauri::AppHandle,
) -> Result<String, String> {
    let pid = profile_id.as_deref();
    let bin = resolve_engine(pid, engine.as_deref(), &app_state, &app_handle).await?;

    let tail_lines = tail.unwrap_or(200);
    let cmd = format!("{} logs --tail {} {}", bin, tail_lines, container_id);
    let output = run_command(pid, &cmd, &app_state, &app_handle).await?;
    Ok(output)
}

#[tauri::command]
pub async fn get_container_stats(
    profile_id: Option<String>,
    engine: Option<String>,
    app_state: State<'_, AppState>,
    app_handle: tauri::AppHandle,
) -> Result<HashMap<String, ContainerStatInfo>, String> {
    let pid = profile_id.as_deref();
    let bin = resolve_engine(pid, engine.as_deref(), &app_state, &app_handle).await?;

    let cmd = format!("{} stats --no-stream --format '{{{{json .}}}}'", bin);
    let output = run_command(pid, &cmd, &app_state, &app_handle).await?;

    let mut map = HashMap::new();
    for line in output.lines() {
        let trimmed = line.trim();
        if trimmed.is_empty() {
            continue;
        }

        if let Ok(parsed) = serde_json::from_str::<serde_json::Value>(trimmed) {
            let id = parsed.get("ID")
                .or_else(|| parsed.get("Container"))
                .and_then(|v| v.as_str())
                .unwrap_or("")
                .to_string();

            let cpu = parsed.get("CPUPerc")
                .and_then(|v| v.as_str())
                .unwrap_or("0%")
                .to_string();

            let memory = parsed.get("MemUsage")
                .and_then(|v| v.as_str())
                .unwrap_or("0B")
                .to_string();

            if !id.is_empty() {
                map.insert(id.clone(), ContainerStatInfo { id, cpu, memory });
            }
        }
    }

    Ok(map)
}
