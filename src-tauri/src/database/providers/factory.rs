use crate::database::{
    traits::Database,
    providers::{sqlite::SQLiteProvider},
    config::{DatabaseConfig, DatabaseProvider, ConnectionConfig},
    error::{DatabaseError, DatabaseResult},
};

/// Database factory để create appropriate provider
#[allow(dead_code)]
pub struct DatabaseFactory;

#[allow(dead_code)]
impl DatabaseFactory {
    /// Create database provider based on configuration
    pub fn create_provider(config: &DatabaseConfig) -> DatabaseResult<Box<dyn Database>> {
        match &config.provider {
            DatabaseProvider::SQLite => {
                if let ConnectionConfig::SQLite { file_path } = &config.connection {
                    Ok(Box::new(SQLiteProvider::new(file_path.clone())))
                } else {
                    Err(DatabaseError::ConfigError("Invalid SQLite configuration".to_string()))
                }
            },
            DatabaseProvider::MySQL => {
                Err(DatabaseError::UnsupportedProvider("MySQL support not implemented yet".to_string()))
            },
            DatabaseProvider::PostgreSQL => {
                Err(DatabaseError::UnsupportedProvider("PostgreSQL support not implemented yet".to_string()))
            },
            DatabaseProvider::MongoDB => {
                Err(DatabaseError::UnsupportedProvider("MongoDB support not implemented yet".to_string()))
            },
        }
    }

    /// Create và connect database provider
    pub async fn create_and_connect(config: &DatabaseConfig) -> DatabaseResult<Box<dyn Database>> {
        let mut provider = Self::create_provider(config)?;
        provider.connect().await?;
        Ok(provider)
    }

    /// Test database connection với configuration
    pub async fn test_config(config: &DatabaseConfig) -> DatabaseResult<()> {
        let mut provider = Self::create_provider(config)?;
        provider.connect().await?;
        provider.test_connection().await?;
        provider.disconnect().await?;
        Ok(())
    }

    /// Get supported providers
    pub fn supported_providers() -> Vec<DatabaseProvider> {
        vec![
            DatabaseProvider::SQLite,
            // Note: Other providers will be added when implemented
        ]
    }

    /// Check if provider is supported
    pub fn is_provider_supported(provider: &DatabaseProvider) -> bool {
        matches!(provider, DatabaseProvider::SQLite)
    }
}
