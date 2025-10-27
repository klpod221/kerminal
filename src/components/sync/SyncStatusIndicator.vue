<template>
  <div
    class="flex items-center gap-2 text-xs"
  >
    <Badge :variant="statusVariant" :size="'sm'" class="animate-pulse-subtle">
      <div class="flex items-center gap-1.5">
        <component :is="Database" :size="12" />
        <span>{{ syncStore.currentDatabase.name }}</span>
      </div>
    </Badge>
  </div>
</template>

<script setup lang="ts">
import { computed } from "vue";
import { Database } from "lucide-vue-next";
import Badge from "../ui/Badge.vue";
import { useSyncStore } from "../../stores/sync";

const syncStore = useSyncStore();

const statusVariant = computed(() => {
  if (!syncStore.currentDatabase) return "gray";

  if (syncStore.currentDatabase.isActive) {
    return "success";
  }

  return "gray";
});
</script>

<style scoped>
@keyframes pulse-subtle {
  0%,
  100% {
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
