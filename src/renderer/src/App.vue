<template>
  <div class="h-screen w-screen flex flex-col bg-[#0D0D0D] overflow-hidden">
    <TopBar
      class="flex-shrink-0"
      :is-dashboard-active="showDashboard"
      :tabs="tabs"
      @open-dashboard="openDashboard"
      @open-terminal="openTerminal"
      @add-tab="addTab"
      @close-tab="closeTab"
      @select-tab="selectTab"
    />

    <div class="flex-grow overflow-hidden">
      <Dashboard :class="{ hidden: !showDashboard }" @create-terminal="addTab" />
      <TerminalManager
        :class="{ hidden: showDashboard }"
        :terminals="terminals"
        :active-terminal-id="activeTerminalId"
        @terminal-ready="onTerminalReady"
      />
    </div>
  </div>
</template>

<script setup lang="ts">
import { ref, onMounted } from 'vue'
import TopBar from './components/TopBar.vue'
import Dashboard from './components/Dashboard.vue'
import TerminalManager from './components/TerminalManager.vue'

interface Tab {
  id: string
  title: string
  active: boolean
}

interface TerminalInstance {
  id: string
  ready: boolean
}

const showDashboard = ref(false)
const tabs = ref<Tab[]>([])

const terminals = ref<TerminalInstance[]>([])

const activeTerminalId = ref('')
let tabCounter = 1

const openDashboard = (): void => {
  // Deactivate all tabs when switching to dashboard
  tabs.value.forEach((tab) => {
    tab.active = false
  })
  activeTerminalId.value = ''
  showDashboard.value = true
}

const openTerminal = (): void => {
  showDashboard.value = false
}

const addTab = (): void => {
  const newTabId = tabCounter.toString()
  const newTab: Tab = {
    id: newTabId,
    title: 'Terminal', // Default title, will be updated by terminal process
    active: true // Make new tab active immediately
  }

  const newTerminal: TerminalInstance = {
    id: newTabId,
    ready: false
  }

  // Deactivate all existing tabs
  tabs.value.forEach((tab) => {
    tab.active = false
  })

  tabs.value.push(newTab)
  terminals.value.push(newTerminal)

  // Switch to new terminal
  activeTerminalId.value = newTabId
  showDashboard.value = false

  // Request new terminal from main process
  window.api?.send('terminal.create', { terminalId: newTabId })

  tabCounter++
}

const closeTab = (tabId: string): void => {
  const tabIndex = tabs.value.findIndex((tab) => tab.id === tabId)
  const terminalIndex = terminals.value.findIndex((terminal) => terminal.id === tabId)

  if (tabIndex !== -1) {
    // Remove terminal instance
    if (terminalIndex !== -1) {
      // Request terminal destruction from main process
      window.api?.send('terminal.destroy', { terminalId: tabId })
      terminals.value.splice(terminalIndex, 1)
    }

    // Remove the tab
    const wasActive = tabs.value[tabIndex].active
    tabs.value.splice(tabIndex, 1)

    // If this was the last tab, close all and switch to dashboard
    if (tabs.value.length === 0) {
      showDashboard.value = true
      activeTerminalId.value = ''
      return
    }

    // If closed tab was active, activate another tab
    if (wasActive) {
      const newActiveIndex = Math.min(tabIndex, tabs.value.length - 1)
      tabs.value[newActiveIndex].active = true
      activeTerminalId.value = tabs.value[newActiveIndex].id
      showDashboard.value = false
    }
  }
}

const selectTab = (tabId: string): void => {
  tabs.value.forEach((tab) => {
    tab.active = tab.id === tabId
  })
  activeTerminalId.value = tabId
  showDashboard.value = false
}

const onTerminalReady = (terminalId: string): void => {
  const terminal = terminals.value.find((t) => t.id === terminalId)
  if (terminal) {
    terminal.ready = true
  }
}

const updateTabTitle = (terminalId: string, title: string): void => {
  const tab = tabs.value.find((t) => t.id === terminalId)
  if (tab) {
    tab.title = title
  }
}

// Auto create first tab when app starts
onMounted(() => {
  addTab()

  // Listen for terminal title changes
  window.api?.on('terminal.titleChanged', (...args: unknown[]) => {
    const data = args[0] as { terminalId: string; title: string }
    updateTabTitle(data.terminalId, data.title)
  })
})
</script>
