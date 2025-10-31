import { api } from "./api";
import type {
  SSHKey,
  CreateSSHKeyRequest,
  UpdateSSHKeyRequest,
} from "../types/ssh";

/**
 * Create a new SSH key
 * @param request - Create SSH key request
 * @returns Created SSH key
 */
export async function createSSHKey(
  request: CreateSSHKeyRequest,
): Promise<SSHKey> {
  return await api.call<SSHKey>("create_ssh_key", request);
}

/**
 * Get all SSH keys
 * @returns List of all SSH keys
 */
export async function getSSHKeys(): Promise<SSHKey[]> {
  return await api.call<SSHKey[]>("get_ssh_keys");
}

/**
 * Get SSH key by ID
 * @param id - SSH key ID
 * @returns SSH key details
 */
export async function getSSHKey(id: string): Promise<SSHKey> {
  return await api.callRaw<SSHKey>("get_ssh_key", id);
}

/**
 * Update SSH key (metadata only - name, description)
 * @param id - SSH key ID
 * @param request - Update request
 * @returns Updated SSH key
 */
export async function updateSSHKey(
  id: string,
  request: UpdateSSHKeyRequest,
): Promise<SSHKey> {
  return await api.callRaw<SSHKey>("update_ssh_key", id, request);
}

/**
 * Delete SSH key
 * @param id - SSH key ID
 * @param force - Force deletion even if profiles are using it
 */
export async function deleteSSHKey(id: string, force = false): Promise<void> {
  return await api.callRaw<void>("delete_ssh_key", id, force);
}

/**
 * Count how many profiles are using a specific key
 * @param keyId - SSH key ID
 * @returns Number of profiles using the key
 */
export async function countProfilesUsingKey(keyId: string): Promise<number> {
  return await api.callRaw<number>("count_profiles_using_key", keyId);
}

/**
 * Import SSH key from file
 * @param name - Display name for the key
 * @param filePath - Path to the private key file
 * @param passphrase - Optional passphrase if key is encrypted
 * @param description - Optional description
 * @returns Imported SSH key
 */
export async function importSSHKeyFromFile(
  name: string,
  filePath: string,
  passphrase?: string,
  description?: string,
): Promise<SSHKey> {
  return await api.callRaw<SSHKey>(
    "import_ssh_key_from_file",
    name,
    filePath,
    passphrase,
    description,
  );
}
