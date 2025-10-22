<template>
  <div>
    <div
      v-if="isLoading"
      class="flex items-center justify-center py-16 text-gray-400"
    >
      <RefreshCw class="w-6 h-6 animate-spin mr-3" />
      Loading conflicts...
    </div>

    <div v-else-if="!conflicts.length" class="text-center py-16">
      <div class="flex justify-center mb-4">
        <CheckCircle2 class="w-16 h-16 text-green-500" />
      </div>
      <h3 class="text-lg font-semibold text-white mb-2">
        No Pending Conflicts
      </h3>
      <p class="text-sm text-gray-400">
        All sync conflicts have been resolved
      </p>
    </div>

    <div v-else class="space-y-3">
      <div class="flex items-center justify-between mb-4">
        <div class="flex items-center gap-2">
          <AlertTriangle class="w-5 h-5 text-yellow-500" />
          <h3 class="text-base font-semibold text-white">Pending Conflicts</h3>
          <Badge variant="warning">{{ conflicts.length }}</Badge>
        </div>
        <Button
          variant="ghost"
          size="sm"
          :icon="RefreshCw"
          :loading="isLoading"
          @click="refresh"
        >
          Refresh
        </Button>
      </div>

      <div class="space-y-2">
        <div
          v-for="conflict in conflicts"
          :key="conflict.id"
          class="bg-gray-800/50 border border-yellow-700/30 rounded-lg p-4 hover:border-yellow-600/50 transition-colors"
        >
          <div class="flex items-start justify-between gap-4">
            <div class="flex-1 min-w-0">
              <div class="flex items-center gap-2 mb-2">
                <FileWarning class="w-4 h-4 text-yellow-500 flex-shrink-0" />
                <h4 class="text-sm font-medium text-gray-100 truncate">
                  {{ formatEntityType(conflict.entityType) }}
                </h4>
                <Badge variant="warning" size="sm">Unresolved</Badge>
              </div>
              <div class="text-xs text-gray-400 space-y-1">
                <div class="flex items-center gap-1.5">
                  <Hash class="w-3 h-3" />
                  <span class="font-mono truncate">{{ conflict.entityId }}</span>
                </div>
                <div class="flex items-center gap-1.5">
                  <Clock class="w-3 h-3" />
                  <span>{{ formatDateOrNever(conflict.createdAt) }}</span>
                </div>
              </div>
            </div>
            <Button
              variant="primary"
              size="sm"
              :icon="GitMerge"
              @click="resolveConflict(conflict.id)"
            >
              Resolve
            </Button>
          </div>
        </div>
      </div>
    </div>
  </div>
</template>

<script setup lang="ts">
import { ref, onMounted, computed } from "vue";
import {
  RefreshCw,
  CheckCircle2,
  AlertTriangle,
  FileWarning,
  Hash,
  Clock,
  GitMerge,
} from "lucide-vue-next";
import Badge from "../ui/Badge.vue";
import Button from "../ui/Button.vue";
import { useSyncStore } from "../../stores/sync";
import { useOverlay } from "../../composables/useOverlay";
import { formatDateOrNever } from "../../utils/formatter";
import { message } from "../../utils/message";
import { getErrorMessage } from "../../utils/helpers";

const syncStore = useSyncStore();
const { openOverlay } = useOverlay();

const isLoading = ref(false);
const conflicts = computed(() => syncStore.pendingConflicts);

const formatEntityType = (type: string): string => {
  const types: Record<string, string> = {
    ssh_profile: "SSH Profile",
    ssh_group: "SSH Group",
    ssh_key: "SSH Key",
    saved_command: "Saved Command",
    saved_command_group: "Command Group",
  };
  return types[type] || type;
};

const loadConflicts = async () => {
  isLoading.value = true;
  try {
    await syncStore.loadConflicts();
  } catch (error) {
    console.error("Failed to load conflicts:", error);
    message.error(getErrorMessage(error, "Failed to load conflicts"));
  } finally {
    isLoading.value = false;
  }
};

const refresh = () => {
  loadConflicts();
};

const resolveConflict = (conflictId: string) => {
  openOverlay("conflict-resolution-modal", { conflictId });
};

onMounted(() => {
  loadConflicts();
});

defineExpose({ loadConflicts });
</script>
