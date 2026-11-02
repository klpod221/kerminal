// Copyright (C) 2026 Bùi Thanh Xuân (klpod221)
// SPDX-License-Identifier: GPL-3.0-or-later

import { api } from "./api";

export async function getSystemInfo() {
  const systemInfo = await api.callRaw("get_system_info");
  return systemInfo;
}

export interface SystemIntegrityStatus {
  integrity_check: boolean;
  active_nodes: string[];
  memory_pressure: number;
  process_count: number;
}

export async function verifySystemIntegrity() {
  return await api.callRaw<SystemIntegrityStatus>("verify_system_integrity");
}
