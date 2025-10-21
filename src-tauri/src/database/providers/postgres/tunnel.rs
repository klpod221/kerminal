#![allow(dead_code)]

use sqlx::Row;

use crate::{
    database::error::{DatabaseError, DatabaseResult},
    models::ssh::SSHTunnel,
};

use super::PostgreSQLProvider;

pub async fn save_ssh_tunnel(provider: &PostgreSQLProvider, model: &SSHTunnel) -> DatabaseResult<()> {
    let pool = provider.get_pool()?;
    let pool = pool.read().await;

    sqlx::query(
        r#"
        INSERT INTO ssh_tunnels (
            id, name, description, profile_id, tunnel_type, local_host, local_port,
            remote_host, remote_port, auto_start, created_at, updated_at,
            device_id, version, sync_status
        ) VALUES (?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?)
        ON DUPLICATE KEY UPDATE
            name = VALUES(name),
            description = VALUES(description),
            profile_id = VALUES(profile_id),
            tunnel_type = VALUES(tunnel_type),
            local_host = VALUES(local_host),
            local_port = VALUES(local_port),
            remote_host = VALUES(remote_host),
            remote_port = VALUES(remote_port),
            auto_start = VALUES(auto_start),
            updated_at = VALUES(updated_at),
            version = VALUES(version),
            sync_status = VALUES(sync_status)
    "#,
    )
    .bind(&model.base.id)
    .bind(&model.name)
    .bind(&model.description)
    .bind(&model.profile_id)
    .bind(serde_json::to_string(&model.tunnel_type).unwrap())
    .bind(&model.local_host)
    .bind(model.local_port as i32)
    .bind(&model.remote_host)
    .bind(model.remote_port.map(|p| p as i32))
    .bind(model.auto_start)
    .bind(model.base.created_at.to_rfc3339())
    .bind(model.base.updated_at.to_rfc3339())
    .bind(&model.base.device_id)
    .bind(model.base.version as i64)
    .bind(serde_json::to_string(&model.base.sync_status).unwrap())
    .execute(&*pool)
    .await
    .map_err(|e| DatabaseError::QueryFailed(e.to_string()))?;

    Ok(())
}

pub async fn find_ssh_tunnel_by_id(
    provider: &PostgreSQLProvider,
    id: &str,
) -> DatabaseResult<Option<SSHTunnel>> {
    let pool = provider.get_pool()?;
    let pool = pool.read().await;

    let row = sqlx::query("SELECT * FROM ssh_tunnels WHERE id = ?")
        .bind(id)
        .fetch_optional(&*pool)
        .await
        .map_err(|e| DatabaseError::QueryFailed(e.to_string()))?;

    if let Some(row) = row {
        let tunnel = SSHTunnel {
            base: crate::models::base::BaseModel {
                id: row.get("id"),
                created_at: chrono::DateTime::parse_from_rfc3339(
                    &row.get::<String, _>("created_at"),
                )
                .map_err(|e| DatabaseError::QueryFailed(e.to_string()))?
                .with_timezone(&chrono::Utc),
                updated_at: chrono::DateTime::parse_from_rfc3339(
                    &row.get::<String, _>("updated_at"),
                )
                .map_err(|e| DatabaseError::QueryFailed(e.to_string()))?
                .with_timezone(&chrono::Utc),
                device_id: row.get("device_id"),
                version: row.get::<i64, _>("version") as u64,
                sync_status: serde_json::from_str(&row.get::<String, _>("sync_status"))
                    .unwrap_or(crate::database::traits::SyncStatus::Synced),
            },
            name: row.get("name"),
            description: row.get("description"),
            profile_id: row.get("profile_id"),
            tunnel_type: serde_json::from_str(&row.get::<String, _>("tunnel_type"))
                .map_err(|e| DatabaseError::QueryFailed(e.to_string()))?,
            local_host: row.get("local_host"),
            local_port: row.get::<i32, _>("local_port") as u16,
            remote_host: row.get("remote_host"),
            remote_port: row.get::<Option<i32>, _>("remote_port").map(|p| p as u16),
            auto_start: row.get("auto_start"),
            status: crate::models::ssh::TunnelStatus::default(),
            error_message: None,
        };
        Ok(Some(tunnel))
    } else {
        Ok(None)
    }
}

