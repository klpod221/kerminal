
use sqlx::Row;

use crate::{
    database::error::{DatabaseError, DatabaseResult},
    models::ssh::{SSHGroup, SSHKey, SSHProfile},
};

use super::PostgreSQLProvider;

pub async fn save_ssh_profile(provider: &PostgreSQLProvider, model: &SSHProfile) -> DatabaseResult<()> {
    let pool_arc = provider.get_pool()?;
    let pool = pool_arc.read().await;

    sqlx::query(
        r#"
        INSERT INTO ssh_profiles (
            id, name, host, port, username, group_id, auth_method, auth_data,
            description, color, timeout, keep_alive, compression, created_at, updated_at,
            device_id, version, sync_status
        ) VALUES (?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?)
        ON DUPLICATE KEY UPDATE
            name = VALUES(name),
            host = VALUES(host),
            port = VALUES(port),
            username = VALUES(username),
            group_id = VALUES(group_id),
            auth_method = VALUES(auth_method),
            auth_data = VALUES(auth_data),
            description = VALUES(description),
            color = VALUES(color),
            timeout = VALUES(timeout),
            keep_alive = VALUES(keep_alive),
            compression = VALUES(compression),
            updated_at = VALUES(updated_at),
            version = VALUES(version),
            sync_status = VALUES(sync_status)
    "#,
    )
    .bind(&model.base.id)
    .bind(&model.name)
    .bind(&model.host)
    .bind(model.port as i32)
    .bind(&model.username)
    .bind(&model.group_id)
    .bind(serde_json::to_string(&model.auth_method).unwrap_or_default())
    .bind(serde_json::to_string(&model.auth_data).unwrap_or_default())
    .bind(&model.description)
    .bind(&model.color)
    .bind(model.timeout.map(|t| t as i32))
    .bind(model.keep_alive)
    .bind(model.compression)
    .bind(model.base.created_at)
    .bind(model.base.updated_at)
    .bind(&model.base.device_id)
    .bind(model.base.version as i64)
    .bind(serde_json::to_string(&model.base.sync_status).unwrap_or_default())
    .execute(&*pool)
    .await
    .map_err(|e| DatabaseError::QueryFailed(e.to_string()))?;

    Ok(())
}

pub async fn find_ssh_profile_by_id(
    provider: &PostgreSQLProvider,
    id: &str,
) -> DatabaseResult<Option<SSHProfile>> {
    let pool = provider.get_pool()?;
    let pool = pool.read().await;

    let row = sqlx::query(
        "SELECT id, name, host, port, username, group_id, auth_method, auth_data, description, color, timeout, keep_alive, compression, created_at, updated_at, device_id, version, sync_status FROM ssh_profiles WHERE id = ?"
    )
    .bind(id)
    .fetch_optional(&*pool)
    .await
    .map_err(|e| DatabaseError::QueryFailed(e.to_string()))?;

    if let Some(row) = row {
        let profile = SSHProfile {
            base: crate::models::base::BaseModel {
                id: row.get("id"),
                created_at: chrono::DateTime::parse_from_rfc3339(
                    &row.get::<String, _>("created_at"),
                )
                .map_err(|e| DatabaseError::ParseError(format!("Parse error: {}", e)))?
                .with_timezone(&chrono::Utc),
                updated_at: chrono::DateTime::parse_from_rfc3339(
                    &row.get::<String, _>("updated_at"),
                )
                .map_err(|e| DatabaseError::ParseError(format!("Parse error: {}", e)))?
                .with_timezone(&chrono::Utc),
                device_id: row.get("device_id"),
                version: row.get::<i64, _>("version") as u64,
                sync_status: serde_json::from_str(&row.get::<String, _>("sync_status"))
                    .unwrap_or(crate::database::traits::SyncStatus::Synced),
            },
            name: row.get("name"),
            host: row.get("host"),
            port: row.get::<i32, _>("port") as u16,
            username: row.get("username"),
            group_id: row.get("group_id"),
            auth_method: serde_json::from_str(&row.get::<String, _>("auth_method"))
                .map_err(DatabaseError::SerializationError)?,
            auth_data: serde_json::from_str(&row.get::<String, _>("auth_data"))
                .map_err(DatabaseError::SerializationError)?,
            timeout: row.get::<Option<i32>, _>("timeout").map(|t| t as u32),
            keep_alive: row.get("keep_alive"),
            compression: row.get("compression"),
            color: row.get("color"),
            description: row.get("description"),
            proxy: None,
        };
        Ok(Some(profile))
    } else {
        Ok(None)
    }
}

