<template>
  <div
    class="title-bar flex items-center h-[30px] min-h-[30px] max-h-[30px] text-white font-sans select-none bg-[#0D0D0D] border-b border-gray-800 flex-shrink-0 relative z-50"
  >
    <!-- Dashboard Icon -->
    <div
      class="no-drag flex items-center px-3 hover:bg-gray-800 cursor-pointer h-full max-h-[30px] transition-colors duration-200 flex-shrink-0"
      :class="{ 'bg-gray-800': isDashboardActive }"
      @click="openDashboard"
    >
      <img
        src="../assets/images/logo_500.png"
        alt="Dashboard"
        class="w-4 h-4 transition-opacity duration-200"
        :class="isDashboardActive ? 'opacity-100' : 'opacity-60 hover:opacity-100'"
      />
    </div>

    <!-- Workspace Icon -->
    <div
      class="no-drag flex items-center px-3 hover:bg-gray-800 cursor-pointer h-full max-h-[30px] transition-colors duration-200 flex-shrink-0"
      :class="{ 'bg-gray-800': !isDashboardActive }"
      @click="openWorkspace"
    >
      <LayoutGrid
        :size="16"
        class="transition-colors duration-200"
        :class="!isDashboardActive ? 'text-blue-400' : 'text-gray-400 hover:text-white'"
      />
    </div>

    <!-- SSH Profiles Icon -->
    <div
      class="no-drag flex items-center px-3 hover:bg-gray-800 cursor-pointer h-full max-h-[30px] transition-colors duration-200 flex-shrink-0"
      @click="toggleSSHDrawer"
    >
      <Server :size="16" class="transition-colors duration-200 text-gray-400 hover:text-white" />
    </div>

    <!-- Draggable space - Always visible with flex-1 -->
    <div class="draggable flex-1 h-full"></div>

    <!-- Window Controls -->
    <div class="flex items-center h-full max-h-[30px] flex-shrink-0">
      <Button
        title="Saved Commands"
        variant="ghost"
        size="sm"
        :icon="BookmarkIcon"
        @click="toggleSavedCommands"
      />
      <Button
        title="SSH Tunnels"
        variant="ghost"
        size="sm"
        :icon="Wifi"
        :class="hasActiveTunnels ? 'text-green-400' : 'text-gray-400'"
        @click="() => emit('toggle-ssh-tunnels')"
      />
      <Button
        title="Sync Settings"
        variant="ghost"
        size="sm"
        :icon="CloudIcon"
        :class="syncStatus?.isConnected ? 'text-green-400' : 'text-gray-400'"
        @click="openSyncSettings"
      />
      <Button
        title="Minimize window"
        variant="ghost"
        size="sm"
        :icon="Minus"
        @click="minimizeWindow"
      />
      <Button
        title="Maximize window"
        variant="ghost"
        size="sm"
        :icon="isMaximized ? Minimize2 : Maximize2"
        @click="maximizeWindow"
      />
      <Button title="Close window" variant="ghost" size="sm" :icon="X" @click="closeWindow" />
    </div>
  </div>
</template>

<script setup lang="ts">
import { ref, onMounted, onBeforeUnmount, watch } from 'vue'
import {
  LayoutGrid,
  X,
  Minus,
  Minimize2,
  Maximize2,
  Server,
  Bookmark,
  Cloud,
  Wifi
} from 'lucide-vue-next'
import Button from './ui/Button.vue'
import type { SyncStatus } from '../types/sync'
import type { SSHTunnelWithProfile } from '../types/ssh'

interface Props {
  isDashboardActive?: boolean
  syncStatusRefresh?: number // Add this to force refresh sync status
}

const { isDashboardActive = false, syncStatusRefresh = 0 } = defineProps<Props>()

const emit = defineEmits<{
  'open-dashboard': []
  'open-workspace': []
  'toggle-ssh-drawer': []
  'toggle-saved-commands': []
  'toggle-ssh-tunnels': []
  'open-sync-settings': []
}>()

// Use icons
const BookmarkIcon = Bookmark
const CloudIcon = Cloud

const isMaximized = ref(false)
const syncStatus = ref<SyncStatus | null>(null)
const hasActiveTunnels = ref(false)

let removeMaximizedListener: (() => void) | null = null
let tunnelStatusInterval: ReturnType<typeof setInterval> | null = null
let syncStatusInterval: ReturnType<typeof setInterval> | null = null

// Load sync status
async function loadSyncStatus(): Promise<void> {
  try {
    syncStatus.value = (await window.api.invoke('sync.getStatus')) as SyncStatus
  } catch (error) {
    console.error('Failed to load sync status:', error)
    syncStatus.value = null
  }
}

// Force refresh sync status (called when sync config is updated)
async function refreshSyncStatus(): Promise<void> {
  // Try to refresh multiple times with delay to handle timing issues
  for (let i = 0; i < 3; i++) {
    await loadSyncStatus()
    if (syncStatus.value?.isConnected) {
      break // Stop retrying if we got a connected status
    }
    if (i < 2) {
      await new Promise((resolve) => setTimeout(resolve, 200)) // Wait 200ms before retry
    }
  }
}

// Load tunnel status
async function loadTunnelStatus(): Promise<void> {
  try {
    const tunnels = (await window.api.invoke('ssh-tunnels.getAll')) as SSHTunnelWithProfile[]
    hasActiveTunnels.value = tunnels.some((tunnel) => tunnel.status === 'running')
  } catch (error) {
    console.error('Failed to load tunnel status:', error)
    hasActiveTunnels.value = false
  }
}

onMounted(() => {
  // Load sync status
  loadSyncStatus()

  // Load tunnel status
  loadTunnelStatus()

  // Update tunnel status every 5 seconds
  tunnelStatusInterval = setInterval(loadTunnelStatus, 5000)

  // Update sync status every 10 seconds
  syncStatusInterval = setInterval(loadSyncStatus, 10000)

  // Listen for maximize state changes from main process
  if (window.api?.on) {
    removeMaximizedListener = window.api.on('window-maximized', (...args: unknown[]) => {
      const maximized = args[0] as boolean
      isMaximized.value = maximized
    })
  }
})

onBeforeUnmount(() => {
  if (removeMaximizedListener) {
    removeMaximizedListener()
  }

  if (tunnelStatusInterval) {
    clearInterval(tunnelStatusInterval)
  }

  if (syncStatusInterval) {
    clearInterval(syncStatusInterval)
  }
})

// Watch for syncStatusRefresh changes to force refresh sync status
watch(
  () => syncStatusRefresh,
  () => {
    refreshSyncStatus()
  }
)

const openDashboard = (): void => {
  emit('open-dashboard')
}

const openWorkspace = (): void => {
  emit('open-workspace')
}

const toggleSSHDrawer = (): void => {
  emit('toggle-ssh-drawer')
}

const toggleSavedCommands = (): void => {
  emit('toggle-saved-commands')
}

const openSyncSettings = (): void => {
  emit('open-sync-settings')
}

const minimizeWindow = (): void => {
  if (window.api?.send) {
    window.api.send('window-minimize', {})
  }
}

const maximizeWindow = (): void => {
  if (window.api?.send) {
    window.api.send('window-maximize', {})
  }
}

const closeWindow = (): void => {
  if (window.api?.send) {
    window.api.send('window-close', {})
  }
}
</script>

<style scoped>
.title-bar {
  -webkit-app-region: no-drag;
}

.draggable {
  -webkit-app-region: drag;
}

.no-drag {
  -webkit-app-region: no-drag;
}
</style>
