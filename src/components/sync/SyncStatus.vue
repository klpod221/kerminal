<template>
  <Card title="Sync Status" subtitle="Monitor synchronization activity">
    <div v-if="!currentDatabase" class="text-gray-400 text-sm">
      No database selected. Please add or select a database first.
    </div>

    <div v-else class="space-y-4">
      <!-- Connection Status -->
      <div class="bg-gray-800 rounded-lg p-4 border border-gray-700">
        <div class="flex items-center justify-between mb-3">
          <h4 class="text-sm font-medium text-gray-100">Connection Status</h4>
          <Badge :variant="currentDatabase.isActive ? 'success' : 'default'">
            {{ currentDatabase.isActive ? "Connected" : "Disconnected" }}
          </Badge>
        </div>
        <div class="text-xs text-gray-400">
          Database: {{ currentDatabase.name }} ({{ currentDatabase.dbType }})
        </div>
      </div>

      <!-- Sync Actions -->
      <div class="border border-gray-700 rounded-lg p-4">
        <h4 class="text-sm font-medium text-gray-100 mb-3">Manual Sync</h4>
        <div class="grid grid-cols-3 gap-2">
          <Button
            variant="outline"
            size="sm"
            :loading="isSyncing && syncDirection === 'push'"
            :disabled="!currentDatabase.isActive || isSyncing"
            @click="handleSync('push')"
          >
            Push
          </Button>
          <Button
            variant="outline"
            size="sm"
            :loading="isSyncing && syncDirection === 'pull'"
            :disabled="!currentDatabase.isActive || isSyncing"
            @click="handleSync('pull')"
          >
            Pull
          </Button>
          <Button
            variant="primary"
            size="sm"
            :loading="isSyncing && syncDirection === 'bidirectional'"
            :disabled="!currentDatabase.isActive || isSyncing"
            @click="handleSync('bidirectional')"
          >
            Sync Both
          </Button>
        </div>
        <p class="text-xs text-gray-400 mt-3">
          Push: Upload local changes • Pull: Download remote changes • Sync
          Both: Two-way sync
        </p>
      </div>

      <!-- Last Sync Info -->
      <div v-if="syncStatus" class="border border-gray-700 rounded-lg p-4">
        <h4 class="text-sm font-medium text-gray-100 mb-3">Last Sync</h4>
        <div class="space-y-2 text-xs">
          <div class="flex justify-between">
            <span class="text-gray-400">Time:</span>
            <span class="text-gray-200">{{
              formatDateOrNever(
                syncStatus.lastSync?.completedAt ||
                  syncStatus.lastSync?.startedAt,
              )
            }}</span>
          </div>
          <div class="flex justify-between">
            <span class="text-gray-400">Direction:</span>
            <span class="text-gray-200">{{
              syncStatus.lastSync?.direction
            }}</span>
          </div>
          <div class="flex justify-between">
            <span class="text-gray-400">Status:</span>
            <Badge :variant="getStatusVariant(syncStatus.lastSync?.status)">
              {{ syncStatus.lastSync?.status }}
            </Badge>
          </div>
          <div class="flex justify-between">
            <span class="text-gray-400">Records Synced:</span>
            <span class="text-gray-200">{{
              syncStatus.lastSync?.recordsSynced || 0
            }}</span>
          </div>
          <div
            v-if="syncStatus.lastSync?.conflictsResolved"
            class="flex justify-between"
          >
            <span class="text-gray-400">Conflicts Resolved:</span>
            <span class="text-gray-200">{{
              syncStatus.lastSync.conflictsResolved
            }}</span>
          </div>
          <div
            v-if="syncStatus.lastSync?.manualConflicts"
            class="flex justify-between"
          >
            <span class="text-gray-400">Manual Conflicts:</span>
            <span class="text-yellow-400">{{
              syncStatus.lastSync.manualConflicts
            }}</span>
          </div>
          <div
            v-if="syncStatus.lastSync?.errorMessage"
            class="mt-2 pt-2 border-t border-gray-700"
          >
            <span class="text-red-400 text-xs">{{
              syncStatus.lastSync.errorMessage
            }}</span>
          </div>
        </div>
      </div>

      <!-- Sync Logs -->
      <div class="border border-gray-700 rounded-lg p-4">
        <div class="flex items-center justify-between mb-3">
          <h4 class="text-sm font-medium text-gray-100">Recent Sync History</h4>
          <Button
            variant="ghost"
            size="sm"
            :loading="isLoadingLogs"
            @click="loadLogs"
          >
            Refresh
          </Button>
        </div>

        <div
          v-if="isLoadingLogs"
          class="text-center py-4 text-gray-400 text-sm"
        >
          Loading sync history...
        </div>

        <div
          v-else-if="!syncLogs.length"
          class="text-center py-4 text-gray-400 text-sm"
        >
          No sync history yet
        </div>

        <div v-else class="space-y-2">
          <div
            v-for="log in syncLogs.slice(0, 5)"
            :key="log.id"
            class="bg-gray-800 rounded p-3 border border-gray-700"
          >
            <div class="flex items-center justify-between mb-2">
              <Badge :variant="getStatusVariant(log.status)">
                {{ log.status }}
              </Badge>
              <span class="text-xs text-gray-400">{{
                formatDateOrNever(log.startedAt)
              }}</span>
            </div>
            <div class="grid grid-cols-3 gap-2 text-xs">
              <div>
                <span class="text-gray-400">Direction:</span>
                <span class="text-gray-200 ml-1">{{ log.direction }}</span>
              </div>
              <div>
                <span class="text-gray-400">Records:</span>
                <span class="text-gray-200 ml-1">{{ log.recordsSynced }}</span>
              </div>
              <div>
                <span class="text-gray-400">Conflicts:</span>
                <span class="text-gray-200 ml-1">{{
                  log.conflictsResolved
                }}</span>
              </div>
            </div>
            <div v-if="log.errorMessage" class="mt-2 text-xs text-red-400">
              Error: {{ log.errorMessage }}
            </div>
          </div>
        </div>

        <div v-if="syncLogs.length > 5" class="mt-3 text-center">
          <Button variant="ghost" size="sm" @click="loadMoreLogs">
            View All History
          </Button>
        </div>
      </div>

      <!-- Statistics -->
      <div v-if="statistics" class="grid grid-cols-2 gap-4">
        <div class="bg-gray-800 rounded-lg p-4 border border-gray-700">
          <div class="text-xs text-gray-400 mb-1">Active Connections</div>
          <div class="text-2xl font-semibold text-gray-100">
            {{ statistics.activeConnections }}
          </div>
        </div>
        <div class="bg-gray-800 rounded-lg p-4 border border-gray-700">
          <div class="text-xs text-gray-400 mb-1">Auto-Sync Enabled</div>
          <div class="text-2xl font-semibold text-green-400">
            {{ statistics.autoSyncEnabledCount }}
          </div>
        </div>
      </div>
    </div>
  </Card>
