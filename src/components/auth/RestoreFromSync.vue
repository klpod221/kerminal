<template>
  <Modal
    id="restore-from-sync"
    :show-close-button="true"
    :close-on-backdrop="false"
    title="Restore from Sync Server"
    size="lg"
  >
    <div class="space-y-4">
      <div class="bg-blue-900/20 border border-blue-700/50 rounded-lg p-4">
        <div class="flex items-start gap-2">
          <Download :size="20" class="text-blue-400 mt-0.5 flex-shrink-0" />
          <div class="flex-1">
            <h4 class="text-sm font-medium text-blue-200 mb-1">Restore Your Data</h4>
            <p class="text-xs text-blue-100/80">
              Connect to your sync server to download existing profiles, SSH keys, and settings.
              You'll need your master password to decrypt the data after download.
            </p>
          </div>
        </div>
      </div>

      <Form ref="restoreForm" @submit="handleRestore">
        <h4 class="text-sm font-medium text-gray-100 border-b border-gray-700 pb-2 mb-2">
          Database Connection
        </h4>

        <Select
          id="database-type"
          v-model="formData.dbType"
          label="Database Type"
          :options="databaseTypeOptions"
          rules="required"
          :disabled="isLoading"
        />

        <div class="grid grid-cols-2 gap-4">
          <Input
            id="host"
            v-model="formData.host"
            label="Host"
            placeholder="localhost"
            rules="required"
            :disabled="isLoading"
          />

          <Input
            id="port"
            v-model.number="formData.port"
            label="Port"
            type="number"
            :placeholder="defaultPort"
            rules="required"
            :disabled="isLoading"
          />
        </div>

        <Input
          id="database"
          v-model="formData.database"
          label="Database Name"
          placeholder="kerminal"
          rules="required"
          :disabled="isLoading"
        />

        <Input
          id="username"
          v-model="formData.username"
          label="Username"
          placeholder="root"
          rules="required"
          :disabled="isLoading"
        />

        <Input
          id="password"
          v-model="formData.password"
          label="Password"
          type="password"
          placeholder="••••••••"
          :disabled="isLoading"
        />

        <h4 class="text-sm font-medium text-gray-100 border-b border-gray-700 pb-2 mb-2 mt-4">
          Device Information
        </h4>

        <Input
          id="device-name"
          v-model="formData.deviceName"
          label="Device Name"
          placeholder="My Laptop"
          rules="required"
          :disabled="isLoading"
        />
      </Form>

      <Collapsible title="How does restore work?" class="text-sm">
        <div class="space-y-2 text-xs text-gray-400">
          <p>1. We'll test the database connection to ensure it's valid</p>
          <p>2. Pull all encrypted data from the sync server</p>
          <p>3. You'll be prompted to enter your master password</p>
          <p>4. Data will be decrypted and restored to this device</p>
          <p>5. This device will be registered for future syncing</p>
        </div>
      </Collapsible>
    </div>

    <template #footer>
      <div class="flex justify-between items-center w-full">
        <Button
          variant="ghost"
          :disabled="isLoading"
          @click="handleBack"
        >
          Back
        </Button>

        <div class="flex gap-2">
          <Button
            variant="secondary"
            :loading="isTesting"
            :disabled="isLoading"
            :icon="Wifi"
            @click="handleTestConnection"
          >
            Test Connection
          </Button>

          <Button
            variant="primary"
            :loading="isLoading"
            :disabled="isTesting"
            :icon="Download"
            @click="handleRestore"
          >
            Restore Data
          </Button>
        </div>
      </div>
    </template>
  </Modal>
</template>

<script setup lang="ts">
import { ref, computed, watch } from "vue";
import { Download, Wifi } from "lucide-vue-next";
import Modal from "../ui/Modal.vue";
import Form from "../ui/Form.vue";
import Input from "../ui/Input.vue";
import Select from "../ui/Select.vue";
import Button from "../ui/Button.vue";
import Collapsible from "../ui/Collapsible.vue";
import { message } from "../../utils/message";
import { getErrorMessage, safeJsonStringify, getCurrentTimestamp } from "../../utils/helpers";
import { useOverlay } from "../../composables/useOverlay";
import { useSyncStore } from "../../stores/sync";
import type { DatabaseType } from "../../types/sync";

