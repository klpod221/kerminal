<template>
  <div class="h-screen w-screen flex flex-col bg-[#0D0D0D] overflow-hidden">
    <TopBar />

    <div class="flex-grow overflow-hidden">
      <MasterPasswordManager />

      <template v-if="authStore.isAuthenticated">
        <Dashboard v-if="viewState.activeView === 'dashboard'" />

        <Workspace v-if="viewState.activeView === 'workspace'" />

        <SSHProfileManager />

        <SavedCommandManager />

        <TunnelManager />

        <SyncManager />
      </template>
    </div>
  </div>
</template>

<script setup lang="ts">
import { onMounted, onUnmounted, watch } from "vue";

// Import components
import TopBar from "./components/TopBar.vue";
import Dashboard from "./components/Dashboard.vue";
import Workspace from "./components/Workspace.vue";
import SSHProfileManager from "./components/ssh-profiles/SSHProfileManager.vue";
import SavedCommandManager from "./components/saved-commands/SavedCommandManager.vue";
import TunnelManager from "./components/tunnels/TunnelManager.vue";
import SyncManager from "./components/sync/SyncManager.vue";
import MasterPasswordManager from "./components/auth/MasterPasswordManager.vue";

// Import stores and composables
import { useOverlay } from "./composables/useOverlay";

import { useViewStateStore } from "./stores/viewState";
import { useAuthStore } from "./stores/auth";

// Initialize stores
const viewState = useViewStateStore();
const authStore = useAuthStore();

const { openOverlay, closeAllOverlays } = useOverlay();

// Initialize auth store when app starts
onMounted(async () => {
  try {
    await authStore.initialize();

    // Try auto-unlock if setup is completed and auto-unlock is enabled
    if (
      !authStore.requiresSetup &&
      !authStore.status.isUnlocked
    ) {
      if (authStore.status.autoUnlockEnabled) {
        const success = await authStore.tryAutoUnlock();

        // Check status again after auto-unlock attempt
        await authStore.checkStatus();

        // If auto-unlock was successful, we should be authenticated now
        if (success && authStore.isAuthenticated) {
          return; // Exit early, don't open any overlays
        }
      } else {
      }
    }

    // Wait a bit to ensure status is fully updated
    await new Promise(resolve => setTimeout(resolve, 100));

    // Re-check status after potential auto-unlock
    if (authStore.isAuthenticated) {
      return;
    }

    // if setup is not completed, ensure the setup view is shown
    if (authStore.requiresSetup) {
      openOverlay("master-password-setup");
      return;
    }

    // If app is locked, show the unlock overlay
    if (authStore.requiresUnlock) {
      openOverlay("master-password-unlock");
      return;
    }
  } catch (error) {
    // Handle auth initialization error silently
  }
});

watch(
  () => [authStore.requiresSetup, authStore.requiresUnlock, authStore.isAuthenticated],
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
  { immediate: false }
);

// Cleanup when app unmounts
onUnmounted(() => {
  authStore.cleanup();
});

// Watch for changes to toggle top bar visibility based on authentication status
watch(
  () => authStore.isAuthenticated,
  (isAuthenticated) => {
    if (!isAuthenticated) {
      // Don't automatically open unlock overlay here - let the main watch handle it
      viewState.toggleTopBar(false);
    } else {
      // When authenticated, ensure all overlays are closed and top bar is shown
      closeAllOverlays();
      viewState.toggleTopBar(true);
    }
  },
);
</script>