</template>

<script setup lang="ts">
import { ref, computed, watch, onMounted, onUnmounted } from "vue";
import Card from "../ui/Card.vue";
import Badge from "../ui/Badge.vue";
import Button from "../ui/Button.vue";
import { message } from "../../utils/message";
import { getErrorMessage } from "../../utils/helpers";
import { formatDateOrNever } from "../../utils/formatter";
import { useSyncStore } from "../../stores/sync";
import type { SyncLogStatus } from "../../types/sync";

const syncStore = useSyncStore();
const isSyncing = ref(false);
const isLoadingLogs = ref(false);
const syncDirection = ref<"push" | "pull" | "bidirectional" | null>(null);

let refreshInterval: ReturnType<typeof setInterval> | null = null;

const currentDatabase = computed(() => syncStore.currentDatabase);
const syncStatus = computed(() => syncStore.syncStatus);
const syncLogs = computed(() => syncStore.syncLogs);
const statistics = computed(() => syncStore.statistics);

const getStatusVariant = (
  status?: SyncLogStatus,
): "success" | "danger" | "warning" | "default" => {
  switch (status) {
    case "Completed":
      return "success";
    case "Failed":
      return "danger";
    case "InProgress":
      return "warning";
    case "Cancelled":
      return "default";
    default:
      return "default";
  }
};

const handleSync = async (direction: "push" | "pull" | "bidirectional") => {
  if (!currentDatabase.value) return;

  isSyncing.value = true;
  syncDirection.value = direction;

  try {
    const log = await syncStore.sync(currentDatabase.value.id, direction);
    message.success(`Sync completed: ${log.recordsSynced} records synced`);

    await loadStatus();
    await loadLogs();
    await loadStatistics();
  } catch (error) {
    console.error("Sync failed:", error);
    message.error(getErrorMessage(error, "Sync failed"));
  } finally {
    isSyncing.value = false;
    syncDirection.value = null;
  }
};

const loadStatus = async () => {
  if (!currentDatabase.value) return;

  try {
    await syncStore.loadSyncStatus(currentDatabase.value.id);
  } catch (error) {
    console.error("Failed to load sync status:", error);
  }
};

const loadLogs = async () => {
  if (!currentDatabase.value) return;

  isLoadingLogs.value = true;
  try {
    await syncStore.loadSyncLogs(currentDatabase.value.id, 10);
  } catch (error) {
    console.error("Failed to load sync logs:", error);
    message.error(getErrorMessage(error, "Failed to load sync logs"));
  } finally {
    isLoadingLogs.value = false;
  }
};

const loadMoreLogs = async () => {
  if (!currentDatabase.value) return;

  try {
    await syncStore.loadSyncLogs(currentDatabase.value.id);
    message.info("Showing all sync history");
  } catch (error) {
    console.error("Failed to load more logs:", error);
    message.error(getErrorMessage(error, "Failed to load sync logs"));
  }
};

const loadStatistics = async () => {
  try {
    await syncStore.loadStatistics();
  } catch (error) {
    console.error("Failed to load statistics:", error);
  }
};

const startAutoRefresh = () => {
  refreshInterval = setInterval(() => {
    if (currentDatabase.value?.isActive) {
      loadStatus();
      loadStatistics();
    }
  }, 30000); // Refresh every 30 seconds
};

const stopAutoRefresh = () => {
  if (refreshInterval) {
    clearInterval(refreshInterval);
    refreshInterval = null;
  }
};

watch(
  () => currentDatabase.value,
  async (newDb) => {
    if (newDb) {
      await loadStatus();
      await loadLogs();
      await loadStatistics();
      startAutoRefresh();
    } else {
      stopAutoRefresh();
    }
  },
  { immediate: true },
);

onMounted(() => {
  if (!syncStore.databases.length) {
    syncStore.loadDatabases();
  }
  startAutoRefresh();
});

onUnmounted(() => {
  stopAutoRefresh();
});
</script>
