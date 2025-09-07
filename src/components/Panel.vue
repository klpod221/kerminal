<template>
  <div class="flex flex-col h-full cursor-pointer relative bg-[#0D0D0D]" @click="handlePanelClick">
    <!-- Active panel background overlay -->
    <div
      class="absolute inset-0 transition-opacity duration-200 pointer-events-none"
      :class="{
        'opacity-100 bg-gradient-to-br from-[#141a20] to-[#0D0D0D]': isActive,
        'opacity-0 bg-[#0D0D0D]': !isActive
      }"
    ></div>
    <!-- Active panel blue tint -->
    <div
      class="absolute inset-0 transition-opacity duration-200 pointer-events-none"
      :class="{
        'opacity-100 bg-blue-500/5': isActive,
        'opacity-0': !isActive
      }"
    ></div>
    <!-- Tab Bar -->
    <TabBar
      class="relative z-10"
      :panel="panel"
      :terminals="terminals"
      :window-width="windowWidth"
      :is-active="isActive"
      @select-tab="selectTab"
      @close-tab="closeTab"
      @add-tab="addTab"
      @split-horizontal="splitHorizontal"
      @split-vertical="splitVertical"
      @close-panel="closePanel"
      @move-tab="moveTab"
      @duplicate-tab="duplicateTab"
      @move-tab-to-new-panel="moveTabToNewPanel"
    />

    <!-- Panel Content -->
    <div class="flex-1 overflow-hidden relative z-10">
      <!-- Terminal Manager -->
      <TerminalManager
        :terminals="activeTerminals"
        :active-terminal-id="panel.activeTabId"
        @terminal-ready="onTerminalReady"
      />
    </div>
  </div>
</template>

<script setup lang="ts">
import { computed } from 'vue'
import TabBar from './ui/TabBar.vue'
import TerminalManager from './TerminalManager.vue'
import type { PanelProps, PanelEmits } from '../types/components'

const props = defineProps<PanelProps>()

const emit = defineEmits<PanelEmits>()

// Filter terminals that belong to this panel's tabs
const activeTerminals = computed(() => {
  const tabIds = props.panel.tabs.map((tab) => tab.id)
  return props.terminals.filter((terminal) => tabIds.includes(terminal.id))
})

const selectTab = (panelId: string, tabId: string): void => {
  emit('selectTab', panelId, tabId)
  emit('panelClick', panelId) // Also make this panel active
}

const closeTab = (panelId: string, tabId: string): void => {
  emit('closeTab', panelId, tabId)
}

const addTab = (panelId: string): void => {
  emit('addTab', panelId)
  emit('panelClick', panelId) // Make this panel active when adding tab
}

const splitHorizontal = (panelId: string): void => {
  emit('splitHorizontal', panelId)
}

const splitVertical = (panelId: string): void => {
  emit('splitVertical', panelId)
}

const closePanel = (panelId: string): void => {
  emit('closePanel', panelId)
}

const moveTab = (
  fromPanelId: string,
  toPanelId: string,
  tabId: string,
  targetTabId?: string
): void => {
  emit('moveTab', fromPanelId, toPanelId, tabId, targetTabId)
}

const duplicateTab = (panelId: string, tabId: string): void => {
  emit('duplicateTab', panelId, tabId)
}

const moveTabToNewPanel = (panelId: string, tabId: string): void => {
  emit('moveTabToNewPanel', panelId, tabId)
}

const onTerminalReady = (terminalId: string): void => {
  emit('terminalReady', terminalId)
}

const handlePanelClick = (): void => {
  // Only activate panel, don't prevent event propagation
  // This allows clicks on panel background to activate the panel
  // while still allowing normal interactions with child elements
  emit('panelClick', props.panel.id)
}
</script>

<style scoped>
/* Panel entrance animation */
.panel-enter-active {
  transition: all 0.4s cubic-bezier(0.25, 0.8, 0.25, 1);
}

.panel-leave-active {
  transition: all 0.3s cubic-bezier(0.4, 0, 0.6, 1);
}

.panel-enter-from {
  opacity: 0;
  transform: scale(0.95) translateY(-10px);
}

.panel-leave-to {
  opacity: 0;
  transform: scale(0.95) translateY(10px);
}

/* Active panel glow effect */
.bg-\[#0D0D0D\]:has(.opacity-100) {
  box-shadow: 0 0 20px rgba(59, 130, 246, 0.1);
  transition: box-shadow 0.3s ease;
}

/* Smooth background transitions */
.absolute.inset-0 {
  transition: all 0.3s ease;
}
</style>
