// Copyright (C) 2026 Bùi Thanh Xuân (klpod221)
// SPDX-License-Identifier: GPL-3.0-or-later

pub mod device;
pub mod requests;

pub use device::{Device, DeviceType, OsInfo};
pub use requests::{ChangeMasterPasswordRequest, VerifyMasterPasswordRequest};
