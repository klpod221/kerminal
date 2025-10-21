#![allow(dead_code)]

use sqlx::Row;

use crate::{
    database::{
        encryption::device_keys::MasterPasswordEntry,
        error::{DatabaseError, DatabaseResult},
    },
    models::auth::Device,
};

use super::SQLiteProvider;

pub async fn save_master_password_entry(
    provider: &SQLiteProvider,
    entry: &MasterPasswordEntry,
) -> DatabaseResult<()> {
    let pool_arc = provider.get_pool()?;
    let pool = pool_arc.read().await;

    sqlx::query(
        r#"
        INSERT OR REPLACE INTO master_passwords (
            device_id, password_salt, verification_hash, auto_unlock,
            auto_lock_timeout, created_at, last_verified_at
        ) VALUES (?, ?, ?, ?, ?, ?, ?)
    "#,
    )
    .bind(&entry.device_id)
    .bind(entry.password_salt.to_vec())
    .bind(&entry.verification_hash)
    .bind(entry.auto_unlock)
    .bind(entry.auto_lock_timeout.map(|t| t as i64))
    .bind(entry.created_at)
    .bind(entry.last_verified_at)
    .execute(&*pool)
    .await
    .map_err(|e| DatabaseError::QueryFailed(e.to_string()))?;

    Ok(())
}

pub async fn get_master_password_entry(
    provider: &SQLiteProvider,
    device_id: &str,
) -> DatabaseResult<Option<MasterPasswordEntry>> {
    let pool_arc = provider.get_pool()?;
    let pool = pool_arc.read().await;

    let row = sqlx::query("SELECT * FROM master_passwords WHERE device_id = ?")
        .bind(device_id)
        .fetch_optional(&*pool)
        .await
        .map_err(|e| DatabaseError::QueryFailed(e.to_string()))?;

    if let Some(row) = row {
        let salt_bytes: Vec<u8> = row.get("password_salt");
        let mut salt_array = [0u8; 32];
        salt_array.copy_from_slice(&salt_bytes[..32]);

        let entry = MasterPasswordEntry {
            device_id: row.get("device_id"),
            password_salt: salt_array,
            verification_hash: row.get("verification_hash"),
            auto_unlock: row.get("auto_unlock"),
            auto_lock_timeout: row
                .get::<Option<i64>, _>("auto_lock_timeout")
                .map(|t| t as u32),
            created_at: row.get("created_at"),
            last_verified_at: row.get("last_verified_at"),
        };
        Ok(Some(entry))
    } else {
        Ok(None)
    }
}

pub async fn delete_master_password_entry(
    provider: &SQLiteProvider,
    device_id: &str,
) -> DatabaseResult<()> {
    let pool_arc = provider.get_pool()?;
    let pool = pool_arc.read().await;

    sqlx::query("DELETE FROM master_passwords WHERE device_id = ?")
        .bind(device_id)
        .execute(&*pool)
        .await
        .map_err(|e| DatabaseError::QueryFailed(e.to_string()))?;

    Ok(())
}

pub async fn get_current_device(provider: &SQLiteProvider) -> DatabaseResult<Option<Device>> {
    let pool_arc = provider.get_pool()?;
    let pool = pool_arc.read().await;

    let row = sqlx::query("SELECT * FROM devices WHERE is_current = true LIMIT 1")
        .fetch_optional(&*pool)
        .await
        .map_err(|e| DatabaseError::QueryFailed(e.to_string()))?;

    if let Some(row) = row {
        let device = Device {
            device_id: row.get("device_id"),
            device_name: row.get("device_name"),
            device_type: serde_json::from_str(&row.get::<String, _>("device_type"))
                .unwrap_or(crate::models::auth::DeviceType::Unknown),
            os_info: crate::models::auth::OsInfo {
                os_type: row.get("os_name"),
                os_version: row.get("os_version"),
                arch: "".to_string(),
                hostname: "".to_string(),
            },
            app_version: env!("CARGO_PKG_VERSION").to_string(),
            created_at: chrono::DateTime::parse_from_rfc3339(&row.get::<String, _>("created_at"))
                .map_err(|e| DatabaseError::QueryFailed(e.to_string()))?
                .with_timezone(&chrono::Utc),
            last_seen: chrono::DateTime::parse_from_rfc3339(&row.get::<String, _>("last_seen_at"))
                .map_err(|e| DatabaseError::QueryFailed(e.to_string()))?
                .with_timezone(&chrono::Utc),
            is_current: row.get("is_current"),
        };
        Ok(Some(device))
    } else {
        Ok(None)
    }
}

pub async fn save_device(provider: &SQLiteProvider, device: &Device) -> DatabaseResult<()> {
    let pool_arc = provider.get_pool()?;
    let pool = pool_arc.read().await;

    sqlx::query("UPDATE devices SET is_current = false")
        .execute(&*pool)
        .await
        .map_err(|e| DatabaseError::QueryFailed(e.to_string()))?;

    sqlx::query(
        r#"
        INSERT OR REPLACE INTO devices (
            device_id, device_name, device_type, os_name, os_version,
            created_at, last_seen_at, is_current
        ) VALUES (?, ?, ?, ?, ?, ?, ?, ?)
        "#,
    )
    .bind(&device.device_id)
    .bind(&device.device_name)
    .bind(serde_json::to_string(&device.device_type).unwrap_or_else(|_| "\"Unknown\"".to_string()))
    .bind(&device.os_info.os_type)
    .bind(&device.os_info.os_version)
    .bind(device.created_at.to_rfc3339())
    .bind(device.last_seen.to_rfc3339())
    .bind(device.is_current)
    .execute(&*pool)
    .await
    .map_err(|e| DatabaseError::QueryFailed(e.to_string()))?;

    Ok(())
}

