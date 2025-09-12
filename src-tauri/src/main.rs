// Prevents additional console window on Windows in release, DO NOT REMOVE!!
#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

mod commands;
mod core;
mod error;
mod models;
mod setup;
mod state;

fn main() {
    tauri::Builder::default()
        .plugin(tauri_plugin_opener::init())
        .invoke_handler(tauri::generate_handler![
            // Dashboard commands
            commands::dashboard::get_system_info
        ])
        .manage(state::AppState::default())
        .setup(setup::init)
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
