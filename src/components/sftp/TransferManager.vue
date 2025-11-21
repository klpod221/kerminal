<template>
  <Modal
    id="sftp-transfer-manager-modal"
    title="Transfer Queue"
    :icon="Activity"
    icon-background="bg-blue-500/20"
    icon-color="text-blue-400"
    size="lg"
  >
    <div class="space-y-3">
      <!-- Header with filters and actions -->
      <div class="flex items-center justify-between mb-2">
        <div class="flex items-center gap-2">
          <Select
            id="transfer-status-filter"
            v-model="statusFilter"
            :options="statusOptions"
            placeholder="All Statuses"
            :space="false"
            size="sm"
          />
          <span class="text-xs text-gray-500">
            {{ filteredTransfers.length }} transfer{{
              filteredTransfers.length !== 1 ? "s" : ""
            }}
          </span>
        </div>
        <div class="flex items-center gap-2">
          <Button
            v-if="canRetryAll"
            variant="ghost"
            size="sm"
            @click="handleRetryAll"
            title="Retry all failed transfers"
          >
            Retry All
          </Button>
          <Button
            v-if="hasCompletedTransfers"
            variant="ghost"
            size="sm"
            :icon="Trash2"
            @click="handleClearCompleted"
            title="Clear completed transfers"
          >
            Clear Completed
          </Button>
        </div>
      </div>

      <!-- Empty state -->
      <div
        v-if="filteredTransfers.length === 0"
        class="text-center py-8 text-gray-500 text-sm"
      >
        {{
          statusFilter
            ? `No ${statusFilter} transfers`
            : "No transfers in queue"
        }}
      </div>

      <!-- Transfer list with drag-and-drop -->
      <div class="space-y-2" @dragover.prevent @drop.prevent="handleDrop">
        <Card
          v-for="(transfer, index) in filteredTransfers"
          :key="transfer.transferId"
          :draggable="canDragTransfer(transfer)"
          no-padding
          :custom-class="getTransferCardClass(transfer, index)"
          @dragstart="handleDragStart(transfer, $event)"
          @dragend="handleDragEnd"
          @dragenter="handleDragEnter(index)"
          @dragleave="handleDragLeave"
        >
          <div class="p-3">
            <!-- Drag handle indicator -->
            <div
              v-if="canDragTransfer(transfer)"
              class="absolute left-1 top-1/2 -translate-y-1/2 opacity-0 group-hover:opacity-100 transition-opacity"
            >
              <GripVertical :size="16" class="text-gray-600" />
            </div>

            <div class="flex items-start gap-3">
            <!-- Transfer icon and direction -->
            <div
              class="flex-shrink-0 w-8 h-8 rounded-lg flex items-center justify-center"
              :class="getDirectionClass(transfer.direction)"
            >
              <component
                :is="transfer.direction === 'upload' ? ArrowUpCircle : ArrowDownCircle"
                :size="16"
              />
            </div>

            <div class="flex-1 min-w-0">
              <!-- Transfer header with priority and status -->
              <div class="flex items-start justify-between mb-1.5">
                <div class="flex-1 min-w-0 mr-2">
                  <div class="flex items-center gap-2 mb-1">
                    <span class="text-sm font-medium text-gray-200 truncate">
                      {{ getFileName(transfer) }}
                    </span>
                    <!-- Priority badge -->
                    <span
                      v-if="transfer.priority > 0"
                      class="px-1.5 py-0.5 rounded text-xs font-medium bg-purple-500/20 text-purple-400"
                      :title="`Priority: ${transfer.priority}`"
                    >
                      P{{ transfer.priority }}
                    </span>
                    <!-- Retry count indicator -->
                    <span
                      v-if="transfer.retryCount > 0"
                      class="px-1.5 py-0.5 rounded text-xs font-medium bg-orange-500/20 text-orange-400"
                      :title="`Retry ${transfer.retryCount}/${transfer.maxRetries}`"
                    >
                      Retry {{ transfer.retryCount }}
                    </span>
                  </div>
                  <div class="text-xs text-gray-500">
                    {{ formatBytes(transfer.transferredBytes) }} /
                    {{ formatBytes(transfer.totalBytes) }}
                    <span v-if="transfer.speedBytesPerSec" class="ml-2">
                      • {{ formatSpeed(transfer.speedBytesPerSec) }}
                    </span>
                    <span v-if="transfer.etaSeconds" class="ml-2">
                      • ETA: {{ formatETA(transfer.etaSeconds) }}
                    </span>
                  </div>
                </div>

                <!-- Status badge -->
                <div
                  class="px-2 py-1 rounded text-xs font-medium whitespace-nowrap flex-shrink-0"
                  :class="getStatusClass(transfer.status)"
                >
                  {{ getStatusLabel(transfer.status) }}
                </div>
              </div>

              <!-- Progress bar -->
              <div class="w-full bg-gray-800 rounded-full h-1.5 mb-2">
                <div
                  class="h-1.5 rounded-full transition-all duration-300"
                  :class="getProgressBarClass(transfer.status)"
                  :style="{
                    width: `${(transfer.transferredBytes / transfer.totalBytes) * 100}%`,
                  }"
                ></div>
              </div>

              <!-- Actions row -->
              <div class="flex items-center justify-between">
                <!-- Priority selector for queued/paused transfers -->
                <div class="flex items-center gap-2">
                  <Select
                    v-if="canChangePriority(transfer)"
                    :id="`transfer-priority-${transfer.transferId}`"
                    :model-value="transfer.priority"
                    :options="priorityOptions"
                    size="sm"
                    class="w-24"
                    @update:model-value="
                      (value) => handlePriorityChange(transfer.transferId, value)
                    "
                  />
                  <span v-else class="text-xs text-gray-600">
                    {{ transfer.direction === "upload" ? "↑" : "↓" }}
                    {{ transfer.direction }}
                  </span>
                </div>

                <!-- Action buttons -->
                <div class="flex items-center gap-1.5">
                  <!-- Pause button -->
                  <Button
                    v-if="transfer.status === 'inprogress'"
                    variant="ghost"
                    size="sm"
                    :icon="Pause"
                    @click="handlePause(transfer.transferId)"
                    title="Pause transfer"
                  />
                  <!-- Resume button -->
                  <Button
                    v-if="transfer.status === 'paused'"
                    variant="ghost"
                    size="sm"
                    :icon="Play"
                    @click="handleResume(transfer.transferId)"
                    title="Resume transfer"
                  />
                  <!-- Retry button -->
                  <Button
                    v-if="
                      transfer.status === 'failed' &&
                      transfer.retryCount < transfer.maxRetries
                    "
                    variant="ghost"
                    size="sm"
                    :icon="RotateCw"
                    @click="handleRetry(transfer.transferId)"
                    title="Retry transfer"
                  />
                  <!-- Cancel button -->
                  <Button
                    v-if="canCancel(transfer)"
                    variant="ghost"
                    size="sm"
                    :icon="X"
                    @click="handleCancel(transfer.transferId)"
                    title="Cancel transfer"
                  />
                </div>
              </div>

              <!-- Error message -->
              <div
                v-if="transfer.error"
                class="mt-2 p-2 rounded bg-red-500/10 border border-red-500/20"
              >
                <p class="text-xs text-red-400">
                  <AlertCircle :size="12" class="inline mr-1" />
                  {{ transfer.error }}
                </p>
              </div>

              <!-- Next retry time -->
              <div
                v-if="transfer.status === 'failed' && transfer.nextRetryAt"
                class="mt-2 text-xs text-gray-500"
              >
                Next retry: {{ formatRetryTime(transfer.nextRetryAt) }}
              </div>
            </div>
            </div>
          </div>
        </Card>
      </div>
    </div>
  </Modal>
