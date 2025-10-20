<template>
  <Modal
    id="external-database-modal"
    :title="databaseId ? 'Edit External Database' : 'Add External Database'"
    size="md"
  >
    <Form ref="databaseForm" @submit="handleSubmit">
      <!-- Basic Information -->
      <h4 class="text-sm font-medium text-gray-100 border-b border-gray-700 pb-2 mb-2">
        Basic Information
      </h4>

      <Input
        id="db-name"
        v-model="database.name"
        label="Database Name"
        placeholder="My Production Database"
        rules="required|min:3|max:50"
        :autofocus="true"
      />

      <Select
        id="db-type"
        v-model="database.dbType"
        label="Database Type"
        placeholder="Select database type"
        :options="databaseTypeOptions"
        rules="required"
        :disabled="!!databaseId"
      />

      <!-- Connection Details -->
      <h4 class="text-sm font-medium text-gray-100 border-b border-gray-700 pb-2 mb-2 mt-4">
        Connection Details
      </h4>

      <div class="grid grid-cols-1 md:grid-cols-3 gap-4">
        <div class="md:col-span-2">
          <Input
            id="db-host"
            v-model="connectionDetails.host"
            label="Host"
            placeholder="localhost or db.example.com"
            rules="required"
          />
        </div>

        <Input
          id="db-port"
          v-model.number="connectionDetails.port"
          label="Port"
          type="number"
          :placeholder="defaultPort"
          :min="1"
          :max="65535"
          rules="required|min_value:1|max_value:65535"
        />
      </div>

      <Input
        id="db-username"
        v-model="connectionDetails.username"
        label="Username"
        placeholder="root"
        rules="required"
      />

      <Input
        id="db-password"
        v-model="connectionDetails.password"
        label="Password"
        type="password"
        :placeholder="databaseId ? 'Leave empty to keep current password' : 'Enter password'"
        :rules="databaseId ? '' : 'required'"
      />
      <div v-if="databaseId" class="text-xs text-gray-400 -mt-2">
        Leave empty to keep the current password
      </div>

      <Input
        id="db-database"
        v-model="connectionDetails.database"
        label="Database Name"
        placeholder="kerminal_sync"
        rules="required"
      />

      <!-- Sync Settings -->
      <Collapsible
        title="Sync Settings"
        subtitle="Configure automatic synchronization"
        :default-expanded="false"
      >
        <Checkbox
          id="auto-sync"
          v-model="syncSettings.autoSync"
          label="Enable Auto Sync"
        />

        <Input
          v-if="syncSettings.autoSync"
          id="sync-interval"
          v-model.number="syncSettings.syncIntervalMinutes"
          label="Sync Interval (minutes)"
          type="number"
          placeholder="15"
          :min="1"
          :max="1440"
          rules="required|min_value:1|max_value:1440"
        />

        <Select
          id="conflict-strategy"
          v-model="syncSettings.conflictResolutionStrategy"
          label="Conflict Resolution Strategy"
          placeholder="Select strategy"
          :options="conflictStrategyOptions"
          rules="required"
        />
      </Collapsible>

      <!-- Actions -->
      <template #footer>
        <div class="flex gap-2">
          <Button
            type="button"
            variant="outline"
            @click="testConnection"
            :loading="isTesting"
          >
            Test Connection
          </Button>
          <Button
            type="submit"
            variant="primary"
            :loading="isLoading"
          >
            {{ databaseId ? "Update Database" : "Add Database" }}
          </Button>
        </div>
      </template>
    </Form>
  </Modal>
</template>

<script setup lang="ts">
import { ref, computed, watch, onMounted } from "vue";
import Modal from "../ui/Modal.vue";
import Form from "../ui/Form.vue";
import Input from "../ui/Input.vue";
import Select from "../ui/Select.vue";
import Checkbox from "../ui/Checkbox.vue";
import Button from "../ui/Button.vue";
import Collapsible from "../ui/Collapsible.vue";
import { message } from "../../utils/message";
import { getErrorMessage } from "../../utils/helpers";
import { useSyncStore } from "../../stores/sync";
import { useOverlay } from "../../composables/useOverlay";
import type {
  DatabaseType,
  ConnectionDetails,
  SyncSettings,
  ConflictResolutionStrategy,
} from "../../types/sync";

const props = defineProps<{
  databaseId?: string | null;
}>();

const syncStore = useSyncStore();
const { closeOverlay, getOverlayProp } = useOverlay();

const databaseId = getOverlayProp(
  "external-database-modal",
  "databaseId",
  props.databaseId,
  null,
);

const databaseForm = ref<InstanceType<typeof Form> | null>(null);
const isLoading = ref(false);
const isTesting = ref(false);

const database = ref({
  name: "",
  dbType: "mysql" as DatabaseType,
});

const connectionDetails = ref<ConnectionDetails>({
  host: "",
  port: 3306,
  username: "",
  password: "",
  database: "",
});

const syncSettings = ref<SyncSettings>({
  autoSync: false,
  syncIntervalMinutes: 15,
  conflictResolutionStrategy: "LastWriteWins" as ConflictResolutionStrategy,
});

