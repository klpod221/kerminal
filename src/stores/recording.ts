import { defineStore } from 'pinia';
import { ref, computed } from 'vue';
import * as recordingService from '../services/recording';
import type { SessionRecording } from '../types/recording';

export const useRecordingStore = defineStore('recording', () => {
  const recordings = ref<SessionRecording[]>([]);
  const activeRecordings = ref<Map<string, string>>(new Map()); // terminalId -> recordingId
  const isLoading = ref(false);

  const isRecording = computed(() => (terminalId: string) => {
    return activeRecordings.value.has(terminalId);
  });

  async function loadRecordings() {
    isLoading.value = true;
    try {
      recordings.value = await recordingService.listRecordings();
    } catch (error) {
      console.error('Failed to load recordings:', error);
    } finally {
      isLoading.value = false;
    }
  }

  async function startRecording(terminalId: string, name?: string, width?: number, height?: number) {
    try {
      const recordingId = await recordingService.startRecording(terminalId, name, width, height);
      activeRecordings.value.set(terminalId, recordingId);
      return recordingId;
    } catch (error) {
      console.error('Failed to start recording:', error);
      throw error;
    }
  }

  async function stopRecording(terminalId: string) {
    try {
      const recording = await recordingService.stopRecording(terminalId);
      activeRecordings.value.delete(terminalId);
      recordings.value.unshift(recording);
      return recording;
    } catch (error) {
      console.error('Failed to stop recording:', error);
      throw error;
    }
  }

  async function deleteRecording(recordingId: string) {
    try {
      await recordingService.deleteRecording(recordingId);
      recordings.value = recordings.value.filter(r => r.id !== recordingId);
    } catch (error) {
      console.error('Failed to delete recording:', error);
      throw error;
    }
  }

  async function exportRecording(recordingId: string, path: string) {
    try {
      return await recordingService.exportRecording(recordingId, path);
    } catch (error) {
      console.error('Failed to export recording:', error);
      throw error;
    }
  }

  return {
    recordings,
    activeRecordings,
    isLoading,
    isRecording,
    loadRecordings,
    startRecording,
    stopRecording,
    deleteRecording,
    exportRecording,
  };
});

