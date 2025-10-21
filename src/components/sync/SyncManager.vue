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
          @click="openEditDatabaseModal"
        >
          Add Database
        </Button>
        <Button
          v-if="selectedDatabaseId"
          variant="outline"
          :icon="Settings"
          @click="openEditDatabaseModal"
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

        <!-- Settings Tab -->
        <div v-show="activeTab === 'settings'">
          <SyncSettings />
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
                      <FileWarning class="w-4 h-4 text-yellow-500 flex-shrink-0" />
                      <h4 class="text-sm font-medium text-gray-100 truncate">
                        {{ formatEntityType(conflict.entityType) }}
                      </h4>
                      <Badge variant="warning" size="sm">Unresolved</Badge>
                    </div>
                    <div class="text-xs text-gray-400 space-y-1">
                      <div class="flex items-center gap-1.5">
                        <Hash class="w-3 h-3" />
                        <span class="font-mono truncate">{{ conflict.entityId }}</span>
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
} from "lucide-vue-next";
import Modal from "../ui/Modal.vue";
import Badge from "../ui/Badge.vue";
import Button from "../ui/Button.vue";
import Select from "../ui/Select.vue";
import NavigationTabs from "../ui/NavigationTabs.vue";
import SyncStatus from "./SyncStatus.vue";
import SyncSettings from "./SyncSettings.vue";
import DeviceManager from "./DeviceManager.vue";
import ExternalDatabaseModal from "./ExternalDatabaseModal.vue";
import ConflictResolutionModal from "./ConflictResolutionModal.vue";
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

const tabs = [
  { id: "status", label: "Status & Logs", icon: Activity },
  { id: "settings", label: "Settings", icon: Settings },
  { id: "conflicts", label: "Conflicts", icon: AlertTriangle },
  { id: "devices", label: "Devices", icon: Laptop },
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

const openEditDatabaseModal = () => {
  openOverlay("external-database-modal", {
    databaseId: selectedDatabaseId.value || null,
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
  (newId) => {
    if (newId) {
      syncStore.setCurrentDatabase(newId);
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

  if (syncStore.databases.length > 0 && !selectedDatabaseId.value) {
    selectedDatabaseId.value = syncStore.databases[0].id;
  }

  if (syncStore.currentDatabaseId) {
    selectedDatabaseId.value = syncStore.currentDatabaseId;
  }

  loadConflicts();
});
</script>
