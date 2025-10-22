mod sync;

use async_trait::async_trait;
use bson::Document;
use chrono::{DateTime, Utc};
use mongodb::{options::ClientOptions, Client, Collection, Database as MongoDatabase};
use serde_json::Value;
use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::RwLock;

use crate::database::{
    error::{DatabaseError, DatabaseResult},
    traits_sync::SyncTarget,
};

pub struct MongoDBProvider {
    connection_string: String,
    database_name: String,
    client: Option<Arc<RwLock<Client>>>,
    database: Option<Arc<RwLock<MongoDatabase>>>,
}

impl MongoDBProvider {
    pub fn new(connection_string: String, database_name: String) -> Self {
        Self {
            connection_string,
            database_name,
            client: None,
            database: None,
        }
    }

    pub(crate) fn get_database(&self) -> DatabaseResult<&Arc<RwLock<MongoDatabase>>> {
        self.database.as_ref()
            .ok_or_else(|| DatabaseError::ConnectionFailed("Database not connected".to_string()))
    }

    pub(crate) async fn get_collection(&self, name: &str) -> DatabaseResult<Collection<Document>> {
        let db_arc = self.get_database()?;
        let db = db_arc.read().await;
        Ok(db.collection::<Document>(name))
    }

    async fn create_sync_collections(&self) -> DatabaseResult<()> {
        let db_arc = self.get_database()?;
        let db = db_arc.read().await;

        let collections = vec![
            "ssh_profiles", "ssh_groups", "ssh_keys",
            "ssh_tunnels", "saved_commands", "saved_command_groups",
        ];

        for collection_name in collections {
            let existing = db.list_collection_names(None).await
                .map_err(|e| DatabaseError::QueryFailed(e.to_string()))?;

            if !existing.contains(&collection_name.to_string()) {
                db.create_collection(collection_name, None).await
                    .map_err(|e| DatabaseError::QueryFailed(e.to_string()))?;
            }
        }
        Ok(())
    }
}

#[async_trait]
impl SyncTarget for MongoDBProvider {
    async fn connect(&mut self) -> DatabaseResult<()> {
        let mut client_options = ClientOptions::parse(&self.connection_string).await
            .map_err(|e| DatabaseError::ConnectionFailed(e.to_string()))?;

        client_options.app_name = Some("Kerminal".to_string());

        let client = Client::with_options(client_options)
            .map_err(|e| DatabaseError::ConnectionFailed(e.to_string()))?;

        let database = client.database(&self.database_name);

        self.client = Some(Arc::new(RwLock::new(client)));
        self.database = Some(Arc::new(RwLock::new(database)));

        self.create_sync_collections().await?;
        Ok(())
    }

    async fn test_connection(&self) -> DatabaseResult<()> {
        let db_arc = self.get_database()?;
        let db = db_arc.read().await;

        db.list_collection_names(None).await
            .map_err(|e| DatabaseError::QueryFailed(e.to_string()))?;
        Ok(())
    }

    async fn push_records(&self, table: &str, records: Vec<Value>) -> DatabaseResult<usize> {
        sync::push_records(self, table, records).await
    }

    async fn pull_records(&self, table: &str, since: Option<DateTime<Utc>>) -> DatabaseResult<Vec<Value>> {
        sync::pull_records(self, table, since).await
    }

    async fn get_record_versions(&self, table: &str, ids: Vec<String>) -> DatabaseResult<HashMap<String, u64>> {
        sync::get_record_versions(self, table, ids).await
    }
}
