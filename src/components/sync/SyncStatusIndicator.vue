<template>
  <div
    v-if="syncStore.databases.length > 0"
    class="flex items-center gap-2 text-xs"
  >
    <Badge
      :variant="statusVariant"
      :size="'sm'"
      class="animate-pulse-subtle"
    >
      <div class="flex items-center gap-1.5">
        <component
          :is="statusIcon"
          :size="12"
          :class="statusIconClass"
        />
        <span>{{ statusText }}</span>
      </div>
    </Badge>

    <div
      v-if="syncStore.currentDatabase"
      class="text-gray-400 flex items-center gap-1"
    >
      <Database :size="12" />
      <span class="max-w-[100px] truncate">
        {{ syncStore.currentDatabase.name }}
      </span>
    </div>
  </div>
</template>

<script setup lang="ts">
import { computed } from "vue";
import { Database, CloudOff, Cloud, RefreshCw, AlertTriangle } from "lucide-vue-next";
import Badge from "../ui/Badge.vue";
import { useSyncStore } from "../../stores/sync";

const syncStore = useSyncStore();

const statusVariant = computed(() => {
  if (!syncStore.currentDatabase) return "gray";

  const status = syncStore.currentDatabase.syncStatus;

  switch (status) {
    case "Connected":
      return "success";
    case "Syncing":
      return "info";
    case "Error":
      return "danger";
    case "Disconnected":
      return "gray";
    default:
      return "gray";
  }
});

const statusIcon = computed(() => {
  if (!syncStore.currentDatabase) return CloudOff;

  const status = syncStore.currentDatabase.syncStatus;

  switch (status) {
    case "Connected":
      return Cloud;
    case "Syncing":
      return RefreshCw;
    case "Error":
      return AlertTriangle;
    case "Disconnected":
      return CloudOff;
    default:
      return CloudOff;
  }
});

const statusIconClass = computed(() => {
  if (!syncStore.currentDatabase) return "";

  const status = syncStore.currentDatabase.syncStatus;

  if (status === "Syncing") {
    return "animate-spin-slow";
  }

  return "";
});

const statusText = computed(() => {
  if (!syncStore.currentDatabase) return "No Database";

  const status = syncStore.currentDatabase.syncStatus;

  switch (status) {
    case "Connected":
      return "Connected";
    case "Syncing":
      return "Syncing";
    case "Error":
      return "Error";
    case "Disconnected":
      return "Disconnected";
    default:
      return "Unknown";
  }
});
</script>

<style scoped>
@keyframes pulse-subtle {
  0%, 100% {
    opacity: 1;
  }
  50% {
    opacity: 0.8;
  }
}

@keyframes spin-slow {
  from {
    transform: rotate(0deg);
  }
  to {
    transform: rotate(360deg);
  }
}

.animate-pulse-subtle {
  animation: pulse-subtle 3s ease-in-out infinite;
}

.animate-spin-slow {
  animation: spin-slow 2s linear infinite;
}
</style>
