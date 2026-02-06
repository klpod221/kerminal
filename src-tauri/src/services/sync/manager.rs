use log::warn;
use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::{Mutex, RwLock};

use crate::database::{
    error::{DatabaseError, DatabaseResult},
    providers::{MongoDBProvider, MySQLProvider, PostgreSQLProvider},
    service::DatabaseService,
    traits_sync::SyncTarget,
};
use crate::models::sync::external_db::{DatabaseType, ExternalDatabaseConfig};

/// Manager for external database connections
pub struct SyncManager {
    database_service: Arc<Mutex<DatabaseService>>,
    active_connections: Arc<RwLock<HashMap<String, Arc<dyn SyncTarget>>>>,
}

impl SyncManager {
    pub fn new(database_service: Arc<Mutex<DatabaseService>>) -> Self {
        Self {
            database_service,
            active_connections: Arc::new(RwLock::new(HashMap::new())),
        }
    }

    /// Connect to an external database
    pub async fn connect(&self, config: &ExternalDatabaseConfig) -> DatabaseResult<()> {
        let connection_string = self.get_decrypted_connection_string(config).await?;

        let provider: Box<dyn SyncTarget> = match config.db_type {
            DatabaseType::MySQL => {
                let mut provider = MySQLProvider::new(connection_string);
                provider.connect().await?;
                Box::new(provider)
            }
            DatabaseType::PostgreSQL => {
                let mut provider = PostgreSQLProvider::new(connection_string);
                provider.connect().await?;
                Box::new(provider)
            }
            DatabaseType::MongoDB => {
                let connection_details = self.decrypt_connection_details(config).await?;
                let database_name = connection_details.database_name.clone();
                let mut provider = MongoDBProvider::new(connection_string, database_name);
                provider.connect().await?;
                Box::new(provider)
            }
        };

        provider.test_connection().await?;

        let mut connections = self.active_connections.write().await;
        connections.insert(config.base.id.clone(), Arc::from(provider));

        {
            let db_service = self.database_service.lock().await;
            let local_db = db_service.get_local_database();
            let local_db_guard = local_db.read().await;

            let update_request = crate::models::sync::settings::UpdateSyncSettingsRequest {
                is_active: Some(true),
                auto_sync_enabled: None,
                sync_interval_minutes: None,
                conflict_strategy: None,
                sync_direction: None,
                selected_database_id: None,
            };

            if let Err(e) = local_db_guard.update_sync_settings(&update_request).await {
                warn!(
                    "[SyncManager] Warning: Failed to update sync settings after connect: {}",
                    e
                );
            }
        }

        Ok(())
    }

    /// Disconnect from an external database
    pub async fn disconnect(&self, database_id: &str) -> DatabaseResult<()> {
        let mut connections = self.active_connections.write().await;
        connections.remove(database_id);
        drop(connections); // Release lock early

        {
            let db_service = self.database_service.lock().await;
            let local_db = db_service.get_local_database();
            let local_db_guard = local_db.read().await;

            let update_request = crate::models::sync::settings::UpdateSyncSettingsRequest {
                is_active: Some(false),
                auto_sync_enabled: None,
                sync_interval_minutes: None,
                conflict_strategy: None,
                sync_direction: None,
                selected_database_id: None,
            };

            if let Err(e) = local_db_guard.update_sync_settings(&update_request).await {
                warn!(
                    "[SyncManager] Warning: Failed to update sync settings after disconnect: {}",
                    e
                );
            }
        }

        Ok(())
    }

    /// Disconnect all active connections
    #[allow(dead_code)]
    pub async fn disconnect_all(&self) -> DatabaseResult<()> {
        let mut connections = self.active_connections.write().await;
        connections.clear();
        Ok(())
    }

    /// Get a connected provider
    pub async fn get_provider(&self, database_id: &str) -> DatabaseResult<Arc<dyn SyncTarget>> {
        let connections = self.active_connections.read().await;

        connections.get(database_id).cloned().ok_or_else(|| {
            DatabaseError::ConnectionFailed(format!(
                "No active connection for database: {}",
                database_id
            ))
        })
    }