pub async fn find_all_ssh_profiles(provider: &PostgreSQLProvider) -> DatabaseResult<Vec<SSHProfile>> {
    let pool = provider.get_pool()?;
    let pool = pool.read().await;

    let rows = sqlx::query(
        "SELECT id, name, host, port, username, group_id, auth_method, auth_data, description, color, timeout, keep_alive, compression, created_at, updated_at, device_id, version, sync_status FROM ssh_profiles ORDER BY name"
    )
    .fetch_all(&*pool)
    .await
    .map_err(|e| DatabaseError::QueryFailed(e.to_string()))?;

    let mut profiles = Vec::new();
    for row in rows {
        let profile = SSHProfile {
            base: crate::models::base::BaseModel {
                id: row.get("id"),
                created_at: chrono::DateTime::parse_from_rfc3339(
                    &row.get::<String, _>("created_at"),
                )
                .map_err(|e| DatabaseError::ParseError(format!("Parse error: {}", e)))?
                .with_timezone(&chrono::Utc),
                updated_at: chrono::DateTime::parse_from_rfc3339(
                    &row.get::<String, _>("updated_at"),
                )
                .map_err(|e| DatabaseError::ParseError(format!("Parse error: {}", e)))?
                .with_timezone(&chrono::Utc),
                device_id: row.get("device_id"),
                version: row.get::<i64, _>("version") as u64,
                sync_status: serde_json::from_str(&row.get::<String, _>("sync_status"))
                    .unwrap_or(crate::database::traits::SyncStatus::Synced),
            },
            name: row.get("name"),
            host: row.get("host"),
            port: row.get::<i32, _>("port") as u16,
            username: row.get("username"),
            group_id: row.get("group_id"),
            auth_method: serde_json::from_str(&row.get::<String, _>("auth_method"))
                .map_err(DatabaseError::SerializationError)?,
            auth_data: serde_json::from_str(&row.get::<String, _>("auth_data"))
                .map_err(DatabaseError::SerializationError)?,
            timeout: row.get::<Option<i32>, _>("timeout").map(|t| t as u32),
            keep_alive: row.get("keep_alive"),
            compression: row.get("compression"),
            color: row.get("color"),
            description: row.get("description"),
            proxy: None,
        };
        profiles.push(profile);
    }

    Ok(profiles)
}

pub async fn update_ssh_profile(
    provider: &PostgreSQLProvider,
    model: &SSHProfile,
) -> DatabaseResult<()> {
    save_ssh_profile(provider, model).await
}

pub async fn delete_ssh_profile(provider: &PostgreSQLProvider, id: &str) -> DatabaseResult<()> {
    let pool_arc = provider.get_pool()?;
    let pool = pool_arc.read().await;

    sqlx::query("DELETE FROM ssh_profiles WHERE id = ?")
        .bind(id)
        .execute(&*pool)
        .await
        .map_err(|e| DatabaseError::QueryFailed(e.to_string()))?;

    Ok(())
}

