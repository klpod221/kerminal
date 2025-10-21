use async_trait::async_trait;
use bson::{doc, Bson, Document};
use mongodb::{
    options::{ClientOptions, FindOptions},
    Client, Collection, Database as MongoDatabase,
};
use std::sync::Arc;
use tokio::sync::RwLock;

use crate::database::{
    error::{DatabaseError, DatabaseResult},
    traits::{Database, DatabaseProviderType, SqlValue, ToSqlValue},
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

    fn get_database(&self) -> DatabaseResult<&Arc<RwLock<MongoDatabase>>> {
        self.database
            .as_ref()
            .ok_or_else(|| DatabaseError::ConnectionFailed("Database not connected".to_string()))
    }

    async fn get_collection(&self, name: &str) -> DatabaseResult<Collection<Document>> {
        let db_arc = self.get_database()?;
        let db = db_arc.read().await;
        Ok(db.collection::<Document>(name))
    }

    fn model_to_document<T: serde::Serialize>(model: &T) -> DatabaseResult<Document> {
        let bson = bson::to_bson(model)
            .map_err(|e| DatabaseError::ParseError(format!("BSON serialization failed: {}", e)))?;

        match bson {
            Bson::Document(doc) => Ok(doc),
            _ => Err(DatabaseError::ParseError(
                "Expected BSON Document".to_string(),
            )),
        }
    }

    fn document_to_model<T: serde::de::DeserializeOwned>(doc: Document) -> DatabaseResult<T> {
        bson::from_document(doc)
            .map_err(|e| DatabaseError::ParseError(format!("BSON deserialization failed: {}", e)))
    }
}

#[async_trait]
impl Database for MongoDBProvider {
    async fn connect(&mut self) -> DatabaseResult<()> {
        let mut client_options = ClientOptions::parse(&self.connection_string)
            .await
            .map_err(|e| DatabaseError::ConnectionFailed(e.to_string()))?;

        client_options.app_name = Some("Kerminal".to_string());

        let client = Client::with_options(client_options)
            .map_err(|e| DatabaseError::ConnectionFailed(e.to_string()))?;

        let database = client.database(&self.database_name);

        self.client = Some(Arc::new(RwLock::new(client)));
        self.database = Some(Arc::new(RwLock::new(database)));

        Ok(())
    }

    async fn disconnect(&mut self) -> DatabaseResult<()> {
        self.database = None;
        self.client = None;
        Ok(())
    }

    fn is_connected(&self) -> bool {
        self.database.is_some()
    }

    async fn test_connection(&self) -> DatabaseResult<()> {
        let db_arc = self.get_database()?;
        let db = db_arc.read().await;

        db.list_collection_names(None)
            .await
            .map_err(|e| DatabaseError::QueryFailed(e.to_string()))?;

        Ok(())
    }

    async fn execute_raw(&self, _query: &str, _params: &[&dyn ToSqlValue]) -> DatabaseResult<u64> {
        Err(DatabaseError::NotImplemented(
            "MongoDB execute_raw not applicable".to_string(),
        ))
    }

    async fn fetch_raw(
        &self,
        _query: &str,
        _params: &[&dyn ToSqlValue],
    ) -> DatabaseResult<Vec<std::collections::HashMap<String, SqlValue>>> {
        Err(DatabaseError::NotImplemented(
            "MongoDB fetch_raw not applicable".to_string(),
        ))
    }

    async fn save_ssh_profile(&self, model: &crate::models::ssh::SSHProfile) -> DatabaseResult<()> {
        let collection = self.get_collection("ssh_profiles").await?;
        let doc = Self::model_to_document(model)?;

        collection
            .insert_one(doc, None)
            .await
            .map_err(|e| DatabaseError::QueryFailed(e.to_string()))?;

        Ok(())
    }

    async fn find_ssh_profile_by_id(
        &self,
        id: &str,
    ) -> DatabaseResult<Option<crate::models::ssh::SSHProfile>> {
        let collection = self.get_collection("ssh_profiles").await?;
        let filter = doc! { "_id": id };

        let result = collection
            .find_one(filter, None)
            .await
            .map_err(|e| DatabaseError::QueryFailed(e.to_string()))?;

        match result {
            Some(doc) => Ok(Some(Self::document_to_model(doc)?)),
            None => Ok(None),
        }
    }

