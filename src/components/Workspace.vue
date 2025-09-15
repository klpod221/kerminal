<template>
  <PanelManager
    :layout="workspaceStore.panelLayout"
    :terminals="workspaceStore.terminals"
    :active-panel-id="workspaceStore.activePanelId"
    @select-tab="workspaceStore.selectTab"
    @close-tab="workspaceStore.closeTab"
    @add-tab="workspaceStore.addTab"
    @split-horizontal="workspaceStore.splitHorizontal"
    @split-vertical="workspaceStore.splitVertical"
    @close-panel="workspaceStore.closePanel"
    @move-tab="workspaceStore.moveTab"
    @duplicate-tab="workspaceStore.duplicateTab"
    @move-tab-to-new-panel="workspaceStore.moveTabToNewPanel"
    @terminal-ready="workspaceStore.terminalReady"
    @set-active-panel="workspaceStore.setActivePanel"
    @layout-updated="workspaceStore.updateLayout"
  />
</template>

<script setup lang="ts">
import { onMounted, onBeforeUnmount } from "vue";
import PanelManager from "./ui/PanelManager.vue";
import { useWorkspaceStore } from "../stores/workspace";

const workspaceStore = useWorkspaceStore();

// Setup component
onMounted(async () => {
  await workspaceStore.initialize();
});

// Cleanup component
onBeforeUnmount(() => {
  workspaceStore.cleanup();
});
</script>
