<template>
  <Modal id="sync-manager-modal" title="Sync Manager" size="2xl">
    <div class="space-y-6">
      <!-- Header with Database Selector -->
      <div class="flex items-start gap-3">
        <div class="flex-1">
          <Select
            id="database-selector"
            v-model="selectedDatabaseId"
            :options="databaseOptions"
            placeholder="Choose a database to manage"
          />
        </div>
        <Button
          variant="primary"
          :icon="Database"
          @click="openAddDatabaseModal"
        >
          Add Database
        </Button>
        <Button
          v-if="selectedDatabaseId"
          variant="outline"
          :icon="Settings"
          @click="openEditDatabaseModal"
          title="Edit database settings"
        />
        <Button
          v-if="selectedDatabaseId"
          variant="danger"
          :icon="Trash2"
          @click="confirmDeleteDatabase"
          title="Delete database"
        />
        <Button
          v-if="selectedDatabaseId"
          :variant="currentDatabase?.isActive ? 'danger' : 'success'"
          :icon="currentDatabase?.isActive ? Unplug : Plug"
          :loading="isConnecting"
          @click="toggleConnection"
        >
          {{ currentDatabase?.isActive ? "Disconnect" : "Connect" }}
        </Button>
      </div>

      <!-- Tabs -->
      <NavigationTabs v-model="activeTab" :tabs="tabs" />

      <!-- Tab Content -->
      <div class="min-h-[400px]">
        <!-- Status Tab -->
        <div v-show="activeTab === 'status'">
          <SyncStatus />
        </div>

        <!-- Global Sync Settings Tab (Phase 9) -->
        <div v-show="activeTab === 'settings'">
          <Card
            title="Global Sync Settings"
            subtitle="Configure sync behavior for all external databases"
          >
            <div
              v-if="isLoadingGlobal"
              class="flex items-center justify-center py-16 text-gray-400"
            >
              <RefreshCw class="w-6 h-6 animate-spin mr-3" />
              Loading settings...
            </div>

            <div v-else class="space-y-4">
              <!-- Master Controls -->
              <div class="border border-gray-700 rounded-lg p-4">
                <div class="flex items-center justify-between mb-4">
                  <div>
                    <h4 class="text-sm font-medium text-gray-100">
                      Master Controls
                    </h4>
                    <p class="text-xs text-gray-400 mt-1">
                      Enable or disable the entire sync system
                    </p>
                  </div>
                  <Badge
                    :variant="globalLocal.isActive ? 'success' : 'default'"
                  >
                    {{ globalLocal.isActive ? "Active" : "Inactive" }}
                  </Badge>
                </div>

                <Checkbox
                  id="global-is-active"
                  v-model="globalLocal.isActive"
                  label="Enable Sync System"
                  helper-text="Master switch for all sync operations"
                  @update:modelValue="markGlobalDirty"
                />

                <Checkbox
                  id="global-auto-sync"
                  v-model="globalLocal.autoSyncEnabled"
                  label="Auto Sync"
                  helper-text="Automatically sync at intervals"
                  :disabled="!globalLocal.isActive"
                  @update:modelValue="markGlobalDirty"
                />

                <div v-if="globalLocal.autoSyncEnabled">
                  <Input
                    id="global-interval"
                    v-model.number="globalLocal.syncIntervalMinutes"
                    label="Sync Interval (minutes)"
                    type="number"
                    :min="1"
                    :max="1440"
                    placeholder="15"
                    helper-text="How often to automatically sync (1-1440 minutes)"
                    @blur="markGlobalDirty"
                  />
                </div>
              </div>

              <div class="border border-gray-700 rounded-lg p-4">
                <!-- Conflict Resolution Strategy -->
                <Select
                  id="global-conflict-strategy"
                  v-model="globalLocal.conflictStrategy"
                  label="Conflict Resolution Strategy"
                  :helper-text="getGlobalStrategyDescription()"
                  :options="globalConflictStrategyOptions"
                  @update:modelValue="markGlobalDirty"
                />

                <!-- Sync Direction -->
                <Select
                  id="global-sync-direction"
                  v-model="globalLocal.syncDirection"
                  label="Sync Direction"
                  :helper-text="getGlobalDirectionDescription()"
                  :options="globalSyncDirectionOptions"
                  @update:modelValue="markGlobalDirty"
                />
              </div>

              <!-- Actions -->
              <div class="flex gap-2 justify-end">
                <Button
                  variant="outline"
                  :disabled="!globalDirty"
                  @click="resetGlobal"
                  >Reset</Button
                >
                <Button
                  variant="primary"
                  :disabled="!globalDirty || globalSaving"
                  @click="saveGlobal"
                  :loading="globalSaving"
                  >Save Settings</Button
                >
              </div>
            </div>
          </Card>
        </div>

        <!-- Conflicts Tab -->
        <div v-show="activeTab === 'conflicts'">
          <div
            v-if="isLoadingConflicts"
            class="flex items-center justify-center py-16 text-gray-400"
          >
            <RefreshCw class="w-6 h-6 animate-spin mr-3" />
            Loading conflicts...
          </div>

          <div v-else-if="!pendingConflicts.length" class="text-center py-16">
            <div class="flex justify-center mb-4">
              <CheckCircle2 class="w-16 h-16 text-green-500" />
            </div>
            <h3 class="text-lg font-semibold text-white mb-2">
              No Pending Conflicts
            </h3>
            <p class="text-sm text-gray-400">
              All sync conflicts have been resolved
            </p>
          </div>

          <div v-else class="space-y-3">
            <div class="flex items-center justify-between mb-4">
              <div class="flex items-center gap-2">
                <AlertTriangle class="w-5 h-5 text-yellow-500" />
                <h3 class="text-base font-semibold text-white">
                  Pending Conflicts
                </h3>
                <Badge variant="warning">{{ pendingConflicts.length }}</Badge>
              </div>
              <Button
                variant="ghost"
                size="sm"
                :icon="RefreshCw"
                :loading="isLoadingConflicts"
                @click="loadConflicts"
              >
                Refresh
              </Button>
            </div>

            <div class="space-y-2">
              <div
                v-for="conflict in pendingConflicts"
                :key="conflict.id"
                class="bg-gray-800/50 border border-yellow-700/30 rounded-lg p-4 hover:border-yellow-600/50 transition-colors"
              >
                <div class="flex items-start justify-between gap-4">
                  <div class="flex-1 min-w-0">
                    <div class="flex items-center gap-2 mb-2">
                      <FileWarning
                        class="w-4 h-4 text-yellow-500 flex-shrink-0"
                      />
                      <h4 class="text-sm font-medium text-gray-100 truncate">
                        {{ formatEntityType(conflict.entityType) }}
                      </h4>
                      <Badge variant="warning" size="sm">Unresolved</Badge>
                    </div>
                    <div class="text-xs text-gray-400 space-y-1">
                      <div class="flex items-center gap-1.5">
                        <Hash class="w-3 h-3" />
                        <span class="font-mono truncate">{{
                          conflict.entityId
                        }}</span>
                      </div>
                      <div class="flex items-center gap-1.5">
                        <Clock class="w-3 h-3" />
                        <span>{{ formatDateOrNever(conflict.createdAt) }}</span>
                      </div>
                    </div>
                  </div>
                  <Button
                    variant="primary"
                    size="sm"
                    :icon="GitMerge"
                    @click="openConflictModal(conflict.id)"
                  >
                    Resolve
                  </Button>
                </div>
              </div>
            </div>
          </div>
        </div>

        <!-- Devices Tab -->
        <div v-show="activeTab === 'devices'">
          <DeviceManager />
        </div>
      </div>
    </div>
  </Modal>

  <!-- Modals -->
  <ExternalDatabaseModal />
  <ConflictResolutionModal />