    async fn find_all_ssh_profiles(&self) -> DatabaseResult<Vec<crate::models::ssh::SSHProfile>> {
        let collection = self.get_collection("ssh_profiles").await?;
        let find_options = FindOptions::builder()
            .sort(doc! { "createdAt": -1 })
            .build();

        let mut cursor = collection
            .find(None, find_options)
            .await
            .map_err(|e| DatabaseError::QueryFailed(e.to_string()))?;

        let mut profiles = Vec::new();
        while cursor
            .advance()
            .await
            .map_err(|e| DatabaseError::QueryFailed(e.to_string()))?
        {
            let doc = cursor
                .deserialize_current()
                .map_err(|e| DatabaseError::ParseError(e.to_string()))?;
            profiles.push(Self::document_to_model(doc)?);
        }

        Ok(profiles)
    }

    async fn update_ssh_profile(
        &self,
        model: &crate::models::ssh::SSHProfile,
    ) -> DatabaseResult<()> {
        let collection = self.get_collection("ssh_profiles").await?;
        let filter = doc! { "_id": &model.base.id };
        let doc = Self::model_to_document(model)?;

        collection
            .replace_one(filter, doc, None)
            .await
            .map_err(|e| DatabaseError::QueryFailed(e.to_string()))?;

        Ok(())
    }

    async fn delete_ssh_profile(&self, id: &str) -> DatabaseResult<()> {
        let collection = self.get_collection("ssh_profiles").await?;
        let filter = doc! { "_id": id };

        collection
            .delete_one(filter, None)
            .await
            .map_err(|e| DatabaseError::QueryFailed(e.to_string()))?;

        Ok(())
    }

    async fn save_ssh_group(&self, model: &crate::models::ssh::SSHGroup) -> DatabaseResult<()> {
        let collection = self.get_collection("ssh_groups").await?;
        let doc = Self::model_to_document(model)?;

        collection
            .insert_one(doc, None)
            .await
            .map_err(|e| DatabaseError::QueryFailed(e.to_string()))?;

        Ok(())
    }

    async fn find_ssh_group_by_id(
        &self,
        id: &str,
    ) -> DatabaseResult<Option<crate::models::ssh::SSHGroup>> {
        let collection = self.get_collection("ssh_groups").await?;
        let filter = doc! { "_id": id };

        let result = collection
            .find_one(filter, None)
            .await
            .map_err(|e| DatabaseError::QueryFailed(e.to_string()))?;

        match result {
            Some(doc) => Ok(Some(Self::document_to_model(doc)?)),
            None => Ok(None),
        }
    }

    async fn find_all_ssh_groups(&self) -> DatabaseResult<Vec<crate::models::ssh::SSHGroup>> {
        let collection = self.get_collection("ssh_groups").await?;
        let find_options = FindOptions::builder().sort(doc! { "order": 1 }).build();

        let mut cursor = collection
            .find(None, find_options)
            .await
            .map_err(|e| DatabaseError::QueryFailed(e.to_string()))?;

        let mut groups = Vec::new();
        while cursor
            .advance()
            .await
            .map_err(|e| DatabaseError::QueryFailed(e.to_string()))?
        {
            let doc = cursor
                .deserialize_current()
                .map_err(|e| DatabaseError::ParseError(e.to_string()))?;
            groups.push(Self::document_to_model(doc)?);
        }

        Ok(groups)
    }

    async fn update_ssh_group(&self, model: &crate::models::ssh::SSHGroup) -> DatabaseResult<()> {
        let collection = self.get_collection("ssh_groups").await?;
        let filter = doc! { "_id": &model.base.id };
        let doc = Self::model_to_document(model)?;

        collection
            .replace_one(filter, doc, None)
            .await
            .map_err(|e| DatabaseError::QueryFailed(e.to_string()))?;

        Ok(())
    }

    async fn delete_ssh_group(&self, id: &str) -> DatabaseResult<()> {
        let collection = self.get_collection("ssh_groups").await?;
        let filter = doc! { "_id": id };

        collection
            .delete_one(filter, None)
            .await
            .map_err(|e| DatabaseError::QueryFailed(e.to_string()))?;

        Ok(())
    }

