<template>
  <Card title="Sync Settings">
    <div
      v-if="isLoading"
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
            <h4 class="text-sm font-medium text-gray-100">Master Controls</h4>
            <p class="text-xs text-gray-400 mt-1">
              Enable or disable the entire sync system
            </p>
          </div>
          <Badge :variant="localSettings.isActive ? 'success' : 'default'">
            {{ localSettings.isActive ? "Active" : "Inactive" }}
          </Badge>
        </div>

        <Checkbox
          id="global-is-active"
          v-model="localSettings.isActive"
          label="Enable Sync System"
          helper-text="Master switch for all sync operations"
          @update:modelValue="markDirty"
        />

        <Checkbox
          id="global-auto-sync"
          v-model="localSettings.autoSyncEnabled"
          label="Auto Sync"
          helper-text="Automatically sync at intervals"
          :disabled="!localSettings.isActive"
          @update:modelValue="markDirty"
        />

        <div v-if="localSettings.autoSyncEnabled">
          <Input
            id="global-interval"
            v-model.number="localSettings.syncIntervalMinutes"
            label="Sync Interval (minutes)"
            type="number"
            :min="1"
            :max="1440"
            placeholder="15"
            helper-text="How often to automatically sync (1-1440 minutes)"
            @blur="markDirty"
          />
        </div>
      </div>

      <!-- Sync Configuration -->
      <div class="border border-gray-700 rounded-lg p-4">
        <Select
          id="global-conflict-strategy"
          v-model="localSettings.conflictStrategy"
          label="Conflict Resolution Strategy"
          :helper-text="getStrategyDescription()"
          :options="conflictStrategyOptions"
          @update:modelValue="markDirty"
        />

        <Select
          id="global-sync-direction"
          v-model="localSettings.syncDirection"
          label="Sync Direction"
          :helper-text="getDirectionDescription()"
          :options="syncDirectionOptions"
          @update:modelValue="markDirty"
        />
      </div>

      <!-- Actions -->
      <div class="flex gap-2 justify-end">
        <Button variant="outline" :disabled="!isDirty" @click="reset">
          Reset
        </Button>
        <Button
          variant="primary"
          :disabled="!isDirty || isSaving"
          :loading="isSaving"
          @click="save"
        >
          Save Settings
        </Button>
      </div>
    </div>
  </Card>
</template>

<script setup lang="ts">
import { ref, onMounted } from "vue";
import { RefreshCw } from "lucide-vue-next";
import Card from "../ui/Card.vue";
import Badge from "../ui/Badge.vue";
import Checkbox from "../ui/Checkbox.vue";
import Input from "../ui/Input.vue";
import Select from "../ui/Select.vue";
import Button from "../ui/Button.vue";
import { syncService } from "../../services/sync";
import { message } from "../../utils/message";
import { getErrorMessage } from "../../utils/helpers";

const isLoading = ref(false);
const isSaving = ref(false);
const isDirty = ref(false);
const settings = ref<any | null>(null);
const localSettings = ref<any>({
  isActive: false,
  autoSyncEnabled: false,
  syncIntervalMinutes: 15,
  conflictStrategy: "manual",
  syncDirection: "both",
});

const conflictStrategyOptions = [
  { value: "lastWriteWins", label: "Last Write Wins" },
  { value: "firstWriteWins", label: "First Write Wins" },
  { value: "localWins", label: "Local Wins" },
  { value: "remoteWins", label: "Remote Wins" },
  { value: "manual", label: "Manual Resolution" },
];

const syncDirectionOptions = [
  { value: "both", label: "Bidirectional" },
  { value: "push", label: "Push Only" },
  { value: "pull", label: "Pull Only" },
];

const loadSettings = async () => {
  isLoading.value = true;
  try {
    const data = await syncService.getGlobalSyncSettings();
    settings.value = data || null;
    if (data) {
      localSettings.value = {
        isActive: data.isActive ?? false,
        autoSyncEnabled: data.autoSyncEnabled ?? false,
        syncIntervalMinutes: data.syncIntervalMinutes ?? 15,
        conflictStrategy: data.conflictStrategy ?? "manual",
        syncDirection: data.syncDirection ?? "both",
      };
    }
    isDirty.value = false;
  } catch (error) {
    console.error("Failed to load global settings:", error);
    message.error(getErrorMessage(error, "Failed to load global settings"));
  } finally {
    isLoading.value = false;
  }
};

const markDirty = () => {
  isDirty.value = true;
};

const reset = () => {
  if (settings.value) {
    localSettings.value = { ...settings.value };
    isDirty.value = false;
    message.info("Settings reset to last saved state");
  }
};

const save = async () => {
  if (!isDirty.value) return;
  isSaving.value = true;
  try {
    const updates = {
      isActive: localSettings.value.isActive,
      autoSyncEnabled: localSettings.value.autoSyncEnabled,
      syncIntervalMinutes: localSettings.value.syncIntervalMinutes,
      conflictStrategy: localSettings.value.conflictStrategy,
      syncDirection: localSettings.value.syncDirection,
    };
    await syncService.updateGlobalSyncSettings(updates);
    await loadSettings();
    message.success("Global sync settings saved successfully");
  } catch (error) {
    console.error("Failed to save global settings:", error);
    message.error(getErrorMessage(error, "Failed to save global settings"));
  } finally {
    isSaving.value = false;
  }
};

const getStrategyDescription = (): string => {
  const strategy = localSettings.value.conflictStrategy;
  const descriptions: Record<string, string> = {
    lastWriteWins: "The most recently modified data will be kept",
    firstWriteWins: "The oldest data will be preserved",
    localWins: "Local data always wins in conflicts",
    remoteWins: "Remote data always wins in conflicts",
    manual: "You will be prompted to manually resolve each conflict",
  };
  return descriptions[strategy] || "";
};

const getDirectionDescription = (): string => {
  const direction = localSettings.value.syncDirection;
  const descriptions: Record<string, string> = {
    both: "Sync data in both directions (local ↔ remote)",
    push: "Only push local data to remote (local → remote)",
    pull: "Only pull remote data to local (local ← remote)",
  };
  return descriptions[direction] || "";
};

onMounted(() => {
  loadSettings();
});

defineExpose({ loadSettings });
</script>
