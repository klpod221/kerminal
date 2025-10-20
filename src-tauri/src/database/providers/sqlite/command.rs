use sqlx::Row;

use crate::{
    database::error::{DatabaseError, DatabaseResult},
    models::saved_command::{SavedCommand, SavedCommandGroup},
};

use super::SQLiteProvider;

pub async fn save_saved_command(
    provider: &SQLiteProvider,
    command: &SavedCommand,
) -> DatabaseResult<()> {
    let pool = provider.get_pool()?;
    let pool = pool.read().await;

    sqlx::query(
        r#"
        INSERT OR REPLACE INTO saved_commands (
            id, name, description, command, group_id, tags, is_favorite,
            usage_count, last_used_at, created_at, updated_at, device_id, version, sync_status
        ) VALUES (?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?)
        "#,
    )
    .bind(&command.base.id)
    .bind(&command.name)
    .bind(&command.description)
    .bind(&command.command)
    .bind(&command.group_id)
    .bind(&command.tags)
    .bind(command.is_favorite)
    .bind(command.usage_count as i64)
    .bind(&command.last_used_at)
    .bind(command.base.created_at.to_rfc3339())
    .bind(command.base.updated_at.to_rfc3339())
    .bind(&command.base.device_id)
    .bind(command.base.version as i64)
    .bind(
        serde_json::to_string(&command.base.sync_status)
            .unwrap_or_else(|_| "\"synced\"".to_string()),
    )
    .execute(&*pool)
    .await
    .map_err(|e| DatabaseError::QueryFailed(e.to_string()))?;

    Ok(())
}

pub async fn find_all_saved_commands(
    provider: &SQLiteProvider,
) -> DatabaseResult<Vec<SavedCommand>> {
    let pool = provider.get_pool()?;
    let pool = pool.read().await;

    let rows = sqlx::query(
        "SELECT id, name, description, command, group_id, tags, is_favorite, usage_count, last_used_at, created_at, updated_at, device_id, version, sync_status FROM saved_commands ORDER BY name"
    )
    .fetch_all(&*pool)
    .await
    .map_err(|e| DatabaseError::QueryFailed(e.to_string()))?;

    let mut commands = Vec::new();
    for row in rows {
        let command = SavedCommand {
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
            command: row.get("command"),
            group_id: row.get("group_id"),
            tags: row.get("tags"),
            is_favorite: row.get("is_favorite"),
            usage_count: row.get::<i64, _>("usage_count") as u32,
            last_used_at: row.get("last_used_at"),
        };
        commands.push(command);
    }

    Ok(commands)
}

pub async fn find_saved_command_by_id(
    provider: &SQLiteProvider,
    id: &str,
) -> DatabaseResult<Option<SavedCommand>> {
    let pool = provider.get_pool()?;
    let pool = pool.read().await;

    let row = sqlx::query(
        "SELECT id, name, description, command, group_id, tags, is_favorite, usage_count, last_used_at, created_at, updated_at, device_id, version, sync_status FROM saved_commands WHERE id = ?"
    )
    .bind(id)
    .fetch_optional(&*pool)
    .await
    .map_err(|e| DatabaseError::QueryFailed(e.to_string()))?;

    if let Some(row) = row {
        Ok(Some(SavedCommand {
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
            command: row.get("command"),
            group_id: row.get("group_id"),
            tags: row.get("tags"),
            is_favorite: row.get("is_favorite"),
            usage_count: row.get::<i64, _>("usage_count") as u32,
            last_used_at: row.get("last_used_at"),
        }))
    } else {
        Ok(None)
    }
}

pub async fn find_saved_commands_by_group_id(
    provider: &SQLiteProvider,
    group_id: Option<&str>,
) -> DatabaseResult<Vec<SavedCommand>> {
    let pool = provider.get_pool()?;
    let pool = pool.read().await;

    let (query, bind_value) = if let Some(group_id) = group_id {
        ("SELECT id, name, description, command, group_id, tags, is_favorite, usage_count, last_used_at, created_at, updated_at, device_id, version, sync_status FROM saved_commands WHERE group_id = ? ORDER BY name", Some(group_id))
    } else {
        ("SELECT id, name, description, command, group_id, tags, is_favorite, usage_count, last_used_at, created_at, updated_at, device_id, version, sync_status FROM saved_commands WHERE group_id IS NULL ORDER BY name", None)
    };

    let mut query_builder = sqlx::query(query);
    if let Some(value) = bind_value {
        query_builder = query_builder.bind(value);
    }

    let rows = query_builder
        .fetch_all(&*pool)
        .await
        .map_err(|e| DatabaseError::QueryFailed(e.to_string()))?;

    let mut commands = Vec::new();
    for row in rows {
        let command = SavedCommand {
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
            command: row.get("command"),
            group_id: row.get("group_id"),
            tags: row.get("tags"),
            is_favorite: row.get("is_favorite"),
            usage_count: row.get::<i64, _>("usage_count") as u32,
            last_used_at: row.get("last_used_at"),
        };
        commands.push(command);
    }

    Ok(commands)
}