    async fn save_ssh_key(&self, model: &crate::models::ssh::SSHKey) -> DatabaseResult<()> {
        let collection = self.get_collection("ssh_keys").await?;
        let doc = Self::model_to_document(model)?;

        collection
            .insert_one(doc, None)
            .await
            .map_err(|e| DatabaseError::QueryFailed(e.to_string()))?;

        Ok(())
    }

    async fn find_ssh_key_by_id(
        &self,
        id: &str,
    ) -> DatabaseResult<Option<crate::models::ssh::SSHKey>> {
        let collection = self.get_collection("ssh_keys").await?;
        let filter = doc! { "_id": id };

        let result = collection
            .find_one(filter, None)
            .await
            .map_err(|e| DatabaseError::QueryFailed(e.to_string()))?;

        match result {
            Some(doc) => Ok(Some(Self::document_to_model(doc)?)),
            None => Ok(None),
        }
    }

    async fn find_all_ssh_keys(&self) -> DatabaseResult<Vec<crate::models::ssh::SSHKey>> {
        let collection = self.get_collection("ssh_keys").await?;
        let find_options = FindOptions::builder()
            .sort(doc! { "createdAt": -1 })
            .build();

        let mut cursor = collection
            .find(None, find_options)
            .await
            .map_err(|e| DatabaseError::QueryFailed(e.to_string()))?;

        let mut keys = Vec::new();
        while cursor
            .advance()
            .await
            .map_err(|e| DatabaseError::QueryFailed(e.to_string()))?
        {
            let doc = cursor
                .deserialize_current()
                .map_err(|e| DatabaseError::ParseError(e.to_string()))?;
            keys.push(Self::document_to_model(doc)?);
        }

        Ok(keys)
    }

    async fn update_ssh_key(&self, model: &crate::models::ssh::SSHKey) -> DatabaseResult<()> {
        let collection = self.get_collection("ssh_keys").await?;
        let filter = doc! { "_id": &model.base.id };
        let doc = Self::model_to_document(model)?;

        collection
            .replace_one(filter, doc, None)
            .await
            .map_err(|e| DatabaseError::QueryFailed(e.to_string()))?;

        Ok(())
    }

    async fn delete_ssh_key(&self, id: &str) -> DatabaseResult<()> {
        let collection = self.get_collection("ssh_keys").await?;
        let filter = doc! { "_id": id };

        collection
            .delete_one(filter, None)
            .await
            .map_err(|e| DatabaseError::QueryFailed(e.to_string()))?;

        Ok(())
    }

    async fn count_profiles_using_key(&self, key_id: &str) -> DatabaseResult<u32> {
        let collection = self.get_collection("ssh_profiles").await?;
        let filter = doc! { "authKeyId": key_id };

        let count = collection
            .count_documents(filter, None)
            .await
            .map_err(|e| DatabaseError::QueryFailed(e.to_string()))? as u32;

        Ok(count)
    }

    async fn save_ssh_tunnel(&self, model: &crate::models::ssh::SSHTunnel) -> DatabaseResult<()> {
        let collection = self.get_collection("ssh_tunnels").await?;
        let doc = Self::model_to_document(model)?;

        collection
            .insert_one(doc, None)
            .await
            .map_err(|e| DatabaseError::QueryFailed(e.to_string()))?;

        Ok(())
    }

    async fn find_ssh_tunnel_by_id(
        &self,
        id: &str,
    ) -> DatabaseResult<Option<crate::models::ssh::SSHTunnel>> {
        let collection = self.get_collection("ssh_tunnels").await?;
        let filter = doc! { "_id": id };

        let result = collection
            .find_one(filter, None)
            .await
            .map_err(|e| DatabaseError::QueryFailed(e.to_string()))?;

        match result {
            Some(doc) => Ok(Some(Self::document_to_model(doc)?)),
            None => Ok(None),
        }
    }

    async fn find_all_ssh_tunnels(&self) -> DatabaseResult<Vec<crate::models::ssh::SSHTunnel>> {
        let collection = self.get_collection("ssh_tunnels").await?;
        let find_options = FindOptions::builder()
            .sort(doc! { "createdAt": -1 })
            .build();

        let mut cursor = collection
            .find(None, find_options)
            .await
            .map_err(|e| DatabaseError::QueryFailed(e.to_string()))?;

        let mut tunnels = Vec::new();
        while cursor
            .advance()
            .await
            .map_err(|e| DatabaseError::QueryFailed(e.to_string()))?
        {
            let doc = cursor
                .deserialize_current()
                .map_err(|e| DatabaseError::ParseError(e.to_string()))?;
            tunnels.push(Self::document_to_model(doc)?);
        }

        Ok(tunnels)
    }

