use crate::database::encryption::aes::AESEncryption;
use crate::database::traits::Database;
use crate::models::saved_command::{SavedCommand, SavedCommandGroup};
use crate::models::ssh::{SSHGroup, SSHKey, SSHProfile, SSHTunnel};
use crate::state::AppState;
use serde::{Deserialize, Serialize};
use tauri::State;

/// Derive encryption key from password and salt using PBKDF2
fn derive_key_from_password(password: &str, salt: &[u8; 32]) -> Result<[u8; 32], String> {
    let mut key = [0u8; 32];

    let _ = pbkdf2::pbkdf2::<hmac::Hmac<sha2::Sha256>>(
        password.as_bytes(),
        salt,
        100_000, // 100k iterations
        &mut key,
    );

    Ok(key)
}

/// Backup data structure
#[derive(Debug, Serialize, Deserialize)]
pub struct BackupData {
    pub version: String,
    pub created_at: String,
    pub profiles: Vec<SSHProfile>,
    pub keys: Vec<SSHKey>,
    pub groups: Vec<SSHGroup>,
    pub tunnels: Vec<SSHTunnel>,
    pub saved_commands: Vec<SavedCommand>,
    pub saved_command_groups: Vec<SavedCommandGroup>,
}

/// Export backup command
#[tauri::command]
pub async fn export_backup(
    state: State<'_, AppState>,
    password: Option<String>,
) -> Result<String, String> {
    let service = state.database_service.lock().await;

    // Fetch all data
    let profiles = service
        .get_ssh_profiles(None)
        .await
        .map_err(|e| e.to_string())?;

    let keys = service.get_ssh_keys().await.map_err(|e| e.to_string())?;

    let groups = service.get_ssh_groups().await.map_err(|e| e.to_string())?;

    let tunnels = service.get_ssh_tunnels().await.map_err(|e| e.to_string())?;

    // Fetch saved commands and groups
    let saved_commands = service
        .get_saved_commands()
        .await
        .map_err(|e| e.to_string())?;

    let saved_command_groups = service
        .get_saved_command_groups()
        .await
        .map_err(|e| e.to_string())?;

    let data = BackupData {
        version: "1.0".to_string(),
        created_at: chrono::Utc::now().to_rfc3339(),
        profiles,
        keys,
        groups,
        tunnels,
        saved_commands,
        saved_command_groups,
    };

    let json = serde_json::to_string_pretty(&data).map_err(|e| e.to_string())?;

    // If password provided, encrypt the backup
    if let Some(pwd) = password {
        let salt = AESEncryption::generate_salt();
        let key = derive_key_from_password(&pwd, &salt)?;
        let encrypted_data = AESEncryption::encrypt(&key, json.as_bytes())
            .map_err(|e| format!("Encryption failed: {}", e))?;

        // Combine salt + encrypted data and encode as base64
        let mut result = Vec::with_capacity(32 + encrypted_data.len());
        result.extend_from_slice(&salt);
        result.extend_from_slice(&encrypted_data);

        Ok(base64::Engine::encode(
            &base64::engine::general_purpose::STANDARD,
            result,
        ))
    } else {
        Ok(json)
    }
}

/// Import backup command
#[tauri::command]
pub async fn import_backup(
    state: State<'_, AppState>,
    backup_content: String,
    password: Option<String>,
) -> Result<String, String> {
    let service = state.database_service.lock().await;
    let local_db_lock = service.get_local_database();
    let local_db = local_db_lock.write().await;

    // Decrypt if password is provided
    let json_content = if let Some(pwd) = password {
        let encrypted_bytes = base64::Engine::decode(
            &base64::engine::general_purpose::STANDARD,
            backup_content.trim(),
        )
        .map_err(|e| format!("Invalid base64 encoding: {}", e))?;

        if encrypted_bytes.len() < 32 {
            return Err("Invalid encrypted backup: too short".to_string());
        }

        let (salt, encrypted_data) = encrypted_bytes.split_at(32);
        let salt_array: [u8; 32] = salt
            .try_into()
            .map_err(|_| "Invalid salt size".to_string())?;
        let key = derive_key_from_password(&pwd, &salt_array)?;

        let decrypted_bytes = AESEncryption::decrypt(&key, encrypted_data)
            .map_err(|e| format!("Decryption failed: {}", e))?;

        String::from_utf8(decrypted_bytes)
            .map_err(|e| format!("Invalid UTF-8 in decrypted data: {}", e))?
    } else {
        backup_content
    };

    let data: BackupData =
        serde_json::from_str(&json_content).map_err(|e| format!("Invalid backup file: {}", e))?;

    // Import Groups
    for group in data.groups {
        local_db
            .save_ssh_group(&group)
            .await
            .map_err(|e| e.to_string())?;
    }

    // Import Keys
    for key in data.keys {
        local_db
            .save_ssh_key(&key)
            .await
            .map_err(|e| e.to_string())?;
    }

    // Import Profiles
    for profile in data.profiles {
        local_db
            .save_ssh_profile(&profile)
            .await
            .map_err(|e| e.to_string())?;
    }

    // Import Tunnels
    for tunnel in data.tunnels {
        local_db
            .save_ssh_tunnel(&tunnel)
            .await
            .map_err(|e| e.to_string())?;
    }

    // Import Saved Commands
    for command in data.saved_commands {
        local_db
            .save_saved_command(&command)
            .await
            .map_err(|e| e.to_string())?;
    }

    // Import Saved Command Groups
    for group in data.saved_command_groups {
        local_db
            .save_saved_command_group(&group)
            .await
            .map_err(|e| e.to_string())?;
    }

    Ok("Backup imported successfully".to_string())
}
