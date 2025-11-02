<template>
  <Modal id="conflict-resolution-modal" title="Resolve Sync Conflict" size="lg">
    <div v-if="!currentConflict" class="text-gray-400 text-center py-8">
      No conflict selected
    </div>

    <div v-else class="space-y-4">
      <!-- Conflict Info -->
      <div class="bg-gray-800 rounded-lg p-4 border border-gray-700">
        <div class="text-sm space-y-2">
          <div>
            <span class="text-gray-400">Entity Type:</span>
            <span class="text-gray-200 ml-2">{{ entityLabel }}</span>
          </div>
          <div>
            <span class="text-gray-400">Entity ID:</span>
            <span class="text-gray-200 ml-2 font-mono text-xs">{{
              currentConflict.entityId
            }}</span>
          </div>
        </div>
      </div>

      <!-- Warning -->
      <div class="bg-yellow-900/20 border border-yellow-700/50 rounded-lg p-4">
        <div class="flex items-start gap-2">
          <div class="text-yellow-500">⚠️</div>
          <div class="flex-1">
            <h4 class="text-sm font-medium text-yellow-200 mb-1">
              Conflict Detected
            </h4>
            <p class="text-xs text-yellow-100/80">
              Both local and remote versions have been modified. Choose which to
              keep.
            </p>
          </div>
        </div>
      </div>

      <!-- Data Comparison -->
      <div class="grid grid-cols-2 gap-4">
        <!-- Local -->
        <div class="border border-gray-700 rounded-lg overflow-hidden">
          <div class="bg-blue-900/30 border-b border-blue-700/50 px-4 py-2">
            <h4 class="text-sm font-medium text-blue-200">Local Version</h4>
          </div>
          <div class="p-4 bg-gray-800/50">
            <pre
              class="text-xs text-gray-300 whitespace-pre-wrap max-h-64 overflow-auto"
              >{{ localDataStr }}</pre
            >
          </div>
        </div>

        <!-- Remote -->
        <div class="border border-gray-700 rounded-lg overflow-hidden">
          <div class="bg-green-900/30 border-b border-green-700/50 px-4 py-2">
            <h4 class="text-sm font-medium text-green-200">Remote Version</h4>
          </div>
          <div class="p-4 bg-gray-800/50">
            <pre
              class="text-xs text-gray-300 whitespace-pre-wrap max-h-64 overflow-auto"
              >{{ remoteDataStr }}</pre
            >
          </div>
        </div>
      </div>

      <!-- Resolution Actions -->
      <div class="space-y-3">
        <div class="text-xs text-gray-400 font-medium uppercase tracking-wide">
          Choose Resolution Strategy
        </div>
        <div class="grid grid-cols-2 gap-2">
          <Button
            variant="primary"
            size="sm"
            :loading="isResolving && choice === 'LocalWins'"
            :disabled="isResolving"
            @click="handleResolveWithStrategy('LocalWins')"
          >
            Keep Local
          </Button>
          <Button
            variant="primary"
            size="sm"
            :loading="isResolving && choice === 'RemoteWins'"
            :disabled="isResolving"
            @click="handleResolveWithStrategy('RemoteWins')"
          >
            Keep Remote
          </Button>
          <Button
            variant="outline"
            size="sm"
            :loading="isResolving && choice === 'LastWriteWins'"
            :disabled="isResolving"
            @click="handleResolveWithStrategy('LastWriteWins')"
          >
            Use Latest (Last Write)
          </Button>
          <Button
            variant="outline"
            size="sm"
            :loading="isResolving && choice === 'FirstWriteWins'"
            :disabled="isResolving"
            @click="handleResolveWithStrategy('FirstWriteWins')"
          >
            Use Oldest (First Write)
          </Button>
        </div>
      </div>
    </div>
  </Modal>
</template>

<script setup lang="ts">
import { ref, computed, watch } from "vue";
import Modal from "../ui/Modal.vue";
import Button from "../ui/Button.vue";
import { message } from "../../utils/message";
import { useSyncStore } from "../../stores/sync";
import { useOverlay } from "../../composables/useOverlay";
import type { ConflictResolutionStrategy } from "../../types/sync";

const props = defineProps<{
  conflictId?: string | null;
}>();

const syncStore = useSyncStore();
const { closeOverlay, getOverlayProp } = useOverlay();

const conflictId = getOverlayProp(
  "conflict-resolution-modal",
  "conflictId",
  props.conflictId,
  null,
);

const isResolving = ref(false);
const choice = ref<ConflictResolutionStrategy | null>(null);

const currentConflict = computed(() => {
  if (!conflictId.value) return null;
  return syncStore.conflicts.find((c) => c.id === conflictId.value);
});

const entityLabel = computed(() => {
  if (!currentConflict.value) return "";
  const labels: Record<string, string> = {
    ssh_profile: "SSH Profile",
    ssh_group: "SSH Group",
    ssh_key: "SSH Key",
  };
  return (
    labels[currentConflict.value.entityType] || currentConflict.value.entityType
  );
});

const localDataStr = computed(() => {
  if (!currentConflict.value) return "";
  try {
    return JSON.stringify(currentConflict.value.localData, null, 2);
  } catch {
    return String(currentConflict.value.localData);
  }
});

const remoteDataStr = computed(() => {
  if (!currentConflict.value) return "";
  try {
    return JSON.stringify(currentConflict.value.remoteData, null, 2);
  } catch {
    return String(currentConflict.value.remoteData);
  }
});

const handleResolveWithStrategy = async (
  strategy: ConflictResolutionStrategy,
) => {
  if (!conflictId.value) return;

  isResolving.value = true;
  choice.value = strategy;

  if (strategy === "LocalWins") {
    await syncStore.resolveConflict(conflictId.value, "local");
  } else if (strategy === "RemoteWins") {
    await syncStore.resolveConflict(conflictId.value, "remote");
  } else {
    await syncStore.resolveConflictResolution(conflictId.value, strategy);
    syncStore.conflicts = syncStore.conflicts.filter(
      (c) => c.id !== conflictId.value,
    );
  }

  const strategyLabel =
    {
      LocalWins: "Local",
      RemoteWins: "Remote",
      LastWriteWins: "Latest (Last Write)",
      FirstWriteWins: "Oldest (First Write)",
      Manual: "Manual",
    }[strategy] || strategy;

  message.success(`Conflict resolved: ${strategyLabel} version kept`);
  closeOverlay("conflict-resolution-modal");
  isResolving.value = false;
  choice.value = null;
};

watch(
  () => conflictId.value,
  () => {
    isResolving.value = false;
    choice.value = null;
  },
);
</script>
