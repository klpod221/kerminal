<template>
  <SavedCommandDrawer />

  <SavedCommandModal />
  <SavedCommandGroupModal />
</template>

<script setup lang="ts">
import { onMounted, onBeforeUnmount } from "vue";
import SavedCommandDrawer from "./SavedCommandDrawer.vue";
import SavedCommandModal from "./SavedCommandModal.vue";
import SavedCommandGroupModal from "./SavedCommandGroupModal.vue";
import { useSavedCommandStore } from "../../stores/savedCommand";

const savedCommandStore = useSavedCommandStore();

/**
 * Initialize saved commands feature:
 * - Load all commands and groups
 * - Start realtime listeners for live updates
 */
const initialize = async () => {
  try {
    await savedCommandStore.loadAll();
    await savedCommandStore.startRealtime();
  } catch (error) {
    console.error("Failed to initialize saved commands:", error);
  }
};

/**
 * Cleanup when component is unmounted:
 * - Stop realtime listeners to prevent memory leaks
 */
const cleanup = () => {
  savedCommandStore.stopRealtime();
};

onMounted(() => {
  initialize();
});

onBeforeUnmount(() => {
  cleanup();
});
</script>
