<template>
  <div class="h-screen w-screen flex flex-col bg-bg-primary overflow-hidden">
    <!-- Global Terminal Host for DOM teleportation -->
    <TerminalHost />
    <div
      v-if="useLegacyRenderer"
      class="fixed bottom-4 right-4 z-9999 font-mono text-green-500 text-opacity-80 text-sm select-none pointer-events-none"
      style="text-shadow: 0 0 5px #0f0"
    >
      {{ getStatusLabel() }}
    </div>
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

        <!-- Tour Overlay -->
        <TourOverlay />
      </template>
    </div>
  </div>
</template>

<script setup lang="ts">
import { onMounted, onUnmounted, watch, defineAsyncComponent } from "vue";
import { message } from "./utils/message";

import TopBar from "./components/TopBar.vue";
import TerminalHost from "./components/ui/TerminalHost.vue";

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
const TourOverlay = defineAsyncComponent(
  () => import("./components/tour/TourOverlay.vue"),
);

import { useOverlay } from "./composables/useOverlay";
import { useGlobalShortcuts } from "./composables/useGlobalShortcuts";
import { useSystemMetrics } from "./composables/useSystemMetrics";

import { useViewStateStore } from "./stores/viewState";
import { useAuthStore } from "./stores/auth";
import { useUpdaterStore } from "./stores/updater";
import { useTourStore } from "./stores/tour";

const viewState = useViewStateStore();
const authStore = useAuthStore();
const updaterStore = useUpdaterStore();
const tourStore = useTourStore();

const { openOverlay, closeAllOverlays } = useOverlay();
const { useLegacyRenderer, getStatusLabel } = useSystemMetrics();

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
    { immediate: true },
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
    void error; // NOSONAR
    // Ignore error during initial auto-unlock attempt
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
  async (isAuthenticated) => {
    if (isAuthenticated) {
      closeAllOverlays();
      viewState.toggleTopBar(true);

      // Check if this is the first time user - show tour
      await tourStore.loadState();
      if (!tourStore.hasCompletedFirstTour) {
        // Small delay to ensure UI is ready
        setTimeout(() => {
          tourStore.startTour();
        }, 500);
      }
    } else {
      viewState.toggleTopBar(false);
    }
  },
);
</script>