    async fn find_ssh_tunnels_by_profile_id(
        &self,
        profile_id: &str,
    ) -> DatabaseResult<Vec<crate::models::ssh::SSHTunnel>> {
        let collection = self.get_collection("ssh_tunnels").await?;
        let filter = doc! { "profileId": profile_id };
        let find_options = FindOptions::builder()
            .sort(doc! { "createdAt": -1 })
            .build();

        let mut cursor = collection
            .find(filter, find_options)
            .await
            .map_err(|e| DatabaseError::QueryFailed(e.to_string()))?;

        let mut tunnels = Vec::new();
        while cursor
            .advance()
            .await
            .map_err(|e| DatabaseError::QueryFailed(e.to_string()))?
        {
            let doc = cursor
                .deserialize_current()
                .map_err(|e| DatabaseError::ParseError(e.to_string()))?;
            tunnels.push(Self::document_to_model(doc)?);
        }

        Ok(tunnels)
    }

    async fn find_auto_start_ssh_tunnels(
        &self,
    ) -> DatabaseResult<Vec<crate::models::ssh::SSHTunnel>> {
        let collection = self.get_collection("ssh_tunnels").await?;
        let filter = doc! { "autoStart": true };
        let find_options = FindOptions::builder()
            .sort(doc! { "createdAt": -1 })
            .build();

        let mut cursor = collection
            .find(filter, find_options)
            .await
            .map_err(|e| DatabaseError::QueryFailed(e.to_string()))?;

        let mut tunnels = Vec::new();
        while cursor
            .advance()
            .await
            .map_err(|e| DatabaseError::QueryFailed(e.to_string()))?
        {
            let doc = cursor
                .deserialize_current()
                .map_err(|e| DatabaseError::ParseError(e.to_string()))?;
            tunnels.push(Self::document_to_model(doc)?);
        }

        Ok(tunnels)
    }

    async fn update_ssh_tunnel(&self, model: &crate::models::ssh::SSHTunnel) -> DatabaseResult<()> {
        let collection = self.get_collection("ssh_tunnels").await?;
        let filter = doc! { "_id": &model.base.id };
        let doc = Self::model_to_document(model)?;

        collection
            .replace_one(filter, doc, None)
            .await
            .map_err(|e| DatabaseError::QueryFailed(e.to_string()))?;

        Ok(())
    }

    async fn delete_ssh_tunnel(&self, id: &str) -> DatabaseResult<()> {
        let collection = self.get_collection("ssh_tunnels").await?;
        let filter = doc! { "_id": id };

        collection
            .delete_one(filter, None)
            .await
            .map_err(|e| DatabaseError::QueryFailed(e.to_string()))?;

        Ok(())
    }

    async fn delete_ssh_tunnels_by_profile_id(&self, profile_id: &str) -> DatabaseResult<()> {
        let collection = self.get_collection("ssh_tunnels").await?;
        let filter = doc! { "profileId": profile_id };

        collection
            .delete_many(filter, None)
            .await
            .map_err(|e| DatabaseError::QueryFailed(e.to_string()))?;

        Ok(())
    }

    async fn create_tables(&self) -> DatabaseResult<()> {
        let db_arc = self.get_database()?;
        let db = db_arc.read().await;

        let collections = vec![
            "ssh_profiles",
            "ssh_groups",
            "ssh_keys",
            "ssh_tunnels",
            "saved_commands",
            "saved_command_groups",
            "master_passwords",
            "devices",
        ];

        for collection_name in collections {
            let _ = db.create_collection(collection_name, None).await.ok();
        }

        Ok(())
    }

    async fn drop_tables(&self) -> DatabaseResult<()> {
        let db_arc = self.get_database()?;
        let db = db_arc.read().await;

        let collections = vec![
            "ssh_profiles",
            "ssh_groups",
            "ssh_keys",
            "ssh_tunnels",
            "saved_commands",
            "saved_command_groups",
            "master_passwords",
            "devices",
        ];

        for collection_name in collections {
            let _ = db
                .collection::<Document>(collection_name)
                .drop(None)
                .await
                .ok();
        }

        Ok(())
    }

