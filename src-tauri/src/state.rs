use crate::database::{DatabaseService, DatabaseServiceConfig};
use crate::services::{auth::AuthService, ssh::SSHService, sync::SyncService, terminal::TerminalManager};
use crate::core::auth_session_manager::AuthSessionManager;
use futures::FutureExt;
use std::sync::Arc;
use tokio::sync::Mutex;

// Application state
pub struct AppState {
    pub database_service: Arc<Mutex<DatabaseService>>,
    pub auth_service: AuthService,
    pub ssh_service: SSHService,
    pub sync_service: SyncService,
    pub terminal_manager: TerminalManager,
    pub auth_session_manager: Arc<Mutex<AuthSessionManager>>,
}

impl AppState {
    /// Create new app state with initialized database service
    pub async fn new() -> Result<Self, String> {
        let config = DatabaseServiceConfig::default();
        let database_service = DatabaseService::new(config)
            .await
            .map_err(|e| format!("Failed to initialize database service: {}", e))?;

        let database_service_arc = Arc::new(Mutex::new(database_service));

        // Create service instances
        let auth_service = AuthService::new(database_service_arc.clone());
        let ssh_service = SSHService::new(database_service_arc.clone());
        let sync_service = SyncService::new(database_service_arc.clone());
        let terminal_manager = TerminalManager::new(database_service_arc.clone());

        // Create auth session manager
        let auth_session_manager = Arc::new(Mutex::new(
            AuthSessionManager::new(database_service_arc.clone())
        ));

        Ok(Self {
            database_service: database_service_arc,
            auth_service,
            ssh_service,
            sync_service,
            terminal_manager,
            auth_session_manager,
        })
    }
}

impl Default for AppState {
    fn default() -> Self {
        // For backward compatibility, create empty state
        // Real initialization should use AppState::new()
        let database_service_arc = Arc::new(Mutex::new(
            DatabaseService::new(DatabaseServiceConfig::default())
                .now_or_never()
                .unwrap()
                .unwrap(),
        ));

        // Create service instances
        let auth_service = AuthService::new(database_service_arc.clone());
        let ssh_service = SSHService::new(database_service_arc.clone());
        let sync_service = SyncService::new(database_service_arc.clone());
        let terminal_manager = TerminalManager::new(database_service_arc.clone());

        // Create auth session manager
        let auth_session_manager = Arc::new(Mutex::new(
            AuthSessionManager::new(database_service_arc.clone())
        ));

        Self {
            database_service: database_service_arc,
            auth_service,
            ssh_service,
            sync_service,
            terminal_manager,
            auth_session_manager,
        }
    }
}
