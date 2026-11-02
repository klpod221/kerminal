// Copyright (C) 2026 Bùi Thanh Xuân (klpod221)
// SPDX-License-Identifier: GPL-3.0-or-later

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