    async fn migrate(&self, _version: u32) -> DatabaseResult<()> {
        Ok(())
    }

    fn provider_type(&self) -> DatabaseProviderType {
        DatabaseProviderType::MongoDB
    }

    fn connection_info(&self) -> String {
        format!("mongodb://{}", self.database_name)
    }

    async fn save_saved_command(
        &self,
        model: &crate::models::saved_command::SavedCommand,
    ) -> DatabaseResult<()> {
        let collection = self.get_collection("saved_commands").await?;
        let doc = Self::model_to_document(model)?;

        let filter = doc! { "_id": &model.base.id };
        collection
            .replace_one(filter, doc, None)
            .await
            .map_err(|e| DatabaseError::QueryFailed(e.to_string()))?;

        Ok(())
    }

    async fn find_saved_command_by_id(
        &self,
        id: &str,
    ) -> DatabaseResult<Option<crate::models::saved_command::SavedCommand>> {
        let collection = self.get_collection("saved_commands").await?;
        let filter = doc! { "_id": id };

        let result = collection
            .find_one(filter, None)
            .await
            .map_err(|e| DatabaseError::QueryFailed(e.to_string()))?;

        match result {
            Some(doc) => Ok(Some(Self::document_to_model(doc)?)),
            None => Ok(None),
        }
    }

    async fn find_all_saved_commands(
        &self,
    ) -> DatabaseResult<Vec<crate::models::saved_command::SavedCommand>> {
        let collection = self.get_collection("saved_commands").await?;

        let mut cursor = collection
            .find(None, None)
            .await
            .map_err(|e| DatabaseError::QueryFailed(e.to_string()))?;

        let mut commands = Vec::new();
        while cursor.advance().await.map_err(|e| DatabaseError::QueryFailed(e.to_string()))? {
            let doc = cursor.deserialize_current().map_err(|e| DatabaseError::ParseError(e.to_string()))?;
            commands.push(Self::document_to_model(doc)?);
        }

        Ok(commands)
    }

    async fn update_saved_command(
        &self,
        model: &crate::models::saved_command::SavedCommand,
    ) -> DatabaseResult<()> {
        self.save_saved_command(model).await
    }

    async fn delete_saved_command(&self, id: &str) -> DatabaseResult<()> {
        let collection = self.get_collection("saved_commands").await?;
        let filter = doc! { "_id": id };

        collection
            .delete_one(filter, None)
            .await
            .map_err(|e| DatabaseError::QueryFailed(e.to_string()))?;

        Ok(())
    }

    async fn increment_command_usage(&self, id: &str) -> DatabaseResult<()> {
        let collection = self.get_collection("saved_commands").await?;
        let filter = doc! { "_id": id };
        let update = doc! { "$inc": { "usageCount": 1 }, "$set": { "lastUsedAt": chrono::Utc::now().to_rfc3339() } };

        collection
            .update_one(filter, update, None)
            .await
            .map_err(|e| DatabaseError::QueryFailed(e.to_string()))?;

        Ok(())
    }

    async fn toggle_command_favorite(&self, id: &str) -> DatabaseResult<()> {
        let current = self.find_saved_command_by_id(id).await?;
        if let Some(mut command) = current {
            command.is_favorite = !command.is_favorite;
            self.save_saved_command(&command).await?;
        }

        Ok(())
    }

    async fn save_saved_command_group(
        &self,
        model: &crate::models::saved_command::SavedCommandGroup,
    ) -> DatabaseResult<()> {
        let collection = self.get_collection("saved_command_groups").await?;
        let doc = Self::model_to_document(model)?;

        let filter = doc! { "_id": &model.base.id };
        collection
            .replace_one(filter, doc, None)
            .await
            .map_err(|e| DatabaseError::QueryFailed(e.to_string()))?;

        Ok(())
    }

    async fn find_saved_command_group_by_id(
        &self,
        id: &str,
    ) -> DatabaseResult<Option<crate::models::saved_command::SavedCommandGroup>> {
        let collection = self.get_collection("saved_command_groups").await?;
        let filter = doc! { "_id": id };

        let result = collection
            .find_one(filter, None)
            .await
            .map_err(|e| DatabaseError::QueryFailed(e.to_string()))?;

        match result {
            Some(doc) => Ok(Some(Self::document_to_model(doc)?)),
            None => Ok(None),
        }
    }