</template>

<script setup lang="ts">
import { ref, computed } from "vue";
import {
  Activity,
  ArrowUpCircle,
  ArrowDownCircle,
  Pause,
  Play,
  RotateCw,
  X,
  GripVertical,
  AlertCircle,
  Trash2,
} from "lucide-vue-next";
import { useSFTPStore } from "../../stores/sftp";
import Modal from "../ui/Modal.vue";
import Button from "../ui/Button.vue";
import Select from "../ui/Select.vue";
import Card from "../ui/Card.vue";
import { message } from "../../utils/message";
import type { TransferProgress, TransferStatus } from "../../types/sftp";

const sftpStore = useSFTPStore();

// Filters
const statusFilter = ref<string>("");
const statusOptions = [
  { value: "", label: "All Statuses" },
  { value: "queued", label: "Queued" },
  { value: "inprogress", label: "In Progress" },
  { value: "paused", label: "Paused" },
  { value: "completed", label: "Completed" },
  { value: "failed", label: "Failed" },
];

// Priority options
const priorityOptions = [
  { value: 0, label: "Normal" },
  { value: 64, label: "Low" },
  { value: 128, label: "Medium" },
  { value: 192, label: "High" },
  { value: 255, label: "Urgent" },
];

// Drag and drop state
const draggingId = ref<string | null>(null);
const dragOverIndex = ref<number | null>(null);