    /// Check if a database is connected
    pub async fn is_connected(&self, database_id: &str) -> bool {
        let connections = self.active_connections.read().await;
        connections.contains_key(database_id)
    }

    /// Ensure connection exists, reconnect if needed
    pub async fn ensure_connection(&self, config: &ExternalDatabaseConfig) -> DatabaseResult<()> {
        if !self.is_connected(&config.base.id).await {
            self.connect(config).await?;
        }
        Ok(())
    }

    /// Get decrypted connection string
    async fn get_decrypted_connection_string(
        &self,
        config: &ExternalDatabaseConfig,
    ) -> DatabaseResult<String> {
        let db_service = self.database_service.lock().await;
        let manager = db_service.get_master_password_manager_arc();

        let encryptor =
            crate::database::encryption::external_db::ExternalDbEncryptor::new(manager.clone());

        let connection_details = encryptor
            .decrypt_connection_details(&config.connection_details_encrypted)
            .await?;

        Ok(connection_details.to_connection_string(&config.db_type))
    }

    /// Decrypt connection details
    async fn decrypt_connection_details(
        &self,
        config: &ExternalDatabaseConfig,
    ) -> DatabaseResult<crate::models::sync::external_db::ConnectionDetails> {
        let db_service = self.database_service.lock().await;
        let manager = db_service.get_master_password_manager_arc();

        let encryptor =
            crate::database::encryption::external_db::ExternalDbEncryptor::new(manager.clone());

        encryptor
            .decrypt_connection_details(&config.connection_details_encrypted)
            .await
    }

    /// Get connection statistics
    pub async fn get_connection_stats(&self) -> ConnectionStats {
        let connections = self.active_connections.read().await;
        ConnectionStats {
            total_connections: connections.len(),
        }
    }

    /// Perform health check on all active connections
    /// Returns health status for each connection
    #[allow(dead_code)]
    pub async fn health_check_all(&self) -> Vec<HealthCheckResult> {
        let connections = self.active_connections.read().await;
        let mut results = Vec::new();

        for (database_id, provider) in connections.iter() {
            let start_time = std::time::Instant::now();
            let status = match provider.test_connection().await {
                Ok(_) => HealthStatus::Healthy,
                Err(e) => HealthStatus::Unhealthy(e.to_string()),
            };
            let latency_ms = start_time.elapsed().as_millis() as u64;

            results.push(HealthCheckResult {
                database_id: database_id.clone(),
                status,
                latency_ms,
                checked_at: chrono::Utc::now(),
            });
        }

        results
    }

    /// Perform health check on a specific connection
    #[allow(dead_code)]
    pub async fn health_check(&self, database_id: &str) -> Option<HealthCheckResult> {
        let connections = self.active_connections.read().await;

        if let Some(provider) = connections.get(database_id) {
            let start_time = std::time::Instant::now();
            let status = match provider.test_connection().await {
                Ok(_) => HealthStatus::Healthy,
                Err(e) => HealthStatus::Unhealthy(e.to_string()),
            };
            let latency_ms = start_time.elapsed().as_millis() as u64;

            Some(HealthCheckResult {
                database_id: database_id.to_string(),
                status,
                latency_ms,
                checked_at: chrono::Utc::now(),
            })
        } else {
            None
        }
    }
}

/// Connection statistics
#[derive(Debug, Clone)]
pub struct ConnectionStats {
    pub total_connections: usize,
}

/// Health check result for a single connection
#[allow(dead_code)]
#[derive(Debug, Clone, serde::Serialize)]
#[serde(rename_all = "camelCase")]
pub struct HealthCheckResult {
    pub database_id: String,
    pub status: HealthStatus,
    pub latency_ms: u64,
    pub checked_at: chrono::DateTime<chrono::Utc>,
}

/// Health status of a connection
#[allow(dead_code)]
#[derive(Debug, Clone, serde::Serialize)]
#[serde(rename_all = "camelCase")]
pub enum HealthStatus {
    Healthy,
    Unhealthy(String),
}

impl Drop for SyncManager {
    fn drop(&mut self) {}
}

#[cfg(test)]
mod tests {
    #[tokio::test]
    async fn test_sync_manager_creation() {}
}
