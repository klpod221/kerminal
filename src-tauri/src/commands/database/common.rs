use crate::database::error::DatabaseError;

/// Convert DatabaseError to String for Tauri compatibility
impl From<DatabaseError> for String {
    fn from(error: DatabaseError) -> Self {
        let app_error: crate::error::AppError = error.into();
        app_error.to_string()
    }
}

/// Unified error conversion macro for database operations
macro_rules! app_result {
    ($expr:expr) => {
        $expr.map_err(|e: crate::database::error::DatabaseError| -> String {
            let app_error: crate::error::AppError = e.into();
            app_error.to_string()
        })
    };
}

pub(crate) use app_result;