</template>

<script setup lang="ts">
import { ref, computed, watch, onMounted } from "vue";
import {
  Activity,
  Settings,
  AlertTriangle,
  Laptop,
  Database,
  Plug,
  Unplug,
  RefreshCw,
  CheckCircle2,
  FileWarning,
  Hash,
  Clock,
  GitMerge,
  Trash2,
} from "lucide-vue-next";
import Modal from "../ui/Modal.vue";
import Badge from "../ui/Badge.vue";
import Button from "../ui/Button.vue";
import Select from "../ui/Select.vue";
import NavigationTabs from "../ui/NavigationTabs.vue";
import Checkbox from "../ui/Checkbox.vue";
import Input from "../ui/Input.vue";
import SyncStatus from "./SyncStatus.vue";
import DeviceManager from "./DeviceManager.vue";
import ExternalDatabaseModal from "./ExternalDatabaseModal.vue";
import ConflictResolutionModal from "./ConflictResolutionModal.vue";
import Card from "../ui/Card.vue";
import { syncService } from "../../services/sync";
import { message } from "../../utils/message";
import { getErrorMessage } from "../../utils/helpers";
import { formatDateOrNever } from "../../utils/formatter";
import { useSyncStore } from "../../stores/sync";
import { useOverlay } from "../../composables/useOverlay";

