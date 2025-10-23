use std::sync::Arc;
use tokio::sync::Mutex;

use crate::database::{error::DatabaseResult, service::DatabaseService};
use crate::models::saved_command::{
    CreateSavedCommandGroupRequest, CreateSavedCommandRequest, SavedCommand, SavedCommandGroup,
    UpdateSavedCommandGroupRequest, UpdateSavedCommandRequest,
};

/// Saved command service for handling saved commands and groups
pub struct SavedCommandService {
    database_service: Arc<Mutex<DatabaseService>>,
}

impl SavedCommandService {
    /// Create new SavedCommandService instance
    pub fn new(database_service: Arc<Mutex<DatabaseService>>) -> Self {
        Self { database_service }
    }

    /// Create new saved command
    pub async fn create_command(
        &self,
        request: CreateSavedCommandRequest,
    ) -> DatabaseResult<SavedCommand> {
        let db_service = self.database_service.lock().await;
        db_service.create_saved_command(request).await
    }

    /// Get all saved commands
    pub async fn get_commands(&self) -> DatabaseResult<Vec<SavedCommand>> {
        let db_service = self.database_service.lock().await;
        db_service.get_saved_commands().await
    }

    /// Get saved command by ID
    pub async fn get_command(&self, id: &str) -> DatabaseResult<SavedCommand> {
        let db_service = self.database_service.lock().await;
        db_service.get_saved_command(id).await
    }

    /// Update saved command
    pub async fn update_command(
        &self,
        id: &str,
        request: UpdateSavedCommandRequest,
    ) -> DatabaseResult<SavedCommand> {
        let db_service = self.database_service.lock().await;
        db_service.update_saved_command(id, request).await
    }

    /// Delete saved command
    pub async fn delete_command(&self, id: &str) -> DatabaseResult<()> {
        let db_service = self.database_service.lock().await;
        db_service.delete_saved_command(id).await
    }

    /// Increment usage count for command
    pub async fn increment_usage(&self, id: &str) -> DatabaseResult<()> {
        let db_service = self.database_service.lock().await;
        db_service.increment_command_usage(id).await
    }

    /// Toggle favorite status
    pub async fn toggle_favorite(&self, id: &str) -> DatabaseResult<SavedCommand> {
        let db_service = self.database_service.lock().await;
        db_service.toggle_command_favorite(id).await
    }

    /// Create new saved command group
    pub async fn create_group(
        &self,
        request: CreateSavedCommandGroupRequest,
    ) -> DatabaseResult<SavedCommandGroup> {
        let db_service = self.database_service.lock().await;
        db_service.create_saved_command_group(request).await
    }

    /// Get all saved command groups
    pub async fn get_groups(&self) -> DatabaseResult<Vec<SavedCommandGroup>> {
        let db_service = self.database_service.lock().await;
        db_service.get_saved_command_groups().await
    }

    /// Get saved command group by ID
    pub async fn get_group(&self, id: &str) -> DatabaseResult<SavedCommandGroup> {
        let db_service = self.database_service.lock().await;
        db_service.get_saved_command_group(id).await
    }

    /// Update saved command group
    pub async fn update_group(
        &self,
        id: &str,
        request: UpdateSavedCommandGroupRequest,
    ) -> DatabaseResult<SavedCommandGroup> {
        let db_service = self.database_service.lock().await;
        db_service.update_saved_command_group(id, request).await
    }

    /// Delete saved command group
    pub async fn delete_group(&self, id: &str) -> DatabaseResult<()> {
        let db_service = self.database_service.lock().await;
        db_service.delete_saved_command_group(id).await
    }
}
