// Copyright (C) 2026 Bùi Thanh Xuân (klpod221)
// SPDX-License-Identifier: GPL-3.0-or-later

import { api } from "./api";

/**
 * Get remote server metrics (CPU, Memory, Disk, Network)
 * @param profileId - The SSH profile ID
 * @returns Remote server metrics
 */
export async function getRemoteServerMetrics(profileId: string): Promise<any> {
  return await api.callRaw<any>("get_remote_server_metrics", { profileId });
}