const syncStore = useSyncStore();
const { openOverlay } = useOverlay();

const activeTab = ref("status");
const selectedDatabaseId = ref<string>("");
const isConnecting = ref(false);
const isLoadingConflicts = ref(false);
// Global sync settings (Phase 9)
const isLoadingGlobal = ref(false);
const globalSaving = ref(false);
const globalDirty = ref(false);
const globalSettings = ref<any | null>(null);
const globalLocal = ref<any>({});

const tabs = [
  { id: "status", label: "Status & Logs", icon: Activity },
  { id: "settings", label: "Settings", icon: Settings },
  { id: "conflicts", label: "Conflicts", icon: AlertTriangle },
  { id: "devices", label: "Devices", icon: Laptop },
];

const globalConflictStrategyOptions = [
  { value: "lastWriteWins", label: "Last Write Wins" },
  { value: "firstWriteWins", label: "First Write Wins" },
  { value: "localWins", label: "Local Wins" },
  { value: "remoteWins", label: "Remote Wins" },
  { value: "manual", label: "Manual Resolution" },
];

const globalSyncDirectionOptions = [
  { value: "both", label: "Bidirectional" },
  { value: "push", label: "Push Only" },
  { value: "pull", label: "Pull Only" },
];

const databaseOptions = computed(() => {
  if (!syncStore.databases.length) {
    return [{ value: "", label: "No databases configured" }];
  }
  return syncStore.databases.map((db) => ({
    value: db.id,
    label: `${db.name} (${db.dbType})`,
  }));
});

const currentDatabase = computed(() => syncStore.currentDatabase);
const pendingConflicts = computed(() => syncStore.pendingConflicts);

const formatEntityType = (type: string): string => {
  const types: Record<string, string> = {
    ssh_profile: "SSH Profile",
    ssh_group: "SSH Group",
    ssh_key: "SSH Key",
    saved_command: "Saved Command",
    saved_command_group: "Command Group",
  };
  return types[type] || type;
};

const openAddDatabaseModal = () => {
  openOverlay("external-database-modal", {
    databaseId: null,
  });
};

const openEditDatabaseModal = () => {
  if (!selectedDatabaseId.value) {
    message.error("Please select a database to edit");
    return;
  }

  const dbExists = syncStore.databases.find(
    (db) => db.id === selectedDatabaseId.value,
  );

  if (!dbExists) {
    message.error("Selected database not found");
    return;
  }

  openOverlay("external-database-modal", {
    databaseId: selectedDatabaseId.value,
  });
};

const openConflictModal = (conflictId: string) => {
  openOverlay("conflict-resolution-modal", { conflictId });
};

const toggleConnection = async () => {
  if (!selectedDatabaseId.value) return;

  isConnecting.value = true;
  try {
    if (currentDatabase.value?.isActive) {
      await syncStore.disconnect(selectedDatabaseId.value);
      message.success("Disconnected from database");
    } else {
      await syncStore.connect(selectedDatabaseId.value);
      message.success("Connected to database");
    }
  } catch (error) {
    console.error("Connection toggle failed:", error);
    message.error(getErrorMessage(error, "Connection failed"));
  } finally {
    isConnecting.value = false;
  }
};

const confirmDeleteDatabase = async () => {
  if (!selectedDatabaseId.value) return;

  const db = syncStore.databases.find((d) => d.id === selectedDatabaseId.value);
  if (!db) return;

  const confirmed = await window.confirm(
    `Are you sure you want to delete "${db.name}"?\n\nThis will remove all sync configuration for this database. This action cannot be undone.`,
  );

  if (!confirmed) return;

  try {
    await syncStore.deleteDatabase(selectedDatabaseId.value);
    message.success(`Database "${db.name}" deleted successfully`);

    // Clear selection and select first available database
    selectedDatabaseId.value = "";
    if (syncStore.databases.length > 0) {
      selectedDatabaseId.value = syncStore.databases[0].id;
    }
  } catch (error) {
    console.error("Failed to delete database:", error);
    message.error(getErrorMessage(error, "Failed to delete database"));
  }
};

const loadConflicts = async () => {
  isLoadingConflicts.value = true;
  try {
    await syncStore.loadConflicts();
  } catch (error) {
    console.error("Failed to load conflicts:", error);
    message.error(getErrorMessage(error, "Failed to load conflicts"));
  } finally {
    isLoadingConflicts.value = false;
  }
};

