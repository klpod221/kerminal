import { defineStore } from "pinia";
import { ref, computed } from "vue";
import { Update } from "@tauri-apps/plugin-updater";
import {
  checkForUpdates,
  getPlatform,
  checkLinuxUpdate,
  UpdateProgress,
} from "../services/updater";

export const useUpdaterStore = defineStore("updater", () => {
  // State
  const isChecking = ref(false);
  const isDownloading = ref(false);
  const availableUpdate = ref<Update | null>(null);
  const downloadProgress = ref<UpdateProgress>({
    downloaded: 0,
    total: 0,
    percentage: 0,
  });
  const lastCheckTime = ref<number | null>(null);
  const currentPlatform = ref<string>("");
  const linuxUpdateInfo = ref<{
    available: boolean;
    version?: string;
    url?: string;
  } | null>(null);

  // Settings
  const autoCheckEnabled = ref(true);
  const skippedVersions = ref<string[]>([]);

  // Computed
  const hasUpdate = computed(() => availableUpdate.value !== null);
  const isLinux = computed(() => currentPlatform.value === "linux");

  // Actions
  function initialize() {
    currentPlatform.value = getPlatform();
  }

  async function checkUpdates(silent = false): Promise<boolean> {
    if (isChecking.value) return false;

    try {
      isChecking.value = true;

      if (isLinux.value) {
        // Check Linux updates via GitHub API
        linuxUpdateInfo.value = await checkLinuxUpdate();
        lastCheckTime.value = Date.now();
        return linuxUpdateInfo.value?.available || false;
      } else {
        // Check updates via Tauri updater for macOS/Windows
        const update = await checkForUpdates();
        availableUpdate.value = update;
        lastCheckTime.value = Date.now();

        // Check if this version was skipped
        if (update && skippedVersions.value.includes(update.version)) {
          if (!silent) {
            console.log(`Update ${update.version} was skipped by user`);
          }
          return false;
        }

        return update !== null;
      }
    } catch (error) {
      if (!silent) {
        console.error("Failed to check for updates:", error);
      }
      return false;
    } finally {
      isChecking.value = false;
    }
  }

  function skipVersion(version: string) {
    if (!skippedVersions.value.includes(version)) {
      skippedVersions.value.push(version);
    }
  }

  function clearSkippedVersions() {
    skippedVersions.value = [];
  }

  function setAutoCheck(enabled: boolean) {
    autoCheckEnabled.value = enabled;
  }

  function updateDownloadProgress(progress: UpdateProgress) {
    downloadProgress.value = progress;
  }

  function setDownloading(downloading: boolean) {
    isDownloading.value = downloading;
  }

  function clearUpdate() {
    availableUpdate.value = null;
    linuxUpdateInfo.value = null;
    downloadProgress.value = {
      downloaded: 0,
      total: 0,
      percentage: 0,
    };
  }

  return {
    // State
    isChecking,
    isDownloading,
    availableUpdate,
    downloadProgress,
    lastCheckTime,
    currentPlatform,
    linuxUpdateInfo,
    autoCheckEnabled,
    skippedVersions,

    // Computed
    hasUpdate,
    isLinux,

    // Actions
    initialize,
    checkUpdates,
    skipVersion,
    clearSkippedVersions,
    setAutoCheck,
    updateDownloadProgress,
    setDownloading,
    clearUpdate,
  };
});
