// Copyright (C) 2026 Bùi Thanh Xuân (klpod221)
// SPDX-License-Identifier: GPL-3.0-or-later

import { api } from "./api";

export interface ContainerInfo {
  id: string;
  name: string;
  image: string;
  state: string;
  status: string;
  ports: string;
  created: string;
  compose_project: string | null;
  compose_service: string | null;
  engine: string;
  cpu_perc?: string;
  mem_usage?: string;
}

export interface ContainerStatInfo {
  id: string;
  cpu: string;
  memory: string;
}

export type ContainerActionType =
  | "start"
  | "stop"
  | "restart"
  | "pause"
  | "unpause"
  | "rm";

/**
 * Fetch list of containers from target machine (local or remote via SSH)
 */
export async function getContainerList(
  profileId?: string,
  engine?: string,
): Promise<ContainerInfo[]> {
  return await api.callRaw<ContainerInfo[]>("get_container_list", {
    profileId: profileId || undefined,
    engine: engine || undefined,
  });
}

/**
 * Perform lifecycle actions on a container (start, stop, restart, pause, unpause, rm)
 */
export async function executeContainerAction(
  profileId: string | undefined,
  action: ContainerActionType,
  containerId: string,
  engine?: string,
): Promise<void> {
  return await api.callRaw<void>("execute_container_action", {
    profileId: profileId || undefined,
    action,
    containerId,
    engine: engine || undefined,
  });
}

/**
 * Fetch logs for a specific container
 */
export async function getContainerLogs(
  profileId: string | undefined,
  containerId: string,
  tail = 200,
  engine?: string,
): Promise<string> {
  return await api.callRaw<string>("get_container_logs", {
    profileId: profileId || undefined,
    containerId,
    tail,
    engine: engine || undefined,
  });
}

/**
 * Fetch live CPU and Memory resource usage for running containers
 */
export async function getContainerStats(
  profileId?: string,
  engine?: string,
): Promise<Record<string, ContainerStatInfo>> {
  return await api.callRaw<Record<string, ContainerStatInfo>>("get_container_stats", {
    profileId: profileId || undefined,
    engine: engine || undefined,
  });
}

/**
 * Backward compatibility adapter for legacy calls
 */
export async function executeRemoteDockerCommand(
  profileId: string,
  action: string,
  containerId?: string,
): Promise<any> {
  if (action === "ps") {
    return await getContainerList(profileId);
  }
  if (containerId) {
    return await executeContainerAction(
      profileId,
      action as ContainerActionType,
      containerId,
    );
  }
  return [];
}
