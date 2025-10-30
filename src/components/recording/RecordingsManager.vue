<template>
  <RecordingsModal />
  <PlaybackModal />
</template>

<script setup lang="ts">
import { onMounted } from 'vue';
import RecordingsModal from './RecordingsModal.vue';
import PlaybackModal from './PlaybackModal.vue';
import { useRecordingStore } from '../../stores/recording';

const recordingStore = useRecordingStore();

onMounted(async () => {
  try {
    await recordingStore.loadRecordings();
  } catch (error) {
    console.error('Failed to load recordings:', error);
  }
});
</script>

