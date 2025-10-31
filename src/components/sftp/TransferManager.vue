<template>
  <div class="p-4 space-y-3">
    <div
      v-if="transfers.length === 0"
      class="text-center py-8 text-gray-500 text-sm"
    >
      No active transfers
    </div>

    <div
      v-for="transfer in transfers"
      :key="transfer.transferId"
      class="p-3 bg-gray-900 rounded-lg border border-gray-800"
    >
      <!-- Transfer header -->
      <div class="flex items-center justify-between mb-2">
        <div class="flex-1 min-w-0">
          <div class="text-sm font-medium text-gray-200 truncate">
            {{ transfer.direction === "upload" ? "↑ Upload" : "↓ Download" }}
            : {{ getFileName(transfer) }}
          </div>
          <div class="text-xs text-gray-500 mt-0.5">
            {{ formatBytes(transfer.transferredBytes) }} /
            {{ formatBytes(transfer.totalBytes) }}
          </div>
        </div>

        <!-- Status badge -->
        <div class="flex items-center gap-2 ml-3">
          <div
            class="px-2 py-1 rounded text-xs font-medium"
            :class="getStatusClass(transfer.status)"
          >
            {{ transfer.status }}
          </div>
        </div>
      </div>

      <!-- Progress bar -->
      <div class="w-full bg-gray-800 rounded-full h-2 mb-2">
        <div
          class="bg-blue-500 h-2 rounded-full transition-all duration-300"
          :style="{
            width: `${(transfer.transferredBytes / transfer.totalBytes) * 100}%`,
          }"
        ></div>
      </div>

      <!-- Transfer info -->
      <div class="flex items-center justify-between text-xs text-gray-500">
        <div>
          <span v-if="transfer.speedBytesPerSec">
            {{ formatSpeed(transfer.speedBytesPerSec) }}
          </span>
          <span v-if="transfer.etaSeconds" class="ml-2">
            ETA: {{ formatETA(transfer.etaSeconds) }}
          </span>
        </div>

        <!-- Actions -->
        <div class="flex items-center gap-2">
          <Button
            v-if="transfer.status === 'paused' || transfer.status === 'failed'"
            variant="ghost"
            size="sm"
            @click="handleResume(transfer.transferId)"
          >
            Resume
          </Button>
          <Button
            v-if="transfer.status === 'inProgress'"
            variant="ghost"
            size="sm"
            @click="handleCancel(transfer.transferId)"
          >
            Cancel
          </Button>
        </div>
      </div>

      <!-- Error message -->
      <div
        v-if="transfer.error"
        class="mt-2 text-xs text-red-400"
      >
        Error: {{ transfer.error }}
      </div>
    </div>
  </div>
</template>

<script setup lang="ts">
import { computed } from "vue";
import { useSFTPStore } from "../../stores/sftp";
import Button from "../ui/Button.vue";
import { showError } from "../../utils/message";
import type { TransferProgress } from "../../types/sftp";

const sftpStore = useSFTPStore();

const transfers = computed(() => sftpStore.activeTransfers);

function getFileName(transfer: TransferProgress): string {
  if (transfer.direction === "upload") {
    return transfer.localPath.split("/").pop() || transfer.localPath;
  }
  return transfer.remotePath.split("/").pop() || transfer.remotePath;
}

function formatBytes(bytes: number): string {
  const units = ["B", "KB", "MB", "GB", "TB"];
  let size = bytes;
  let unitIndex = 0;
  while (size >= 1024 && unitIndex < units.length - 1) {
    size /= 1024;
    unitIndex++;
  }
  return `${size.toFixed(1)} ${units[unitIndex]}`;
}

function formatSpeed(bytesPerSec: number): string {
  return `${formatBytes(bytesPerSec)}/s`;
}

function formatETA(seconds: number): string {
  if (seconds < 60) {
    return `${seconds}s`;
  }
  const minutes = Math.floor(seconds / 60);
  const remainingSeconds = seconds % 60;
  return `${minutes}m ${remainingSeconds}s`;
}

function getStatusClass(status: TransferProgress["status"]): string {
  switch (status) {
    case "completed":
      return "bg-green-500/20 text-green-400";
    case "failed":
      return "bg-red-500/20 text-red-400";
    case "paused":
      return "bg-yellow-500/20 text-yellow-400";
    case "inProgress":
      return "bg-blue-500/20 text-blue-400";
    default:
      return "bg-gray-500/20 text-gray-400";
  }
}

async function handleResume(transferId: string) {
  try {
    await sftpStore.resumeTransfer(transferId);
  } catch (error) {
    console.error("Failed to resume transfer:", error);
    showError("Failed to resume transfer");
  }
}

async function handleCancel(transferId: string) {
  try {
    await sftpStore.cancelTransfer(transferId);
  } catch (error) {
    console.error("Failed to cancel transfer:", error);
    showError("Failed to cancel transfer");
  }
}
</script>

