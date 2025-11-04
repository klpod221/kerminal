<template>
  <Modal id="ssh-key-manager-modal" title="SSH Key Manager" size="xl">
    <!-- Empty State -->
    <EmptyState
      v-if="sshKeyStore.keys.length === 0 && !sshKeyStore.isLoading"
      :icon="Key"
      :icon-size="64"
      title="No SSH Keys"
      description="Add your first SSH key to securely connect to remote servers"
      action-text="Add SSH Key"
      :action-icon="Plus"
      action-variant="primary"
      @action="openKeyModal()"
    />

    <!-- Key List -->
    <div v-else class="space-y-4">
      <!-- Header -->
      <div class="flex items-center justify-between mb-4">
        <div class="text-sm text-gray-400">
          {{ sshKeyStore.keys.length }} key(s) available
        </div>
        <Button
          variant="primary"
          :icon="Plus"
          size="sm"
          @click="openKeyModal()"
        >
          Add SSH Key
        </Button>
      </div>

      <!-- Loading -->
      <SkeletonList
        v-if="sshKeyStore.isLoading"
        :items="4"
        :show-avatar="false"
        :show-actions="true"
      />

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
                <h3 class="text-white font-semibold truncate">
                  {{ key.name }}
                </h3>
                <div class="flex items-center gap-2 mt-1">
                  <Badge variant="info" size="xs">
                    {{ key.keyType }}
                  </Badge>
                  <Badge v-if="key.passphrase" variant="warning" size="xs">
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
            <div
              class="flex items-center justify-between text-xs text-gray-500 pt-2 border-t border-gray-700"
            >
              <div>Created: {{ formatDateOrNever(key.createdAt) }}</div>
              <div class="flex items-center gap-2">
                <div v-if="key.lastUsed" class="text-green-400">
                  Last used: {{ formatDateOrNever(key.lastUsed) }}
                </div>
                <div v-else class="text-gray-500">Never used</div>
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
import EmptyState from "../ui/EmptyState.vue";
import SkeletonList from "../ui/SkeletonList.vue";
import { Key, Plus, Edit3, Trash2 } from "lucide-vue-next";
import { useSshKeyStore } from "../../stores/sshKey";
import { useOverlay } from "../../composables/useOverlay";
import type { SSHKey } from "../../types/ssh";
import { showConfirm } from "../../utils/message";

const sshKeyStore = useSshKeyStore();
const { openOverlay, isOverlayVisible } = useOverlay();

const openKeyModal = (key?: SSHKey) => {
  openOverlay("ssh-key-modal", { keyId: key?.id || null });
};

const confirmDelete = async (key: SSHKey) => {
  const count = await sshKeyStore.countProfilesUsing(key.id);

  let confirmMessage = `Are you sure you want to delete the SSH key "${key.name}"?`;
  if (count > 0) {
    confirmMessage += `\n\nWarning: This key is currently used by ${count} profile(s). Deleting it may affect SSH connections.`;
  }

  const confirmed = await showConfirm("Delete SSH Key", confirmMessage);

  if (confirmed) {
    try {
      await sshKeyStore.deleteKey(key.id);
    } catch (error) {
      console.error("Failed to delete SSH key:", error);
    }
  }
};

onMounted(() => {
  sshKeyStore.loadKeys();
});

watch(
  () => isOverlayVisible("ssh-key-modal"),
  (isVisible, wasVisible) => {
    if (wasVisible && !isVisible) {
      sshKeyStore.loadKeys();
    }
  },
);
</script>
