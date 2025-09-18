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

use crate::state::AppState;

#[tokio::main]
async fn main() {
    // Initialize app state with database service
    let app_state = AppState::new()
        .await
        .expect("Failed to initialize application state");

    tauri::Builder::default()
        .plugin(tauri_plugin_opener::init())
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
            commands::database::auth::setup_master_password,
            commands::database::auth::verify_master_password,
            commands::database::auth::try_auto_unlock,
            commands::database::auth::lock_session,
            commands::database::auth::change_master_password,
            commands::database::auth::reset_master_password,
            commands::database::auth::get_master_password_status,
            commands::database::auth::get_current_device,
            commands::database::auth::update_master_password_config,
            // Database commands - SSH Groups & Profiles
            commands::database::ssh::create_ssh_group,
            commands::database::ssh::get_ssh_groups,
            commands::database::ssh::get_ssh_group,
            commands::database::ssh::update_ssh_group,
            commands::database::ssh::delete_ssh_group,
            commands::database::ssh::create_ssh_profile,
            commands::database::ssh::get_ssh_profiles,
            commands::database::ssh::get_ssh_profile,
            commands::database::ssh::update_ssh_profile,
            commands::database::ssh::delete_ssh_profile,
            commands::database::ssh::move_profile_to_group,
            commands::database::ssh::duplicate_ssh_profile,
            // Database commands - System
            commands::database::system::get_database_stats
        ])
        .setup(setup::init)
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
