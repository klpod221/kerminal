<template>
  <Modal
    id="sftp-sync-compare-modal"
    title="Sync & Compare Directories"
    :icon="GitCompare"
    icon-background="bg-purple-500/20"
    icon-color="text-purple-400"
    size="xl"
  >
    <div class="space-y-4">
      <!-- Paths Info -->
      <div class="grid grid-cols-2 gap-4 p-4 bg-gray-900/50 rounded-lg border border-gray-800">
        <div>
          <div class="text-xs text-gray-500 mb-1">Local Path</div>
          <div class="text-sm font-mono text-gray-300 truncate">
            {{ localPath }}
          </div>
        </div>
        <div>
          <div class="text-xs text-gray-500 mb-1">Remote Path</div>
          <div class="text-sm font-mono text-gray-300 truncate">
            {{ remotePath }}
          </div>
        </div>
      </div>

      <!-- Compare Button -->
      <div class="flex justify-end">
        <Button
          :loading="comparing"
          :disabled="!sftpStore.activeSessionId"
          @click="handleCompare"
        >
          <template v-if="!hasResults">Compare Directories</template>
          <template v-else>Refresh Comparison</template>
        </Button>
      </div>

      <!-- Results -->
      <div v-if="hasResults" class="space-y-4">
        <!-- Summary -->
        <div class="grid grid-cols-2 sm:grid-cols-4 gap-2">
          <div class="p-3 bg-gray-900/50 rounded border border-gray-800">
            <div class="text-xs text-gray-500 mb-1">Only Local</div>
            <div class="text-lg font-semibold text-blue-400">
              {{ summary.onlyLocal }}
            </div>
          </div>
          <div class="p-3 bg-gray-900/50 rounded border border-gray-800">
            <div class="text-xs text-gray-500 mb-1">Only Remote</div>
            <div class="text-lg font-semibold text-green-400">
              {{ summary.onlyRemote }}
            </div>
          </div>
          <div class="p-3 bg-gray-900/50 rounded border border-gray-800">
            <div class="text-xs text-gray-500 mb-1">Different</div>
            <div class="text-lg font-semibold text-yellow-400">
              {{ summary.different }}
            </div>
          </div>
          <div class="p-3 bg-gray-900/50 rounded border border-gray-800">
            <div class="text-xs text-gray-500 mb-1">Identical</div>
            <div class="text-lg font-semibold text-gray-400">
              {{ summary.identical }}
            </div>
          </div>
        </div>

        <!-- Differences List -->
        <div class="border border-gray-800 rounded-lg overflow-hidden">
          <div class="max-h-96 overflow-y-auto">
            <div
              v-for="(diff, index) in filteredDiffs"
              :key="index"
              class="p-3 border-b border-gray-800 hover:bg-gray-900/50 transition-colors"
              :class="getDiffClass(diff.diffType)"
            >
              <div class="flex items-start justify-between gap-3">
                <div class="flex-1 min-w-0">
                  <div class="flex items-center gap-2 mb-1">
                    <component
                      :is="getDiffIcon(diff.diffType)"
                      :size="16"
                      :class="getDiffIconColor(diff.diffType)"
                    />
                    <span class="text-sm font-medium text-gray-200 truncate">
                      {{ diff.path }}
                    </span>
                    <span
                      class="px-2 py-0.5 rounded text-xs font-medium"
                      :class="getDiffBadgeClass(diff.diffType)"
                    >
                      {{ getDiffLabel(diff.diffType) }}
                    </span>
                  </div>
                  <div class="text-xs text-gray-500 space-y-0.5">
                    <div v-if="diff.localEntry">
                      Local: {{ formatFileInfo(diff.localEntry) }}
                    </div>
                    <div v-if="diff.remoteEntry">
                      Remote: {{ formatFileInfo(diff.remoteEntry) }}
                    </div>
                  </div>
                </div>
              </div>
            </div>
          </div>
        </div>

        <!-- Sync Options -->
        <div class="p-4 bg-gray-900/50 rounded-lg border border-gray-800">
          <h4 class="text-sm font-medium text-gray-200 mb-3">
            Sync Options
          </h4>
          <Form ref="syncForm" @submit="handleSync">
            <Select
              id="sync-direction"
              v-model="syncDirection"
              label="Sync Direction"
              :options="syncDirectionOptions"
              rules="required"
            />
            <div class="space-y-2 mt-4">
              <label class="flex items-center gap-2 cursor-pointer">
                <input
                  type="checkbox"
                  v-model="syncOptions.deleteExtraFiles"
                  class="rounded border-gray-600 text-blue-600 focus:ring-blue-500"
                />
                <span class="text-sm text-gray-300">
                  Delete extra files (files not in source)
                </span>
              </label>
              <label class="flex items-center gap-2 cursor-pointer">
                <input
                  type="checkbox"
                  v-model="syncOptions.preservePermissions"
                  class="rounded border-gray-600 text-blue-600 focus:ring-blue-500"
                />
                <span class="text-sm text-gray-300">
                  Preserve file permissions
                </span>
              </label>
              <label class="flex items-center gap-2 cursor-pointer">
                <input
                  type="checkbox"
                  v-model="syncOptions.preserveSymlinks"
                  class="rounded border-gray-600 text-blue-600 focus:ring-blue-500"
                />
                <span class="text-sm text-gray-300">
                  Preserve symbolic links
                </span>
              </label>
            </div>
          </Form>
        </div>
      </div>

      <!-- Loading State -->
      <div
        v-if="comparing"
        class="flex items-center justify-center py-12"
      >
        <div class="text-center">
          <div class="animate-spin rounded-full border-4 border-gray-700 border-t-blue-500 w-8 h-8 mx-auto mb-4"></div>
          <div class="text-sm text-gray-400">Comparing directories...</div>
        </div>
      </div>

      <!-- Empty State -->
      <div
        v-if="!hasResults && !comparing"
        class="flex items-center justify-center py-12"
      >
        <div class="text-center text-gray-500 text-sm">
          Click "Compare Directories" to analyze differences
        </div>
      </div>
    </div>

    <template #footer>
      <Button variant="ghost" @click="closeModal">Close</Button>
      <Button
        v-if="hasResults"
        variant="primary"
        :loading="syncing"
        :disabled="!syncDirection"
        @click="handleSync"
      >
        Sync Directories
      </Button>
    </template>
  </Modal>
