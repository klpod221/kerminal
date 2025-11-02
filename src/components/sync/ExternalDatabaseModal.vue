<template>
  <Modal
    id="external-database-modal"
    :title="databaseId ? 'Edit External Database' : 'Add External Database'"
    size="md"
  >
    <Form ref="databaseForm" @submit="handleSubmit">
      <!-- Basic Information -->
      <h4
        class="text-sm font-medium text-gray-100 border-b border-gray-700 pb-2 mb-2"
      >
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
      <h4
        class="text-sm font-medium text-gray-100 border-b border-gray-700 pb-2 mb-2 mt-4"
      >
        Connection Details
      </h4>

      <Input
        id="db-protocol"
        v-model="connectionDetails.protocol"
        label="Protocol (Optional)"
        :placeholder="defaultProtocol"
        :helper-text="`Leave empty to use default protocol (${defaultProtocol})`"
      />

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
          :min="0"
          :max="65535"
          rules="required|min_value:0|max_value:65535"
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
        :placeholder="
          databaseId ? 'Leave empty to keep current password' : 'Enter password'
        "
        :rules="databaseId ? '' : 'required'"
      />
      <div v-if="databaseId" class="text-xs text-gray-400 -mt-2">
        Leave empty to keep the current password
      </div>

      <Input
        id="db-database"
        v-model="connectionDetails.databaseName"
        label="Database Name"
        placeholder="kerminal_sync"
        rules="required"
      />

      <Input
        id="db-options"
        v-model="connectionDetails.options"
        label="Connection Options (Optional)"
        placeholder="retryWrites=true&w=majority&appName=Cluster0"
      />
      <div class="text-xs text-gray-400 -mt-2">
        Query parameters for the connection string (e.g., sslmode=require,
        retryWrites=true)
      </div>
    </Form>

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
          @click="handleSubmit"
        >
          {{ databaseId ? "Update Database" : "Add Database" }}
        </Button>
      </div>
    </template>
  </Modal>
</template>

<script setup lang="ts">
import { ref, computed, watch, onMounted } from "vue";
import Modal from "../ui/Modal.vue";
import Form from "../ui/Form.vue";
import Input from "../ui/Input.vue";
import Select from "../ui/Select.vue";
import Button from "../ui/Button.vue";
import { message } from "../../utils/message";
import { safeJsonParse, getCurrentTimestamp } from "../../utils/helpers";
import { useSyncStore } from "../../stores/sync";
import { useOverlay } from "../../composables/useOverlay";
import type {
  DatabaseType,
  ConnectionDetails,
  DatabaseSyncSettings,
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
  databaseName: "",
  protocol: "",
  options: "",
});

const syncSettings = ref<DatabaseSyncSettings>({
  autoSync: false,
  syncIntervalMinutes: 15,
  conflictResolutionStrategy: "LastWriteWins" as ConflictResolutionStrategy,
});

const databaseTypeOptions = [
  { value: "mysql", label: "MySQL" },
  { value: "postgresql", label: "PostgreSQL" },
  { value: "mongodb", label: "MongoDB" },
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

const defaultProtocol = computed(() => {
  switch (database.value.dbType) {
    case "mysql":
      return "mysql";
    case "postgresql":
      return "postgresql";
    case "mongodb":
      return "mongodb";
    default:
      return "";
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
    const data = await syncStore.getDatabaseWithDetails(databaseId.value);

    if (!data || !data.config) {
      throw new Error("Database not found or invalid data");
    }

    database.value = {
      name: data.config.name,
      dbType: data.config.dbType,
    };

    connectionDetails.value = {
      host: data.connectionDetails.host,
      port: data.connectionDetails.port,
      username: data.connectionDetails.username,
      password: "",
      databaseName: data.connectionDetails.databaseName,
      protocol: data.connectionDetails.protocol || "",
      options: data.connectionDetails.options || "",
    };

    syncSettings.value = {
      autoSync: data.config.autoSyncEnabled,
      syncIntervalMinutes: 15,
      conflictResolutionStrategy: "LastWriteWins",
    };

    if (data.config.syncSettings) {
      const parsed = safeJsonParse(data.config.syncSettings, {
        syncIntervalMinutes: 15,
        conflictResolutionStrategy: "LastWriteWins" as const,
      });
      syncSettings.value.syncIntervalMinutes = parsed.syncIntervalMinutes || 15;
      syncSettings.value.conflictResolutionStrategy =
        (parsed.conflictResolutionStrategy as ConflictResolutionStrategy) ||
        "LastWriteWins";
    }
  } finally {
    isLoading.value = false;
  }
};

const testConnection = async () => {
  const isValid = await databaseForm.value?.validate();
  if (!isValid) return;

  isTesting.value = true;
  try {
    const success = await syncStore.testConnection(
      database.value.dbType,
      connectionDetails.value,
      databaseId.value || undefined,
    );

    if (success) {
      message.success("Connection successful!");
    } else {
      message.error("Connection failed!");
    }
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
      connectionDetailsEncrypted: "",
      syncSettings: "",
      isActive: false,
      autoSyncEnabled: syncSettings.value.autoSync,
      createdAt: getCurrentTimestamp(),
      updatedAt: getCurrentTimestamp(),
      deviceId: "",
      version: 0,
      syncStatus: "idle",
    };

    if (databaseId.value) {
      interface UpdateDatabasePayload {
        name?: string;
        connectionDetails?: ConnectionDetails;
        autoSync?: boolean;
        syncIntervalMinutes?: number;
        conflictResolutionStrategy?: ConflictResolutionStrategy;
      }
      const updatePayload: UpdateDatabasePayload = {
        name: database.value.name,
        autoSync: syncSettings.value.autoSync,
        syncIntervalMinutes: syncSettings.value.syncIntervalMinutes,
        conflictResolutionStrategy:
          syncSettings.value.conflictResolutionStrategy,
      };

      if (connectionDetails.value.password) {
        updatePayload.connectionDetails = connectionDetails.value;
      }

      await syncStore.updateDatabase(databaseId.value, updatePayload);
      message.success("Database updated successfully");
    } else {
      await syncStore.addDatabase(
        dbData,
        connectionDetails.value,
        syncSettings.value,
      );
      message.success("Database added successfully");
    }

    closeModal();
  } finally {
    isLoading.value = false;
  }
};

const closeModal = () => {
  database.value = {
    name: "",
    dbType: "mysql",
  };
  connectionDetails.value = {
    host: "",
    port: 3306,
    username: "",
    password: "",
    databaseName: "",
    protocol: "",
    options: "",
  };
  syncSettings.value = {
    autoSync: false,
    syncIntervalMinutes: 15,
    conflictResolutionStrategy: "LastWriteWins",
  };
  closeOverlay("external-database-modal");
};

watch(
  () => databaseId.value,
  (newId) => {
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
        databaseName: "",
        protocol: "",
        options: "",
      };
      syncSettings.value = {
        autoSync: false,
        syncIntervalMinutes: 15,
        conflictResolutionStrategy: "LastWriteWins",
      };
    }
  },
  { immediate: true },
);

onMounted(() => {
  if (!syncStore.databases.length) {
    syncStore.loadDatabases();
  }
});
</script>
