<template>
  <!-- Empty State -->
  <EmptyState
    v-if="recordingStore.recordings.length === 0 && !recordingStore.isLoading"
    :icon="Video"
    :icon-size="64"
    title="No Session Recordings"
    description="Start recording a terminal session to see it here. Click the record button in any terminal tab."
  />

  <!-- Recordings List -->
  <div v-else class="space-y-4">
    <!-- Header -->
    <div class="flex items-center justify-between mb-4">
      <div class="flex items-center gap-4">
        <div class="text-sm text-gray-400">
          {{ recordingStore.recordings.length }} recording(s) saved
        </div>
        <div class="text-xs text-gray-500">
          Total: {{ formatTotalSize() }}
        </div>
      </div>
      <Input
        id="search-recordings"
        v-model="searchQuery"
        type="search"
        placeholder="Search recordings..."
        :left-icon="Search"
        :helper="false"
        class="max-w-xs"
        size="sm"
      />
    </div>

    <!-- Loading -->
    <div v-if="recordingStore.isLoading" class="text-center py-8">
      <div
        class="animate-spin rounded-full h-8 w-8 border-b-2 border-blue-500 mx-auto"
      ></div>
      <p class="text-gray-400 mt-4">Loading recordings...</p>
    </div>

    <!-- Recordings Grid -->
    <div v-else class="grid grid-cols-1 md:grid-cols-2 gap-4">
      <Card
        v-for="recording in filteredRecordings"
        :key="recording.id"
        :hover="true"
        class="relative"
      >
        <div class="space-y-3">
          <!-- Header -->
          <div class="flex items-start justify-between">
            <div class="flex-1 min-w-0">
              <h3 class="text-white font-semibold truncate">
                {{ recording.sessionName }}
              </h3>
              <div class="flex items-center gap-2 mt-1">
                <Badge variant="info" size="xs">
                  {{ recording.terminalType }}
                </Badge>
                <Badge variant="gray" size="xs">
                  {{ formatDuration(recording.durationMs) }}
                </Badge>
              </div>
            </div>
            <div class="flex items-center gap-1">
              <Button
                variant="ghost"
                size="sm"
                :icon="Play"
                title="Play recording"
                @click="handlePlay(recording)"
              />
              <Button
                variant="ghost"
                size="sm"
                :icon="Download"
                title="Export recording"
                @click="handleExport(recording)"
              />
              <Button
                variant="ghost"
                size="sm"
                :icon="Trash2"
                title="Delete recording"
                @click="handleDelete(recording)"
              />
            </div>
          </div>

          <!-- Details -->
          <div class="space-y-1">
            <div class="flex items-center justify-between text-xs text-gray-400">
              <div class="flex items-center gap-1">
                <Clock :size="12" />
                <span>{{ formatDate(recording.startedAt) }}</span>
              </div>
              <div class="flex items-center gap-1">
                <HardDrive :size="12" />
                <span>{{ formatFileSize(recording.fileSize) }}</span>
              </div>
            </div>
          </div>

          <!-- Footer -->
          <div
            class="flex items-center justify-between text-xs text-gray-500 pt-2 border-t border-gray-700"
          >
            <div>{{ recording.width }}x{{ recording.height }}</div>
            <div class="text-gray-400">
              {{ formatRelativeTime(recording.startedAt) }}
            </div>
          </div>
        </div>
      </Card>
    </div>
  </div>
</template>

<script setup lang="ts">
import { ref, computed, onMounted } from 'vue';
import { Video, Download, Trash2, Search, Clock, HardDrive, Play } from 'lucide-vue-next';
import Card from '../ui/Card.vue';
import Badge from '../ui/Badge.vue';
import Button from '../ui/Button.vue';
import Input from '../ui/Input.vue';
import EmptyState from '../ui/EmptyState.vue';
import { useRecordingStore } from '../../stores/recording';
import { useOverlay } from '../../composables/useOverlay';
import { message, showConfirm } from '../../utils/message';
import type { SessionRecording } from '../../types/recording';
import { save } from '@tauri-apps/plugin-dialog';

const recordingStore = useRecordingStore();
const { openOverlay } = useOverlay();
const searchQuery = ref('');

const filteredRecordings = computed(() => {
  if (!searchQuery.value) return recordingStore.recordings;
  const query = searchQuery.value.toLowerCase();
  return recordingStore.recordings.filter(
    (r) =>
      r.sessionName.toLowerCase().includes(query) ||
      r.terminalType.toLowerCase().includes(query)
  );
});

function formatDate(dateStr: string) {
  const date = new Date(dateStr);
  return date.toLocaleDateString('en-US', {
    month: 'short',
    day: 'numeric',
    year: 'numeric',
    hour: '2-digit',
    minute: '2-digit',
  });
}

function formatRelativeTime(dateStr: string) {
  const date = new Date(dateStr);
  const now = new Date();
  const diffMs = now.getTime() - date.getTime();
  const diffMins = Math.floor(diffMs / 60000);
  const diffHours = Math.floor(diffMins / 60);
  const diffDays = Math.floor(diffHours / 24);

  if (diffMins < 1) return 'Just now';
  if (diffMins < 60) return `${diffMins}m ago`;
  if (diffHours < 24) return `${diffHours}h ago`;
  if (diffDays < 7) return `${diffDays}d ago`;
  return date.toLocaleDateString();
}

function formatDuration(ms?: number) {
  if (!ms) return '0:00';
  const totalSeconds = Math.floor(ms / 1000);
  const hours = Math.floor(totalSeconds / 3600);
  const minutes = Math.floor((totalSeconds % 3600) / 60);
  const seconds = totalSeconds % 60;

  if (hours > 0) {
    return `${hours}h ${minutes}m`;
  }
  return `${minutes}:${seconds.toString().padStart(2, '0')}`;
}

function formatFileSize(bytes: number) {
  if (bytes < 1024) return `${bytes} B`;
  if (bytes < 1024 * 1024) return `${(bytes / 1024).toFixed(1)} KB`;
  return `${(bytes / (1024 * 1024)).toFixed(1)} MB`;
}

function formatTotalSize() {
  const total = recordingStore.recordings.reduce((sum, r) => sum + r.fileSize, 0);
  return formatFileSize(total);
}

function handlePlay(recording: SessionRecording) {
  openOverlay('playback-modal', { recordingId: recording.id });
}

async function handleExport(recording: SessionRecording) {
  try {
    // Use Tauri dialog to select save location
    const filePath = await save({
      defaultPath: `${recording.sessionName.replace(/[^a-z0-9]/gi, '_')}.cast`,
      filters: [{
        name: 'Asciicast Recording',
        extensions: ['cast']
      }]
    });

    if (!filePath) {
      // User cancelled
      return;
    }

    // Use backend to copy file
    await recordingStore.exportRecording(recording.id, filePath);
    message.success('Recording exported successfully');
  } catch (error) {
    console.error('Failed to export recording:', error);
    message.error('Failed to export recording');
  }
}

async function handleDelete(recording: SessionRecording) {
  const confirmed = await showConfirm(
    "Delete Recording",
    `Are you sure you want to delete "${recording.sessionName}"?\n\nThis action cannot be undone.`,
  );
  if (confirmed) {
    deleteRecording(recording);
  }
}

async function deleteRecording(recording: SessionRecording) {
  try {
    await recordingStore.deleteRecording(recording.id);
    message.success('Recording deleted successfully');
  } catch (error) {
    console.error('Failed to delete recording:', error);
    message.error('Failed to delete recording');
  }
}

onMounted(() => {
  recordingStore.loadRecordings();
});
</script>