</template>

<script setup lang="ts">
import { ref, computed } from "vue";
import {
  GitCompare,
  File,
  Folder,
  ArrowLeftRight,
  FileX,
  Check,
  AlertCircle,
} from "lucide-vue-next";
import Modal from "../ui/Modal.vue";
import Button from "../ui/Button.vue";
import Form from "../ui/Form.vue";
import Select from "../ui/Select.vue";
import { useOverlay } from "../../composables/useOverlay";
import { useSFTPStore } from "../../stores/sftp";
import { message } from "../../utils/message";
import { getErrorMessage } from "../../utils/helpers";
import type { DiffEntry, DiffType, SyncDirection, SyncOperation } from "../../types/sftp";

const { closeOverlay, getOverlayProp } = useOverlay();
const sftpStore = useSFTPStore();

const syncForm = ref<InstanceType<typeof Form> | null>(null);
const comparing = ref(false);
const syncing = ref(false);
const diffs = ref<DiffEntry[]>([]);

const localPath = getOverlayProp<string>(
  "sftp-sync-compare-modal",
  "localPath",
  "",
  "/",
);

const remotePath = getOverlayProp<string>(
  "sftp-sync-compare-modal",
  "remotePath",
  "",
  "/",
);

const syncDirection = ref<SyncDirection>("localToRemote");
const syncOptions = ref({
  deleteExtraFiles: false,
  preservePermissions: true,
  preserveSymlinks: true,
  maxFileSize: null as number | null,
  excludePatterns: [] as string[],
});

const syncDirectionOptions = [
  { value: "localToRemote", label: "Local → Remote" },
  { value: "remoteToLocal", label: "Remote → Local" },
  { value: "bidirectional", label: "Bidirectional" },
];

const hasResults = computed(() => diffs.value.length > 0);

const summary = computed(() => {
  return {
    onlyLocal: diffs.value.filter((d) => d.diffType === "onlyLocal").length,
    onlyRemote: diffs.value.filter((d) => d.diffType === "onlyRemote").length,
    different:
      diffs.value.filter(
        (d) =>
          d.diffType === "sizeDiffers" ||
          d.diffType === "timeDiffers" ||
          d.diffType === "permissionsDiffer",
      ).length,
    identical: diffs.value.filter((d) => d.diffType === "identical").length,
  };
});

