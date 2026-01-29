<template>
  <ThemeSelectorModal />
  <CustomThemeModal />
  <FontSettingsModal />
  <KeyboardShortcutsModal />
  <BackupRestoreModal />
  <UpdaterModal />
</template>

<script setup lang="ts">
import { onMounted, onBeforeUnmount } from "vue";
import ThemeSelectorModal from "./ThemeSelectorModal.vue";
import CustomThemeModal from "./CustomThemeModal.vue";
import FontSettingsModal from "./FontSettingsModal.vue";
import KeyboardShortcutsModal from "./KeyboardShortcutsModal.vue";
import BackupRestoreModal from "./BackupRestoreModal.vue";
import UpdaterModal from "./UpdaterModal.vue";
import { useSettingsStore } from "../../stores/settings";

const settingsStore = useSettingsStore();

/**
 * Initialize settings feature:
 * - Load settings from storage
 * - Start realtime listeners for live updates
 */
const initialize = async () => {
  try {
    await settingsStore.loadSettings();
    await settingsStore.startRealtime();
  } catch (error) {
    console.error("Failed to initialize settings:", error);
  }
};

/**
 * Cleanup when component is unmounted:
 * - Stop realtime listeners to prevent memory leaks
 */
const cleanup = () => {
  settingsStore.stopRealtime();
};

onMounted(() => {
  initialize();
});

onBeforeUnmount(() => {
  cleanup();
});
</script>
