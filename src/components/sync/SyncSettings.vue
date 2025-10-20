<template>
  <Card title="Sync Settings" subtitle="Configure synchronization preferences">
    <div v-if="!currentDatabase" class="text-gray-400 text-sm">
      No database selected. Please add or select a database first.
    </div>

    <div v-else class="space-y-4">
      <!-- Current Database Info -->
      <div class="bg-gray-800 rounded-lg p-4 border border-gray-700">
        <div class="flex items-center justify-between">
          <div>
            <h4 class="text-sm font-medium text-gray-100">{{ currentDatabase.name }}</h4>
            <p class="text-xs text-gray-400 mt-1">
              {{ getDatabaseTypeLabel(currentDatabase.dbType) }}
            </p>
          </div>
          <Badge :variant="currentDatabase.isActive ? 'success' : 'default'">
            {{ currentDatabase.isActive ? 'Connected' : 'Disconnected' }}
          </Badge>
        </div>
      </div>

      <!-- Auto Sync Toggle -->
      <div class="border border-gray-700 rounded-lg p-4">
        <div class="flex items-center justify-between mb-4">
          <div>
            <h4 class="text-sm font-medium text-gray-100">Auto Sync</h4>
            <p class="text-xs text-gray-400 mt-1">
              Automatically sync data at regular intervals
            </p>
          </div>
          <Checkbox
            id="auto-sync-enabled"
            v-model="autoSyncEnabled"
            @update:modelValue="handleAutoSyncToggle"
          />
        </div>

        <!-- Sync Interval (shown when auto-sync is enabled) -->
        <div v-if="autoSyncEnabled" class="mt-4 pt-4 border-t border-gray-700">
          <Input
            id="sync-interval"
            v-model.number="syncInterval"
            label="Sync Interval (minutes)"
            type="number"
            :min="1"
            :max="1440"
            placeholder="15"
            @blur="handleIntervalChange"
          />
          <p class="text-xs text-gray-400 mt-2">
            Minimum: 1 minute, Maximum: 24 hours (1440 minutes)
          </p>
        </div>
      </div>

      <!-- Conflict Resolution Strategy -->
      <div class="border border-gray-700 rounded-lg p-4">
        <h4 class="text-sm font-medium text-gray-100 mb-2">
          Conflict Resolution Strategy
        </h4>
        <p class="text-xs text-gray-400 mb-4">
          Choose how to handle data conflicts when syncing
        </p>

        <Select
          id="conflict-strategy"
          v-model="conflictStrategy"
          :options="conflictStrategyOptions"
          @update:modelValue="handleStrategyChange"
        />

        <!-- Strategy Description -->
        <div class="mt-4 bg-gray-900 rounded-lg p-3 border border-gray-700">
          <p class="text-xs text-gray-300">{{ strategyDescription }}</p>
        </div>
      </div>

      <!-- Last Sync Info -->
      <div v-if="currentDatabase.lastSyncAt" class="border border-gray-700 rounded-lg p-4">
        <h4 class="text-sm font-medium text-gray-100 mb-2">Last Sync</h4>
        <p class="text-xs text-gray-400">
          {{ formatDate(currentDatabase.lastSyncAt) }}
        </p>
      </div>

      <!-- Actions -->
      <div class="flex gap-2">
        <Button
          variant="outline"
          :loading="isLoading"
          @click="handleReset"
        >
          Reset to Defaults
        </Button>
        <Button
          variant="primary"
          :loading="isLoading"
          @click="handleSave"
        >
          Save Settings
        </Button>
      </div>
    </div>
  </Card>
</template>

<script setup lang="ts">
import { ref, computed, watch, onMounted } from "vue";
import Card from "../ui/Card.vue";
import Badge from "../ui/Badge.vue";
import Checkbox from "../ui/Checkbox.vue";
import Input from "../ui/Input.vue";
import Select from "../ui/Select.vue";
import Button from "../ui/Button.vue";
import { message } from "../../utils/message";
import { getErrorMessage } from "../../utils/helpers";
import { useSyncStore } from "../../stores/sync";
import type { ConflictResolutionStrategy, DatabaseType } from "../../types/sync";

const syncStore = useSyncStore();
const isLoading = ref(false);

const autoSyncEnabled = ref(false);
const syncInterval = ref(15);
const conflictStrategy = ref<ConflictResolutionStrategy>("LastWriteWins");

const currentDatabase = computed(() => syncStore.currentDatabase);