    async fn find_all_saved_command_groups(
        &self,
    ) -> DatabaseResult<Vec<crate::models::saved_command::SavedCommandGroup>> {
        let collection = self.get_collection("saved_command_groups").await?;

        let mut cursor = collection
            .find(None, None)
            .await
            .map_err(|e| DatabaseError::QueryFailed(e.to_string()))?;

        let mut groups = Vec::new();
        while cursor.advance().await.map_err(|e| DatabaseError::QueryFailed(e.to_string()))? {
            let doc = cursor.deserialize_current().map_err(|e| DatabaseError::ParseError(e.to_string()))?;
            groups.push(Self::document_to_model(doc)?);
        }

        Ok(groups)
    }

    async fn update_saved_command_group(
        &self,
        model: &crate::models::saved_command::SavedCommandGroup,
    ) -> DatabaseResult<()> {
        self.save_saved_command_group(model).await
    }

    async fn delete_saved_command_group(&self, id: &str) -> DatabaseResult<()> {
        let collection = self.get_collection("saved_command_groups").await?;
        let filter = doc! { "_id": id };

        collection
            .delete_one(filter, None)
            .await
            .map_err(|e| DatabaseError::QueryFailed(e.to_string()))?;

        Ok(())
    }

    async fn save_master_password_entry(
        &self,
        entry: &crate::database::encryption::device_keys::MasterPasswordEntry,
    ) -> DatabaseResult<()> {
        let collection = self.get_collection("master_passwords").await?;
        
        let mut doc_fields = vec![
            ("_id", bson::Bson::String(entry.device_id.clone())),
            ("passwordSalt", bson::Bson::Binary(bson::Binary {
                subtype: bson::spec::BinarySubtype::Generic,
                bytes: entry.password_salt.to_vec(),
            })),
            ("verificationHash", bson::Bson::String(entry.verification_hash.clone())),
            ("autoUnlock", bson::Bson::Boolean(entry.auto_unlock)),
            ("createdAt", bson::Bson::String(entry.created_at.to_rfc3339())),
        ];

        if let Some(timeout) = entry.auto_lock_timeout {
            doc_fields.push(("autoLockTimeout", bson::Bson::Int64(timeout as i64)));
        }

        if let Some(last_verified) = &entry.last_verified_at {
            doc_fields.push(("lastVerifiedAt", bson::Bson::String(last_verified.to_rfc3339())));
        }

        let doc: Document = doc_fields.into_iter().map(|(k, v)| (k.to_string(), v)).collect();

        let filter = doc! { "_id": &entry.device_id };
        collection
            .replace_one(filter, doc, None)
            .await
            .map_err(|e| DatabaseError::QueryFailed(e.to_string()))?;

        Ok(())
    }

    async fn get_master_password_entry(
        &self,
    ) -> DatabaseResult<Option<crate::database::encryption::device_keys::MasterPasswordEntry>> {
        let collection = self.get_collection("master_passwords").await?;

        let result = collection
            .find_one(None, None)
            .await
            .map_err(|e| DatabaseError::QueryFailed(e.to_string()))?;

        match result {
            Some(doc) => {
                let salt_binary = doc.get_binary_generic("passwordSalt")
                    .map_err(|e| DatabaseError::ParseError(e.to_string()))?;
                let mut salt_array = [0u8; 32];
                salt_array.copy_from_slice(&salt_binary[..32]);

                let created_at_str = doc.get_str("createdAt")
                    .map_err(|e| DatabaseError::ParseError(e.to_string()))?;
                let created_at = chrono::DateTime::parse_from_rfc3339(created_at_str)
                    .map_err(|e| DatabaseError::ParseError(e.to_string()))?
                    .with_timezone(&chrono::Utc);

                let last_verified_at = doc.get_str("lastVerifiedAt").ok()
                    .and_then(|s| chrono::DateTime::parse_from_rfc3339(s).ok())
                    .map(|dt| dt.with_timezone(&chrono::Utc));

                Ok(Some(crate::database::encryption::device_keys::MasterPasswordEntry {
                    device_id: doc.get_str("_id")
                        .map_err(|e| DatabaseError::ParseError(e.to_string()))?.to_string(),
                    password_salt: salt_array,
                    verification_hash: doc.get_str("verificationHash")
                        .map_err(|e| DatabaseError::ParseError(e.to_string()))?.to_string(),
                    auto_unlock: doc.get_bool("autoUnlock")
                        .map_err(|e| DatabaseError::ParseError(e.to_string()))?,
                    auto_lock_timeout: doc.get_i64("autoLockTimeout").ok().map(|t| t as u32),
                    created_at,
                    last_verified_at,
                }))
            }
            None => Ok(None),
        }
    }