pub async fn find_all_ssh_tunnels(provider: &PostgreSQLProvider) -> DatabaseResult<Vec<SSHTunnel>> {
    let pool = provider.get_pool()?;
    let pool = pool.read().await;

    let rows = sqlx::query("SELECT * FROM ssh_tunnels ORDER BY name")
        .fetch_all(&*pool)
        .await
        .map_err(|e| DatabaseError::QueryFailed(e.to_string()))?;

    let mut tunnels = Vec::new();
    for row in rows {
        let tunnel = SSHTunnel {
            base: crate::models::base::BaseModel {
                id: row.get("id"),
                created_at: chrono::DateTime::parse_from_rfc3339(
                    &row.get::<String, _>("created_at"),
                )
                .map_err(|e| DatabaseError::QueryFailed(e.to_string()))?
                .with_timezone(&chrono::Utc),
                updated_at: chrono::DateTime::parse_from_rfc3339(
                    &row.get::<String, _>("updated_at"),
                )
                .map_err(|e| DatabaseError::QueryFailed(e.to_string()))?
                .with_timezone(&chrono::Utc),
                device_id: row.get("device_id"),
                version: row.get::<i64, _>("version") as u64,
                sync_status: serde_json::from_str(&row.get::<String, _>("sync_status"))
                    .unwrap_or(crate::database::traits::SyncStatus::Synced),
            },
            name: row.get("name"),
            description: row.get("description"),
            profile_id: row.get("profile_id"),
            tunnel_type: serde_json::from_str(&row.get::<String, _>("tunnel_type"))
                .map_err(|e| DatabaseError::QueryFailed(e.to_string()))?,
            local_host: row.get("local_host"),
            local_port: row.get::<i32, _>("local_port") as u16,
            remote_host: row.get("remote_host"),
            remote_port: row.get::<Option<i32>, _>("remote_port").map(|p| p as u16),
            auto_start: row.get("auto_start"),
            status: crate::models::ssh::TunnelStatus::default(),
            error_message: None,
        };
        tunnels.push(tunnel);
    }

    Ok(tunnels)
}

pub async fn find_ssh_tunnels_by_profile_id(
    provider: &PostgreSQLProvider,
    profile_id: &str,
) -> DatabaseResult<Vec<SSHTunnel>> {
    let pool = provider.get_pool()?;
    let pool = pool.read().await;

    let rows = sqlx::query("SELECT * FROM ssh_tunnels WHERE profile_id = ? ORDER BY name")
        .bind(profile_id)
        .fetch_all(&*pool)
        .await
        .map_err(|e| DatabaseError::QueryFailed(e.to_string()))?;

    let mut tunnels = Vec::new();
    for row in rows {
        let tunnel = SSHTunnel {
            base: crate::models::base::BaseModel {
                id: row.get("id"),
                created_at: chrono::DateTime::parse_from_rfc3339(
                    &row.get::<String, _>("created_at"),
                )
                .map_err(|e| DatabaseError::QueryFailed(e.to_string()))?
                .with_timezone(&chrono::Utc),
                updated_at: chrono::DateTime::parse_from_rfc3339(
                    &row.get::<String, _>("updated_at"),
                )
                .map_err(|e| DatabaseError::QueryFailed(e.to_string()))?
                .with_timezone(&chrono::Utc),
                device_id: row.get("device_id"),
                version: row.get::<i64, _>("version") as u64,
                sync_status: serde_json::from_str(&row.get::<String, _>("sync_status"))
                    .unwrap_or(crate::database::traits::SyncStatus::Synced),
            },
            name: row.get("name"),
            description: row.get("description"),
            profile_id: row.get("profile_id"),
            tunnel_type: serde_json::from_str(&row.get::<String, _>("tunnel_type"))
                .map_err(|e| DatabaseError::QueryFailed(e.to_string()))?,
            local_host: row.get("local_host"),
            local_port: row.get::<i32, _>("local_port") as u16,
            remote_host: row.get("remote_host"),
            remote_port: row.get::<Option<i32>, _>("remote_port").map(|p| p as u16),
            auto_start: row.get("auto_start"),
            status: crate::models::ssh::TunnelStatus::default(),
            error_message: None,
        };
        tunnels.push(tunnel);
    }

    Ok(tunnels)
}

