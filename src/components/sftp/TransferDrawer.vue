<!--
  Copyright (C) 2026 Bùi Thanh Xuân (klpod221)
  SPDX-License-Identifier: GPL-3.0-or-later
-->

<template>
  <div
    v-if="isOpen"
    class="relative flex flex-col border-t border-gray-800 bg-bg-secondary font-mono text-xs text-gray-300 select-none"
    :style="{ height: `${drawerHeight}px` }"
  >
    <!-- Resize Drag Handle -->
    <div
      class="group absolute top-0 left-0 right-0 h-1.5 -translate-y-1/2 cursor-row-resize z-20 flex items-center justify-center hover:bg-blue-500/40 transition-colors"
      @mousedown="startResize"
    >
      <div class="h-0.5 w-10 rounded-full bg-gray-600 group-hover:bg-blue-400 transition-colors"></div>
    </div>

    <!-- Header -->
    <div class="flex items-center justify-between border-b border-gray-800 bg-bg-quaternary px-4 py-2 shrink-0">
      <div class="flex items-center gap-2 font-medium text-white">
        <ArrowUpDown class="h-4 w-4 text-accent-blue" />
        <span>Transfer Queue</span>
        <Badge
          v-if="activeCount > 0"
          variant="primary"
          :text="`${activeCount} active`"
        />
      </div>

      <div class="flex items-center gap-1.5">
        <Button
          v-if="activeCount > 0"
          variant="ghost"
          size="sm"
          :icon="Pause"
          text="Pause All"
          class="h-7 text-xs text-gray-400 hover:text-white"
          @click="$emit('pause-all')"
        />
        <Button
          v-else-if="transfers.some(t => t.status === 'paused')"
          variant="ghost"
          size="sm"
          :icon="Play"
          text="Resume All"
          class="h-7 text-xs text-gray-400 hover:text-white"
          @click="$emit('resume-all')"
        />

        <Button
          variant="ghost"
          size="sm"
          :icon="Trash2"
          title="Clear completed transfers"
          class="h-7 w-7 p-0 text-gray-400 hover:text-white"
          @click="$emit('clear-completed')"
        />

        <Button
          variant="ghost"
          size="sm"
          :icon="RotateCw"
          title="Refresh queue"
          class="h-7 w-7 p-0 text-gray-400 hover:text-white"
          :class="{ 'animate-spin': loading }"
          @click="$emit('refresh')"
        />

        <Button
          variant="ghost"
          size="sm"
          :icon="X"
          title="Close drawer"
          class="h-7 w-7 p-0 text-gray-400 hover:text-white"
          @click="$emit('close')"
        />
      </div>
    </div>

    <!-- Transfer items -->
    <div class="flex-1 overflow-y-auto p-2 space-y-1.5 min-h-0">
      <div
        v-if="transfers.length === 0"
        class="flex h-full items-center justify-center text-gray-500"
      >
        <span>No active or completed transfers</span>
      </div>

      <div
        v-for="item in transfers"
        :key="item.transferId"
        class="flex items-center justify-between rounded-lg border border-gray-800/80 bg-bg-primary/80 p-2.5 transition-colors hover:border-gray-700 hover:bg-bg-primary"
      >
        <!-- Direction icon & Info -->
        <div class="flex items-center gap-3 min-w-0 flex-1">
          <div
            class="flex h-7 w-7 shrink-0 items-center justify-center rounded-md border"
            :class="item.direction === 'upload' ? 'bg-cyan-500/10 border-cyan-500/20 text-cyan-400' : 'bg-purple-500/10 border-purple-500/20 text-purple-400'"
          >
            <Upload v-if="item.direction === 'upload'" class="h-3.5 w-3.5" />
            <Download v-else class="h-3.5 w-3.5" />
          </div>

          <div class="min-w-0 flex-1">
            <div class="flex items-center justify-between gap-2">
              <span class="truncate font-medium text-gray-200" :title="item.localPath">
                {{ getFileName(item.direction === 'upload' ? item.localPath : item.remotePath) }}
              </span>
              <span class="shrink-0 text-2xs text-gray-400 font-mono">
                {{ formatBytes(item.transferredBytes) }} / {{ formatBytes(item.totalBytes) }}
                ({{ calculatePercent(item) }}%)
              </span>
            </div>

            <!-- Progress Bar -->
            <div class="mt-1.5 h-1.5 w-full overflow-hidden rounded-full bg-gray-800">
              <div
                class="h-full transition-all duration-200"
                :class="getProgressBarClass(item.status)"
                :style="{ width: `${calculatePercent(item)}%` }"
              ></div>
            </div>

            <!-- Status & Speed -->
            <div class="mt-1 flex items-center justify-between text-2xs text-gray-500">
              <div class="flex items-center gap-2">
                <span :class="getStatusColor(item.status)" class="capitalize font-medium">
                  {{ item.status }}
                </span>
                <span v-if="(item.speedBytesPerSec || (item as any).bytesPerSecond) && ((item.speedBytesPerSec || (item as any).bytesPerSecond) > 0)" class="text-gray-400">
                  {{ formatBytes(item.speedBytesPerSec || (item as any).bytesPerSecond) }}/s
                </span>
                <span v-if="item.error" class="truncate text-red-400" :title="item.error">
                  {{ item.error }}
                </span>
              </div>
            </div>
          </div>
        </div>

        <!-- Action Controls -->
        <div class="ml-3 flex shrink-0 items-center gap-1">
          <Button
            v-if="item.status === 'inprogress'"
            variant="ghost"
            size="sm"
            :icon="Pause"
            title="Pause"
            class="h-6 w-6 p-0 text-gray-400 hover:text-yellow-400"
            @click="$emit('pause', item.transferId)"
          />

          <Button
            v-if="item.status === 'paused'"
            variant="ghost"
            size="sm"
            :icon="Play"
            title="Resume"
            class="h-6 w-6 p-0 text-gray-400 hover:text-green-400"
            @click="$emit('resume', item.transferId)"
          />

          <Button
            v-if="item.status === 'failed'"
            variant="ghost"
            size="sm"
            :icon="RotateCw"
            title="Retry"
            class="h-6 w-6 p-0 text-gray-400 hover:text-blue-400"
            @click="$emit('retry', item.transferId)"
          />

          <Button
            v-if="item.status === 'inprogress' || item.status === 'queued' || item.status === 'paused'"
            variant="ghost"
            size="sm"
            :icon="X"
            title="Cancel"
            class="h-6 w-6 p-0 text-gray-400 hover:text-red-400"
            @click="$emit('cancel', item.transferId)"
          />
        </div>
      </div>
    </div>
  </div>
