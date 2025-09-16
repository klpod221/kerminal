// Prevents additional console window on Windows in release, DO NOT REMOVE!!
#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

mod commands;
mod core;
mod database;
mod error;
mod models;
mod services;
mod setup;
mod state;

use crate::services::terminal::TerminalManager;
use crate::state::AppState;
use crate::database::{DatabaseService, DatabaseServiceConfig};
use std::sync::Arc;

#[tokio::main]
async fn main() {
    // Initialize app state with database service
    let app_state = AppState::new().await
        .expect("Failed to initialize application state");

    // Extract the database service for terminal manager
    let database_service = {
        let db_lock = app_state.database_service.lock().await;
        // We need to create a new Arc<DatabaseService> here
        // For now, let's work around this by managing DatabaseService separately
        Arc::new(DatabaseService::new(DatabaseServiceConfig::default()).await
            .expect("Failed to create database service for terminal manager"))
    };

    let terminal_manager = TerminalManager::new(database_service.clone());

    tauri::Builder::default()
        .plugin(tauri_plugin_opener::init())
        .manage(terminal_manager)
        .manage(database_service.clone())
        .manage(app_state)
        .invoke_handler(tauri::generate_handler![
            // Dashboard commands
            commands::dashboard::get_system_info,
            // Terminal commands
            commands::terminal::create_terminal,
            commands::terminal::create_ssh_terminal,
            commands::terminal::write_to_terminal,
            commands::terminal::resize_terminal,
            commands::terminal::close_terminal,
            commands::terminal::get_terminal_info,
            commands::terminal::list_terminals,
            commands::terminal::close_all_terminals,
            // Buffer commands
            commands::buffer::get_terminal_buffer,
            commands::buffer::has_terminal_buffer,
            commands::buffer::get_buffer_stats,
            commands::buffer::cleanup_terminal_buffers,
            // System commands
            commands::system::get_user_hostname,
            // Database commands - Master Password
            commands::database::is_master_password_setup,
            commands::database::setup_master_password,
            commands::database::verify_master_password,
            commands::database::try_auto_unlock,
            commands::database::lock_session,
            // Database commands - SSH Groups
            commands::database::create_ssh_group,
            commands::database::get_ssh_groups,
            commands::database::get_ssh_group,
            commands::database::update_ssh_group,
            commands::database::delete_ssh_group,
            // Database commands - SSH Profiles
            commands::database::create_ssh_profile,
            commands::database::get_ssh_profiles,
            commands::database::get_ssh_profile,
            commands::database::update_ssh_profile,
            commands::database::delete_ssh_profile,
            commands::database::move_profile_to_group,
            commands::database::duplicate_ssh_profile,
            // Database commands - Utilities
            commands::database::get_database_stats,
            commands::database::get_current_device
        ])
        .setup(setup::init)
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