watch(
  () => selectedDatabaseId.value,
  async (newId) => {
    if (newId) {
      syncStore.setCurrentDatabase(newId);

      // Save selected database to global settings for persistence
      try {
        await syncService.updateGlobalSyncSettings({
          selectedDatabaseId: newId,
        });
      } catch (error) {
        console.error("Failed to save selected database:", error);
        // Non-critical error, just log it
      }
    }
  },
);

watch(
  () => syncStore.currentDatabaseId,
  (newId) => {
    if (newId && newId !== selectedDatabaseId.value) {
      selectedDatabaseId.value = newId;
    }
  },
);

onMounted(async () => {
  await syncStore.loadDatabases();

  // Load global settings first to restore last selected database
  try {
    await loadGlobalSettings();

    // Restore last selected database from global settings
    if (globalSettings.value?.selectedDatabaseId) {
      const dbExists = syncStore.databases.find(
        (db) => db.id === globalSettings.value.selectedDatabaseId,
      );
      if (dbExists) {
        selectedDatabaseId.value = globalSettings.value.selectedDatabaseId;
      }
    }
  } catch (e) {
    // ignore, loadGlobalSettings handles its own errors
  }

  // Fallback: Select first database if none selected
  if (!selectedDatabaseId.value && syncStore.databases.length > 0) {
    selectedDatabaseId.value = syncStore.databases[0].id;
  }

  // Override with store's current database if set
  if (syncStore.currentDatabaseId) {
    selectedDatabaseId.value = syncStore.currentDatabaseId;
  }

  loadConflicts();
});

// Load global settings when user switches to the Global tab
watch(
  () => activeTab.value,
  (tab) => {
    if (tab === "global" && !globalSettings.value) {
      loadGlobalSettings();
    }
  },
);

async function loadGlobalSettings() {
  isLoadingGlobal.value = true;
  try {
    const data = await syncService.getGlobalSyncSettings();
    globalSettings.value = data || null;
    if (data) {
      globalLocal.value = {
        isActive: data.isActive ?? false,
        autoSyncEnabled: data.autoSyncEnabled ?? false,
        syncIntervalMinutes: data.syncIntervalMinutes ?? 15,
        conflictStrategy: data.conflictStrategy ?? "manual",
        syncDirection: data.syncDirection ?? "both",
      };
    } else {
      globalLocal.value = {
        isActive: false,
        autoSyncEnabled: false,
        syncIntervalMinutes: 15,
        conflictStrategy: "manual",
        syncDirection: "both",
      };
    }
    globalDirty.value = false;
  } catch (error) {
    console.error("Failed to load global settings:", error);
    message.error(getErrorMessage(error, "Failed to load global settings"));
  } finally {
    isLoadingGlobal.value = false;
  }
}

function markGlobalDirty() {
  globalDirty.value = true;
}

function resetGlobal() {
  if (globalSettings.value) {
    globalLocal.value = { ...globalSettings.value };
    globalDirty.value = false;
    message.info("Settings reset to last saved state");
  }
}

async function saveGlobal() {
  if (!globalDirty.value) return;
  globalSaving.value = true;
  try {
    const updates = {
      isActive: globalLocal.value.isActive,
      autoSyncEnabled: globalLocal.value.autoSyncEnabled,
      syncIntervalMinutes: globalLocal.value.syncIntervalMinutes,
      conflictStrategy: globalLocal.value.conflictStrategy,
      syncDirection: globalLocal.value.syncDirection,
    };
    await syncService.updateGlobalSyncSettings(updates);
    await loadGlobalSettings();
    message.success("Global sync settings saved successfully");
  } catch (error) {
    console.error("Failed to save global settings:", error);
    message.error(getErrorMessage(error, "Failed to save global settings"));
  } finally {
    globalSaving.value = false;
  }
}

function getGlobalStrategyDescription(): string {
  const strategy = globalLocal.value.conflictStrategy;
  const descriptions: Record<string, string> = {
    lastWriteWins: "The most recently modified data will be kept",
    firstWriteWins: "The oldest data will be preserved",
    localWins: "Local data always wins in conflicts",
    remoteWins: "Remote data always wins in conflicts",
    manual: "You will be prompted to manually resolve each conflict",
  };
  return descriptions[strategy] || "";
}

function getGlobalDirectionDescription(): string {
  const direction = globalLocal.value.syncDirection;
  const descriptions: Record<string, string> = {
    both: "Sync data in both directions (local ↔ remote)",
    push: "Only push local data to remote (local → remote)",
    pull: "Only pull remote data to local (local ← remote)",
  };
  return descriptions[direction] || "";
}
</script>
