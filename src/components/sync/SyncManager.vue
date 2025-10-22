<template>
  <Modal id="sync-manager-modal" title="Sync Manager" size="2xl">
    <div class="space-y-6">
      <!-- Header with Database Selector and Actions -->
      <Card>
        <div class="flex items-center gap-3 mb-3">
          <Database class="w-5 h-5 text-gray-400 flex-shrink-0" />
          <div class="flex-1">
            <Select
              id="database-selector"
              v-model="selectedDatabaseId"
              :options="databaseOptions"
              placeholder="Choose a database to manage"
              :space="false"
            />
          </div>
          <Button
            variant="primary"
            :icon="Database"
            @click="openAddDatabaseModal"
          >
            Add Database
          </Button>
        </div>

        <!-- Database Actions (shown when a database is selected) -->
        <div
          v-if="selectedDatabaseId"
          class="flex items-center justify-between pt-3 border-t border-gray-700"
        >
          <div class="flex items-center gap-2">
            <Button
              variant="ghost"
              size="sm"
              :icon="Settings"
              @click="openEditDatabaseModal"
              title="Edit database settings"
            >
              Edit
            </Button>
            <Button
              variant="ghost"
              size="sm"
              :icon="Trash2"
              @click="confirmDeleteDatabase"
              title="Delete database"
            >
              Delete
            </Button>
          </div>
          <div class="flex items-center gap-2">
            <Button
              :variant="currentDatabase?.isActive ? 'danger' : 'success'"
              size="sm"
              :icon="currentDatabase?.isActive ? Unplug : Plug"
              :loading="isConnecting"
              @click="toggleConnection"
            >
              {{ currentDatabase?.isActive ? "Disconnect" : "Connect" }}
            </Button>
          </div>
        </div>
      </Card>

      <!-- Tabs -->
      <NavigationTabs v-model="activeTab" :tabs="tabs" />

      <!-- Tab Content -->
      <div class="min-h-[400px]">
        <!-- Status Tab -->
        <div v-show="activeTab === 'status'">
          <SyncStatus />
        </div>

        <!-- Global Sync Settings Tab -->
        <div v-show="activeTab === 'settings'">
          <SyncSettings ref="syncSettingsRef" />
        </div>

        <!-- Conflicts Tab -->
        <div v-show="activeTab === 'conflicts'">
          <ConflictList ref="conflictListRef" />
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
  Trash2,
} from "lucide-vue-next";
import Modal from "../ui/Modal.vue";
import Badge from "../ui/Badge.vue";
import Button from "../ui/Button.vue";
import Select from "../ui/Select.vue";
import Card from "../ui/Card.vue";
import NavigationTabs from "../ui/NavigationTabs.vue";
import SyncStatus from "./SyncStatus.vue";
import SyncSettings from "./SyncSettings.vue";
import ConflictList from "./ConflictList.vue";
import DeviceManager from "./DeviceManager.vue";
import ExternalDatabaseModal from "./ExternalDatabaseModal.vue";
import ConflictResolutionModal from "./ConflictResolutionModal.vue";
import { syncService } from "../../services/sync";
import { message } from "../../utils/message";
import { getErrorMessage } from "../../utils/helpers";
import { useSyncStore } from "../../stores/sync";
import { useOverlay } from "../../composables/useOverlay";

const syncStore = useSyncStore();
const { openOverlay } = useOverlay();

const activeTab = ref("status");
const selectedDatabaseId = ref<string>("");
const isConnecting = ref(false);
const syncSettingsRef = ref<InstanceType<typeof SyncSettings>>();
const conflictListRef = ref<InstanceType<typeof ConflictList>>();

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

    selectedDatabaseId.value = "";
    if (syncStore.databases.length > 0) {
      selectedDatabaseId.value = syncStore.databases[0].id;
    }
  } catch (error) {
    console.error("Failed to delete database:", error);
    message.error(getErrorMessage(error, "Failed to delete database"));
  }
};

watch(
  () => selectedDatabaseId.value,
  async (newId) => {
    if (newId) {
      syncStore.setCurrentDatabase(newId);

      try {
        await syncService.updateGlobalSyncSettings({
          selectedDatabaseId: newId,
        });
      } catch (error) {
        console.error("Failed to save selected database:", error);
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

  try {
    const globalSettings = await syncService.getGlobalSyncSettings();
    if (globalSettings?.selectedDatabaseId) {
      const dbExists = syncStore.databases.find(
        (db) => db.id === globalSettings.selectedDatabaseId,
      );
      if (dbExists) {
        selectedDatabaseId.value = globalSettings.selectedDatabaseId;
      }
    }
  } catch (e) {
    // ignore
  }

  if (!selectedDatabaseId.value && syncStore.databases.length > 0) {
    selectedDatabaseId.value = syncStore.databases[0].id;
  }

  if (syncStore.currentDatabaseId) {
    selectedDatabaseId.value = syncStore.currentDatabaseId;
  }
});
</script>
