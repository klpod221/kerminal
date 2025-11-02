<template>
  <SSHProfileDrawer />

  <SSHGroupModal />
  <SSHProfileModal />
  <SSHConfigPasswordModal />

  <SSHKeyManager />
  <SSHKeyModal />
</template>

<script setup lang="ts">
import { onMounted, onBeforeUnmount } from "vue";
import SSHProfileDrawer from "./SSHProfileDrawer.vue";
import SSHGroupModal from "./SSHGroupModal.vue";
import SSHProfileModal from "./SSHProfileModal.vue";
import SSHConfigPasswordModal from "./SSHConfigPasswordModal.vue";
import SSHKeyManager from "./SSHKeyManager.vue";
import SSHKeyModal from "./SSHKeyModal.vue";
import { useSSHStore } from "../../stores/ssh";
import { useSshKeyStore } from "../../stores/sshKey";

const sshStore = useSSHStore();
const sshKeyStore = useSshKeyStore();

/**
 * Initialize SSH profiles feature:
 * - Load all profiles, groups, and SSH keys
 * - Start realtime listeners for live updates
 */
const initialize = async () => {
  try {
    await Promise.all([sshStore.loadAll(), sshKeyStore.loadKeys()]);
    await Promise.all([sshStore.startRealtime(), sshKeyStore.startRealtime()]);
  } catch (error) {
    console.error("Failed to initialize SSH data:", error);
  }
};

/**
 * Cleanup when component is unmounted:
 * - Stop realtime listeners to prevent memory leaks
 */
const cleanup = () => {
  sshStore.stopRealtime();
  sshKeyStore.stopRealtime();
};

onMounted(() => {
  initialize();
});

onBeforeUnmount(() => {
  cleanup();
});
</script>
