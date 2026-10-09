// Copyright (C) 2026 Bùi Thanh Xuân (klpod221)
// SPDX-License-Identifier: GPL-3.0-or-later

pub mod config_host;
pub mod group;
pub mod key;
pub mod profile;
pub mod tunnel;

pub use config_host::SSHConfigHost;
pub use group::{CreateSSHGroupRequest, DeleteGroupAction, SSHGroup, UpdateSSHGroupRequest};
pub use key::{CreateSSHKeyRequest, SSHKey, UpdateSSHKeyRequest};
pub use profile::{
    AuthData, CreateSSHProfileRequest, SSHProfile, TestSSHConnectionRequest,
    UpdateSSHProfileRequest,
};
pub use tunnel::{
    CreateSSHTunnelRequest, SSHTunnel, TunnelStatus, TunnelType, TunnelWithStatus,
    UpdateSSHTunnelRequest,
};