    async fn update_master_password_last_verified(&self, device_id: &str) -> DatabaseResult<()> {
        let collection = self.get_collection("master_passwords").await?;
        let filter = doc! { "_id": device_id };
        let update = doc! { "$set": { "lastVerifiedAt": chrono::Utc::now().to_rfc3339() } };

        collection
            .update_one(filter, update, None)
            .await
            .map_err(|e| DatabaseError::QueryFailed(e.to_string()))?;

        Ok(())
    }

    async fn delete_master_password_entry(&self, device_id: &str) -> DatabaseResult<()> {
        let collection = self.get_collection("master_passwords").await?;
        let filter = doc! { "_id": device_id };

        collection
            .delete_one(filter, None)
            .await
            .map_err(|e| DatabaseError::QueryFailed(e.to_string()))?;

        Ok(())
    }

    async fn save_device(&self, device: &crate::models::auth::Device) -> DatabaseResult<()> {
        let collection = self.get_collection("devices").await?;
        
        collection
            .update_many(doc! {}, doc! { "$set": { "isCurrent": false } }, None)
            .await
            .map_err(|e| DatabaseError::QueryFailed(e.to_string()))?;

        let doc = Self::model_to_document(device)?;

        let filter = doc! { "_id": &device.device_id };
        collection
            .replace_one(filter, doc, None)
            .await
            .map_err(|e| DatabaseError::QueryFailed(e.to_string()))?;

        Ok(())
    }

    async fn get_device_by_id(
        &self,
        device_id: &str,
    ) -> DatabaseResult<Option<crate::models::auth::Device>> {
        let collection = self.get_collection("devices").await?;
        let filter = doc! { "_id": device_id };

        let result = collection
            .find_one(filter, None)
            .await
            .map_err(|e| DatabaseError::QueryFailed(e.to_string()))?;

        match result {
            Some(doc) => Ok(Some(Self::document_to_model(doc)?)),
            None => Ok(None),
        }
    }

    async fn get_current_device(&self) -> DatabaseResult<Option<crate::models::auth::Device>> {
        let collection = self.get_collection("devices").await?;
        let filter = doc! { "isCurrent": true };

        let result = collection
            .find_one(filter, None)
            .await
            .map_err(|e| DatabaseError::QueryFailed(e.to_string()))?;

        match result {
            Some(doc) => Ok(Some(Self::document_to_model(doc)?)),
            None => Ok(None),
        }
    }

    async fn get_all_devices(&self) -> DatabaseResult<Vec<crate::models::auth::Device>> {
        let collection = self.get_collection("devices").await?;

        let mut cursor = collection
            .find(None, FindOptions::builder().sort(doc! { "lastSeenAt": -1 }).build())
            .await
            .map_err(|e| DatabaseError::QueryFailed(e.to_string()))?;

        let mut devices = Vec::new();
        while cursor.advance().await.map_err(|e| DatabaseError::QueryFailed(e.to_string()))? {
            let doc = cursor.deserialize_current().map_err(|e| DatabaseError::ParseError(e.to_string()))?;
            devices.push(Self::document_to_model(doc)?);
        }

        Ok(devices)
    }

    async fn update_device_last_seen(&self, device_id: &str) -> DatabaseResult<()> {
        let collection = self.get_collection("devices").await?;
        let filter = doc! { "_id": device_id };
        let update = doc! { "$set": { "lastSeenAt": chrono::Utc::now().to_rfc3339() } };

        collection
            .update_one(filter, update, None)
            .await
            .map_err(|e| DatabaseError::QueryFailed(e.to_string()))?;

        Ok(())
    }

    async fn delete_device(&self, device_id: &str) -> DatabaseResult<()> {
        let collection = self.get_collection("devices").await?;
        let filter = doc! { "_id": device_id };

        collection
            .delete_one(filter, None)
            .await
            .map_err(|e| DatabaseError::QueryFailed(e.to_string()))?;

        Ok(())
    }
}
