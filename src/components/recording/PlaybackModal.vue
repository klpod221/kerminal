<template>
  <Modal
    id="playback-modal"
    :title="currentRecording?.sessionName || 'Playback Recording'"
    size="2xl"
    :icon="Play"
  >
    <div class="flex flex-col h-full space-y-4">
      <!-- Recording Info Header -->
      <div
        v-if="currentRecording"
        class="flex items-center justify-between pb-3 border-b border-gray-700/50"
      >
        <div class="flex items-center gap-3">
          <div
            class="flex items-center justify-center w-10 h-10 rounded-lg bg-blue-500/10"
          >
            <Play :size="20" class="text-blue-400" />
          </div>
          <div>
            <div class="flex items-center gap-2">
              <Badge variant="info" size="xs">
                {{ currentRecording.terminalType }}
              </Badge>
              <span class="text-xs text-gray-500">•</span>
              <span class="text-xs text-gray-400"
                >{{ currentRecording.width }}×{{
                  currentRecording.height
                }}</span
              >
            </div>
            <div class="flex items-center gap-2 mt-1 text-xs text-gray-500">
              <Clock :size="12" />
              <span>{{ formatDate(currentRecording.startedAt) }}</span>
            </div>
          </div>
        </div>
        <div class="text-right">
          <div class="text-sm font-medium text-white">
            {{ formatDuration(currentRecording.durationMs) }}
          </div>
          <div class="text-xs text-gray-500">
            {{ formatFileSize(currentRecording.fileSize) }}
          </div>
        </div>
      </div>

      <!-- Asciinema Player -->
      <div
        class="flex-1 bg-black rounded-lg overflow-hidden border border-gray-700/50 shadow-xl"
      >
        <div ref="playerRef" class="w-full h-full"></div>
      </div>
    </div>
  </Modal>
</template>

<script setup lang="ts">
import { ref, onUnmounted, watch } from "vue";
import { Play, Clock } from "lucide-vue-next";
import * as AsciinemaPlayer from "asciinema-player";
import Modal from "../ui/Modal.vue";
import Badge from "../ui/Badge.vue";
import { useOverlay } from "../../composables/useOverlay";
import { useRecordingStore } from "../../stores/recording";
import type { SessionRecording } from "../../types/recording";
import "asciinema-player/dist/bundle/asciinema-player.css";

const { getOverlayProp, isOverlayVisible } = useOverlay();
const recordingStore = useRecordingStore();

const recordingId = getOverlayProp<string>(
  "playback-modal",
  "recordingId",
  "",
  "",
);

const playerRef = ref<HTMLElement | null>(null);
const currentRecording = ref<SessionRecording | null>(null);

// eslint-disable-next-line @typescript-eslint/no-explicit-any
let player: any = null;

function formatDuration(ms?: number) {
  if (!ms) return "0:00";
  const totalSeconds = Math.floor(ms / 1000);
  const hours = Math.floor(totalSeconds / 3600);
  const minutes = Math.floor((totalSeconds % 3600) / 60);
  const seconds = totalSeconds % 60;

  if (hours > 0) {
    return `${hours}h ${minutes}m`;
  }
  return `${minutes}:${seconds.toString().padStart(2, "0")}`;
}

function formatDate(dateStr: string) {
  const date = new Date(dateStr);
  return date.toLocaleDateString("en-US", {
    month: "short",
    day: "numeric",
    year: "numeric",
    hour: "2-digit",
    minute: "2-digit",
  });
}

function formatFileSize(bytes: number) {
  if (bytes < 1024) return `${bytes} B`;
  if (bytes < 1024 * 1024) return `${(bytes / 1024).toFixed(1)} KB`;
  return `${(bytes / (1024 * 1024)).toFixed(1)} MB`;
}

async function loadRecording() {
  const recId = String(recordingId.value);
  if (!recId) {
    return;
  }

  try {
    await recordingStore.loadRecordings();

    const recording = recordingStore.recordings.find((r) => r.id === recId);
    if (!recording) {
      throw new Error(`Recording not found: ${recId}`);
    }

    currentRecording.value = recording;

    await initPlayer();
  } catch (error) {
    console.error("Failed to load recording:", error);
  }
}

async function initPlayer() {
  if (!playerRef.value || !currentRecording.value) {
    return;
  }

  try {
    const castContent = await recordingStore.readCastFile(
      currentRecording.value.filePath,
    );

    const lines = castContent.trim().split("\n");
    if (lines.length < 1) {
      throw new Error("Invalid cast file: empty");
    }

    const blob = new Blob([castContent], { type: "application/x-asciicast" });
    const blobUrl = URL.createObjectURL(blob);

    player = AsciinemaPlayer.create(blobUrl, playerRef.value, {
      loop: false,
      autoPlay: true,
      preload: true,
      fit: "width",
      fontSize: "14px",
      theme: "asciinema",
    });
  } catch (error) {
    console.error("Failed to initialize player:", error);
  }
}

function cleanup() {
  if (player) {
    try {
      player.dispose();
    } catch (e) {
      console.error("Failed to dispose player:", e);
    }
    player = null;
  }
  currentRecording.value = null;
}

watch(
  () => isOverlayVisible("playback-modal"),
  async (visible) => {
    if (visible) {
      await new Promise((resolve) => setTimeout(resolve, 100));
      await loadRecording();
    } else {
      cleanup();
    }
  },
  { immediate: true },
);

onUnmounted(() => {
  cleanup();
});
</script>

<style>
/* Override asciinema player styles to match app theme */
:deep(.asciinema-player-wrapper) {
  background: #000000;
  border-radius: 0.5rem;
}

:deep(.asciinema-player) {
  border-radius: 0.5rem;
}

:deep(.control-bar) {
  background: rgba(0, 0, 0, 0.8);
  backdrop-filter: blur(8px);
  border-top: 1px solid rgba(75, 85, 99, 0.3);
}

:deep(.playback-button),
:deep(.fullscreen-button) {
  color: rgb(156, 163, 175);
  transition: color 200ms;
}

:deep(.playback-button:hover),
:deep(.fullscreen-button:hover) {
  color: rgb(255, 255, 255);
}

:deep(.progressbar) {
  background: rgb(55, 65, 81);
}

:deep(.progressbar .bar) {
  background: rgb(59, 130, 246);
}

:deep(.timer) {
  color: rgb(156, 163, 175);
  font-family: "FiraCode Nerd Font", "Menlo", monospace;
}
</style>
