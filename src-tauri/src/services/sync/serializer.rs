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

use serde_json::Value;

use crate::database::error::{DatabaseError, DatabaseResult};
use crate::models::saved_command::{SavedCommand, SavedCommandGroup};
use crate::models::ssh::{SSHGroup, SSHKey, SSHProfile, SSHTunnel};

/// Helper trait for converting models to/from sync records
pub trait SyncSerializable {
    fn to_json(&self) -> DatabaseResult<Value>;
    fn from_json(value: &Value) -> DatabaseResult<Self>
    where
        Self: Sized;
}

impl SyncSerializable for SSHProfile {
    fn to_json(&self) -> DatabaseResult<Value> {
        serde_json::to_value(self).map_err(DatabaseError::SerializationError)
    }

    fn from_json(value: &Value) -> DatabaseResult<Self> {
        serde_json::from_value(value.clone()).map_err(DatabaseError::SerializationError)
    }
}

impl SyncSerializable for SSHGroup {
    fn to_json(&self) -> DatabaseResult<Value> {
        serde_json::to_value(self).map_err(DatabaseError::SerializationError)
    }

    fn from_json(value: &Value) -> DatabaseResult<Self> {
        serde_json::from_value(value.clone()).map_err(DatabaseError::SerializationError)
    }
}

impl SyncSerializable for SSHKey {
    fn to_json(&self) -> DatabaseResult<Value> {
        serde_json::to_value(self).map_err(DatabaseError::SerializationError)
    }

    fn from_json(value: &Value) -> DatabaseResult<Self> {
        serde_json::from_value(value.clone()).map_err(DatabaseError::SerializationError)
    }
}

impl SyncSerializable for SSHTunnel {
    fn to_json(&self) -> DatabaseResult<Value> {
        serde_json::to_value(self).map_err(DatabaseError::SerializationError)
    }

    fn from_json(value: &Value) -> DatabaseResult<Self> {
        serde_json::from_value(value.clone()).map_err(DatabaseError::SerializationError)
    }
}

impl SyncSerializable for SavedCommand {
    fn to_json(&self) -> DatabaseResult<Value> {
        serde_json::to_value(self).map_err(DatabaseError::SerializationError)
    }

    fn from_json(value: &Value) -> DatabaseResult<Self> {
        serde_json::from_value(value.clone()).map_err(DatabaseError::SerializationError)
    }
}

impl SyncSerializable for SavedCommandGroup {
    fn to_json(&self) -> DatabaseResult<Value> {
        serde_json::to_value(self).map_err(DatabaseError::SerializationError)
    }

    fn from_json(value: &Value) -> DatabaseResult<Self> {
        serde_json::from_value(value.clone()).map_err(DatabaseError::SerializationError)
    }
}
