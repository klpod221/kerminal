use crate::database::{
    error::{DatabaseError, DatabaseResult},
    providers::sqlite::SQLiteProvider,
};
use crate::models::terminal::profile::TerminalProfile;
use sqlx::Row;
use std::collections::HashMap;

pub async fn save_terminal_profile(
    provider: &SQLiteProvider,
    profile: &TerminalProfile,
) -> DatabaseResult<()> {
    let pool_arc = provider.get_pool()?;
    let pool = pool_arc.read().await;

    let env_json = serde_json::to_string(&profile.env)?;

    sqlx::query(
        r#"
        INSERT OR REPLACE INTO terminal_profiles (
            id, name, shell, working_dir, env, icon, color, command, created_at, updated_at
        ) VALUES (?, ?, ?, ?, ?, ?, ?, ?, ?, ?)
        "#,
    )
    .bind(&profile.id)
    .bind(&profile.name)
    .bind(&profile.shell)
    .bind(&profile.working_dir)
    .bind(env_json)
    .bind(&profile.icon)
    .bind(&profile.color)
    .bind(&profile.command)
    .bind(profile.created_at)
    .bind(profile.updated_at)
    .execute(&*pool)
    .await
    .map_err(|e| DatabaseError::QueryFailed(e.to_string()))?;

    Ok(())
}

pub async fn find_terminal_profile_by_id(
    provider: &SQLiteProvider,
    id: &str,
) -> DatabaseResult<Option<TerminalProfile>> {
    let pool_arc = provider.get_pool()?;
    let pool = pool_arc.read().await;

    let row = sqlx::query(
        r#"
        SELECT id, name, shell, working_dir, env, icon, color, command, created_at, updated_at
        FROM terminal_profiles
        WHERE id = ?
        "#,
    )
    .bind(id)
    .fetch_optional(&*pool)
    .await
    .map_err(|e| DatabaseError::QueryFailed(e.to_string()))?;

    if let Some(row) = row {
        let env_str: String = row.try_get("env").unwrap_or_default();
        let env: Option<HashMap<String, String>> = serde_json::from_str(&env_str).ok();

        Ok(Some(TerminalProfile {
            id: row.try_get("id").unwrap_or_default(),
            name: row.try_get("name").unwrap_or_default(),
            shell: row.try_get("shell").unwrap_or_default(),
            working_dir: row.try_get("working_dir").ok(),
            env,
            icon: row.try_get("icon").ok(),
            color: row.try_get("color").ok(),
            command: row.try_get("command").ok(),
            created_at: row.try_get("created_at").unwrap_or_default(),
            updated_at: row.try_get("updated_at").unwrap_or_default(),
        }))
    } else {
        Ok(None)
    }
}

pub async fn find_all_terminal_profiles(
    provider: &SQLiteProvider,
) -> DatabaseResult<Vec<TerminalProfile>> {
    let pool_arc = provider.get_pool()?;
    let pool = pool_arc.read().await;

    let rows = sqlx::query(
        r#"
        SELECT id, name, shell, working_dir, env, icon, color, command, created_at, updated_at
        FROM terminal_profiles
        ORDER BY name ASC
        "#,
    )
    .fetch_all(&*pool)
    .await
    .map_err(|e| DatabaseError::QueryFailed(e.to_string()))?;

    let mut profiles = Vec::new();
    for row in rows {
        let env_str: String = row.try_get("env").unwrap_or_default();
        let env: Option<HashMap<String, String>> = serde_json::from_str(&env_str).ok();

        profiles.push(TerminalProfile {
            id: row.try_get("id").unwrap_or_default(),
            name: row.try_get("name").unwrap_or_default(),
            shell: row.try_get("shell").unwrap_or_default(),
            working_dir: row.try_get("working_dir").ok(),
            env,
            icon: row.try_get("icon").ok(),
            color: row.try_get("color").ok(),
            command: row.try_get("command").ok(),
            created_at: row.try_get("created_at").unwrap_or_default(),
            updated_at: row.try_get("updated_at").unwrap_or_default(),
        });
    }

    Ok(profiles)
}

pub async fn delete_terminal_profile(provider: &SQLiteProvider, id: &str) -> DatabaseResult<()> {
    let pool_arc = provider.get_pool()?;
    let pool = pool_arc.read().await;

    sqlx::query("DELETE FROM terminal_profiles WHERE id = ?")
        .bind(id)
        .execute(&*pool)
        .await
        .map_err(|e| DatabaseError::QueryFailed(e.to_string()))?;

    Ok(())
}