pub async fn save_ssh_group(provider: &PostgreSQLProvider, model: &SSHGroup) -> DatabaseResult<()> {
    let pool_arc = provider.get_pool()?;
    let pool = pool_arc.read().await;

    sqlx::query(
        r#"
        INSERT INTO ssh_groups (
            id, name, description, color,
            created_at, updated_at, device_id, version, sync_status
        ) VALUES (?, ?, ?, ?, ?, ?, ?, ?, ?)
        ON DUPLICATE KEY UPDATE
            name = VALUES(name),
            description = VALUES(description),
            color = VALUES(color),
            updated_at = VALUES(updated_at),
            version = VALUES(version),
            sync_status = VALUES(sync_status)
    "#,
    )
    .bind(&model.base.id)
    .bind(&model.name)
    .bind(&model.description)
    .bind(&model.color)
    .bind(model.base.created_at)
    .bind(model.base.updated_at)
    .bind(&model.base.device_id)
    .bind(model.base.version as i64)
    .bind(serde_json::to_string(&model.base.sync_status).unwrap_or_default())
    .execute(&*pool)
    .await
    .map_err(|e| DatabaseError::QueryFailed(e.to_string()))?;

    Ok(())
}

pub async fn find_ssh_group_by_id(
    provider: &PostgreSQLProvider,
    id: &str,
) -> DatabaseResult<Option<SSHGroup>> {
    let pool = provider.get_pool()?;
    let pool = pool.read().await;

    let row = sqlx::query(
        "SELECT id, name, description, color, created_at, updated_at, device_id, version, sync_status FROM ssh_groups WHERE id = ?"
    )
    .bind(id)
    .fetch_optional(&*pool)
    .await
    .map_err(|e| DatabaseError::QueryFailed(e.to_string()))?;

    if let Some(row) = row {
        let group = SSHGroup {
            base: crate::models::base::BaseModel {
                id: row.get("id"),
                created_at: chrono::DateTime::parse_from_rfc3339(
                    &row.get::<String, _>("created_at"),
                )
                .map_err(|e| DatabaseError::ParseError(format!("Parse error: {}", e)))?
                .with_timezone(&chrono::Utc),
                updated_at: chrono::DateTime::parse_from_rfc3339(
                    &row.get::<String, _>("updated_at"),
                )
                .map_err(|e| DatabaseError::ParseError(format!("Parse error: {}", e)))?
                .with_timezone(&chrono::Utc),
                device_id: row.get("device_id"),
                version: row.get::<i64, _>("version") as u64,
                sync_status: serde_json::from_str(&row.get::<String, _>("sync_status"))
                    .unwrap_or(crate::database::traits::SyncStatus::Synced),
            },
            name: row.get("name"),
            description: row.get("description"),
            color: row.get("color"),
        };
        Ok(Some(group))
    } else {
        Ok(None)
    }
}

pub async fn find_all_ssh_groups(provider: &PostgreSQLProvider) -> DatabaseResult<Vec<SSHGroup>> {
    let pool = provider.get_pool()?;
    let pool = pool.read().await;

    let rows = sqlx::query(
        "SELECT id, name, description, color, created_at, updated_at, device_id, version, sync_status FROM ssh_groups ORDER BY name"
    )
    .fetch_all(&*pool)
    .await
    .map_err(|e| DatabaseError::QueryFailed(e.to_string()))?;

    let mut groups = Vec::new();
    for row in rows {
        let group = SSHGroup {
            base: crate::models::base::BaseModel {
                id: row.get("id"),
                created_at: chrono::DateTime::parse_from_rfc3339(
                    &row.get::<String, _>("created_at"),
                )
                .map_err(|e| DatabaseError::ParseError(format!("Parse error: {}", e)))?
                .with_timezone(&chrono::Utc),
                updated_at: chrono::DateTime::parse_from_rfc3339(
                    &row.get::<String, _>("updated_at"),
                )
                .map_err(|e| DatabaseError::ParseError(format!("Parse error: {}", e)))?
                .with_timezone(&chrono::Utc),
                device_id: row.get("device_id"),
                version: row.get::<i64, _>("version") as u64,
                sync_status: serde_json::from_str(&row.get::<String, _>("sync_status"))
                    .unwrap_or(crate::database::traits::SyncStatus::Synced),
            },
            name: row.get("name"),
            description: row.get("description"),
            color: row.get("color"),
        };
        groups.push(group);
    }

    Ok(groups)
}

