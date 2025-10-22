
use chrono::{DateTime, Utc};
use sqlx::Row;

use crate::{
    database::{error::DatabaseResult, traits::SyncStatus},
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
                id, name, db_type, connection_details_encrypted, sync_settings,
                is_active, auto_sync_enabled, last_sync_at, created_at, updated_at, device_id, version, sync_status
            ) VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10, ?11, ?12, ?13)
            ON CONFLICT(id) DO UPDATE SET
                name = excluded.name,
                db_type = excluded.db_type,
                connection_details_encrypted = excluded.connection_details_encrypted,
                sync_settings = excluded.sync_settings,
                is_active = excluded.is_active,
                auto_sync_enabled = excluded.auto_sync_enabled,
                last_sync_at = excluded.last_sync_at,
                updated_at = excluded.updated_at,
                version = excluded.version,
                sync_status = excluded.sync_status
        "#,
        )
        .bind(&config.base.id)
        .bind(&config.name)
        .bind(config.db_type.to_string())
        .bind(&config.connection_details_encrypted)
        .bind(&config.sync_settings)
        .bind(config.is_active)
        .bind(config.auto_sync_enabled)
        .bind(config.last_sync_at)
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
            SELECT id, name, db_type, connection_details_encrypted, sync_settings,
                is_active, last_sync_at, created_at, updated_at, device_id, version, sync_status
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
            SELECT id, name, db_type, connection_details_encrypted, sync_settings,
                is_active, last_sync_at, created_at, updated_at, device_id, version, sync_status
            FROM external_databases
            ORDER BY created_at DESC
        "#,
        )
        .fetch_all(&*pool)
        .await
        .map_err(|e| crate::database::error::DatabaseError::QueryFailed(e.to_string()))?;

        let mut configs = Vec::new();
        for row in rows {
            configs.push(self.map_external_database_row(&row)?);
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
        let sync_settings: String = row
            .try_get("sync_settings")
            .map_err(|e| crate::database::error::DatabaseError::QueryFailed(e.to_string()))?;
        let is_active: bool = row
            .try_get("is_active")
            .map_err(|e| crate::database::error::DatabaseError::QueryFailed(e.to_string()))?;
        let auto_sync_enabled: bool = row
            .try_get("auto_sync_enabled")
            .unwrap_or(false); // Default to false for backward compatibility
        let last_sync_at: Option<String> = row
            .try_get("last_sync_at")
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

        let last_sync_at_parsed = if let Some(dt_str) = last_sync_at {
            Some(
                DateTime::parse_from_rfc3339(&dt_str)
                    .map_err(|e| crate::database::error::DatabaseError::QueryFailed(e.to_string()))?
                    .with_timezone(&Utc),
            )
        } else {
            None
        };

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
            sync_settings,
            is_active,
            auto_sync_enabled,
            last_sync_at: last_sync_at_parsed,
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
}
