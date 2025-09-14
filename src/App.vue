<template>
  <div class="h-screen w-screen flex flex-col bg-[#0D0D0D] overflow-hidden">
    <TopBar />

    <div class="flex-grow overflow-hidden">
      <Dashboard v-if="viewState.activeView === 'dashboard'" />

      <Workspace v-show="viewState.activeView === 'workspace'" />
    </div>
  </div>
</template>

<script setup lang="ts">
// Import components
import TopBar from "./components/TopBar.vue";
import Dashboard from "./components/Dashboard.vue";
import Workspace from "./components/Workspace.vue";

// Import store
import { useViewStateStore } from "./stores/viewState";

// Initialize store
const viewState = useViewStateStore();
</script>
