<template>
  <Card>
    <div v-if="!currentDatabase" class="text-gray-400 text-sm">
      No database selected. Please add or select a database first.
    </div>

    <div v-else class="space-y-4">
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
        <ul class="text-xs text-gray-400 mt-3">
          <li>• Push: Upload local changes to the remote database.</li>
          <li>• Pull: Download remote changes to the local database.</li>
          <li>• Sync Both: Perform a two-way synchronization.</li>
        </ul>
      </div>

      <!-- Last Sync Info -->
      <div class="border border-gray-700 rounded-lg p-4">
        <div class="flex items-center mb-3">
          <h4 class="text-sm font-medium text-gray-100">Last Sync</h4>
        </div>

        <div
          v-if="isLoadingStatus"
          class="text-center py-4 text-gray-400 text-sm"
        >
          Loading sync status...
        </div>

        <div v-else-if="syncStatus?.lastSync" class="space-y-2 text-xs">
          <div class="flex justify-between">
            <span class="text-gray-400">Time:</span>
            <span class="text-gray-200">{{
              formatDateOrNever(
                syncStatus.lastSync.completedAt ||
                  syncStatus.lastSync.startedAt,
              )
            }}</span>
          </div>
          <div class="flex justify-between">
            <span class="text-gray-400">Direction:</span>
            <span class="text-gray-200">{{
              syncStatus.lastSync.direction
            }}</span>
          </div>
          <div class="flex justify-between">
            <span class="text-gray-400">Status:</span>
            <Badge :variant="getStatusVariant(syncStatus.lastSync.status)">
              {{ syncStatus.lastSync.status }}
            </Badge>
          </div>
          <div class="flex justify-between">
            <span class="text-gray-400">Records Synced:</span>
            <span class="text-gray-200">{{
              syncStatus.lastSync.recordsSynced || 0
            }}</span>
          </div>
          <div
            v-if="syncStatus.lastSync.conflictsResolved"
            class="flex justify-between"
          >
            <span class="text-gray-400">Conflicts Resolved:</span>
            <span class="text-gray-200">{{
              syncStatus.lastSync.conflictsResolved
            }}</span>
          </div>
          <div
            v-if="syncStatus.lastSync.manualConflicts"
            class="flex justify-between"
          >
            <span class="text-gray-400">Manual Conflicts:</span>
            <span class="text-yellow-400">{{
              syncStatus.lastSync.manualConflicts
            }}</span>
          </div>
          <div
            v-if="syncStatus.lastSync.errorMessage"
            class="mt-2 pt-2 border-t border-gray-700"
          >
            <span class="text-red-400 text-xs">{{
              syncStatus.lastSync.errorMessage
            }}</span>
          </div>
        </div>
        <div v-else class="text-center py-4 text-gray-400 text-sm">
          No sync history yet
        </div>
      </div>

      <!-- Sync Logs -->
      <div class="border border-gray-700 rounded-lg p-4">
        <div class="flex items-center mb-3">
          <h4 class="text-sm font-medium text-gray-100">Recent Sync History</h4>
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
import { useSSHStore } from "../../stores/ssh";
import { useSshKeyStore } from "../../stores/sshKey";
import { useSavedCommandStore } from "../../stores/savedCommand";
import { useTunnelStore } from "../../stores/tunnel";
import type { SyncLogStatus } from "../../types/sync";

const syncStore = useSyncStore();
const sshStore = useSSHStore();
const sshKeyStore = useSshKeyStore();
const savedCommandStore = useSavedCommandStore();
const tunnelStore = useTunnelStore();
const isSyncing = ref(false);
const isLoadingLogs = ref(false);
const isLoadingStatus = ref(false);
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

    // Reload sync status and statistics
    await Promise.all([loadStatus(), loadLogs(), loadStatistics()]);

    // Reload all application data in background
    reloadAllData();
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

  isLoadingStatus.value = true;
  try {
    await syncStore.loadSyncStatus(currentDatabase.value.id);
  } catch (error) {
    console.error("Failed to load sync status:", error);
  } finally {
    isLoadingStatus.value = false;
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

const loadStatistics = async () => {
  try {
    await syncStore.loadStatistics();
  } catch (error) {
    console.error("Failed to load statistics:", error);
  }
};

const reloadAllData = async () => {
  try {
    // Reload SSH profiles and groups
    await sshStore.loadProfiles();
    await sshStore.loadGroups();

    // Reload SSH keys
    await sshKeyStore.loadKeys();

    // Reload saved commands and groups
    await savedCommandStore.loadCommands();
    await savedCommandStore.loadGroups();

    // Reload tunnels
    await tunnelStore.loadTunnels();

    console.log("All data reloaded successfully after sync");
  } catch (error) {
    console.error("Failed to reload data:", error);
    // Don't show error message to user since this is background refresh
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
  async (newDb: any) => {
    if (newDb) {
      await Promise.all([loadStatus(), loadLogs(), loadStatistics()]);
      startAutoRefresh();
    } else {
      stopAutoRefresh();
    }
  },
  { immediate: true },
);

// Watch for database connection changes
watch(
  () => currentDatabase.value?.isActive,
  async (isActive: boolean | undefined) => {
    if (isActive && currentDatabase.value) {
      await Promise.all([loadStatus(), loadLogs(), loadStatistics()]);
    }
  },
);

onMounted(async () => {
  if (!syncStore.databases.length) {
    await syncStore.loadDatabases();
  }

  // Load initial data if database is already selected
  if (currentDatabase.value) {
    await Promise.all([loadStatus(), loadLogs(), loadStatistics()]);
  }

  startAutoRefresh();
});

onUnmounted(() => {
  stopAutoRefresh();
});
</script>