const filteredTransfers = computed(() => {
  const transfers = Array.from(sftpStore.browserState.activeTransfers.values());
  if (!statusFilter.value) {
    return transfers.filter((t) => t.status !== "cancelled");
  }
  return transfers.filter(
    (t) => t.status === statusFilter.value && t.status !== "cancelled"
  );
});

const hasCompletedTransfers = computed(() => {
  return Array.from(sftpStore.browserState.activeTransfers.values()).some(
    (t) => t.status === "completed"
  );
});

const canRetryAll = computed(() => {
  return Array.from(sftpStore.browserState.activeTransfers.values()).some(
    (t) => t.status === "failed" && t.retryCount < t.maxRetries
  );
});

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
  if (minutes < 60) {
    return `${minutes}m ${remainingSeconds}s`;
  }
  const hours = Math.floor(minutes / 60);
  const remainingMinutes = minutes % 60;
  return `${hours}h ${remainingMinutes}m`;
}

function formatRetryTime(isoString: string): string {
  const retryTime = new Date(isoString);
  const now = new Date();
  const diffMs = retryTime.getTime() - now.getTime();
  const diffSecs = Math.max(0, Math.floor(diffMs / 1000));
  return formatETA(diffSecs);
}

function getStatusLabel(status: TransferStatus): string {
  const labels: Record<TransferStatus, string> = {
    queued: "Queued",
    inprogress: "In Progress",
    paused: "Paused",
    completed: "Completed",
    failed: "Failed",
    cancelled: "Cancelled",
  };
  return labels[status] || status;
}

function getStatusClass(status: TransferStatus): string {
  switch (status) {
    case "completed":
      return "bg-green-500/20 text-green-400";
    case "failed":
      return "bg-red-500/20 text-red-400";
    case "paused":
      return "bg-yellow-500/20 text-yellow-400";
    case "inprogress":
      return "bg-blue-500/20 text-blue-400";
    case "queued":
      return "bg-purple-500/20 text-purple-400";
    default:
      return "bg-gray-500/20 text-gray-400";
  }
}

function getProgressBarClass(status: TransferStatus): string {
  switch (status) {
    case "completed":
      return "bg-green-500";
    case "failed":
      return "bg-red-500";
    case "paused":
      return "bg-yellow-500";
    case "inprogress":
      return "bg-blue-500";
    default:
      return "bg-gray-500";
  }
}

function getDirectionClass(direction: "upload" | "download"): string {
  if (direction === "upload") {
    return "bg-blue-500/20 text-blue-400";
  }
  return "bg-green-500/20 text-green-400";
}

function canChangePriority(transfer: TransferProgress): boolean {
  return transfer.status === "queued" || transfer.status === "paused";
}

function canCancel(transfer: TransferProgress): boolean {
  return (
    transfer.status === "inprogress" ||
    transfer.status === "paused" ||
    transfer.status === "queued"
  );
}

function canDragTransfer(transfer: TransferProgress): boolean {
  return transfer.status === "queued" || transfer.status === "paused";
}