</template>

<script setup lang="ts">
import { ref, computed } from "vue";
import type { TransferProgress } from "../../types/sftp";
import Button from "../ui/Button.vue";
import Badge from "../ui/Badge.vue";
import {
  ArrowUpDown,
  Upload,
  Download,
  RotateCw,
  Pause,
  Play,
  X,
  Trash2,
} from "lucide-vue-next";

interface Props {
  isOpen: boolean;
  transfers: TransferProgress[];
  loading?: boolean;
}

const props = withDefaults(defineProps<Props>(), {
  loading: false,
});

defineEmits<{
  (e: "close"): void;
  (e: "refresh"): void;
  (e: "pause-all"): void;
  (e: "resume-all"): void;
  (e: "clear-completed"): void;
  (e: "pause", id: string): void;
  (e: "resume", id: string): void;
  (e: "cancel", id: string): void;
  (e: "retry", id: string): void;
}>();

const drawerHeight = ref<number>(240);
let isResizing = false;
let startY = 0;
let startHeight = 0;

function startResize(e: MouseEvent) {
  isResizing = true;
  startY = e.clientY;
  startHeight = drawerHeight.value;
  document.body.style.cursor = "row-resize";
  document.body.style.userSelect = "none";

  const onMouseMove = (ev: MouseEvent) => {
    if (!isResizing) return;
    const deltaY = startY - ev.clientY; // drag up -> increase height
    const minH = 140;
    const maxH = Math.floor(window.innerHeight * 0.8);
    drawerHeight.value = Math.max(minH, Math.min(maxH, startHeight + deltaY));
  };

  const onMouseUp = () => {
    isResizing = false;
    document.body.style.cursor = "";
    document.body.style.userSelect = "";
    window.removeEventListener("mousemove", onMouseMove);
    window.removeEventListener("mouseup", onMouseUp);
  };

  window.addEventListener("mousemove", onMouseMove);
  window.addEventListener("mouseup", onMouseUp);
}

const activeCount = computed(() => {
  return props.transfers.filter((t) => t.status === "inprogress").length;
});

function calculatePercent(item: TransferProgress): number {
  if (!item.totalBytes || item.totalBytes <= 0) {
    return item.status === "completed" ? 100 : 0;
  }
  const pct = Math.round((item.transferredBytes / item.totalBytes) * 100);
  return Math.min(100, Math.max(0, pct));
}

function getFileName(path: string): string {
  return path.split(/[/\\]/).pop() || path;
}

function getProgressBarClass(status: string): string {
  switch (status) {
    case "completed":
      return "bg-green-500";
    case "failed":
      return "bg-red-500";
    case "paused":
      return "bg-yellow-500";
    default:
      return "bg-gradient-to-r from-cyan-500 to-purple-500";
  }
}

function getStatusColor(status: string): string {
  switch (status) {
    case "completed":
      return "text-green-400";
    case "failed":
      return "text-red-400";
    case "paused":
      return "text-yellow-400";
    case "inprogress":
      return "text-blue-400";
    default:
      return "text-gray-400";
  }
}

function formatBytes(bytes: number): string {
  if (!bytes || bytes === 0) return "0 B";
  const k = 1024;
  const sizes = ["B", "KB", "MB", "GB", "TB"];
  const i = Math.floor(Math.log(bytes) / Math.log(k));
  return `${parseFloat((bytes / Math.pow(k, i)).toFixed(1))} ${sizes[i]}`;
}
</script>

