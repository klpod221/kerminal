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

use std::sync::Arc;
use tokio::sync::RwLock;

use crate::{
    database::{
        encryption::MasterPasswordManager,
        error::{DatabaseResult, EncryptionError},
        traits::EncryptionService,
    },
    models::sync::external_db::ConnectionDetails,
};

pub struct ExternalDbEncryptor {
    master_password_manager: Arc<RwLock<MasterPasswordManager>>,
}

impl ExternalDbEncryptor {
    pub fn new(master_password_manager: Arc<RwLock<MasterPasswordManager>>) -> Self {
        Self {
            master_password_manager,
        }
    }

    pub async fn encrypt_connection_details(
        &self,
        details: &ConnectionDetails,
    ) -> DatabaseResult<String> {
        let json = serde_json::to_string(details).map_err(|e| {
            EncryptionError::EncryptionFailed(format!(
                "Failed to serialize connection details: {}",
                e
            ))
        })?;

        let manager = self.master_password_manager.read().await;
        let encrypted = manager.encrypt_string(&json, Some("__shared__")).await?;

        Ok(encrypted)
    }

    pub async fn decrypt_connection_details(
        &self,
        encrypted: &str,
    ) -> DatabaseResult<ConnectionDetails> {
        let manager = self.master_password_manager.read().await;

        let decrypted = match manager.decrypt_string(encrypted, Some("__shared__")).await {
            Ok(data) => data,
            Err(_) => manager.decrypt_string(encrypted, None).await?,
        };

        let details: ConnectionDetails = serde_json::from_str(&decrypted).map_err(|e| {
            EncryptionError::DecryptionFailed(format!(
                "Failed to deserialize connection details: {}",
                e
            ))
        })?;

        Ok(details)
    }
}