function getTransferCardClass(transfer: TransferProgress, index: number): string {
  const classes = ["group relative transition-all duration-200"];
  
  if (canDragTransfer(transfer)) {
    classes.push("cursor-move");
  }
  
  if (draggingId.value === transfer.transferId) {
    classes.push("opacity-50");
  }
  
  if (dragOverIndex.value === index && draggingId.value !== transfer.transferId) {
    classes.push("border-blue-500 bg-blue-500/5");
  }
  
  return classes.join(" ");
}

// Transfer actions
async function handlePause(transferId: string) {
  try {
    await sftpStore.pauseTransfer(transferId);
    message.success("Transfer paused");
  } catch (error) {
    console.error("Failed to pause transfer:", error);
  }
}

async function handleResume(transferId: string) {
  try {
    await sftpStore.resumeTransfer(transferId);
    message.success("Transfer resumed");
  } catch (error) {
    console.error("Failed to resume transfer:", error);
  }
}

async function handleRetry(transferId: string) {
  try {
    await sftpStore.retryTransfer(transferId);
    message.success("Transfer retry initiated");
  } catch (error) {
    console.error("Failed to retry transfer:", error);
  }
}

async function handleCancel(transferId: string) {
  try {
    await sftpStore.cancelTransfer(transferId);
    message.success("Transfer cancelled");
  } catch (error) {
    console.error("Failed to cancel transfer:", error);
  }
}

async function handlePriorityChange(transferId: string, priority: number) {
  try {
    await sftpStore.setTransferPriority(transferId, priority);
    message.success(`Priority updated to ${priorityOptions.find((o) => o.value === priority)?.label || priority}`);
  } catch (error) {
    console.error("Failed to update priority:", error);
  }
}

async function handleRetryAll() {
  const failedTransfers = Array.from(
    sftpStore.browserState.activeTransfers.values()
  ).filter((t) => t.status === "failed" && t.retryCount < t.maxRetries);

  for (const transfer of failedTransfers) {
    try {
      await sftpStore.retryTransfer(transfer.transferId);
    } catch (error) {
      console.error(`Failed to retry transfer ${transfer.transferId}:`, error);
    }
  }
  message.success(`Retrying ${failedTransfers.length} failed transfers`);
}

async function handleClearCompleted() {
  const completedIds = Array.from(
    sftpStore.browserState.activeTransfers.entries()
  )
    .filter(([_, transfer]) => transfer.status === "completed")
    .map(([id]) => id);

  for (const id of completedIds) {
    sftpStore.browserState.activeTransfers.delete(id);
  }
  message.success(`Cleared ${completedIds.length} completed transfers`);
}

// Drag and drop handlers
function handleDragStart(transfer: TransferProgress, event: DragEvent) {
  if (!canDragTransfer(transfer)) {
    event.preventDefault();
    return;
  }
  draggingId.value = transfer.transferId;
  if (event.dataTransfer) {
    event.dataTransfer.effectAllowed = "move";
    event.dataTransfer.setData("text/plain", transfer.transferId);
  }
}

function handleDragEnd() {
  draggingId.value = null;
  dragOverIndex.value = null;
}

function handleDragEnter(index: number) {
  dragOverIndex.value = index;
}

function handleDragLeave() {
  dragOverIndex.value = null;
}

async function handleDrop(event: DragEvent) {
  event.preventDefault();
  const draggedId = draggingId.value;
  const targetIndex = dragOverIndex.value;

  if (draggedId && targetIndex !== null) {
    try {
      // Get current order of draggable transfers (queued/paused only)
      const draggableTransfers = filteredTransfers.value.filter((t) =>
        canDragTransfer(t)
      );
      const draggedIndex = draggableTransfers.findIndex(
        (t) => t.transferId === draggedId
      );

      if (draggedIndex === -1) return;

      // Reorder
      const newOrder = [...draggableTransfers];
      const [removed] = newOrder.splice(draggedIndex, 1);
      newOrder.splice(targetIndex, 0, removed);

      // Update queue order in backend
      await sftpStore.reorderQueue(newOrder.map((t) => t.transferId));
      message.success("Queue reordered");
    } catch (error) {
      console.error("Failed to reorder queue:", error);
      message.error("Failed to reorder queue");
    }
  }

  draggingId.value = null;
  dragOverIndex.value = null;
}
</script>
