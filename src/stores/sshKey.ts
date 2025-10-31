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

export const useSshKeyStore = defineStore("sshKey", () => {
  const keys = ref<SSHKey[]>([]);
  const loading = ref(false);

  const keyCount = computed(() => keys.value.length);

  const keysByName = computed(() => {
    return [...keys.value].sort((a, b) => a.name.localeCompare(b.name));
  });

  /**
   * Load all SSH keys from backend
   */
  async function loadKeys(): Promise<void> {
    loading.value = true;
    try {
      keys.value = await sshKeyService.getSSHKeys();
    } catch (error) {
      message.error(`Failed to load SSH keys: ${error}`);
      throw error;
    } finally {
      loading.value = false;
    }
  }

  /**
   * Create a new SSH key
   */
  async function createKey(request: CreateSSHKeyRequest): Promise<SSHKey> {
    loading.value = true;
    try {
      const key = await sshKeyService.createSSHKey(request);
      keys.value.push(key);
      message.success(`SSH key "${key.name}" created successfully`);
      return key;
    } catch (error) {
      message.error(`Failed to create SSH key: ${error}`);
      throw error;
    } finally {
      loading.value = false;
    }
  }

  /**
   * Update an existing SSH key
   */
  async function updateKey(
    id: string,
    request: UpdateSSHKeyRequest,
  ): Promise<SSHKey> {
    loading.value = true;
    try {
      const updatedKey = await sshKeyService.updateSSHKey(id, request);
      const index = keys.value.findIndex((k) => k.id === id);
      if (index !== -1) {
        keys.value[index] = updatedKey;
      }
      message.success(`SSH key "${updatedKey.name}" updated successfully`);
      return updatedKey;
    } catch (error) {
      message.error(`Failed to update SSH key: ${error}`);
      throw error;
    } finally {
      loading.value = false;
    }
  }

  /**
   * Delete an SSH key
   */
  async function deleteKey(id: string, force = false): Promise<void> {
    loading.value = true;
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
      message.error(`Failed to delete SSH key: ${error}`);
      throw error;
    } finally {
      loading.value = false;
    }
  }

  /**
   * Import SSH key from file (file content, not file path)
   * keyType will be auto-detected from key content
   */
  async function importKeyFromFile(
    name: string,
    fileContent: string,
    passphrase?: string,
  ): Promise<SSHKey> {
    loading.value = true;
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
      message.error(`Failed to import SSH key: ${error}`);
      throw error;
    } finally {
      loading.value = false;
    }
  }

  /**
   * Get key by ID
   */
  function getKeyById(id: string): SSHKey | undefined {
    return keys.value.find((k) => k.id === id);
  }

  /**
   * Count profiles using a key
   */
  async function countProfilesUsing(keyId: string): Promise<number> {
    try {
      return await sshKeyService.countProfilesUsingKey(keyId);
    } catch (error) {
      message.error(`Failed to count profiles: ${error}`);
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
      const u3 = await api.listen<{ id: string }>(
        "ssh_key_deleted",
        ({ id }) => removeKey(id),
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
    loading,
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
