<template>
  <SSHProfileDrawer />

  <SSHGroupModal />
  <SSHProfileModal />

  <SSHKeyManager />
  <SSHKeyModal />
</template>

<script setup lang="ts">
import SSHProfileDrawer from "./SSHProfileDrawer.vue";
import SSHGroupModal from "./SSHGroupModal.vue";
import SSHProfileModal from "./SSHProfileModal.vue";
import SSHKeyManager from "./SSHKeyManager.vue";
import SSHKeyModal from "./SSHKeyModal.vue";

import { useSSHStore } from "../../stores/ssh";

// Store
const sshStore = useSSHStore();

const loadAllData = async () => {
  try {
    await sshStore.loadAll();
  } catch (error) {
    console.error("Failed to load SSH profiles:", error);
  }
};

// Load data when component is mounted
loadAllData();
</script>
