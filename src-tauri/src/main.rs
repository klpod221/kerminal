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

    // Extract a clone of the database service for terminal manager
    let database_service = app_state.database_service.clone();
    let terminal_manager = TerminalManager::new(database_service);

    tauri::Builder::default()
        .plugin(tauri_plugin_opener::init())
        .manage(terminal_manager)
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
            commands::database::setup_master_password,
            commands::database::verify_master_password,
            commands::database::try_auto_unlock,
            commands::database::lock_session,
            commands::database::get_master_password_status,
            commands::database::change_master_password,
            commands::database::reset_master_password,
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
            commands::database::get_database_stats
        ])
        .setup(setup::init)
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
