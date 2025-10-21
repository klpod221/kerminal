
use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::{Mutex, RwLock};

use crate::database::{
    error::{DatabaseError, DatabaseResult},
    providers::{MongoDBProvider, MySQLProvider, PostgreSQLProvider},
    service::DatabaseService,
    traits::Database,
};
use crate::models::sync::external_db::{DatabaseType, ExternalDatabaseConfig};

/// Manager for external database connections
pub struct SyncManager {
    database_service: Arc<Mutex<DatabaseService>>,
    active_connections: Arc<RwLock<HashMap<String, Arc<dyn Database>>>>,
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
        println!("SyncManager::connect: Starting connection for database: {}", config.base.id);
        println!("SyncManager::connect: Decrypting connection string...");

        let connection_string = self.get_decrypted_connection_string(config).await?;
        println!("SyncManager::connect: Connection string decrypted successfully");

        let provider: Box<dyn Database> = match config.db_type {
            DatabaseType::MySQL => {
                println!("SyncManager::connect: Creating MySQL provider");
                let mut provider = MySQLProvider::new(connection_string);
                provider.connect().await?;
                Box::new(provider)
            }
            DatabaseType::PostgreSQL => {
                println!("SyncManager::connect: Creating PostgreSQL provider");
                let mut provider = PostgreSQLProvider::new(connection_string);
                provider.connect().await?;
                Box::new(provider)
            }
            DatabaseType::MongoDB => {
                println!("SyncManager::connect: Creating MongoDB provider");
                let connection_details = self.decrypt_connection_details(config).await?;
                let database_name = connection_details.database.clone();
                let mut provider = MongoDBProvider::new(connection_string, database_name);
                provider.connect().await?;
                Box::new(provider)
            }
        };

        println!("SyncManager::connect: Testing connection...");
        // Test connection
        provider.test_connection().await?;
        println!("SyncManager::connect: Connection test successful");

        println!("SyncManager::connect: Storing active connection...");
        // Store in active connections
        let mut connections = self.active_connections.write().await;
        connections.insert(config.base.id.clone(), Arc::from(provider));
        println!("SyncManager::connect: Connection stored successfully");

        Ok(())
    }

    /// Disconnect from an external database
    pub async fn disconnect(&self, database_id: &str) -> DatabaseResult<()> {
        let mut connections = self.active_connections.write().await;
        connections.remove(database_id);
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
    pub async fn get_provider(&self, database_id: &str) -> DatabaseResult<Arc<dyn Database>> {
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
}

/// Connection statistics
#[derive(Debug, Clone)]
pub struct ConnectionStats {
    pub total_connections: usize,
}

impl Drop for SyncManager {
    fn drop(&mut self) {
        // Note: Can't use async in Drop, connections will be cleaned up when Arc is dropped
    }
}

#[cfg(test)]
mod tests {
    #[tokio::test]
    async fn test_sync_manager_creation() {
        // Test implementation requires mock DatabaseService
    }
}