const databaseTypeOptions = [
  { value: "mysql", label: "MySQL" },
  { value: "postgresql", label: "PostgreSQL" },
  { value: "mongodb", label: "MongoDB" },
];

const conflictStrategyOptions = [
  { value: "LastWriteWins", label: "Last Write Wins (Newest)" },
  { value: "FirstWriteWins", label: "First Write Wins (Oldest)" },
  { value: "LocalWins", label: "Local Wins (Prefer This Device)" },
  { value: "RemoteWins", label: "Remote Wins (Prefer Server)" },
  { value: "Manual", label: "Manual Resolution (Ask Me)" },
];

const defaultPort = computed(() => {
  switch (database.value.dbType) {
    case "mysql":
      return "3306";
    case "postgresql":
      return "5432";
    case "mongodb":
      return "27017";
    default:
      return "3306";
  }
});

watch(
  () => database.value.dbType,
  () => {
    if (database.value.dbType === "mongodb") {
      connectionDetails.value.port = 27017;
    } else if (database.value.dbType === "mysql") {
      connectionDetails.value.port = 3306;
    } else if (database.value.dbType === "postgresql") {
      connectionDetails.value.port = 5432;
    }
  },
);

const loadDatabase = async () => {
  if (!databaseId.value) return;

  isLoading.value = true;
  try {
    const db = syncStore.databases.find((d) => d.id === databaseId.value);
    if (db) {
      database.value = {
        name: db.name,
        dbType: db.dbType,
      };

      syncSettings.value = {
        autoSync: db.autoSyncEnabled,
        syncIntervalMinutes: 15,
        conflictResolutionStrategy: "LastWriteWins",
      };

      if (db.syncSettings) {
        try {
          const parsed = JSON.parse(db.syncSettings);
          syncSettings.value = {
            ...syncSettings.value,
            ...parsed,
          };
        } catch (e) {
          console.error("Failed to parse sync settings:", e);
        }
      }

      connectionDetails.value.password = "";
    }
  } catch (error) {
    console.error("Error loading database:", error);
    message.error(getErrorMessage(error, "Failed to load database"));
  } finally {
    isLoading.value = false;
  }
};

const testConnection = async () => {
  const isValid = await databaseForm.value?.validate();
  if (!isValid) return;

  isTesting.value = true;
  try {
    let testId = databaseId.value;

    if (!testId) {
      const tempDb = await syncStore.addDatabase({
        name: `__test_${Date.now()}`,
        dbType: database.value.dbType,
        connectionDetailsEncrypted: JSON.stringify(connectionDetails.value),
        syncSettings: JSON.stringify(syncSettings.value),
        isActive: false,
        autoSyncEnabled: false,
        createdAt: new Date().toISOString(),
        updatedAt: new Date().toISOString(),
        deviceId: "",
        version: 0,
        syncStatus: "idle",
      });
      testId = tempDb.id;
    }

    const success = await syncStore.testConnection(testId);

    if (!databaseId.value && testId) {
      await syncStore.deleteDatabase(testId);
    }

    if (success) {
      message.success("Connection successful!");
    } else {
      message.error("Connection failed!");
    }
  } catch (error) {
    console.error("Connection test failed:", error);
    message.error(getErrorMessage(error, "Connection test failed"));
  } finally {
    isTesting.value = false;
  }
};

const handleSubmit = async () => {
  const isValid = await databaseForm.value?.validate();
  if (!isValid) return;

  isLoading.value = true;
  try {
    const dbData = {
      name: database.value.name,
      dbType: database.value.dbType,
      connectionDetailsEncrypted: JSON.stringify(connectionDetails.value),
      syncSettings: JSON.stringify(syncSettings.value),
      isActive: false,
      autoSyncEnabled: syncSettings.value.autoSync,
      createdAt: new Date().toISOString(),
      updatedAt: new Date().toISOString(),
      deviceId: "",
      version: 0,
      syncStatus: "idle",
    };

    if (databaseId.value) {
      await syncStore.updateDatabase(databaseId.value, {
        ...dbData,
        ...(connectionDetails.value.password && {
          connectionDetailsEncrypted: JSON.stringify(connectionDetails.value),
        }),
      });
      message.success("Database updated successfully");
    } else {
      await syncStore.addDatabase(dbData);
      message.success("Database added successfully");
    }

    closeOverlay("external-database-modal");
  } catch (error) {
    console.error("Error saving database:", error);
    message.error(getErrorMessage(error, "Failed to save database"));
  } finally {
    isLoading.value = false;
  }
};

watch(() => databaseId.value, (newId) => {
  if (newId) {
    loadDatabase();
  } else {
    database.value = {
      name: "",
      dbType: "mysql",
    };
    connectionDetails.value = {
      host: "",
      port: 3306,
      username: "",
      password: "",
      database: "",
    };
    syncSettings.value = {
      autoSync: false,
      syncIntervalMinutes: 15,
      conflictResolutionStrategy: "LastWriteWins",
    };
  }
}, { immediate: true });

onMounted(() => {
  if (!syncStore.databases.length) {
    syncStore.loadDatabases();
  }
});
</script>
