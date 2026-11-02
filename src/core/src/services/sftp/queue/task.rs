// Copyright (C) 2026 Bùi Thanh Xuân (klpod221)
// SPDX-License-Identifier: GPL-3.0-or-later

use crate::models::sftp::transfer::TransferDirection;

/// Transfer internal metadata for tracking execution and resume
#[allow(dead_code)]
#[derive(Debug, Clone)]
pub struct TransferMetadata {
    pub transfer_id: String,
    pub session_id: String,
    pub local_path: String,
    pub remote_path: String,
    pub direction: TransferDirection,
    pub partial_path: Option<String>,
}

impl TransferMetadata {
    pub fn new(
        transfer_id: String,
        session_id: String,
        local_path: String,
        remote_path: String,
        direction: TransferDirection,
    ) -> Self {
        Self {
            transfer_id,
            session_id,
            local_path,
            remote_path,
            direction,
            partial_path: None,
        }
    }
}
