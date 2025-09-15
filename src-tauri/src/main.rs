// Prevents additional console window on Windows in release, DO NOT REMOVE!!
#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

mod commands;
mod core;
mod error;
mod models;
mod services;
mod setup;
mod state;

use crate::services::terminal::TerminalManager;

fn main() {
    let terminal_manager = TerminalManager::new();

    tauri::Builder::default()
        .plugin(tauri_plugin_opener::init())
        .manage(terminal_manager)
        .invoke_handler(tauri::generate_handler![
            // Dashboard commands
            commands::dashboard::get_system_info,
            // Terminal commands
            commands::terminal::create_terminal,
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
            commands::system::get_user_hostname
        ])
        .manage(state::AppState::default())
        .setup(setup::init)
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
