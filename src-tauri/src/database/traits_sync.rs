/*
 * Kerminal - Modern Terminal Emulator & SSH Manager
 * Copyright (C) 2026 Bùi Thanh Xuân (klpod221)
 *
 * This program is free software: you can redistribute it and/or modify
 * it under the terms of the GNU General Public License as published by
 * the Free Software Foundation, either version 3 of the License, or
 * (at your option) any later version.
 *
 * This program is distributed in the hope that it will be useful,
 * but WITHOUT ANY WARRANTY; without even the implied warranty of
 * MERCHANTABILITY or FITNESS FOR A PARTICULAR PURPOSE.  See the
 * GNU General Public License for more details.
 *
 * You should have received a copy of the GNU General Public License
 * along with this program.  If not, see <https://www.gnu.org/licenses/>.
 */

use async_trait::async_trait;
use chrono::{DateTime, Utc};
use serde_json::Value;

use crate::database::error::DatabaseResult;

/// Simplified trait for sync target databases
/// These databases only serve as sync endpoints - no business logic
#[async_trait]
pub trait SyncTarget: Send + Sync {
    /// Connect to the database
    async fn connect(&mut self) -> DatabaseResult<()>;

    /// Test the database connection
    async fn test_connection(&self) -> DatabaseResult<()>;

    /// Push records to remote database
    /// Records are JSON-serialized entities from local SQLite
    async fn push_records(&self, table: &str, records: Vec<Value>) -> DatabaseResult<usize>;

    /// Pull records from remote database modified since timestamp
    /// Returns JSON-serialized entities
    async fn pull_records(
        &self,
        table: &str,
        since: Option<DateTime<Utc>>,
    ) -> DatabaseResult<Vec<Value>>;

    /// Get record versions for conflict detection
    /// Get version information for conflict detection
    async fn get_record_versions(
        &self,
        table: &str,
        ids: Vec<String>,
    ) -> DatabaseResult<std::collections::HashMap<String, u64>>;
}
