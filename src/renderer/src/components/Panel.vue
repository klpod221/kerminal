<template>
  <div class="flex flex-col h-full bg-[#0D0D0D]" :class="{ 'border border-blue-500': isActive }">
    <!-- Tab Bar -->
    <TabBar
      :panel="panel"
      :window-width="windowWidth"
      @select-tab="selectTab"
      @close-tab="closeTab"
      @add-tab="addTab"
      @split-horizontal="splitHorizontal"
      @split-vertical="splitVertical"
      @close-panel="closePanel"
      @move-tab="moveTab"
    />

    <!-- Panel Content -->
    <div class="flex-1 overflow-hidden">
      <!-- Dashboard -->
      <Dashboard
        v-if="showDashboard"
        @create-terminal="() => addTab(panel.id)"
        @open-ssh-profiles="openSSHProfiles"
      />

      <!-- Terminal Manager -->
      <TerminalManager
        v-else
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
import Dashboard from './Dashboard.vue'
import TerminalManager from './TerminalManager.vue'
import type { Panel, TerminalInstance } from '../types/panel'

interface Props {
  panel: Panel
  terminals: TerminalInstance[]
  windowWidth: number
  isActive: boolean
  showDashboard: boolean
}

const props = defineProps<Props>()

const emit = defineEmits<{
  selectTab: [panelId: string, tabId: string]
  closeTab: [panelId: string, tabId: string]
  addTab: [panelId: string]
  splitHorizontal: [panelId: string]
  splitVertical: [panelId: string]
  closePanel: [panelId: string]
  moveTab: [fromPanelId: string, toPanelId: string, tabId: string, targetTabId?: string]
  terminalReady: [terminalId: string]
  openSSHProfiles: []
  panelClick: [panelId: string]
}>()

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

const onTerminalReady = (terminalId: string): void => {
  emit('terminalReady', terminalId)
}

const openSSHProfiles = (): void => {
  emit('openSSHProfiles')
}
</script>