const filteredDiffs = computed(() => {
  // Filter out identical files
  return diffs.value.filter((d) => d.diffType !== "identical");
});

async function handleCompare() {
  if (!sftpStore.activeSessionId) {
    message.error("No active SFTP session");
    return;
  }

  comparing.value = true;
  try {
    const results = await sftpStore.compareDirectories(
      sftpStore.activeSessionId,
      localPath.value,
      remotePath.value,
    );
    diffs.value = results;
    message.success(
      `Comparison complete: ${results.length} files analyzed`,
    );
  } catch (error) {
    console.error("Failed to compare directories:", error);
    message.error(
      getErrorMessage(error, "Failed to compare directories"),
    );
  } finally {
    comparing.value = false;
  }
}

async function handleSync() {
  if (!sftpStore.activeSessionId || !syncDirection.value) {
    message.error("Please select sync direction");
    return;
  }

  const isValid = await syncForm.value?.validate();
  if (!isValid) return;

  syncing.value = true;
  try {
    const operation: SyncOperation = {
      direction: syncDirection.value,
      localPath: localPath.value,
      remotePath: remotePath.value,
      ...syncOptions.value,
    };
    await sftpStore.syncDirectories(sftpStore.activeSessionId, operation);
    message.success("Sync completed successfully");
    // Refresh comparison after sync
    await handleCompare();
  } catch (error) {
    console.error("Failed to sync directories:", error);
    message.error(getErrorMessage(error, "Failed to sync directories"));
  } finally {
    syncing.value = false;
  }
}

function formatFileInfo(entry: any): string {
  const parts: string[] = [];
  if (entry.size !== null) {
    parts.push(formatBytes(entry.size));
  }
  if (entry.fileType === "directory") {
    parts.push("dir");
  }
  if (entry.permissions) {
    parts.push(`0o${entry.permissions.toString(8)}`);
  }
  return parts.join(" • ");
}

function formatBytes(bytes: number): string {
  const units = ["B", "KB", "MB", "GB"];
  let size = bytes;
  let unitIndex = 0;
  while (size >= 1024 && unitIndex < units.length - 1) {
    size /= 1024;
    unitIndex++;
  }
  return `${size.toFixed(1)} ${units[unitIndex]}`;
}

function getDiffLabel(type: DiffType): string {
  switch (type) {
    case "onlyLocal":
      return "Only Local";
    case "onlyRemote":
      return "Only Remote";
    case "sizeDiffers":
      return "Size Differs";
    case "timeDiffers":
      return "Time Differs";
    case "permissionsDiffer":
      return "Permissions Differ";
    case "identical":
      return "Identical";
    default:
      return "Unknown";
  }
}

function getDiffIcon(type: DiffType) {
  switch (type) {
    case "onlyLocal":
      return File;
    case "onlyRemote":
      return File;
    case "sizeDiffers":
    case "timeDiffers":
    case "permissionsDiffer":
      return AlertCircle;
    default:
      return Check;
  }
}

function getDiffIconColor(type: DiffType): string {
  switch (type) {
    case "onlyLocal":
      return "text-blue-400";
    case "onlyRemote":
      return "text-green-400";
    case "sizeDiffers":
    case "timeDiffers":
    case "permissionsDiffer":
      return "text-yellow-400";
    default:
      return "text-gray-400";
  }
}

function getDiffBadgeClass(type: DiffType): string {
  switch (type) {
    case "onlyLocal":
      return "bg-blue-500/20 text-blue-400";
    case "onlyRemote":
      return "bg-green-500/20 text-green-400";
    case "sizeDiffers":
    case "timeDiffers":
    case "permissionsDiffer":
      return "bg-yellow-500/20 text-yellow-400";
    default:
      return "bg-gray-500/20 text-gray-400";
  }
}

function getDiffClass(type: DiffType): string {
  switch (type) {
    case "onlyLocal":
      return "border-l-2 border-blue-500";
    case "onlyRemote":
      return "border-l-2 border-green-500";
    case "sizeDiffers":
    case "timeDiffers":
    case "permissionsDiffer":
      return "border-l-2 border-yellow-500";
    default:
      return "";
  }
}

function closeModal() {
  diffs.value = [];
  syncDirection.value = "localToRemote";
  closeOverlay("sftp-sync-compare-modal");
}
</script>

