import { api } from "./api";

/**
 * Execute a remote docker command
 * @param profileId - The SSH profile ID
 * @param action - The action to perform (list, start, stop, etc)
 * @param containerId - The container ID (if applicable)
 * @returns Docker command result
 */
export async function executeRemoteDockerCommand(
  profileId: string,
  action: string,
  containerId?: string,
): Promise<any> {
  return await api.callRaw<any>("execute_remote_docker_command", {
    profileId,
    action,
    containerId,
  });
}
