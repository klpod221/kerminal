use crate::core::auth_session_manager::AuthSessionManager;
use crate::database::{DatabaseService, DatabaseServiceConfig};
use crate::services::{
    auth::AuthService,
    saved_command::SavedCommandService,
    ssh::{SSHConnectionPool, SSHKeyService, SSHService},
    sync::SyncService,
    terminal::TerminalManager,
    tunnel::TunnelService,
};
use futures::FutureExt;
use std::sync::Arc;
use tokio::sync::{Mutex, RwLock};

pub struct AppState {
    pub database_service: Arc<Mutex<DatabaseService>>,
    pub auth_service: AuthService,
    pub ssh_service: SSHService,
    pub ssh_key_service: Arc<Mutex<SSHKeyService>>,
    pub ssh_connection_pool: Arc<SSHConnectionPool>,
    pub tunnel_service: TunnelService,
    pub saved_command_service: SavedCommandService,
    pub sync_service: Arc<SyncService>,
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
        let ssh_key_service =
            Arc::new(Mutex::new(SSHKeyService::new(database_service_arc.clone())));
        let ssh_service = SSHService::new(database_service_arc.clone(), ssh_key_service.clone());
        let tunnel_service = TunnelService::new_with_auto_start(database_service_arc.clone()).await;
        let saved_command_service = SavedCommandService::new(database_service_arc.clone());

        // Create RwLock wrapper for sync service (allows parallel reads)
        let database_service_rw = Arc::new(RwLock::new(
            DatabaseService::new(DatabaseServiceConfig::default())
                .await
                .map_err(|e| format!("Failed to initialize sync database service: {}", e))?,
        ));

        // Create and initialize sync service
        let sync_service = Arc::new(SyncService::new(database_service_rw));
        sync_service
            .initialize()
            .await
            .map_err(|e| format!("Failed to initialize sync service: {}", e))?;

        let terminal_manager = TerminalManager::new_with_ssh_key_service(
            database_service_arc.clone(),
            ssh_key_service.clone(),
        );

        let auth_session_manager = Arc::new(Mutex::new(AuthSessionManager::new(
            database_service_arc.clone(),
        )));

        let ssh_connection_pool = Arc::new(SSHConnectionPool::default());

        Ok(Self {
            database_service: database_service_arc,
            auth_service,
            ssh_service,
            ssh_key_service,
            ssh_connection_pool,
            tunnel_service,
            saved_command_service,
            sync_service,
            terminal_manager,
            auth_session_manager,
        })
    }
}

impl Default for AppState {
    fn default() -> Self {
        let database_service_arc = Arc::new(Mutex::new(
            DatabaseService::new(DatabaseServiceConfig::default())
                .now_or_never()
                .unwrap()
                .unwrap(),
        ));

        let auth_service = AuthService::new(database_service_arc.clone());
        let ssh_key_service =
            Arc::new(Mutex::new(SSHKeyService::new(database_service_arc.clone())));
        let ssh_service = SSHService::new(database_service_arc.clone(), ssh_key_service.clone());
        let tunnel_service = TunnelService::new(database_service_arc.clone());
        let saved_command_service = SavedCommandService::new(database_service_arc.clone());

        let database_service_rw = Arc::new(RwLock::new(
            DatabaseService::new(DatabaseServiceConfig::default())
                .now_or_never()
                .unwrap()
                .unwrap(),
        ));
        let sync_service = Arc::new(SyncService::new(database_service_rw));

        let terminal_manager = TerminalManager::new_with_ssh_key_service(
            database_service_arc.clone(),
            ssh_key_service.clone(),
        );

        let auth_session_manager = Arc::new(Mutex::new(AuthSessionManager::new(
            database_service_arc.clone(),
        )));

        let ssh_connection_pool = Arc::new(SSHConnectionPool::default());

        Self {
            database_service: database_service_arc,
            auth_service,
            ssh_service,
            ssh_key_service,
            ssh_connection_pool,
            tunnel_service,
            saved_command_service,
            sync_service,
            terminal_manager,
            auth_session_manager,
        }
    }
}
