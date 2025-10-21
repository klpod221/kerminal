#![allow(dead_code)]

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
                is_active, last_sync_at, created_at, updated_at, device_id, version, sync_status
            ) VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10, ?11, ?12)
            ON CONFLICT(id) DO UPDATE SET
                name = excluded.name,
                db_type = excluded.db_type,
                connection_details_encrypted = excluded.connection_details_encrypted,
                sync_settings = excluded.sync_settings,
                is_active = excluded.is_active,
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
        .bind(&config.last_sync_at)
        .bind(&config.base.created_at)
        .bind(&config.base.updated_at)
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

    pub async fn update_external_database_last_sync(
        &self,
        id: &str,
        last_sync: DateTime<Utc>,
    ) -> DatabaseResult<()> {
        let pool = self.get_pool()?;
        let pool = pool.read().await;

        sqlx::query(
            r#"
            UPDATE external_databases
            SET last_sync_at = ?1, updated_at = ?2
            WHERE id = ?3
        "#,
        )
        .bind(&last_sync)
        .bind(&Utc::now())
        .bind(id)
        .execute(&*pool)
        .await
        .map_err(|e| crate::database::error::DatabaseError::QueryFailed(e.to_string()))?;

        Ok(())
    }

    pub async fn toggle_external_database_active(
        &self,
        id: &str,
        is_active: bool,
    ) -> DatabaseResult<()> {
        let pool = self.get_pool()?;
        let pool = pool.read().await;

        sqlx::query(
            r#"
            UPDATE external_databases
            SET is_active = ?1, updated_at = ?2
            WHERE id = ?3
        "#,
        )
        .bind(is_active)
        .bind(&Utc::now())
        .bind(id)
        .execute(&*pool)
        .await
        .map_err(|e| crate::database::error::DatabaseError::QueryFailed(e.to_string()))?;

        Ok(())
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

    pub async fn save_sync_operation(&self, operation: &SyncOperation) -> DatabaseResult<()> {
        let pool = self.get_pool()?;
        let pool = pool.read().await;

        sqlx::query(
            r#"
            INSERT INTO sync_operations (
                id, operation_type, entity_type, entity_id, source_db, target_db,
                status, error_message, started_at, completed_at
            ) VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10)
            ON CONFLICT(id) DO UPDATE SET
                status = excluded.status,
                error_message = excluded.error_message,
                completed_at = excluded.completed_at
        "#,
        )
        .bind(&operation.id)
        .bind(operation.operation_type.to_string())
        .bind(&operation.entity_type)
        .bind(&operation.entity_id)
        .bind(&operation.source_db)
        .bind(&operation.target_db)
        .bind(operation.status.to_string())
        .bind(&operation.error_message)
        .bind(&operation.started_at)
        .bind(&operation.completed_at)
        .execute(&*pool)
        .await
        .map_err(|e| crate::database::error::DatabaseError::QueryFailed(e.to_string()))?;

        Ok(())
    }

    pub async fn find_sync_operations_by_entity(
        &self,
        entity_type: &str,
        entity_id: &str,
    ) -> DatabaseResult<Vec<SyncOperation>> {
        let pool = self.get_pool()?;
        let pool = pool.read().await;

        let rows = sqlx::query(
            r#"
            SELECT id, operation_type, entity_type, entity_id, source_db, target_db,
                status, error_message, started_at, completed_at
            FROM sync_operations
            WHERE entity_type = ?1 AND entity_id = ?2
            ORDER BY started_at DESC
        "#,
        )
        .bind(entity_type)
        .bind(entity_id)
        .fetch_all(&*pool)
        .await
        .map_err(|e| crate::database::error::DatabaseError::QueryFailed(e.to_string()))?;

        let mut operations = Vec::new();
        for row in rows {
            operations.push(self.map_sync_operation_row(&row)?);
        }

        Ok(operations)
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

    pub async fn save_sync_conflict(&self, conflict: &SyncConflict) -> DatabaseResult<()> {
        let pool = self.get_pool()?;
        let pool = pool.read().await;

        let resolution_strategy = conflict.resolution_strategy.as_ref().map(|s| s.to_string());

        sqlx::query(
            r#"
            INSERT INTO sync_conflicts (
                id, entity_type, entity_id, local_version, remote_version,
                local_data, remote_data, resolution_strategy, resolved, created_at, resolved_at
            ) VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10, ?11)
            ON CONFLICT(id) DO UPDATE SET
                resolution_strategy = excluded.resolution_strategy,
                resolved = excluded.resolved,
                resolved_at = excluded.resolved_at
        "#,
        )
        .bind(&conflict.id)
        .bind(&conflict.entity_type)
        .bind(&conflict.entity_id)
        .bind(conflict.local_version as i64)
        .bind(conflict.remote_version as i64)
        .bind(&conflict.local_data)
        .bind(&conflict.remote_data)
        .bind(&resolution_strategy)
        .bind(conflict.resolved)
        .bind(&conflict.created_at)
        .bind(&conflict.resolved_at)
        .execute(&*pool)
        .await
        .map_err(|e| crate::database::error::DatabaseError::QueryFailed(e.to_string()))?;

        Ok(())
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

    pub async fn find_conflict_by_entity(
        &self,
        entity_type: &str,
        entity_id: &str,
    ) -> DatabaseResult<Option<SyncConflict>> {
        let pool = self.get_pool()?;
        let pool = pool.read().await;

        let row = sqlx::query(
            r#"
            SELECT id, entity_type, entity_id, local_version, remote_version,
                local_data, remote_data, resolution_strategy, resolved, created_at, resolved_at
            FROM sync_conflicts
            WHERE entity_type = ?1 AND entity_id = ?2 AND resolved = false
            ORDER BY created_at DESC
            LIMIT 1
        "#,
        )
        .bind(entity_type)
        .bind(entity_id)
        .fetch_optional(&*pool)
        .await
        .map_err(|e| crate::database::error::DatabaseError::QueryFailed(e.to_string()))?;

        if let Some(row) = row {
            let conflict = self.map_sync_conflict_row(&row)?;
            Ok(Some(conflict))
        } else {
            Ok(None)
        }
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
        .bind(&Utc::now())
        .bind(id)
        .execute(&*pool)
        .await
        .map_err(|e| crate::database::error::DatabaseError::QueryFailed(e.to_string()))?;

        Ok(())
    }

    pub async fn delete_conflict(&self, id: &str) -> DatabaseResult<()> {
        let pool = self.get_pool()?;
        let pool = pool.read().await;

        sqlx::query(
            r#"
            DELETE FROM sync_conflicts WHERE id = ?1
        "#,
        )
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
            .map_err(|e| crate::database::error::DatabaseError::QueryFailed(e))?;

        let sync_status = SyncStatus::from_str(&sync_status_str)
            .map_err(|e| crate::database::error::DatabaseError::QueryFailed(e))?;

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
            .map_err(|e| crate::database::error::DatabaseError::QueryFailed(e))?;

        let status = crate::models::sync::SyncOperationStatus::from_str(&status_str)
            .map_err(|e| crate::database::error::DatabaseError::QueryFailed(e))?;

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
                    .map_err(|e| crate::database::error::DatabaseError::QueryFailed(e))?,
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
