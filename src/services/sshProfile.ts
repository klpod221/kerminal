import { api } from "./api";
import type {
  SSHProfile,
  SSHGroup,
  CreateSSHProfileRequest,
  UpdateSSHProfileRequest,
  DeleteGroupAction,
  CreateSSHGroupRequest,
  UpdateSSHGroupRequest,
} from "../types/ssh";

// === SSH Profile Services ===

/**
 * Create a new SSH profile
 * @param request - SSH profile creation request
 * @returns The created SSH profile
 */
export async function createSSHProfile(
  request: CreateSSHProfileRequest,
): Promise<SSHProfile> {
  console.log("Creating SSH profile with request:", request);
  return await api.call("create_ssh_profile", request);
}

/**
 * Get all SSH profiles
 * @returns Array of SSH profiles
 */
export async function getSSHProfiles(): Promise<SSHProfile[]> {
  return await api.call("get_ssh_profiles");
}

/**
 * Get SSH profile by ID
 * @param id - Profile ID
 * @returns SSH profile
 */
export async function getSSHProfile(id: string): Promise<SSHProfile> {
  return await api.callRaw("get_ssh_profile", id);
}

/**
 * Update SSH profile
 * @param id - Profile ID
 * @param request - Update request
 * @returns Updated SSH profile
 */
export async function updateSSHProfile(
  id: string,
  request: UpdateSSHProfileRequest,
): Promise<SSHProfile> {
  return await api.callRaw("update_ssh_profile", id, request);
}

/**
 * Delete SSH profile
 * @param id - Profile ID
 */
export async function deleteSSHProfile(id: string): Promise<void> {
  return await api.callRaw("delete_ssh_profile", id);
}

/**
 * Move profile to different group
 * @param profileId - Profile ID
 * @param groupId - Target group ID (null for ungrouped)
 */
export async function moveProfileToGroup(
  profileId: string,
  groupId: string | null,
): Promise<void> {
  return await api.callRaw("move_profile_to_group", profileId, groupId);
}

/**
 * Duplicate SSH profile with new name
 * @param id - Original profile ID
 * @param newName - New profile name
 * @returns Duplicated SSH profile
 */
export async function duplicateSSHProfile(
  id: string,
  newName: string,
): Promise<SSHProfile> {
  return await api.callRaw("duplicate_ssh_profile", id, newName);
}

// === SSH Group Services ===

/**
 * Create a new SSH group
 * @param request - SSH group creation request
 * @returns The created SSH group
 */
export async function createSSHGroup(
  request: CreateSSHGroupRequest,
): Promise<SSHGroup> {
  return await api.call("create_ssh_group", request);
}

/**
 * Get all SSH groups
 * @returns Array of SSH groups
 */
export async function getSSHGroups(): Promise<SSHGroup[]> {
  return await api.call("get_ssh_groups");
}

/**
 * Get SSH group by ID
 * @param id - Group ID
 * @returns SSH group
 */
export async function getSSHGroup(id: string): Promise<SSHGroup> {
  return await api.callRaw("get_ssh_group", id);
}

/**
 * Update SSH group
 * @param id - Group ID
 * @param request - Update request
 * @returns Updated SSH group
 */
export async function updateSSHGroup(
  id: string,
  request: UpdateSSHGroupRequest,
): Promise<SSHGroup> {
  return await api.callRaw("update_ssh_group", id, request);
}

/**
 * Delete SSH group with action for existing profiles
 * @param id - Group ID
 * @param action - Action for existing profiles
 */
export async function deleteSSHGroup(
  id: string,
  action: DeleteGroupAction,
): Promise<void> {
  return await api.callRaw("delete_ssh_group", id, action);
}

/**
 * Cleanup idle SSH connections from the pool
 */
export async function cleanupIdleConnections(): Promise<void> {
  return await api.call("cleanup_idle_connections");
}

/**
 * Clear all connections from the pool
 */
export async function clearConnectionPool(): Promise<void> {
  return await api.call("clear_connection_pool");
}

/**
 * Get current connection pool size
 * @returns Number of active connections in pool
 */
export async function getConnectionPoolSize(): Promise<number> {
  return await api.call("get_connection_pool_size");
}
