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
