
use chrono::{DateTime, Utc};
use sqlx::Row;

use crate::{
    database::error::{DatabaseError, DatabaseResult},
    database::traits::SyncStatus,
    models::sync::{
        ConflictResolutionStrategy, ExternalDatabaseConfig, SyncConflict, SyncOperation,
    },
};

use super::SQLiteProvider;

impl SQLiteProvider {
    pub async fn save_external_database(
        &self,
        config: &ExternalDatabaseConfig,
    ) -> DatabaseResult<()> {
        let pool = self.get_pool()?;
        let pool = pool.read().await;

        sqlx::query(
            r#"
            INSERT INTO external_databases (
                id, name, db_type, connection_details_encrypted,
                created_at, updated_at, device_id, version, sync_status
            ) VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9)
            ON CONFLICT(id) DO UPDATE SET
                name = excluded.name,
                db_type = excluded.db_type,
                connection_details_encrypted = excluded.connection_details_encrypted,
                updated_at = excluded.updated_at,
                version = excluded.version,
                sync_status = excluded.sync_status
        "#,
        )
        .bind(&config.base.id)
        .bind(&config.name)
        .bind(config.db_type.to_string())
        .bind(&config.connection_details_encrypted)
        .bind(config.base.created_at)
        .bind(config.base.updated_at)
        .bind(&config.base.device_id)
        .bind(config.base.version as i64)
        .bind(config.base.sync_status.to_string())
        .execute(&*pool)
        .await
        .map_err(|e| crate::database::error::DatabaseError::QueryFailed(e.to_string()))?;

