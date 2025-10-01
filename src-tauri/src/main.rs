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
    let app_state = AppState::new()
        .await
        .expect("Failed to initialize application state");

    tauri::Builder::default()
        .plugin(tauri_plugin_opener::init())
        .manage(app_state)
        .invoke_handler(tauri::generate_handler![
            commands::dashboard::get_system_info,
            commands::terminal::create_terminal,
            commands::terminal::create_ssh_terminal,
            commands::terminal::write_to_terminal,
            commands::terminal::write_batch_to_terminal,
            commands::terminal::resize_terminal,
            commands::terminal::close_terminal,
            commands::terminal::get_terminal_info,
            commands::terminal::list_terminals,
            commands::buffer::get_terminal_buffer,
            commands::buffer::get_terminal_buffer_chunk,
            commands::buffer::has_terminal_buffer,
            commands::buffer::get_buffer_stats,
            commands::buffer::cleanup_terminal_buffers,
            commands::system::get_user_hostname,
            commands::database::auth::setup_master_password,
            commands::database::auth::verify_master_password,
            commands::database::auth::try_auto_unlock,
            commands::database::auth::lock_session,
            commands::database::auth::change_master_password,
            commands::database::auth::reset_master_password,
            commands::database::auth::get_master_password_status,
            commands::database::auth::is_session_valid,
            commands::database::auth::get_master_password_config,
            commands::database::auth::get_current_device,
            commands::database::auth::update_master_password_config,
            commands::auth_events::notify_session_unlocked,
            commands::auth_events::notify_session_locked,
            commands::auth_events::get_auth_session_status,
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
            commands::database::ssh::test_ssh_connection,
            commands::database::system::get_database_stats
        ])
        .setup(setup::init)
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