pub async fn find_auto_start_ssh_tunnels(
    provider: &PostgreSQLProvider,
) -> DatabaseResult<Vec<SSHTunnel>> {
    let pool = provider.get_pool()?;
    let pool = pool.read().await;

    let rows = sqlx::query("SELECT * FROM ssh_tunnels WHERE auto_start = true ORDER BY name")
        .fetch_all(&*pool)
        .await
        .map_err(|e| DatabaseError::QueryFailed(e.to_string()))?;

    let mut tunnels = Vec::new();
    for row in rows {
        let tunnel = SSHTunnel {
            base: crate::models::base::BaseModel {
                id: row.get("id"),
                created_at: chrono::DateTime::parse_from_rfc3339(
                    &row.get::<String, _>("created_at"),
                )
                .map_err(|e| DatabaseError::QueryFailed(e.to_string()))?
                .with_timezone(&chrono::Utc),
                updated_at: chrono::DateTime::parse_from_rfc3339(
                    &row.get::<String, _>("updated_at"),
                )
                .map_err(|e| DatabaseError::QueryFailed(e.to_string()))?
                .with_timezone(&chrono::Utc),
                device_id: row.get("device_id"),
                version: row.get::<i64, _>("version") as u64,
                sync_status: serde_json::from_str(&row.get::<String, _>("sync_status"))
                    .unwrap_or(crate::database::traits::SyncStatus::Synced),
            },
            name: row.get("name"),
            description: row.get("description"),
            profile_id: row.get("profile_id"),
            tunnel_type: serde_json::from_str(&row.get::<String, _>("tunnel_type"))
                .map_err(|e| DatabaseError::QueryFailed(e.to_string()))?,
            local_host: row.get("local_host"),
            local_port: row.get::<i32, _>("local_port") as u16,
            remote_host: row.get("remote_host"),
            remote_port: row.get::<Option<i32>, _>("remote_port").map(|p| p as u16),
            auto_start: row.get("auto_start"),
            status: crate::models::ssh::TunnelStatus::default(),
            error_message: None,
        };
        tunnels.push(tunnel);
    }

    Ok(tunnels)
}

pub async fn update_ssh_tunnel(
    provider: &PostgreSQLProvider,
    model: &SSHTunnel,
) -> DatabaseResult<()> {
    save_ssh_tunnel(provider, model).await
}

pub async fn delete_ssh_tunnel(provider: &PostgreSQLProvider, id: &str) -> DatabaseResult<()> {
    let pool_arc = provider.get_pool()?;
    let pool = pool_arc.read().await;

    sqlx::query("DELETE FROM ssh_tunnels WHERE id = ?")
        .bind(id)
        .execute(&*pool)
        .await
        .map_err(|e| DatabaseError::QueryFailed(e.to_string()))?;

    Ok(())
}

pub async fn delete_ssh_tunnels_by_profile_id(
    provider: &PostgreSQLProvider,
    profile_id: &str,
) -> DatabaseResult<()> {
    let pool_arc = provider.get_pool()?;
    let pool = pool_arc.read().await;

    sqlx::query("DELETE FROM ssh_tunnels WHERE profile_id = ?")
        .bind(profile_id)
        .execute(&*pool)
        .await
        .map_err(|e| DatabaseError::QueryFailed(e.to_string()))?;

    Ok(())
}