        Ok(())
    }

    pub async fn find_external_database_by_id(
        &self,
        id: &str,
    ) -> DatabaseResult<Option<ExternalDatabaseConfig>> {
        let pool = self.get_pool()?;
        let pool = pool.read().await;

        let row = sqlx::query(
            r#"
            SELECT id, name, db_type, connection_details_encrypted,
                created_at, updated_at, device_id, version, sync_status
            FROM external_databases
            WHERE id = ?1
        "#,
        )
        .bind(id)
        .fetch_optional(&*pool)
        .await
        .map_err(|e| crate::database::error::DatabaseError::QueryFailed(e.to_string()))?;

        if let Some(row) = row {
            let config = self.map_external_database_row(&row)?;
            Ok(Some(config))
        } else {
            Ok(None)
        }
    }

    pub async fn find_all_external_databases(&self) -> DatabaseResult<Vec<ExternalDatabaseConfig>> {
        let pool = self.get_pool()?;
        let pool = pool.read().await;

        let rows = sqlx::query(
            r#"
            SELECT id, name, db_type, connection_details_encrypted,
                created_at, updated_at, device_id, version, sync_status
            FROM external_databases
            ORDER BY created_at DESC
        "#,
        )
        .fetch_all(&*pool)
        .await
        .map_err(|e| crate::database::error::DatabaseError::QueryFailed(e.to_string()))?;

        // Get global sync settings to check if sync is active
        let is_active = sqlx::query_scalar::<_, bool>(
            r#"
            SELECT is_active FROM sync_settings WHERE id = 'global'
        "#,
        )
        .fetch_optional(&*pool)
        .await
        .map_err(|e| crate::database::error::DatabaseError::QueryFailed(e.to_string()))?
        .unwrap_or(false);

        let mut configs = Vec::new();
        for row in rows {
            let mut config = self.map_external_database_row(&row)?;
            config.is_active = is_active; // Apply global sync active status
            configs.push(config);
        }

        Ok(configs)
    }

    pub async fn delete_external_database(&self, id: &str) -> DatabaseResult<()> {
        let pool = self.get_pool()?;
        let pool = pool.read().await;

        sqlx::query(
            r#"
            DELETE FROM external_databases WHERE id = ?1
        "#,
        )
        .bind(id)
        .execute(&*pool)
        .await
        .map_err(|e| crate::database::error::DatabaseError::QueryFailed(e.to_string()))?;

        Ok(())
    }

    pub async fn find_recent_sync_operations(
        &self,
        limit: i32,
    ) -> DatabaseResult<Vec<SyncOperation>> {
        let pool = self.get_pool()?;
        let pool = pool.read().await;

        let rows = sqlx::query(
            r#"
            SELECT id, operation_type, entity_type, entity_id, source_db, target_db,
                status, error_message, started_at, completed_at
            FROM sync_operations
            ORDER BY started_at DESC
            LIMIT ?1
        "#,
        )
        .bind(limit)
        .fetch_all(&*pool)
        .await
        .map_err(|e| crate::database::error::DatabaseError::QueryFailed(e.to_string()))?;

        let mut operations = Vec::new();
        for row in rows {
            operations.push(self.map_sync_operation_row(&row)?);
        }

        Ok(operations)
    }

    pub async fn find_unresolved_conflicts(&self) -> DatabaseResult<Vec<SyncConflict>> {
        let pool = self.get_pool()?;
        let pool = pool.read().await;

        let rows = sqlx::query(
            r#"
            SELECT id, entity_type, entity_id, local_version, remote_version,
                local_data, remote_data, resolution_strategy, resolved, created_at, resolved_at
            FROM sync_conflicts
            WHERE resolved = false
            ORDER BY created_at DESC
        "#,
        )
        .fetch_all(&*pool)
        .await
        .map_err(|e| crate::database::error::DatabaseError::QueryFailed(e.to_string()))?;

        let mut conflicts = Vec::new();
        for row in rows {
            conflicts.push(self.map_sync_conflict_row(&row)?);
        }

        Ok(conflicts)
    }

    pub async fn resolve_conflict(
        &self,
        id: &str,
        strategy: ConflictResolutionStrategy,
    ) -> DatabaseResult<()> {
        let pool = self.get_pool()?;
        let pool = pool.read().await;

        sqlx::query(
            r#"
            UPDATE sync_conflicts
            SET resolution_strategy = ?1, resolved = true, resolved_at = ?2
            WHERE id = ?3
        "#,
        )
        .bind(strategy.to_string())
        .bind(Utc::now())
        .bind(id)
        .execute(&*pool)
        .await
        .map_err(|e| crate::database::error::DatabaseError::QueryFailed(e.to_string()))?;

        Ok(())
    }

    fn map_external_database_row(
        &self,
        row: &sqlx::sqlite::SqliteRow,
    ) -> DatabaseResult<ExternalDatabaseConfig> {
        use std::str::FromStr;

        let id: String = row
            .try_get("id")
            .map_err(|e| crate::database::error::DatabaseError::QueryFailed(e.to_string()))?;
        let name: String = row
            .try_get("name")
            .map_err(|e| crate::database::error::DatabaseError::QueryFailed(e.to_string()))?;
        let db_type_str: String = row
            .try_get("db_type")
            .map_err(|e| crate::database::error::DatabaseError::QueryFailed(e.to_string()))?;
        let connection_details_encrypted: String = row
            .try_get("connection_details_encrypted")
            .map_err(|e| crate::database::error::DatabaseError::QueryFailed(e.to_string()))?;
        let created_at: String = row
            .try_get("created_at")
            .map_err(|e| crate::database::error::DatabaseError::QueryFailed(e.to_string()))?;
        let updated_at: String = row
            .try_get("updated_at")
            .map_err(|e| crate::database::error::DatabaseError::QueryFailed(e.to_string()))?;
        let device_id: String = row
            .try_get("device_id")
            .map_err(|e| crate::database::error::DatabaseError::QueryFailed(e.to_string()))?;
        let version: i64 = row
            .try_get("version")
            .map_err(|e| crate::database::error::DatabaseError::QueryFailed(e.to_string()))?;
        let sync_status_str: String = row
            .try_get("sync_status")
            .map_err(|e| crate::database::error::DatabaseError::QueryFailed(e.to_string()))?;

        let db_type = crate::models::sync::DatabaseType::from_str(&db_type_str)
            .map_err(crate::database::error::DatabaseError::QueryFailed)?;

        let sync_status = SyncStatus::from_str(&sync_status_str)
            .map_err(crate::database::error::DatabaseError::QueryFailed)?;

        Ok(ExternalDatabaseConfig {
            base: crate::models::base::BaseModel {
                id,
                created_at: DateTime::parse_from_rfc3339(&created_at)
                    .map_err(|e| crate::database::error::DatabaseError::QueryFailed(e.to_string()))?
                    .with_timezone(&Utc),
                updated_at: DateTime::parse_from_rfc3339(&updated_at)
                    .map_err(|e| crate::database::error::DatabaseError::QueryFailed(e.to_string()))?
                    .with_timezone(&Utc),
                device_id,
                version: version as u64,
                sync_status,
            },
            name,
            db_type,
            connection_details_encrypted,
            is_active: false, // Will be updated by join or separate query
        })
    }

    fn map_sync_operation_row(
        &self,
        row: &sqlx::sqlite::SqliteRow,
    ) -> DatabaseResult<SyncOperation> {
        use std::str::FromStr;

        let id: String = row
            .try_get("id")
            .map_err(|e| crate::database::error::DatabaseError::QueryFailed(e.to_string()))?;
        let operation_type_str: String = row
            .try_get("operation_type")
            .map_err(|e| crate::database::error::DatabaseError::QueryFailed(e.to_string()))?;
        let entity_type: String = row
            .try_get("entity_type")
            .map_err(|e| crate::database::error::DatabaseError::QueryFailed(e.to_string()))?;
        let entity_id: String = row
            .try_get("entity_id")
            .map_err(|e| crate::database::error::DatabaseError::QueryFailed(e.to_string()))?;
        let source_db: String = row
            .try_get("source_db")
            .map_err(|e| crate::database::error::DatabaseError::QueryFailed(e.to_string()))?;
        let target_db: String = row
            .try_get("target_db")
            .map_err(|e| crate::database::error::DatabaseError::QueryFailed(e.to_string()))?;
        let status_str: String = row
            .try_get("status")
            .map_err(|e| crate::database::error::DatabaseError::QueryFailed(e.to_string()))?;
        let error_message: Option<String> = row
            .try_get("error_message")
            .map_err(|e| crate::database::error::DatabaseError::QueryFailed(e.to_string()))?;
        let started_at: String = row
            .try_get("started_at")
            .map_err(|e| crate::database::error::DatabaseError::QueryFailed(e.to_string()))?;
        let completed_at: Option<String> = row
            .try_get("completed_at")
            .map_err(|e| crate::database::error::DatabaseError::QueryFailed(e.to_string()))?;

        let operation_type = crate::models::sync::SyncOperationType::from_str(&operation_type_str)
            .map_err(crate::database::error::DatabaseError::QueryFailed)?;

        let status = crate::models::sync::SyncOperationStatus::from_str(&status_str)
            .map_err(crate::database::error::DatabaseError::QueryFailed)?;

        let completed_at_parsed = if let Some(dt_str) = completed_at {
            Some(
                DateTime::parse_from_rfc3339(&dt_str)
                    .map_err(|e| crate::database::error::DatabaseError::QueryFailed(e.to_string()))?
                    .with_timezone(&Utc),
            )
        } else {
            None
        };

        Ok(SyncOperation {
            id,
            operation_type,
            entity_type,
            entity_id,
            source_db,
            target_db,
            status,
            error_message,
            started_at: DateTime::parse_from_rfc3339(&started_at)
                .map_err(|e| crate::database::error::DatabaseError::QueryFailed(e.to_string()))?
                .with_timezone(&Utc),
            completed_at: completed_at_parsed,
        })
    }

    fn map_sync_conflict_row(&self, row: &sqlx::sqlite::SqliteRow) -> DatabaseResult<SyncConflict> {
        use std::str::FromStr;

        let id: String = row
            .try_get("id")
            .map_err(|e| crate::database::error::DatabaseError::QueryFailed(e.to_string()))?;
        let entity_type: String = row
            .try_get("entity_type")
            .map_err(|e| crate::database::error::DatabaseError::QueryFailed(e.to_string()))?;
        let entity_id: String = row
            .try_get("entity_id")
            .map_err(|e| crate::database::error::DatabaseError::QueryFailed(e.to_string()))?;
        let local_version: i64 = row
            .try_get("local_version")
            .map_err(|e| crate::database::error::DatabaseError::QueryFailed(e.to_string()))?;
        let remote_version: i64 = row
            .try_get("remote_version")
            .map_err(|e| crate::database::error::DatabaseError::QueryFailed(e.to_string()))?;
        let local_data: String = row
            .try_get("local_data")
            .map_err(|e| crate::database::error::DatabaseError::QueryFailed(e.to_string()))?;
        let remote_data: String = row
            .try_get("remote_data")
            .map_err(|e| crate::database::error::DatabaseError::QueryFailed(e.to_string()))?;
        let resolution_strategy_str: Option<String> = row
            .try_get("resolution_strategy")
            .map_err(|e| crate::database::error::DatabaseError::QueryFailed(e.to_string()))?;
        let resolved: bool = row
            .try_get("resolved")
            .map_err(|e| crate::database::error::DatabaseError::QueryFailed(e.to_string()))?;
        let created_at: String = row
            .try_get("created_at")
            .map_err(|e| crate::database::error::DatabaseError::QueryFailed(e.to_string()))?;
        let resolved_at: Option<String> = row
            .try_get("resolved_at")
            .map_err(|e| crate::database::error::DatabaseError::QueryFailed(e.to_string()))?;

        let resolution_strategy = if let Some(strategy_str) = resolution_strategy_str {
            Some(
                ConflictResolutionStrategy::from_str(&strategy_str)
                    .map_err(crate::database::error::DatabaseError::QueryFailed)?,
            )
        } else {
            None
        };

        let resolved_at_parsed = if let Some(dt_str) = resolved_at {
            Some(
                DateTime::parse_from_rfc3339(&dt_str)
                    .map_err(|e| crate::database::error::DatabaseError::QueryFailed(e.to_string()))?
                    .with_timezone(&Utc),
            )
        } else {
            None
        };

        Ok(SyncConflict {
            id,
            entity_type,
            entity_id,
            local_version: local_version as u64,
            remote_version: remote_version as u64,
            local_data,
            remote_data,
            resolution_strategy,
            resolved,
            created_at: DateTime::parse_from_rfc3339(&created_at)
                .map_err(|e| crate::database::error::DatabaseError::QueryFailed(e.to_string()))?
                .with_timezone(&Utc),
            resolved_at: resolved_at_parsed,
        })
    }

    /// Get global sync settings (single instance)
    #[allow(dead_code)]
    pub async fn get_global_sync_settings(
        &self,
    ) -> DatabaseResult<Option<crate::models::sync::SyncSettings>> {
        println!("SQLite::get_global_sync_settings: Starting query");

        let pool = self.get_pool()?;
        let pool = pool.read().await;

        println!("SQLite::get_global_sync_settings: Pool acquired");

        let row = sqlx::query(
            r#"
            SELECT id, is_active, auto_sync_enabled, sync_interval_minutes,
                conflict_strategy, sync_direction, selected_database_id, last_sync_at,
                created_at, updated_at
            FROM sync_settings
            WHERE id = 'global'
        "#,
        )
        .fetch_optional(&*pool)
        .await
        .map_err(|e| {
            println!("SQLite::get_global_sync_settings: Query failed: {}", e);
            crate::database::error::DatabaseError::QueryFailed(e.to_string())
        })?;

        println!("SQLite::get_global_sync_settings: Query executed successfully");

        if let Some(row) = row {
            let settings = self.map_sync_settings_row(&row)?;
            Ok(Some(settings))
        } else {
            Ok(None)
        }
    }

    /// Save global sync settings (upsert)
    #[allow(dead_code)]
    pub async fn save_global_sync_settings(
        &self,
        settings: &crate::models::sync::SyncSettings,
    ) -> DatabaseResult<()> {
        let pool = self.get_pool()?;
        let pool = pool.read().await;

        sqlx::query(
            r#"
            INSERT INTO sync_settings (
                id, is_active, auto_sync_enabled, sync_interval_minutes,
                conflict_strategy, sync_direction, selected_database_id, last_sync_at,
                created_at, updated_at
            ) VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10)
            ON CONFLICT(id) DO UPDATE SET
                is_active = excluded.is_active,
                auto_sync_enabled = excluded.auto_sync_enabled,
                sync_interval_minutes = excluded.sync_interval_minutes,
                conflict_strategy = excluded.conflict_strategy,
                sync_direction = excluded.sync_direction,
                selected_database_id = excluded.selected_database_id,
                last_sync_at = excluded.last_sync_at,
                updated_at = excluded.updated_at
        "#,
        )
        .bind(&settings.id)
        .bind(settings.is_active)
        .bind(settings.auto_sync_enabled)
        .bind(settings.sync_interval_minutes as i64)
        .bind(settings.conflict_strategy.to_string())
        .bind(settings.sync_direction.to_string())
        .bind(&settings.selected_database_id)
        .bind(settings.last_sync_at.map(|dt| dt.to_rfc3339()))
        .bind(settings.created_at.to_rfc3339())
        .bind(settings.updated_at.to_rfc3339())
        .execute(&*pool)
        .await
        .map_err(|e| crate::database::error::DatabaseError::QueryFailed(e.to_string()))?;

        Ok(())
    }

    /// Update global sync settings with partial data
    #[allow(dead_code)]
    pub async fn update_sync_settings(
        &self,
        request: &crate::models::sync::UpdateSyncSettingsRequest,
    ) -> DatabaseResult<()> {
        use std::str::FromStr;

        println!("SQLite::update_sync_settings: Starting update");

        // Get current settings
        let mut settings = self
            .get_global_sync_settings()
            .await?
            .unwrap_or_else(|| crate::models::sync::SyncSettings::new());

        println!("SQLite::update_sync_settings: Current settings loaded");

        // Apply updates
        if let Some(is_active) = request.is_active {
            settings.is_active = is_active;
        }
        if let Some(auto_sync_enabled) = request.auto_sync_enabled {
            settings.auto_sync_enabled = auto_sync_enabled;
        }
        if let Some(sync_interval_minutes) = request.sync_interval_minutes {
            settings.sync_interval_minutes = sync_interval_minutes;
        }
        if let Some(ref conflict_strategy) = request.conflict_strategy {
            settings.conflict_strategy = ConflictResolutionStrategy::from_str(conflict_strategy)
                .map_err(crate::database::error::DatabaseError::QueryFailed)?;
        }
        if let Some(ref sync_direction) = request.sync_direction {
            settings.sync_direction =
                crate::models::sync::settings::SyncDirection::from_str(sync_direction)
                    .map_err(crate::database::error::DatabaseError::QueryFailed)?;
        }
        if let Some(ref selected_database_id) = request.selected_database_id {
            settings.selected_database_id = Some(selected_database_id.clone());
        }

        settings.touch();

        // Save updated settings
        self.save_global_sync_settings(&settings).await
    }

    #[allow(dead_code)]
    fn map_sync_settings_row(
        &self,
        row: &sqlx::sqlite::SqliteRow,
    ) -> DatabaseResult<crate::models::sync::SyncSettings> {
        use std::str::FromStr;

        let id: String = row
            .try_get("id")
            .map_err(|e| crate::database::error::DatabaseError::QueryFailed(e.to_string()))?;
        let is_active: bool = row
            .try_get("is_active")
            .map_err(|e| crate::database::error::DatabaseError::QueryFailed(e.to_string()))?;
        let auto_sync_enabled: bool = row
            .try_get("auto_sync_enabled")
            .map_err(|e| crate::database::error::DatabaseError::QueryFailed(e.to_string()))?;
        let sync_interval_minutes: i64 = row
            .try_get("sync_interval_minutes")
            .map_err(|e| crate::database::error::DatabaseError::QueryFailed(e.to_string()))?;
        let conflict_strategy_str: String = row
            .try_get("conflict_strategy")
            .map_err(|e| crate::database::error::DatabaseError::QueryFailed(e.to_string()))?;
        let sync_direction_str: String = row
            .try_get("sync_direction")
            .map_err(|e| crate::database::error::DatabaseError::QueryFailed(e.to_string()))?;
        let selected_database_id: Option<String> = row
            .try_get("selected_database_id")
            .map_err(|e| crate::database::error::DatabaseError::QueryFailed(e.to_string()))?;
        let last_sync_at: Option<String> = row
            .try_get("last_sync_at")
            .map_err(|e| crate::database::error::DatabaseError::QueryFailed(e.to_string()))?;
        let created_at: String = row
            .try_get("created_at")
            .map_err(|e| crate::database::error::DatabaseError::QueryFailed(e.to_string()))?;
        let updated_at: String = row
            .try_get("updated_at")
            .map_err(|e| crate::database::error::DatabaseError::QueryFailed(e.to_string()))?;

        let conflict_strategy = ConflictResolutionStrategy::from_str(&conflict_strategy_str)
            .map_err(crate::database::error::DatabaseError::QueryFailed)?;

        let sync_direction = crate::models::sync::settings::SyncDirection::from_str(&sync_direction_str)
            .map_err(crate::database::error::DatabaseError::QueryFailed)?;

        let last_sync_at_parsed = if let Some(dt_str) = last_sync_at {
            Some(
                DateTime::parse_from_rfc3339(&dt_str)
                    .map_err(|e| crate::database::error::DatabaseError::QueryFailed(e.to_string()))?
                    .with_timezone(&Utc),
            )
        } else {
            None
        };

        Ok(crate::models::sync::SyncSettings {
            id,
            is_active,
            auto_sync_enabled,
            sync_interval_minutes: sync_interval_minutes as u32,
            conflict_strategy,
            sync_direction,
            selected_database_id,
            last_sync_at: last_sync_at_parsed,
            created_at: DateTime::parse_from_rfc3339(&created_at)
                .map_err(|e| crate::database::error::DatabaseError::QueryFailed(e.to_string()))?
                .with_timezone(&Utc),
            updated_at: DateTime::parse_from_rfc3339(&updated_at)
                .map_err(|e| crate::database::error::DatabaseError::QueryFailed(e.to_string()))?
                .with_timezone(&Utc),
        })
    }

    /// Save conflict resolution for manual resolution
    pub async fn save_conflict_resolution(
        &self,
        resolution: &crate::models::sync::conflict::ConflictResolution,
    ) -> DatabaseResult<()> {
        let pool = self.get_pool()?;
        let pool = pool.read().await;

        sqlx::query(
            r#"
            INSERT INTO conflict_resolutions (
                id, entity_type, entity_id, local_data, remote_data,
                resolution_strategy, resolved_at, created_at
            ) VALUES (?, ?, ?, ?, ?, ?, ?, ?)
            "#,
        )
        .bind(&resolution.id)
        .bind(&resolution.entity_type)
        .bind(&resolution.entity_id)
        .bind(resolution.local_data.to_string())
        .bind(resolution.remote_data.to_string())
        .bind(resolution.resolution_strategy.as_ref().map(|s| s.to_string()))
        .bind(resolution.resolved_at.map(|dt| dt.to_rfc3339()))
        .bind(resolution.created_at.to_rfc3339())
        .execute(&*pool)
        .await
        .map_err(|e| DatabaseError::QueryFailed(e.to_string()))?;

        Ok(())
    }

    /// Get all unresolved conflict resolutions
    pub async fn get_unresolved_conflict_resolutions(
        &self,
    ) -> DatabaseResult<Vec<crate::models::sync::conflict::ConflictResolution>> {
        let pool = self.get_pool()?;
        let pool = pool.read().await;

        let rows = sqlx::query(
            r#"
            SELECT id, entity_type, entity_id, local_data, remote_data,
                   resolution_strategy, resolved_at, created_at
            FROM conflict_resolutions
            WHERE resolved_at IS NULL
            ORDER BY created_at DESC
            "#,
        )
        .fetch_all(&*pool)
        .await
        .map_err(|e| DatabaseError::QueryFailed(e.to_string()))?;

        let mut resolutions = Vec::new();
        for row in rows {
            let id: String = row.get("id");
            let entity_type: String = row.get("entity_type");
            let entity_id: String = row.get("entity_id");
            let local_data: String = row.get("local_data");
            let remote_data: String = row.get("remote_data");
            let resolution_strategy: Option<String> = row.get("resolution_strategy");
            let resolved_at: Option<String> = row.get("resolved_at");
            let created_at: String = row.get("created_at");

            resolutions.push(crate::models::sync::conflict::ConflictResolution {
                id,
                entity_type,
                entity_id,
                local_data: serde_json::from_str(&local_data)
                    .map_err(|e| DatabaseError::SerializationError(e))?,
                remote_data: serde_json::from_str(&remote_data)
                    .map_err(|e| DatabaseError::SerializationError(e))?,
                resolution_strategy: resolution_strategy
                    .and_then(|s| s.parse::<ConflictResolutionStrategy>().ok()),
                resolved_at: resolved_at
                    .and_then(|s| DateTime::parse_from_rfc3339(&s).ok())
                    .map(|dt| dt.with_timezone(&Utc)),
                created_at: DateTime::parse_from_rfc3339(&created_at)
                    .map_err(|e| DatabaseError::QueryFailed(e.to_string()))?
                    .with_timezone(&Utc),
            });
        }

        Ok(resolutions)
    }

    /// Resolve a conflict resolution
    pub async fn resolve_conflict_resolution(
        &self,
        id: &str,
        strategy: ConflictResolutionStrategy,
    ) -> DatabaseResult<()> {
        let pool = self.get_pool()?;
        let pool = pool.read().await;

        sqlx::query(
            r#"
            UPDATE conflict_resolutions
            SET resolution_strategy = ?, resolved_at = ?
            WHERE id = ?
            "#,
        )
        .bind(strategy.to_string())
        .bind(Utc::now().to_rfc3339())
        .bind(id)
        .execute(&*pool)
        .await
        .map_err(|e| DatabaseError::QueryFailed(e.to_string()))?;

        Ok(())
    }

    /// Delete resolved conflict resolutions older than specified days
    pub async fn cleanup_resolved_conflicts(&self, days: i64) -> DatabaseResult<usize> {
        let pool = self.get_pool()?;
        let pool = pool.read().await;

        let cutoff = Utc::now() - chrono::Duration::days(days);
        let result = sqlx::query(
            r#"
            DELETE FROM conflict_resolutions
            WHERE resolved_at IS NOT NULL AND resolved_at < ?
            "#,
        )
        .bind(cutoff.to_rfc3339())
        .execute(&*pool)
        .await
        .map_err(|e| DatabaseError::QueryFailed(e.to_string()))?;

        Ok(result.rows_affected() as usize)
    }
}
