<template>
  <div class="h-screen w-screen flex flex-col bg-[#0D0D0D] overflow-hidden">
    <TopBar />

    <div class="flex-grow overflow-hidden">
      <MasterPasswordManager />

      <template v-if="authStore.isAuthenticated">
        <Dashboard v-if="viewState.activeView === 'dashboard'" />

        <Workspace v-if="viewState.activeView === 'workspace'" />

        <SSHProfileManager />
      </template>
    </div>
  </div>
</template>

<script setup lang="ts">
import { onMounted, onUnmounted, watch } from "vue";

// Import components
import TopBar from "./components/TopBar.vue";
import MasterPasswordManager from "./components/MasterPasswordManager.vue";
import Dashboard from "./components/Dashboard.vue";
import Workspace from "./components/Workspace.vue";
import SSHProfileManager from "./components/SSHProfileManager.vue";

// Import stores and composables
import { useOverlay } from "./composables/useOverlay";

import { useViewStateStore } from "./stores/viewState";
import { useAuthStore } from "./stores/auth";

// Initialize stores
const viewState = useViewStateStore();
const authStore = useAuthStore();

const { openOverlay } = useOverlay();

// Initialize auth store when app starts
onMounted(async () => {
  try {
    await authStore.initialize();

    // current status
    console.log("Auth status:", authStore.isAuthenticated, authStore.status);

    // Try auto-unlock if setup is completed and auto-unlock is enabled
    if (
      !authStore.requiresSetup &&
      !authStore.status.isUnlocked &&
      authStore.status.autoUnlockEnabled
    ) {
      console.log("Attempting auto-unlock...");
      await authStore.tryAutoUnlock();
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
    console.error("Failed to initialize auth:", error);
  }
});

// Cleanup when app unmounts
onUnmounted(() => {
  authStore.cleanup();
});

// Watch for changes to toggle top bar visibility based on authentication status
watch(
  () => authStore.isAuthenticated,
  (isAuthenticated) => {
    if (!isAuthenticated) {
      openOverlay("master-password-unlock");
      viewState.toggleTopBar(false);
    } else {
      viewState.toggleTopBar(true);
    }
  }
);
</script>
