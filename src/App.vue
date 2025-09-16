<template>
  <div class="h-screen w-screen flex flex-col bg-[#0D0D0D] overflow-hidden">
    <TopBar />

    <div class="flex-grow overflow-hidden">
      <MasterPasswordManager />

      <Dashboard v-if="viewState.activeView === 'dashboard'" />

      <Workspace v-show="viewState.activeView === 'workspace'" />

      <SSHProfileManager />
    </div>
  </div>
</template>

<script setup lang="ts">
import { onMounted, onUnmounted } from "vue";

// Import components
import TopBar from "./components/TopBar.vue";
import MasterPasswordManager from "./components/MasterPasswordManager.vue";
import Dashboard from "./components/Dashboard.vue";
import Workspace from "./components/Workspace.vue";
import SSHProfileManager from "./components/SSHProfileManager.vue";

import { useOverlay } from './composables/useOverlay';

// Import stores
import { useViewStateStore } from "./stores/viewState";
import { useAuthStore } from "./stores/auth";

// Initialize stores
const viewState = useViewStateStore();
const authStore = useAuthStore();

// Note: useOverlay is imported but not used here, available for future use
useOverlay();

// Initialize auth store when app starts
onMounted(async () => {
  try {
    await authStore.initialize();

    // Try auto-unlock if setup is completed
    if (authStore.status.isSetup && !authStore.status.isUnlocked && authStore.status.autoUnlockEnabled) {
      await authStore.tryAutoUnlock();
    }

    // current status
    console.log("Auth status:", authStore.status);
  } catch (error) {
    console.error("Failed to initialize auth:", error);
  }
});

// Cleanup when app unmounts
onUnmounted(() => {
  authStore.cleanup();
});
</script>