const { closeOverlay, openOverlay } = useOverlay();
const syncStore = useSyncStore();

const restoreForm = ref<InstanceType<typeof Form> | null>(null);
const isLoading = ref(false);
const isTesting = ref(false);

const formData = ref({
  dbType: "mysql" as DatabaseType,
  host: "",
  port: 3306,
  username: "",
  password: "",
  database: "",
  deviceName: "",
});

const databaseTypeOptions = [
  { value: "mysql", label: "MySQL" },
  { value: "postgresql", label: "PostgreSQL" },
  { value: "mongodb", label: "MongoDB" },
];

const defaultPort = computed(() => {
  switch (formData.value.dbType) {
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
  () => formData.value.dbType,
  () => {
    if (formData.value.dbType === "mongodb") {
      formData.value.port = 27017;
    } else if (formData.value.dbType === "mysql") {
      formData.value.port = 3306;
    } else if (formData.value.dbType === "postgresql") {
      formData.value.port = 5432;
    }
  },
);

const handleBack = () => {
  closeOverlay("restore-from-sync");
  openOverlay("initial-setup");
};

const handleTestConnection = async () => {
  const isValid = await restoreForm.value?.validate();
  if (!isValid) return;

  isTesting.value = true;

  try {
    const tempConfig = {
      name: "Temp Test",
      dbType: formData.value.dbType,
      connectionDetailsEncrypted: safeJsonStringify({
        host: formData.value.host,
        port: formData.value.port,
        username: formData.value.username,
        password: formData.value.password,
        database: formData.value.database,
      }),
      syncSettings: safeJsonStringify({
        autoSync: false,
        syncIntervalMinutes: 15,
        conflictResolutionStrategy: "LastWriteWins",
      }),
      isActive: false,
      autoSyncEnabled: false,
      lastSyncAt: undefined,
      createdAt: getCurrentTimestamp(),
      updatedAt: getCurrentTimestamp(),
      deviceId: "",
      version: 1,
      syncStatus: "Disconnected",
    };

    const tempDb = await syncStore.addDatabase(tempConfig);
    const success = await syncStore.testConnection(tempDb.id);

    await syncStore.deleteDatabase(tempDb.id);

    if (success) {
      message.success("Connection successful!");
    } else {
      message.error("Connection failed");
    }
  } catch (error) {
    console.error("Connection test failed:", error);
    message.error(getErrorMessage(error, "Connection test failed"));
  } finally {
    isTesting.value = false;
  }
};

const handleRestore = async () => {
  const isValid = await restoreForm.value?.validate();
  if (!isValid) return;

  isLoading.value = true;

  try {
    const databaseConfig = {
      name: "Sync Server",
      dbType: formData.value.dbType,
      connectionDetailsEncrypted: safeJsonStringify({
        host: formData.value.host,
        port: formData.value.port,
        username: formData.value.username,
        password: formData.value.password,
        database: formData.value.database,
      }),
      syncSettings: safeJsonStringify({
        autoSync: true,
        syncIntervalMinutes: 15,
        conflictResolutionStrategy: "LastWriteWins",
      }),
      isActive: false,
      autoSyncEnabled: true,
      lastSyncAt: undefined,
      createdAt: getCurrentTimestamp(),
      updatedAt: getCurrentTimestamp(),
      deviceId: "",
      version: 0,
      syncStatus: "idle",
    };

    const newDb = await syncStore.addDatabase(databaseConfig);

    message.info("Connecting to sync server...");
    await syncStore.connect(newDb.id);

    message.info("Pulling data from sync server...");
    await syncStore.sync(newDb.id, "pull");

    message.success("Data restored successfully! Please setup your master password.");

    closeOverlay("restore-from-sync");
    openOverlay("master-password-setup");
  } catch (error) {
    console.error("Restore failed:", error);
    message.error(getErrorMessage(error, "Failed to restore data"));
  } finally {
    isLoading.value = false;
  }
};
</script>
