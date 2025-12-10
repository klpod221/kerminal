<template>
  <div class="h-screen w-screen flex flex-col bg-bg-primary overflow-hidden">
    <TopBar />

    <div class="grow overflow-hidden">
      <MasterPasswordManager />

      <template v-if="authStore.isAuthenticated">
        <Dashboard v-if="viewState.activeView === 'dashboard'" />

        <Workspace v-if="viewState.activeView === 'workspace'" />

        <SFTPBrowser v-if="viewState.activeView === 'sftp'" />

        <SSHProfileManager />

        <SavedCommandManager />

        <RecordingsManager />

        <TunnelManager />

        <SyncManager />

        <SettingsManager />

        <TerminalProfileManager />

        <CommandPaletteManager />
      </template>
    </div>
  </div>
</template>

<script setup lang="ts">
import { onMounted, onUnmounted, watch, defineAsyncComponent } from "vue";

import TopBar from "./components/TopBar.vue";
const Dashboard = defineAsyncComponent(
  () => import("./components/Dashboard.vue"),
);
const Workspace = defineAsyncComponent(
  () => import("./components/Workspace.vue"),
);
const SFTPBrowser = defineAsyncComponent(
  () => import("./components/sftp/SFTPBrowser.vue"),
);
const SSHProfileManager = defineAsyncComponent(
  () => import("./components/ssh-profiles/SSHProfileManager.vue"),
);
const SavedCommandManager = defineAsyncComponent(
  () => import("./components/saved-commands/SavedCommandManager.vue"),
);
const RecordingsManager = defineAsyncComponent(
  () => import("./components/recording/RecordingsManager.vue"),
);
const TunnelManager = defineAsyncComponent(
  () => import("./components/tunnels/TunnelManager.vue"),
);
const SyncManager = defineAsyncComponent(
  () => import("./components/sync/SyncManager.vue"),
);
const MasterPasswordManager = defineAsyncComponent(
  () => import("./components/auth/MasterPasswordManager.vue"),
);
const SettingsManager = defineAsyncComponent(
  () => import("./components/settings/SettingsManager.vue"),
);
const TerminalProfileManager = defineAsyncComponent(
  () => import("./components/terminal-profiles/TerminalProfileManager.vue"),
);
const CommandPaletteManager = defineAsyncComponent(
  () => import("./components/CommandPaletteManager.vue"),
);

import { useOverlay } from "./composables/useOverlay";
import { useGlobalShortcuts } from "./composables/useGlobalShortcuts";

import { useViewStateStore } from "./stores/viewState";
import { useAuthStore } from "./stores/auth";
import { useUpdaterStore } from "./stores/updater";

const viewState = useViewStateStore();
const authStore = useAuthStore();
const updaterStore = useUpdaterStore();

const { openOverlay, closeAllOverlays } = useOverlay();

// Initialize global keyboard shortcuts once at app level
useGlobalShortcuts();

onMounted(async () => {
  try {
    await authStore.initialize();

    if (!authStore.requiresSetup && !authStore.status.isUnlocked) {
      if (authStore.status.autoUnlockEnabled) {
        const success = await authStore.tryAutoUnlock();

        await authStore.checkStatus();

        if (success && authStore.isAuthenticated) {
          return; // Exit early, don't open any overlays
        }
      }
    }

    await new Promise((resolve) => setTimeout(resolve, 100));

    if (authStore.isAuthenticated) {
      return;
    }

    if (authStore.requiresSetup) {
      openOverlay("master-password-setup");
      return;
    }

    if (authStore.requiresUnlock) {
      openOverlay("master-password-unlock");
      return;
    }
  } catch (error) {
    // Ignore error during initial auto-unlock attempt
    console.debug("Auto-unlock failed silently:", error);
  }

  // Initialize updater and check for updates after a short delay
  setTimeout(async () => {
    if (authStore.isAuthenticated) {
      updaterStore.initialize();
      await updaterStore.checkUpdates(true); // Silent check

      // If update is available, show modal
      if (updaterStore.hasUpdate) {
        openOverlay("updater-modal");
      }
    }
  }, 30000); // Check for updates 30 seconds after start
});

watch(
  () => [
    authStore.requiresSetup,
    authStore.requiresUnlock,
    authStore.isAuthenticated,
  ],
  ([requiresSetup, requiresUnlock, isAuthenticated]) => {
    if (isAuthenticated) {
      closeAllOverlays();
      return;
    }

    if (requiresSetup) {
      openOverlay("master-password-setup");
    } else if (requiresUnlock) {
      openOverlay("master-password-unlock");
    }
  },
  { immediate: false },
);

onUnmounted(() => {
  authStore.cleanup();
});

watch(
  () => authStore.isAuthenticated,
  (isAuthenticated) => {
    if (isAuthenticated) {
      closeAllOverlays();
      viewState.toggleTopBar(true);
    } else {
      viewState.toggleTopBar(false);
    }
  },
);
</script>
