<template>
  <div class="h-screen w-screen flex flex-col bg-bg-primary overflow-hidden">
    <TopBar />

    <div class="grow overflow-hidden">
      <MasterPasswordManager />

      <template v-if="authStore.isAuthenticated">
        <Dashboard v-if="viewState.activeView === 'dashboard'" />

        <Workspace v-show="viewState.activeView === 'workspace'" class="h-full" />

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
import { message } from "./utils/message";

import TopBar from "./components/TopBar.vue";

const Dashboard = defineAsyncComponent(
  () => import("./components/Dashboard.vue"),
);
import Workspace from "./components/Workspace.vue";
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

let unlisten: (() => void) | undefined;

onMounted(async () => {
  // Initialize updater store (detect platform)
  updaterStore.initialize();

  // Start listening for updates via store
  await updaterStore.startListening();

  // Watch for update availability to trigger modal
  watch(
    () => updaterStore.hasUpdate,
    (hasUpdate) => {
      if (hasUpdate) {
        message.success("Update available!");
        openOverlay("updater-modal");
      }
    },
    { immediate: true }
  );

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
});

onUnmounted(() => {
  if (unlisten) unlisten();
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
