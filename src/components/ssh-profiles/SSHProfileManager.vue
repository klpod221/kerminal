<template>
  <SSHProfileDrawer />

  <SSHGroupModal />
  <SSHProfileModal />
  <SSHConfigPasswordModal />

  <SSHKeyManager />
  <SSHKeyModal />
</template>

<script setup lang="ts">
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

const loadAllData = async () => {
  try {
    await Promise.all([sshStore.loadAll(), sshKeyStore.loadKeys()]);
    await Promise.all([sshStore.startRealtime(), sshKeyStore.startRealtime()]);
  } catch (error) {
    console.error("Failed to load SSH data:", error);
  }
};

loadAllData();
</script>
