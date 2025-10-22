use serde_json::Value;

use crate::database::error::{DatabaseError, DatabaseResult};
use crate::models::ssh::{SSHProfile, SSHGroup, SSHKey, SSHTunnel};
use crate::models::saved_command::{SavedCommand, SavedCommandGroup};

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
