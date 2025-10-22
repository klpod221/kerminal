<template>
  <TunnelList />
  <TunnelModal />
</template>

<script setup lang="ts">
import TunnelList from "./TunnelList.vue";
import TunnelModal from "./TunnelModal.vue";

import { useTunnelStore } from "../../stores/tunnel";

const tunnelStore = useTunnelStore();

const loadAllData = async () => {
  try {
    await tunnelStore.loadTunnels();
  } catch (error) {
    console.error("Failed to load SSH tunnels:", error);
  }
};

loadAllData();
</script>
