import { defineStore } from "pinia";
import { ref, computed } from "vue";
import type {
  SSHKey,
  CreateSSHKeyRequest,
  UpdateSSHKeyRequest,
} from "../types/ssh";
import * as sshKeyService from "../services/sshKey";
import { api } from "../services/api";
import { message } from "../utils/message";
import { handleError, type ErrorContext } from "../utils/errorHandler";

export const useSshKeyStore = defineStore("sshKey", () => {
  const keys = ref<SSHKey[]>([]);
  const isLoading = ref(false);

  const keyCount = computed(() => keys.value.length);

  const keysByName = computed(() => {
    return [...keys.value].sort((a, b) => a.name.localeCompare(b.name));
  });

  /**
   * Load all SSH keys from backend with error handling
   */
  async function loadKeys(): Promise<void> {
    isLoading.value = true;
    const context: ErrorContext = {
      operation: "Load SSH Keys",
    };

    try {
      keys.value = await sshKeyService.getSSHKeys();
    } catch (error) {
      const errorMessage = handleError(error, context);
      message.error(errorMessage);
      throw new Error(errorMessage);
    } finally {
      isLoading.value = false;
    }
  }

  /**
   * Create a new SSH key with error handling
   * @param request - Key creation request
   * @returns Created key
   */
  async function createKey(request: CreateSSHKeyRequest): Promise<SSHKey> {
    isLoading.value = true;
    const context: ErrorContext = {
      operation: "Create SSH Key",
      context: { name: request.name },
    };

    try {
      const key = await sshKeyService.createSSHKey(request);
      keys.value.push(key);
      message.success(`SSH key "${key.name}" created successfully`);
      return key;
    } catch (error) {
      const errorMessage = handleError(error, context);
      message.error(errorMessage);
      throw new Error(errorMessage);
    } finally {
      isLoading.value = false;
    }
  }

  /**
   * Update an existing SSH key with error handling
   * @param id - Key ID to update
   * @param request - Update request
   * @returns Updated key
   */
  async function updateKey(
    id: string,
    request: UpdateSSHKeyRequest,
  ): Promise<SSHKey> {
    isLoading.value = true;
    const context: ErrorContext = {
      operation: "Update SSH Key",
      context: { keyId: id },
    };

    try {
      const updatedKey = await sshKeyService.updateSSHKey(id, request);
      const index = keys.value.findIndex((k) => k.id === id);
      if (index !== -1) {
        keys.value[index] = updatedKey;
      }
      message.success(`SSH key "${updatedKey.name}" updated successfully`);
      return updatedKey;
    } catch (error) {
      const errorMessage = handleError(error, context);
      message.error(errorMessage);
      throw new Error(errorMessage);
    } finally {
      isLoading.value = false;
    }
  }

  /**
   * Delete an SSH key with error handling
   * @param id - Key ID to delete
   * @param force - Force delete even if used by profiles
   */
  async function deleteKey(id: string, force = false): Promise<void> {
    isLoading.value = true;
    const context: ErrorContext = {
      operation: "Delete SSH Key",
      context: { keyId: id, force },
    };

    try {
      const count = await sshKeyService.countProfilesUsingKey(id);

      if (count > 0 && !force) {
        const key = keys.value.find((k) => k.id === id);
        throw new Error(
          `Cannot delete "${key?.name}": ${count} profile(s) are using it`,
        );
      }

      await sshKeyService.deleteSSHKey(id, force);
      keys.value = keys.value.filter((k) => k.id !== id);
      message.success("SSH key deleted successfully");
    } catch (error) {
      const errorMessage = handleError(error, context);
      message.error(errorMessage);
      throw new Error(errorMessage);
    } finally {
      isLoading.value = false;
    }
  }

  /**
   * Import SSH key from file with error handling
   * keyType will be auto-detected from key content
   * @param name - Key name
   * @param fileContent - Key file content
   * @param passphrase - Optional passphrase
   * @returns Imported key
   */
  async function importKeyFromFile(
    name: string,
    fileContent: string,
    passphrase?: string,
  ): Promise<SSHKey> {
    isLoading.value = true;
    const context: ErrorContext = {
      operation: "Import SSH Key",
      context: { name },
    };

    try {
      const key = await sshKeyService.createSSHKey({
        name,
        privateKey: fileContent,
        passphrase,
      });
      keys.value.push(key);
      message.success(`SSH key "${key.name}" imported successfully`);
      return key;
    } catch (error) {
      const errorMessage = handleError(error, context);
      message.error(errorMessage);
      throw new Error(errorMessage);
    } finally {
      isLoading.value = false;
    }
  }

  /**
   * Get key by ID
   */
  function getKeyById(id: string): SSHKey | undefined {
    return keys.value.find((k) => k.id === id);
  }

  /**
   * Count profiles using a key with error handling
   * @param keyId - Key ID to check
   * @returns Number of profiles using the key
   */
  async function countProfilesUsing(keyId: string): Promise<number> {
    const context: ErrorContext = {
      operation: "Count Profiles Using Key",
      context: { keyId },
    };

    try {
      return await sshKeyService.countProfilesUsingKey(keyId);
    } catch (error) {
      const errorMessage = handleError(error, context);
      message.error(errorMessage);
      return 0;
    }
  }

  /**
   * Clear all keys (used on logout)
   */
  function clearKeys(): void {
    keys.value = [];
  }

  const upsertKey = (key: SSHKey) => {
    if (!key?.id) return;
    const i = keys.value.findIndex((k) => k?.id === key.id);
    if (i === -1) {
      keys.value = [...keys.value, key];
    } else {
      keys.value[i] = { ...keys.value[i]!, ...key };
    }
  };

  const removeKey = (id: string) => {
    keys.value = keys.value.filter((k) => k?.id !== id);
  };

  let unsubscribeRealtime: (() => void) | null = null;

  const startRealtime = async (): Promise<void> => {
    if (unsubscribeRealtime) return;
    try {
      const u1 = await api.listen<SSHKey>("ssh_key_created", upsertKey);
      const u2 = await api.listen<SSHKey>("ssh_key_updated", upsertKey);
      const u3 = await api.listen<{ id: string }>("ssh_key_deleted", ({ id }) =>
        removeKey(id),
      );
      unsubscribeRealtime = () => {
        u1();
        u2();
        u3();
      };
    } catch (e) {
      console.error("Failed to subscribe SSH key realtime events:", e);
    }
  };

  const stopRealtime = (): void => {
    if (unsubscribeRealtime) {
      unsubscribeRealtime();
      unsubscribeRealtime = null;
    }
  };

  return {
    keys,
    isLoading,
    keyCount,
    keysByName,
    loadKeys,
    createKey,
    updateKey,
    deleteKey,
    importKeyFromFile,
    getKeyById,
    countProfilesUsing,
    clearKeys,

    startRealtime,
    stopRealtime,
  };
});