pub async fn delete_saved_command(provider: &SQLiteProvider, id: &str) -> DatabaseResult<()> {
    let pool = provider.get_pool()?;
    let pool = pool.read().await;

    sqlx::query("DELETE FROM saved_commands WHERE id = ?")
        .bind(id)
        .execute(&*pool)
        .await
        .map_err(|e| DatabaseError::QueryFailed(e.to_string()))?;

    Ok(())
}

pub async fn save_saved_command_group(
    provider: &SQLiteProvider,
    group: &SavedCommandGroup,
) -> DatabaseResult<()> {
    let pool = provider.get_pool()?;
    let pool = pool.read().await;

    sqlx::query(
        r#"
        INSERT OR REPLACE INTO saved_command_groups (
            id, name, description, color, icon, created_at, updated_at, device_id, version, sync_status
        ) VALUES (?, ?, ?, ?, ?, ?, ?, ?, ?, ?)
        "#,
    )
    .bind(&group.base.id)
    .bind(&group.name)
    .bind(&group.description)
    .bind(&group.color)
    .bind(&group.icon)
    .bind(group.base.created_at.to_rfc3339())
    .bind(group.base.updated_at.to_rfc3339())
    .bind(&group.base.device_id)
    .bind(group.base.version as i64)
    .bind(
        serde_json::to_string(&group.base.sync_status)
            .unwrap_or_else(|_| "\"synced\"".to_string()),
    )
    .execute(&*pool)
    .await
    .map_err(|e| DatabaseError::QueryFailed(e.to_string()))?;

    Ok(())
}

pub async fn find_all_saved_command_groups(
    provider: &SQLiteProvider,
) -> DatabaseResult<Vec<SavedCommandGroup>> {
    let pool = provider.get_pool()?;
    let pool = pool.read().await;

    let rows = sqlx::query(
        "SELECT id, name, description, color, icon, created_at, updated_at, device_id, version, sync_status FROM saved_command_groups ORDER BY name"
    )
    .fetch_all(&*pool)
    .await
    .map_err(|e| DatabaseError::QueryFailed(e.to_string()))?;

    let mut groups = Vec::new();
    for row in rows {
        let group = SavedCommandGroup {
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
            icon: row.get("icon"),
        };
        groups.push(group);
    }

    Ok(groups)
}

pub async fn find_saved_command_group_by_id(
    provider: &SQLiteProvider,
    id: &str,
) -> DatabaseResult<Option<SavedCommandGroup>> {
    let pool = provider.get_pool()?;
    let pool = pool.read().await;

    let row = sqlx::query(
        "SELECT id, name, description, color, icon, created_at, updated_at, device_id, version, sync_status FROM saved_command_groups WHERE id = ?"
    )
    .bind(id)
    .fetch_optional(&*pool)
    .await
    .map_err(|e| DatabaseError::QueryFailed(e.to_string()))?;

    if let Some(row) = row {
        Ok(Some(SavedCommandGroup {
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
            icon: row.get("icon"),
        }))
    } else {
        Ok(None)
    }
}

pub async fn delete_saved_command_group(provider: &SQLiteProvider, id: &str) -> DatabaseResult<()> {
    let pool = provider.get_pool()?;
    let pool = pool.read().await;

    sqlx::query("DELETE FROM saved_command_groups WHERE id = ?")
        .bind(id)
        .execute(&*pool)
        .await
        .map_err(|e| DatabaseError::QueryFailed(e.to_string()))?;

    Ok(())
}

impl SQLiteProvider {
    pub async fn save_saved_command(&self, command: &SavedCommand) -> DatabaseResult<()> {
        save_saved_command(self, command).await
    }

    pub async fn find_all_saved_commands(&self) -> DatabaseResult<Vec<SavedCommand>> {
        find_all_saved_commands(self).await
    }

    pub async fn find_saved_command_by_id(&self, id: &str) -> DatabaseResult<Option<SavedCommand>> {
        find_saved_command_by_id(self, id).await
    }

    pub async fn find_saved_commands_by_group_id(
        &self,
        group_id: Option<&str>,
    ) -> DatabaseResult<Vec<SavedCommand>> {
        find_saved_commands_by_group_id(self, group_id).await
    }

    pub async fn delete_saved_command(&self, id: &str) -> DatabaseResult<()> {
        delete_saved_command(self, id).await
    }

    pub async fn save_saved_command_group(&self, group: &SavedCommandGroup) -> DatabaseResult<()> {
        save_saved_command_group(self, group).await
    }

    pub async fn find_all_saved_command_groups(&self) -> DatabaseResult<Vec<SavedCommandGroup>> {
        find_all_saved_command_groups(self).await
    }

    pub async fn find_saved_command_group_by_id(
        &self,
        id: &str,
    ) -> DatabaseResult<Option<SavedCommandGroup>> {
        find_saved_command_group_by_id(self, id).await
    }

    pub async fn delete_saved_command_group(&self, id: &str) -> DatabaseResult<()> {
        delete_saved_command_group(self, id).await
    }
}
