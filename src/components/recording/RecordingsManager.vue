<template>
  <RecordingsModal />
  <PlaybackModal />
</template>

<script setup lang="ts">
import { onMounted, onBeforeUnmount } from "vue";
import RecordingsModal from "./RecordingsModal.vue";
import PlaybackModal from "./PlaybackModal.vue";
import { useRecordingStore } from "../../stores/recording";

const recordingStore = useRecordingStore();

/**
 * Initialize recordings feature:
 * - Load all session recordings
 * - Start realtime listeners for live updates
 */
const initialize = async () => {
  try {
    await recordingStore.loadRecordings();
    await recordingStore.startRealtime();
  } catch (error) {
    console.error("Failed to initialize recordings:", error);
  }
};

/**
 * Cleanup when component is unmounted:
 * - Stop realtime listeners to prevent memory leaks
 */
const cleanup = () => {
  recordingStore.stopRealtime();
};

onMounted(() => {
  initialize();
});

onBeforeUnmount(() => {
  cleanup();
});
</script>
