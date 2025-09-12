use tauri::{App, Manager};

// Initialize application state and perform setup tasks
pub fn init(app: &mut App) -> std::result::Result<(), Box<dyn std::error::Error>> {
    println!("Application starting up...");

    // Setup application title
    let window = app.get_webview_window("main").unwrap();
    window.set_title("Kerminal").unwrap();

    Ok(())
}
