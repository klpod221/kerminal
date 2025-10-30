<template>
  <div class="h-screen w-screen flex flex-col bg-[#0D0D0D] overflow-hidden">
    <TopBar />

    <div class="grow overflow-hidden">
      <MasterPasswordManager />

      <template v-if="authStore.isAuthenticated">
        <Dashboard v-if="viewState.activeView === 'dashboard'" />

        <Workspace v-if="viewState.activeView === 'workspace'" />

        <div v-if="viewState.activeView === 'sftp'" class="h-full w-full flex items-center justify-center text-white">
          SFTP feature coming soon!
        </div>

        <SSHProfileManager />

        <SavedCommandManager />

        <TunnelManager />

        <SyncManager />

        <ThemeSelectorModal />
        <CustomThemeModal />
        <FontSettingsModal />
      </template>
    </div>
  </div>
</template>

<script setup lang="ts">
import { onMounted, onUnmounted, watch } from "vue";

import TopBar from "./components/TopBar.vue";
import Dashboard from "./components/Dashboard.vue";
import Workspace from "./components/Workspace.vue";
import SSHProfileManager from "./components/ssh-profiles/SSHProfileManager.vue";
import SavedCommandManager from "./components/saved-commands/SavedCommandManager.vue";
import TunnelManager from "./components/tunnels/TunnelManager.vue";
import SyncManager from "./components/sync/SyncManager.vue";
import MasterPasswordManager from "./components/auth/MasterPasswordManager.vue";
import ThemeSelectorModal from "./components/settings/ThemeSelectorModal.vue";
import CustomThemeModal from "./components/settings/CustomThemeModal.vue";
import FontSettingsModal from "./components/settings/FontSettingsModal.vue";

import { useOverlay } from "./composables/useOverlay";

import { useViewStateStore } from "./stores/viewState";
import { useAuthStore } from "./stores/auth";

const viewState = useViewStateStore();
const authStore = useAuthStore();

const { openOverlay, closeAllOverlays } = useOverlay();

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
      } else {
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
  } catch (error) {}
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
    if (!isAuthenticated) {
      viewState.toggleTopBar(false);
    } else {
      closeAllOverlays();
      viewState.toggleTopBar(true);
    }
  },
);
</script>
