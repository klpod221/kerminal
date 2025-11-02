<template>
  <TunnelList />
  <TunnelModal />
</template>

<script setup lang="ts">
import { onMounted, onBeforeUnmount } from "vue";
import TunnelList from "./TunnelList.vue";
import TunnelModal from "./TunnelModal.vue";
import { useTunnelStore } from "../../stores/tunnel";

const tunnelStore = useTunnelStore();

/**
 * Initialize tunnels feature:
 * - Load all tunnels
 * - Start realtime listeners for live updates
 */
const initialize = async () => {
  try {
    await tunnelStore.loadTunnels();
    await tunnelStore.startRealtime();
  } catch (error) {
    console.error("Failed to initialize tunnels:", error);
  }
};

/**
 * Cleanup when component is unmounted:
 * - Stop realtime listeners to prevent memory leaks
 */
const cleanup = () => {
  tunnelStore.stopRealtime();
};

onMounted(() => {
  initialize();
});

onBeforeUnmount(() => {
  cleanup();
});
</script>
