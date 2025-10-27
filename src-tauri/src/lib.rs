#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

mod commands;
mod core;
mod database;
mod error;
mod models;
mod services;
mod setup;
mod state;

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn main() {
    tauri::Builder::default()
        .plugin(tauri_plugin_clipboard_manager::init())
        .plugin(tauri_plugin_opener::init())
        .plugin(tauri_plugin_process::init())
        .invoke_handler(tauri::generate_handler![
                commands::dashboard::get_system_info,
                commands::terminal::create_terminal,
                commands::terminal::create_ssh_terminal,
                commands::terminal::create_ssh_config_terminal,
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
                commands::auth_events::notify_session_unlocked,
                commands::auth_events::notify_session_locked,
                commands::auth_events::get_auth_session_status,
                commands::database::auth::setup_master_password,
                commands::database::auth::verify_master_password,
                commands::database::auth::try_auto_unlock,
                commands::database::auth::lock_session,
                commands::database::auth::change_master_password,
                commands::database::auth::reset_master_password,
                commands::database::auth::get_master_password_status,
                commands::database::auth::is_session_valid,
                commands::database::auth::get_master_password_config,
                commands::database::auth::update_master_password_config,
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
                commands::database::ssh::create_ssh_key,
                commands::database::ssh::get_ssh_keys,
                commands::database::ssh::update_ssh_key,
                commands::database::ssh::delete_ssh_key,
                commands::database::ssh::count_profiles_using_key,
                commands::database::ssh::import_ssh_key_from_file,
                commands::database::ssh::cleanup_idle_connections,
                commands::database::ssh::clear_connection_pool,
                commands::database::ssh::get_connection_pool_size,
                commands::database::ssh::get_ssh_config_hosts,
                commands::database::tunnel::create_tunnel,
                commands::database::tunnel::get_tunnels,
                commands::database::tunnel::get_tunnel,
                commands::database::tunnel::update_tunnel,
                commands::database::tunnel::delete_tunnel,
                commands::database::tunnel::start_tunnel,
                commands::database::tunnel::stop_tunnel,
                commands::database::tunnel::get_tunnel_status,
                commands::database::saved_command::create_saved_command,
                commands::database::saved_command::get_saved_commands,
                commands::database::saved_command::get_saved_command,
                commands::database::saved_command::update_saved_command,
                commands::database::saved_command::delete_saved_command,
                commands::database::saved_command::increment_command_usage,
                commands::database::saved_command::toggle_command_favorite,
                commands::database::saved_command::create_saved_command_group,
                commands::database::saved_command::get_saved_command_groups,
                commands::database::saved_command::get_saved_command_group,
                commands::database::saved_command::update_saved_command_group,
                commands::database::saved_command::delete_saved_command_group,
                commands::database::external_db::add_external_database,
                commands::database::external_db::get_external_databases,
                commands::database::external_db::get_external_database_with_details,
                commands::database::external_db::update_external_database,
                commands::database::external_db::delete_external_database,
                commands::database::external_db::test_external_database_connection,
                commands::database::external_db::connect_to_database,
                commands::database::external_db::disconnect_from_database,
                commands::database::sync::sync_now,
                commands::database::sync::get_sync_status,
                commands::database::sync::get_sync_logs,
                commands::database::sync::enable_auto_sync,
                commands::database::sync::disable_auto_sync,
                commands::database::sync::get_sync_statistics,
                commands::database::sync::get_sync_service_statistics,
                commands::database::sync::get_unresolved_conflict_resolutions,
                commands::database::sync::resolve_conflict_resolution,
                commands::database::sync::cleanup_resolved_conflicts,
                commands::database::sync::get_global_sync_settings,
                commands::database::sync::update_global_sync_settings,
                commands::database::sync::get_current_device,
                commands::database::sync::get_all_devices,
                commands::database::sync::register_device
            ])
            .setup(setup::init)
            .run(tauri::generate_context!())
            .expect("error while running tauri application");
}
