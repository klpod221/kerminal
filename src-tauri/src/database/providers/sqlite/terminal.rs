/*
 * Kerminal - Modern Terminal Emulator & SSH Manager
 * Copyright (C) 2026 Bùi Thanh Xuân (klpod221)
 *
 * This program is free software: you can redistribute it and/or modify
 * it under the terms of the GNU General Public License as published by
 * the Free Software Foundation, either version 3 of the License, or
 * (at your option) any later version.
 *
 * This program is distributed in the hope that it will be useful,
 * but WITHOUT ANY WARRANTY; without even the implied warranty of
 * MERCHANTABILITY or FITNESS FOR A PARTICULAR PURPOSE.  See the
 * GNU General Public License for more details.
 *
 * You should have received a copy of the GNU General Public License
 * along with this program.  If not, see <https://www.gnu.org/licenses/>.
 */

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
            id, name, shell, working_dir, env, icon, color, command, is_default, created_at, updated_at
        ) VALUES (?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?)
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
    .bind(profile.is_default)
    .bind(profile.created_at)
    .bind(profile.updated_at)
    .execute(&*pool)
    .await
    .map_err(|e| DatabaseError::QueryFailed(e.to_string()))?;

    Ok(())
}

/// Helper to parse a TerminalProfile from a SQLite row
fn parse_terminal_profile(row: &sqlx::sqlite::SqliteRow) -> TerminalProfile {
    let env_str: String = row.try_get("env").unwrap_or_default();
    let env: Option<HashMap<String, String>> = serde_json::from_str(&env_str).ok();

    TerminalProfile {
        id: row.try_get("id").unwrap_or_default(),
        name: row.try_get("name").unwrap_or_default(),
        shell: row.try_get("shell").unwrap_or_default(),
        working_dir: row.try_get("working_dir").ok(),
        env,
        icon: row.try_get("icon").ok(),
        color: row.try_get("color").ok(),
        command: row.try_get("command").ok(),
        is_default: row.try_get("is_default").unwrap_or(false),
        created_at: row.try_get("created_at").unwrap_or_default(),
        updated_at: row.try_get("updated_at").unwrap_or_default(),
    }
}

pub async fn find_terminal_profile_by_id(
    provider: &SQLiteProvider,
    id: &str,
) -> DatabaseResult<Option<TerminalProfile>> {
    let pool_arc = provider.get_pool()?;
    let pool = pool_arc.read().await;

    let row = sqlx::query(
        r#"
        SELECT id, name, shell, working_dir, env, icon, color, command, is_default, created_at, updated_at
        FROM terminal_profiles
        WHERE id = ?
        "#,
    )
    .bind(id)
    .fetch_optional(&*pool)
    .await
    .map_err(|e| DatabaseError::QueryFailed(e.to_string()))?;

    Ok(row.as_ref().map(parse_terminal_profile))
}

pub async fn find_all_terminal_profiles(
    provider: &SQLiteProvider,
) -> DatabaseResult<Vec<TerminalProfile>> {
    let pool_arc = provider.get_pool()?;
    let pool = pool_arc.read().await;

    let rows = sqlx::query(
        r#"
        SELECT id, name, shell, working_dir, env, icon, color, command, is_default, created_at, updated_at
        FROM terminal_profiles
        ORDER BY name ASC
        "#,
    )
    .fetch_all(&*pool)
    .await
    .map_err(|e| DatabaseError::QueryFailed(e.to_string()))?;

    Ok(rows.iter().map(parse_terminal_profile).collect())
}

/// Find the profile marked as default, if any
pub async fn find_default_terminal_profile(
    provider: &SQLiteProvider,
) -> DatabaseResult<Option<TerminalProfile>> {
    let pool_arc = provider.get_pool()?;
    let pool = pool_arc.read().await;

    let row = sqlx::query(
        r#"
        SELECT id, name, shell, working_dir, env, icon, color, command, is_default, created_at, updated_at
        FROM terminal_profiles
        WHERE is_default = 1
        LIMIT 1
        "#,
    )
    .fetch_optional(&*pool)
    .await
    .map_err(|e| DatabaseError::QueryFailed(e.to_string()))?;

    Ok(row.as_ref().map(parse_terminal_profile))
}

/// Set a profile as default, clearing all others
pub async fn set_default_terminal_profile(
    provider: &SQLiteProvider,
    id: &str,
) -> DatabaseResult<()> {
    let pool_arc = provider.get_pool()?;
    let pool = pool_arc.read().await;

    // Clear all defaults first
    sqlx::query("UPDATE terminal_profiles SET is_default = 0 WHERE is_default = 1")
        .execute(&*pool)
        .await
        .map_err(|e| DatabaseError::QueryFailed(e.to_string()))?;

    // Set the new default
    sqlx::query("UPDATE terminal_profiles SET is_default = 1 WHERE id = ?")
        .bind(id)
        .execute(&*pool)
        .await
        .map_err(|e| DatabaseError::QueryFailed(e.to_string()))?;

    Ok(())
}

/// Clear default status from all profiles
pub async fn clear_default_terminal_profile(provider: &SQLiteProvider) -> DatabaseResult<()> {
    let pool_arc = provider.get_pool()?;
    let pool = pool_arc.read().await;

    sqlx::query("UPDATE terminal_profiles SET is_default = 0 WHERE is_default = 1")
        .execute(&*pool)
        .await
        .map_err(|e| DatabaseError::QueryFailed(e.to_string()))?;

    Ok(())
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
