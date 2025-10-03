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
                  <span
                    class="text-xs px-2 py-1 rounded"
                    :class="getKeyTypeColor(key.keyType)"
                  >
                    {{ key.keyType }}
                  </span>
                  <span
                    v-if="key.passphrase"
                    class="text-xs px-2 py-1 rounded bg-yellow-500/20 text-yellow-400"
                  >
                    Protected
                  </span>
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
                Created: {{ formatDate(key.createdAt) }}
              </div>
              <div v-if="key.lastUsed">
                Last used: {{ formatDate(key.lastUsed) }}
              </div>
            </div>
          </div>
        </Card>
      </div>
    </div>

    <!-- Delete Confirmation Modal -->
    <Modal
      v-if="deleteState.isVisible"
      id="delete-key-confirm"
      title="Delete SSH Key"
      size="md"
      @close="cancelDelete"
    >
      <div class="space-y-4">
        <p class="text-gray-300">
          Are you sure you want to delete the SSH key
          <strong class="text-white">{{ deleteState.key?.name }}</strong>?
        </p>

        <div
          v-if="deleteState.usageCount > 0"
          class="p-4 bg-yellow-500/10 border border-yellow-500/30 rounded-lg"
        >
          <div class="flex items-start gap-2">
            <AlertTriangle class="text-yellow-400 flex-shrink-0 mt-1" :size="20" />
            <div>
              <p class="text-yellow-400 font-semibold">Warning</p>
              <p class="text-gray-300 text-sm mt-1">
                This key is currently used by
                <strong>{{ deleteState.usageCount }}</strong>
                profile(s). Deleting it may affect SSH connections.
              </p>
            </div>
          </div>
        </div>
      </div>

      <template #footer>
        <div class="flex justify-end gap-2">
          <Button variant="secondary" @click="cancelDelete">Cancel</Button>
          <Button
            variant="danger"
            :loading="deleteState.isDeleting"
            @click="executeDelete"
          >
            Delete Key
          </Button>
        </div>
      </template>
    </Modal>
  </Modal>
</template>

<script setup lang="ts">
import { ref, onMounted, watch } from "vue";
import Modal from "../ui/Modal.vue";
import Card from "../ui/Card.vue";
import Button from "../ui/Button.vue";
import { Key, Plus, Edit3, Trash2, AlertTriangle } from "lucide-vue-next";
import { useSshKeyStore } from "../../stores/sshKey";
import { useOverlay } from "../../composables/useOverlay";
import type { SSHKey } from "../../types/ssh";

// Store and composables
const sshKeyStore = useSshKeyStore();
const { openOverlay, isOverlayVisible } = useOverlay();

// State
const deleteState = ref({
  isVisible: false,
  key: null as SSHKey | null,
  usageCount: 0,
  isDeleting: false,
});

// Functions
const openKeyModal = (key?: SSHKey) => {
  openOverlay("ssh-key-modal", { keyId: key?.id || null });
};

const confirmDelete = async (key: SSHKey) => {
  const count = await sshKeyStore.countProfilesUsing(key.id);
  deleteState.value = {
    isVisible: true,
    key,
    usageCount: count,
    isDeleting: false,
  };
};

const cancelDelete = () => {
  deleteState.value = {
    isVisible: false,
    key: null,
    usageCount: 0,
    isDeleting: false,
  };
};

const executeDelete = async () => {
  if (!deleteState.value.key) return;

  deleteState.value.isDeleting = true;
  try {
    await sshKeyStore.deleteKey(deleteState.value.key.id);
    cancelDelete();
  } catch (error) {
    deleteState.value.isDeleting = false;
  }
};

const getKeyTypeColor = (keyType: string): string => {
  const colors: Record<string, string> = {
    RSA: "bg-blue-500/20 text-blue-400",
    Ed25519: "bg-green-500/20 text-green-400",
    ECDSA: "bg-purple-500/20 text-purple-400",
    DSA: "bg-orange-500/20 text-orange-400",
  };
  return colors[keyType] || "bg-gray-500/20 text-gray-400";
};

const formatFingerprint = (fingerprint: string): string => {
  if (!fingerprint) return "";
  // Format: SHA256:ab:cd:ef:12:34:56...
  return fingerprint.length > 60
    ? fingerprint.substring(0, 60) + "..."
    : fingerprint;
};

const formatDate = (dateString: string): string => {
  if (!dateString) return "Never";
  const date = new Date(dateString);
  const now = new Date();
  const diffMs = now.getTime() - date.getTime();
  const diffMins = Math.floor(diffMs / 60000);
  const diffHours = Math.floor(diffMs / 3600000);
  const diffDays = Math.floor(diffMs / 86400000);

  if (diffMins < 1) return "Just now";
  if (diffMins < 60) return `${diffMins}m ago`;
  if (diffHours < 24) return `${diffHours}h ago`;
  if (diffDays < 7) return `${diffDays}d ago`;

  return date.toLocaleDateString();
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