pub async fn get_all_devices(provider: &SQLiteProvider) -> DatabaseResult<Vec<Device>> {
    let pool_arc = provider.get_pool()?;
    let pool = pool_arc.read().await;

    let rows = sqlx::query("SELECT * FROM devices ORDER BY last_seen_at DESC")
        .fetch_all(&*pool)
        .await
        .map_err(|e| DatabaseError::QueryFailed(e.to_string()))?;

    let mut devices = Vec::new();
    for row in rows {
        let device = Device {
            device_id: row.get("device_id"),
            device_name: row.get("device_name"),
            device_type: serde_json::from_str(&row.get::<String, _>("device_type"))
                .unwrap_or(crate::models::auth::DeviceType::Unknown),
            os_info: crate::models::auth::OsInfo {
                os_type: row.get("os_name"),
                os_version: row.get("os_version"),
                arch: "".to_string(),
                hostname: "".to_string(),
            },
            app_version: env!("CARGO_PKG_VERSION").to_string(),
            created_at: chrono::DateTime::parse_from_rfc3339(&row.get::<String, _>("created_at"))
                .map_err(|e| DatabaseError::QueryFailed(e.to_string()))?
                .with_timezone(&chrono::Utc),
            last_seen: chrono::DateTime::parse_from_rfc3339(&row.get::<String, _>("last_seen_at"))
                .map_err(|e| DatabaseError::QueryFailed(e.to_string()))?
                .with_timezone(&chrono::Utc),
            is_current: row.get("is_current"),
        };
        devices.push(device);
    }

    Ok(devices)
}

pub async fn update_device_last_seen(
    provider: &SQLiteProvider,
    device_id: &str,
) -> DatabaseResult<()> {
    let pool_arc = provider.get_pool()?;
    let pool = pool_arc.read().await;

    let now = chrono::Utc::now().to_rfc3339();

    sqlx::query("UPDATE devices SET last_seen_at = ? WHERE device_id = ?")
        .bind(&now)
        .bind(device_id)
        .execute(&*pool)
        .await
        .map_err(|e| DatabaseError::QueryFailed(e.to_string()))?;

    Ok(())
}

pub async fn delete_device(provider: &SQLiteProvider, device_id: &str) -> DatabaseResult<()> {
    let pool_arc = provider.get_pool()?;
    let pool = pool_arc.read().await;

    sqlx::query("DELETE FROM devices WHERE device_id = ?")
        .bind(device_id)
        .execute(&*pool)
        .await
        .map_err(|e| DatabaseError::QueryFailed(e.to_string()))?;

    Ok(())
}

pub async fn get_device_by_id(
    provider: &SQLiteProvider,
    device_id: &str,
) -> DatabaseResult<Option<Device>> {
    let pool_arc = provider.get_pool()?;
    let pool = pool_arc.read().await;

    let row = sqlx::query("SELECT * FROM devices WHERE device_id = ? LIMIT 1")
        .bind(device_id)
        .fetch_optional(&*pool)
        .await
        .map_err(|e| DatabaseError::QueryFailed(e.to_string()))?;

    if let Some(row) = row {
        let device = Device {
            device_id: row.get("device_id"),
            device_name: row.get("device_name"),
            device_type: serde_json::from_str(&row.get::<String, _>("device_type"))
                .unwrap_or(crate::models::auth::DeviceType::Unknown),
            os_info: crate::models::auth::OsInfo {
                os_type: row.get("os_name"),
                os_version: row.get("os_version"),
                arch: "".to_string(),
                hostname: "".to_string(),
            },
            app_version: env!("CARGO_PKG_VERSION").to_string(),
            created_at: chrono::DateTime::parse_from_rfc3339(&row.get::<String, _>("created_at"))
                .map_err(|e| DatabaseError::QueryFailed(e.to_string()))?
                .with_timezone(&chrono::Utc),
            last_seen: chrono::DateTime::parse_from_rfc3339(&row.get::<String, _>("last_seen_at"))
                .map_err(|e| DatabaseError::QueryFailed(e.to_string()))?
                .with_timezone(&chrono::Utc),
            is_current: row.get("is_current"),
        };
        Ok(Some(device))
    } else {
        Ok(None)
    }
}

impl SQLiteProvider {
    pub async fn save_master_password_entry(
        &self,
        entry: &MasterPasswordEntry,
    ) -> DatabaseResult<()> {
        save_master_password_entry(self, entry).await
    }

    pub async fn get_master_password_entry(
        &self,
        device_id: &str,
    ) -> DatabaseResult<Option<MasterPasswordEntry>> {
        get_master_password_entry(self, device_id).await
    }

    pub async fn delete_master_password_entry(&self, device_id: &str) -> DatabaseResult<()> {
        delete_master_password_entry(self, device_id).await
    }

    pub async fn get_current_device(&self) -> DatabaseResult<Option<Device>> {
        get_current_device(self).await
    }

    pub async fn get_device_by_id(&self, device_id: &str) -> DatabaseResult<Option<Device>> {
        get_device_by_id(self, device_id).await
    }

    pub async fn save_device(&self, device: &Device) -> DatabaseResult<()> {
        save_device(self, device).await
    }

    pub async fn get_all_devices(&self) -> DatabaseResult<Vec<Device>> {
        get_all_devices(self).await
    }

    pub async fn update_device_last_seen(&self, device_id: &str) -> DatabaseResult<()> {
        update_device_last_seen(self, device_id).await
    }

    pub async fn delete_device(&self, device_id: &str) -> DatabaseResult<()> {
        delete_device(self, device_id).await
    }
}
