import { defineStore } from "pinia";
import { ref, computed } from "vue";
import { Update } from "@tauri-apps/plugin-updater";
import { Store } from "@tauri-apps/plugin-store";
import {
  checkForUpdates,
  getPlatform,
  checkLinuxUpdate,
  listenToUpdateEvents,
  UpdateProgress,
  LinuxUpdateInfo,
  TauriUpdateInfo,
} from "../services/updater";

// Tauri store instance for updater settings
let store: Store | null = null;

const initStore = async () => {
  store ??= await Store.load("updater-settings.json");
  return store;
};

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
  const dontShowUpdateModal = ref(false);

  // Computed
  const isLinux = computed(() => currentPlatform.value === "linux");
  const hasUpdate = computed(() => {
    if (isLinux.value) {
      return linuxUpdateInfo.value?.available || false;
    }
    return availableUpdate.value !== null;
  });

  // Actions
  async function initialize() {
    currentPlatform.value = getPlatform();

    // Load saved settings from Tauri store
    try {
      const storeInstance = await initStore();
      const savedDontShow = await storeInstance.get<boolean>("dont-show-update-modal");
      if (savedDontShow !== null && savedDontShow !== undefined) {
        dontShowUpdateModal.value = savedDontShow;
      }
      const savedSkipped = await storeInstance.get<string[]>("skipped-versions");
      if (savedSkipped) {
        skippedVersions.value = savedSkipped;
      }
    } catch (error) {
      console.error("Failed to load updater settings:", error);
    }

    // Start listening for update events
    await startListening();
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

        if (update && skippedVersions.value.includes(update.version)) {
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
      saveSkippedVersions();
    }
  }

  function clearSkippedVersions() {
    skippedVersions.value = [];
    saveSkippedVersions();
  }

  async function saveSkippedVersions() {
    try {
      const storeInstance = await initStore();
      await storeInstance.set("skipped-versions", skippedVersions.value);
      await storeInstance.save();
    } catch (error) {
      console.error("Failed to save skipped versions:", error);
    }
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

  function setLinuxUpdateInfo(info: {
    available: boolean;
    version?: string;
    url?: string;
  }) {
    linuxUpdateInfo.value = info;
  }

  async function setDontShowUpdateModal(value: boolean) {
    dontShowUpdateModal.value = value;
    try {
      const storeInstance = await initStore();
      await storeInstance.set("dont-show-update-modal", value);
      await storeInstance.save();
    } catch (error) {
      console.error("Failed to save dont-show-update-modal:", error);
    }
  }

  let unlisten: (() => void) | null = null;

  async function startListening() {
    if (unlisten) return; // Already listening

    unlisten = await listenToUpdateEvents(
      (data: LinuxUpdateInfo | TauriUpdateInfo) => {
        if (isLinux.value) {
          // Linux: data is LinuxUpdateInfo with available, version, url
          setLinuxUpdateInfo(data as LinuxUpdateInfo);
        } else {
          // Windows/macOS: data is TauriUpdateInfo with version, date, body
          // We need to trigger a manual check to get the actual Update object
          // since the event only contains metadata
          checkUpdates(true);
        }
      },
    );
  }

  function stopListening() {
    if (unlisten) {
      unlisten();
      unlisten = null;
    }
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
    dontShowUpdateModal,

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
    setLinuxUpdateInfo,
    setDontShowUpdateModal,
    startListening,
    stopListening,
  };
});
