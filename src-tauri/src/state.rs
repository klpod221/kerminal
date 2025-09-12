use std::sync::Mutex;

// Database struct
pub struct DbConnection {
}

// Application state
pub struct AppState {
    pub db: Mutex<DbConnection>,
}

impl Default for AppState {
    fn default() -> Self {
        AppState {
            db: Mutex::new(DbConnection {}),
        }
    }
}
