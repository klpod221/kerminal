<template>
  <SSHProfileDrawer />
</template>

<script setup lang="ts">
import SSHProfileDrawer from "./SSHProfileDrawer.vue";
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