pub async fn update_ssh_group(provider: &PostgreSQLProvider, model: &SSHGroup) -> DatabaseResult<()> {
    save_ssh_group(provider, model).await
}

pub async fn delete_ssh_group(provider: &PostgreSQLProvider, id: &str) -> DatabaseResult<()> {
    let pool_arc = provider.get_pool()?;
    let pool = pool_arc.read().await;

    sqlx::query("DELETE FROM ssh_groups WHERE id = ?")
        .bind(id)
        .execute(&*pool)
        .await
        .map_err(|e| DatabaseError::QueryFailed(e.to_string()))?;

    Ok(())
}

pub async fn save_ssh_key(provider: &PostgreSQLProvider, model: &SSHKey) -> DatabaseResult<()> {
    let pool_arc = provider.get_pool()?;
    let pool = pool_arc.read().await;

    sqlx::query(
        r#"
        INSERT INTO ssh_keys (
            id, name, key_type, private_key, public_key, passphrase, fingerprint,
            description, last_used, created_at, updated_at, device_id, version, sync_status
        ) VALUES (?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?)
        ON DUPLICATE KEY UPDATE
            name = VALUES(name),
            key_type = VALUES(key_type),
            private_key = VALUES(private_key),
            public_key = VALUES(public_key),
            passphrase = VALUES(passphrase),
            fingerprint = VALUES(fingerprint),
            description = VALUES(description),
            last_used = VALUES(last_used),
            updated_at = VALUES(updated_at),
            version = VALUES(version),
            sync_status = VALUES(sync_status)
    "#,
    )
    .bind(&model.base.id)
    .bind(&model.name)
    .bind(serde_json::to_string(&model.key_type).unwrap_or_default())
    .bind(&model.private_key)
    .bind(&model.public_key)
    .bind(&model.passphrase)
    .bind(&model.fingerprint)
    .bind(&model.description)
    .bind(model.last_used.as_ref().map(|t| t.to_rfc3339()))
    .bind(model.base.created_at)
    .bind(model.base.updated_at)
    .bind(&model.base.device_id)
    .bind(model.base.version as i64)
    .bind(serde_json::to_string(&model.base.sync_status).unwrap_or_default())
    .execute(&*pool)
    .await
    .map_err(|e| DatabaseError::QueryFailed(e.to_string()))?;

    Ok(())
}

pub async fn find_ssh_key_by_id(
    provider: &PostgreSQLProvider,
    id: &str,
) -> DatabaseResult<Option<SSHKey>> {
    let pool = provider.get_pool()?;
    let pool = pool.read().await;

    let row = sqlx::query(
        "SELECT id, name, key_type, private_key, public_key, passphrase, fingerprint, description, last_used, created_at, updated_at, device_id, version, sync_status FROM ssh_keys WHERE id = ?"
    )
    .bind(id)
    .fetch_optional(&*pool)
    .await
    .map_err(|e| DatabaseError::QueryFailed(e.to_string()))?;

    if let Some(row) = row {
        let key = SSHKey {
            base: crate::models::base::BaseModel {
                id: row.get("id"),
                created_at: chrono::DateTime::parse_from_rfc3339(
                    &row.get::<String, _>("created_at"),
                )
                .map_err(|e| DatabaseError::ParseError(format!("Parse error: {}", e)))?
                .with_timezone(&chrono::Utc),
                updated_at: chrono::DateTime::parse_from_rfc3339(
                    &row.get::<String, _>("updated_at"),
                )
                .map_err(|e| DatabaseError::ParseError(format!("Parse error: {}", e)))?
                .with_timezone(&chrono::Utc),
                device_id: row.get("device_id"),
                version: row.get::<i64, _>("version") as u64,
                sync_status: serde_json::from_str(&row.get::<String, _>("sync_status"))
                    .unwrap_or(crate::database::traits::SyncStatus::Synced),
            },
            name: row.get("name"),
            key_type: serde_json::from_str(&row.get::<String, _>("key_type"))
                .map_err(DatabaseError::SerializationError)?,
            private_key: row.get("private_key"),
            public_key: row.get("public_key"),
            passphrase: row.get("passphrase"),
            fingerprint: row.get("fingerprint"),
            description: row.get("description"),
            last_used: row
                .get::<Option<String>, _>("last_used")
                .and_then(|s| chrono::DateTime::parse_from_rfc3339(&s).ok())
                .map(|dt| dt.with_timezone(&chrono::Utc)),
        };
        Ok(Some(key))
    } else {
        Ok(None)
    }
}

