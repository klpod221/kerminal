<template>
  <Modal
    id="updater-modal"
    :show-close-button="!isDownloading"
    title="Software Update"
    :icon="Download"
    icon-background="bg-green-500/20"
    icon-color="text-green-400"
    size="md"
  >
    <div class="space-y-4">
      <!-- Update Available (Non-Linux) -->
      <div v-if="!isLinux && availableUpdate" class="space-y-4">
        <div class="flex items-center justify-between text-sm">
          <span class="text-gray-400">Current Version:</span>
          <span class="text-white font-mono">{{ currentVersion }}</span>
        </div>
        <div class="flex items-center justify-between text-sm">
          <span class="text-gray-400">New Version:</span>
          <span class="text-green-400 font-mono">{{
            availableUpdate.version
          }}</span>
        </div>

        <!-- Release Notes -->
        <div v-if="availableUpdate.body" class="mt-4">
          <div class="text-sm text-gray-400 mb-2">What's New:</div>
          <div
            class="bg-dark-700/50 rounded-lg p-3 text-sm text-gray-300 max-h-48 overflow-y-auto"
          >
            <pre class="whitespace-pre-wrap">{{ availableUpdate.body }}</pre>
          </div>
        </div>

        <!-- Download Progress -->
        <div v-if="isDownloading" class="mt-4">
          <div class="flex items-center justify-between text-sm mb-2">
            <span class="text-gray-400">Downloading...</span>
            <span class="text-white"
              >{{ downloadProgress.percentage.toFixed(0) }}%</span
            >
          </div>
          <div class="w-full bg-dark-700 rounded-full h-2">
            <div
              class="bg-green-500 h-2 rounded-full transition-all duration-300"
              :style="{ width: `${downloadProgress.percentage}%` }"
            ></div>
          </div>
          <div class="flex items-center justify-between text-xs mt-1">
            <span class="text-gray-500">{{
              formatBytes(downloadProgress.downloaded)
            }}</span>
            <span class="text-gray-500">{{
              formatBytes(downloadProgress.total)
            }}</span>
          </div>
        </div>

        <!-- Actions -->
        <div class="flex gap-3 mt-4">
          <Button
            v-if="!isDownloading"
            variant="secondary"
            @click="handleSkip"
            class="flex-1"
          >
            Skip This Version
          </Button>
          <Button
            variant="primary"
            :icon="Download"
            :loading="isDownloading"
            :disabled="isDownloading"
            @click="handleUpdate"
            class="flex-1"
          >
            {{ isDownloading ? "Downloading..." : "Update Now" }}
          </Button>
        </div>
      </div>

      <!-- Linux Update Available -->
      <div v-if="isLinux && linuxUpdateInfo?.available" class="space-y-4">
        <div class="flex items-center justify-between text-sm">
          <span class="text-gray-400">Current Version:</span>
          <span class="text-white font-mono">{{ currentVersion }}</span>
        </div>
        <div class="flex items-center justify-between text-sm">
          <span class="text-gray-400">New Version:</span>
          <span class="text-green-400 font-mono">{{
            linuxUpdateInfo.version
          }}</span>
        </div>

        <p class="text-sm text-gray-400">
          A new version is available. Please update using your package manager
          or download from GitHub.
        </p>

        <div class="flex gap-3">
          <Button variant="secondary" @click="handleClose" class="flex-1">
            Later
          </Button>
          <Button
            variant="primary"
            :icon="ExternalLink"
            @click="handleOpenGitHub"
            class="flex-1"
          >
            Download from GitHub
          </Button>
        </div>
      </div>

      <!-- No Updates -->
      <div v-if="!hasUpdate && !isChecking" class="space-y-4">
        <p class="text-sm text-gray-400">
          You are running the latest version of Kerminal.
        </p>
        <div class="flex items-center justify-between text-sm mt-4">
          <span class="text-gray-400">Current Version:</span>
          <span class="text-white font-mono">{{ currentVersion }}</span>
        </div>
      </div>

      <!-- Checking -->
      <div v-if="isChecking" class="flex items-center justify-center py-4">
        <RefreshCw class="w-6 h-6 text-blue-400 animate-spin" />
        <span class="ml-3 text-sm text-gray-400">Checking for updates...</span>
      </div>
    </div>
  </Modal>
</template>

<script setup lang="ts">
import { computed } from "vue";
import {
  Download,
  RefreshCw,
  ExternalLink,
} from "lucide-vue-next";
import { openUrl } from "@tauri-apps/plugin-opener";
import Modal from "../ui/Modal.vue";
import Button from "../ui/Button.vue";
import { useOverlay } from "../../composables/useOverlay";
import { useUpdaterStore } from "../../stores/updater";
import { restartApp } from "../../services/updater";
import { message } from "../../utils/message";
import { version } from "../../../package.json";

const { closeOverlay } = useOverlay();
const updaterStore = useUpdaterStore();

const currentVersion = `v${version}`;

const isChecking = computed(() => updaterStore.isChecking);
const isDownloading = computed(() => updaterStore.isDownloading);
const availableUpdate = computed(() => updaterStore.availableUpdate);
const downloadProgress = computed(() => updaterStore.downloadProgress);
const hasUpdate = computed(() => updaterStore.hasUpdate);
const isLinux = computed(() => updaterStore.isLinux);
const linuxUpdateInfo = computed(() => updaterStore.linuxUpdateInfo);

const formatBytes = (bytes: number): string => {
  if (bytes === 0) return "0 B";
  const k = 1024;
  const sizes = ["B", "KB", "MB", "GB"];
  const i = Math.floor(Math.log(bytes) / Math.log(k));
  return `${Number.parseFloat((bytes / Math.pow(k, i)).toFixed(2))} ${sizes[i]}`;
};

const handleUpdate = async () => {
  if (!availableUpdate.value) return;

  try {
    updaterStore.setDownloading(true);

    let downloaded = 0;
    let total = 0;

    // Use the Update object's native downloadAndInstall method
    await availableUpdate.value.downloadAndInstall((event) => {
      switch (event.event) {
        case "Started":
          total = event.data.contentLength || 0;
          updaterStore.updateDownloadProgress({
            downloaded: 0,
            total,
            percentage: 0,
          });
          break;
        case "Progress":
          downloaded += event.data.chunkLength;
          updaterStore.updateDownloadProgress({
            downloaded,
            total,
            percentage: total > 0 ? (downloaded / total) * 100 : 0,
          });
          break;
        case "Finished":
          updaterStore.updateDownloadProgress({
            downloaded: total,
            total,
            percentage: 100,
          });
          break;
      }
    });

    message.success("Update downloaded! Restarting...");

    // Wait a moment before restarting
    setTimeout(async () => {
      await restartApp();
    }, 1000);
  } catch (error) {
    console.error("Update failed:", error);
    message.error("Failed to download update: " + error);
    updaterStore.setDownloading(false);
  }
};

const handleSkip = () => {
  if (availableUpdate.value) {
    updaterStore.skipVersion(availableUpdate.value.version);
    message.info(`Version ${availableUpdate.value.version} will be skipped`);
  }
  handleClose();
};

const handleOpenGitHub = async () => {
  if (linuxUpdateInfo.value?.url) {
    await openUrl(linuxUpdateInfo.value.url);
  }
  handleClose();
};

const handleClose = () => {
  if (!isDownloading.value) {
    closeOverlay("updater-modal");
  }
};
</script>