const conflictStrategyOptions = [
  { value: "LastWriteWins", label: "Last Write Wins" },
  { value: "FirstWriteWins", label: "First Write Wins" },
  { value: "LocalWins", label: "Local Wins" },
  { value: "RemoteWins", label: "Remote Wins" },
  { value: "Manual", label: "Manual Resolution" },
];

const strategyDescription = computed(() => {
  const descriptions: Record<ConflictResolutionStrategy, string> = {
    LastWriteWins: "The most recently modified data will be kept. Changes with newer timestamps override older ones.",
    FirstWriteWins: "The oldest data will be preserved. Changes with older timestamps are kept, newer ones are discarded.",
    LocalWins: "Data on this device will always be kept in case of conflicts. Server data is ignored.",
    RemoteWins: "Server data will always be kept in case of conflicts. Local changes are discarded.",
    Manual: "You will be prompted to manually resolve each conflict. Review and choose which data to keep.",
  };
  return descriptions[conflictStrategy.value] || "";
});

const getDatabaseTypeLabel = (type: DatabaseType): string => {
  const labels: Record<DatabaseType, string> = {
    mysql: "MySQL Database",
    postgresql: "PostgreSQL Database",
    mongodb: "MongoDB Database",
  };
  return labels[type] || type;
};

const formatDate = (dateStr: string): string => {
  const date = new Date(dateStr);
  const now = new Date();
  const diff = now.getTime() - date.getTime();
  const seconds = Math.floor(diff / 1000);
  const minutes = Math.floor(seconds / 60);
  const hours = Math.floor(minutes / 60);
  const days = Math.floor(hours / 24);

  if (seconds < 60) return "Just now";
  if (minutes < 60) return `${minutes} minute${minutes > 1 ? 's' : ''} ago`;
  if (hours < 24) return `${hours} hour${hours > 1 ? 's' : ''} ago`;
  if (days < 7) return `${days} day${days > 1 ? 's' : ''} ago`;

  return date.toLocaleDateString() + ' ' + date.toLocaleTimeString();
};

const loadSettings = () => {
  if (!currentDatabase.value) return;

  autoSyncEnabled.value = currentDatabase.value.autoSyncEnabled;

  if (currentDatabase.value.syncSettings) {
    try {
      const settings = JSON.parse(currentDatabase.value.syncSettings);
      syncInterval.value = settings.syncIntervalMinutes || 15;
      conflictStrategy.value = settings.conflictResolutionStrategy || "LastWriteWins";
    } catch (e) {
      console.error("Failed to parse sync settings:", e);
    }
  }
};

const handleAutoSyncToggle = async (enabled: boolean) => {
  if (!currentDatabase.value) return;

  isLoading.value = true;
  try {
    if (enabled) {
      await syncStore.enableAutoSync(currentDatabase.value.id);
      message.success("Auto-sync enabled");
    } else {
      await syncStore.disableAutoSync(currentDatabase.value.id);
      message.success("Auto-sync disabled");
    }
  } catch (error) {
    console.error("Failed to toggle auto-sync:", error);
    message.error(getErrorMessage(error, "Failed to toggle auto-sync"));
    autoSyncEnabled.value = !enabled;
  } finally {
    isLoading.value = false;
  }
};

const handleIntervalChange = async () => {
  if (!currentDatabase.value) return;
  if (syncInterval.value < 1) syncInterval.value = 1;
  if (syncInterval.value > 1440) syncInterval.value = 1440;
};

const handleStrategyChange = () => {
  // Strategy will be saved when user clicks Save Settings
};

const handleSave = async () => {
  if (!currentDatabase.value) return;

  isLoading.value = true;
  try {
    const settings = {
      autoSync: autoSyncEnabled.value,
      syncIntervalMinutes: syncInterval.value,
      conflictResolutionStrategy: conflictStrategy.value,
    };

    await syncStore.updateDatabase(currentDatabase.value.id, {
      syncSettings: JSON.stringify(settings),
    });

    message.success("Settings saved successfully");
  } catch (error) {
    console.error("Failed to save settings:", error);
    message.error(getErrorMessage(error, "Failed to save settings"));
  } finally {
    isLoading.value = false;
  }
};

const handleReset = () => {
  autoSyncEnabled.value = false;
  syncInterval.value = 15;
  conflictStrategy.value = "LastWriteWins";
  message.info("Settings reset to defaults");
};

watch(() => currentDatabase.value, (newDb) => {
  if (newDb) {
    loadSettings();
  }
}, { immediate: true });

onMounted(() => {
  if (!syncStore.databases.length) {
    syncStore.loadDatabases();
  }
  loadSettings();
});
</script>