pub async fn find_all_ssh_keys(provider: &PostgreSQLProvider) -> DatabaseResult<Vec<SSHKey>> {
    let pool = provider.get_pool()?;
    let pool = pool.read().await;

    let rows = sqlx::query(
        "SELECT id, name, key_type, private_key, public_key, passphrase, fingerprint, description, last_used, created_at, updated_at, device_id, version, sync_status FROM ssh_keys ORDER BY name"
    )
    .fetch_all(&*pool)
    .await
    .map_err(|e| DatabaseError::QueryFailed(e.to_string()))?;

    let mut keys = Vec::new();
    for row in rows {
        let key = SSHKey {
            base: crate::models::base::BaseModel {
                id: row.get("id"),
                created_at: chrono::DateTime::parse_from_rfc3339(
                    &row.get::<String, _>("created_at"),
                )
                .map_err(|e| DatabaseError::ParseError(format!("Parse error: {}", e)))?
                .with_timezone(&chrono::Utc),
                updated_at: chrono::DateTime::parse_from_rfc3339(
                    &row.get::<String, _>("updated_at"),
                )
                .map_err(|e| DatabaseError::ParseError(format!("Parse error: {}", e)))?
                .with_timezone(&chrono::Utc),
                device_id: row.get("device_id"),
                version: row.get::<i64, _>("version") as u64,
                sync_status: serde_json::from_str(&row.get::<String, _>("sync_status"))
                    .unwrap_or(crate::database::traits::SyncStatus::Synced),
            },
            name: row.get("name"),
            key_type: serde_json::from_str(&row.get::<String, _>("key_type"))
                .map_err(DatabaseError::SerializationError)?,
            private_key: row.get("private_key"),
            public_key: row.get("public_key"),
            passphrase: row.get("passphrase"),
            fingerprint: row.get("fingerprint"),
            description: row.get("description"),
            last_used: row
                .get::<Option<String>, _>("last_used")
                .and_then(|s| chrono::DateTime::parse_from_rfc3339(&s).ok())
                .map(|dt| dt.with_timezone(&chrono::Utc)),
        };
        keys.push(key);
    }

    Ok(keys)
}

pub async fn update_ssh_key(provider: &PostgreSQLProvider, model: &SSHKey) -> DatabaseResult<()> {
    save_ssh_key(provider, model).await
}

pub async fn delete_ssh_key(provider: &PostgreSQLProvider, id: &str) -> DatabaseResult<()> {
    let pool_arc = provider.get_pool()?;
    let pool = pool_arc.read().await;

    sqlx::query("DELETE FROM ssh_keys WHERE id = ?")
        .bind(id)
        .execute(&*pool)
        .await
        .map_err(|e| DatabaseError::QueryFailed(e.to_string()))?;

    Ok(())
}

pub async fn count_profiles_using_key(provider: &PostgreSQLProvider, key_id: &str) -> DatabaseResult<u32> {
    let pool = provider.get_pool()?;
    let pool = pool.read().await;

    let row = sqlx::query(
        "SELECT COUNT(*) as count FROM ssh_profiles WHERE auth_method = 'KeyReference' AND JSON_EXTRACT(auth_data, '$.keyId') = ?"
    )
    .bind(key_id)
    .fetch_one(&*pool)
    .await
    .map_err(|e| DatabaseError::QueryFailed(e.to_string()))?;

    let count: i64 = row.get("count");
    Ok(count as u32)
}
