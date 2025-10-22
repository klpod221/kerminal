<template>
  <div class="flex items-center gap-1">
    <div class="w-2 h-2 rounded-full" :class="statusColor"></div>
    <span class="text-xs capitalize" :class="textColor">
      {{ status }}
    </span>
  </div>
</template>

<script setup lang="ts">
import { computed } from "vue";
import type { TunnelStatus } from "../../types/tunnel";

interface Props {
  status: TunnelStatus;
}

const props = defineProps<Props>();

const statusColor = computed(() => {
  switch (props.status) {
    case "running":
      return "bg-green-500";
    case "stopped":
      return "bg-gray-500";
    case "error":
      return "bg-red-500";
    default:
      return "bg-gray-500";
  }
});

const textColor = computed(() => {
  switch (props.status) {
    case "running":
      return "text-green-400";
    case "stopped":
      return "text-gray-400";
    case "error":
      return "text-red-400";
    default:
      return "text-gray-400";
  }
});
</script>
