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
    @split-panel-by-drop="workspaceStore.splitPanelByDrop"
    @clone-tab-and-split="workspaceStore.cloneTabAndSplit"
  />
</template>

<script setup lang="ts">
import { onMounted, onBeforeUnmount } from "vue";
import PanelManager from "./ui/PanelManager.vue";
import { useWorkspaceStore } from "../stores/workspace";
import { useGlobalShortcuts } from "../composables/useGlobalShortcuts";

const workspaceStore = useWorkspaceStore();

// Initialize global keyboard shortcuts (singleton, safe to call multiple times)
useGlobalShortcuts();

onMounted(async () => {
  await workspaceStore.initialize();
});

onBeforeUnmount(() => {
  workspaceStore.cleanup();
});
</script>
