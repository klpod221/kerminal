<template>
  <Modal
    id="ssh-key-manager-modal"
    title="SSH Key Manager"
    size="xl"
  >
    <!-- Empty State -->
    <div
      v-if="sshKeyStore.keys.length === 0 && !sshKeyStore.loading"
      class="text-center py-12"
    >
      <Key :size="64" class="mx-auto text-gray-600 mb-4" />
      <h3 class="text-lg font-semibold text-white mb-2">No SSH Keys</h3>
      <p class="text-gray-400 mb-6">
        Add your first SSH key to securely connect to remote servers
      </p>
      <Button variant="primary" :icon="Plus" @click="openKeyModal()">
        Add SSH Key
      </Button>
    </div>

    <!-- Key List -->
    <div v-else class="space-y-4">
      <!-- Header -->
      <div class="flex items-center justify-between mb-4">
        <div class="text-sm text-gray-400">
          {{ sshKeyStore.keys.length }} key(s) available
        </div>
        <Button variant="primary" :icon="Plus" size="sm" @click="openKeyModal()">
          Add SSH Key
        </Button>
      </div>

      <!-- Loading -->
      <div v-if="sshKeyStore.loading" class="text-center py-8">
        <div class="animate-spin rounded-full h-8 w-8 border-b-2 border-blue-500 mx-auto"></div>
        <p class="text-gray-400 mt-4">Loading SSH keys...</p>
      </div>

      <!-- Keys Grid -->
      <div v-else class="grid grid-cols-1 md:grid-cols-2 gap-4">
        <Card
          v-for="key in sshKeyStore.keys"
          :key="key.id"
          :hover="true"
          class="relative"
        >
          <div class="space-y-3">
            <!-- Header -->
            <div class="flex items-start justify-between">
              <div class="flex-1 min-w-0">
                <h3 class="text-white font-semibold truncate">{{ key.name }}</h3>
                <div class="flex items-center gap-2 mt-1">
                  <Badge variant="info" size="xs">
                    {{ key.keyType }}
                  </Badge>
                  <Badge
                    v-if="key.passphrase"
                    variant="warning"
                    size="xs"
                  >
                    Protected
                  </Badge>
                </div>
              </div>
              <div class="flex items-center gap-1">
                <Button
                  variant="ghost"
                  size="sm"
                  :icon="Edit3"
                  title="Edit key"
                  @click="openKeyModal(key)"
                />
                <Button
                  variant="ghost"
                  size="sm"
                  :icon="Trash2"
                  title="Delete key"
                  @click="confirmDelete(key)"
                />
              </div>
            </div>

            <!-- Fingerprint -->
            <div class="space-y-1">
              <div class="text-xs text-gray-400">Fingerprint:</div>
              <div
                class="text-xs font-mono text-gray-300 bg-gray-800 px-2 py-1 rounded break-all"
              >
                {{ formatFingerprint(key.fingerprint) }}
              </div>
            </div>

            <!-- Description -->
            <div v-if="key.description" class="text-sm text-gray-400">
              {{ key.description }}
            </div>

            <!-- Footer -->
            <div class="flex items-center justify-between text-xs text-gray-500 pt-2 border-t border-gray-700">
              <div>
                Created: {{ formatDateOrNever(key.createdAt) }}
              </div>
              <div class="flex items-center gap-2">
                <div v-if="key.lastUsed" class="text-green-400">
                  Last used: {{ formatDateOrNever(key.lastUsed) }}
                </div>
                <div v-else class="text-gray-500">
                  Never used
                </div>
              </div>
            </div>
          </div>
        </Card>
      </div>
    </div>


  </Modal>
</template>

<script setup lang="ts">
import { onMounted, watch } from "vue";
import Modal from "../ui/Modal.vue";
import { formatFingerprint, formatDateOrNever } from "../../utils/formatter";
import Card from "../ui/Card.vue";
import Badge from "../ui/Badge.vue";
import Button from "../ui/Button.vue";
import { Key, Plus, Edit3, Trash2 } from "lucide-vue-next";
import { useSshKeyStore } from "../../stores/sshKey";
import { useOverlay } from "../../composables/useOverlay";
import type { SSHKey } from "../../types/ssh";

// Store and composables
const sshKeyStore = useSshKeyStore();
const { openOverlay, isOverlayVisible } = useOverlay();



// Functions
const openKeyModal = (key?: SSHKey) => {
  openOverlay("ssh-key-modal", { keyId: key?.id || null });
};

const confirmDelete = async (key: SSHKey) => {
  const count = await sshKeyStore.countProfilesUsing(key.id);

  let confirmMessage = `Are you sure you want to delete the SSH key "${key.name}"?`;
  if (count > 0) {
    confirmMessage += `\n\nWarning: This key is currently used by ${count} profile(s). Deleting it may affect SSH connections.`;
  }

  const confirmed = confirm(confirmMessage);

  if (confirmed) {
    try {
      await sshKeyStore.deleteKey(key.id);
    } catch (error) {
      console.error("Failed to delete SSH key:", error);
    }
  }
};

// Load keys on mount
onMounted(() => {
  sshKeyStore.loadKeys();
});

// Watch for ssh-key-modal visibility to reload keys when it closes
watch(
  () => isOverlayVisible("ssh-key-modal"),
  (isVisible, wasVisible) => {
    // Reload keys when modal closes (was visible, now not visible)
    if (wasVisible && !isVisible) {
      sshKeyStore.loadKeys();
    }
  }
);
</script>
