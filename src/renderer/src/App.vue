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
      @toggle-ssh-drawer="toggleSSHDrawer"
    />

    <div class="flex-grow overflow-hidden">
      <Dashboard
        :class="{ hidden: !showDashboard }"
        @create-terminal="addTab"
        @open-ssh-profiles="toggleSSHDrawer"
      />
      <TerminalManager
        :class="{ hidden: showDashboard }"
        :terminals="terminals"
        :active-terminal-id="activeTerminalId"
        @terminal-ready="onTerminalReady"
      />
    </div>

    <!-- SSH Profiles Drawer -->
    <SSHProfileDrawer
      ref="sshProfileDrawerRef"
      v-model:visible="showSSHDrawer"
      @connect-profile="connectToSSHProfile"
      @edit-profile="editSSHProfile"
      @create-profile="createSSHProfile"
      @create-profile-in-group="createSSHProfileInGroup"
      @manage-groups="openCreateSSHGroup"
      @edit-group="editSSHGroup"
      @delete-group="deleteSSHGroup"
      @delete-profile="deleteSSHProfile"
    />

    <!-- SSH Profile Modal -->
    <SSHProfileModal
      v-model:visible="showSSHProfileModal"
      :profile="editingProfile"
      :groups="sshGroups"
      :preselected-group="selectedGroupForNewProfile"
      @save="saveSSHProfile"
      @update="updateSSHProfile"
      @close="handleSSHProfileModalClose"
    />

    <!-- SSH Group Modal -->
    <SSHGroupModal
      :show="showSSHGroupModal"
      :group="editingGroup"
      @close="handleSSHGroupModalClose"
      @save="saveSSHGroup"
      @update="updateSSHGroup"
    />
  </div>
</template>

<script setup lang="ts">
import { ref, onMounted, onUnmounted } from 'vue'
import TopBar from './components/TopBar.vue'
import Dashboard from './components/Dashboard.vue'
import TerminalManager from './components/TerminalManager.vue'
import SSHProfileDrawer from './components/SSHProfileDrawer.vue'
import SSHProfileModal from './components/SSHProfileModal.vue'
import SSHGroupModal from './components/SSHGroupModal.vue'
import type { SSHProfileWithConfig, SSHGroup, SSHProfile, SSHGroupWithProfiles } from './types/ssh'

interface Tab {
  id: string
  title: string
  active: boolean
  color?: string
}

interface TerminalInstance {
  id: string
  ready: boolean
}

const showDashboard = ref(false)
const tabs = ref<Tab[]>([])
const showSSHDrawer = ref(false)
const showSSHProfileModal = ref(false)
const showSSHGroupModal = ref(false)

const terminals = ref<TerminalInstance[]>([])
const sshProfileDrawerRef = ref()
const editingProfile = ref<SSHProfileWithConfig | null>(null)
const editingGroup = ref<SSHGroup | null>(null)
const sshGroups = ref<SSHGroup[]>([])
const selectedGroupForNewProfile = ref<SSHGroupWithProfiles | null>(null)

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

// SSH-related methods
const toggleSSHDrawer = (): void => {
  showSSHDrawer.value = !showSSHDrawer.value
}

const loadSSHGroups = async (): Promise<void> => {
  try {
    const groups = (await window.api.invoke('ssh-groups.getAll')) as SSHGroup[]
    sshGroups.value = groups.map((group: SSHGroup) => ({
      ...group,
      created: new Date(group.created),
      updated: new Date(group.updated)
    }))
  } catch (error) {
    console.error('Failed to load SSH groups:', error)
    sshGroups.value = []
  }
}

const refreshAllData = async (): Promise<void> => {
  try {
    await loadSSHGroups()
    console.log('All data refreshed successfully')
  } catch (error) {
    console.error('Failed to refresh data:', error)
  }
}

const connectToSSHProfile = (profile: SSHProfileWithConfig): void => {
  const newTabId = tabCounter.toString()
  const newTab: Tab = {
    id: newTabId,
    title: profile.name,
    active: true,
    color: profile.color
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
  showSSHDrawer.value = false

  // Request new SSH terminal from main process
  window.api?.send('terminal.createSSH', { terminalId: newTabId, profileId: profile.id })

  tabCounter++
}

const editSSHProfile = (profile: SSHProfileWithConfig): void => {
  editingProfile.value = profile
  showSSHProfileModal.value = true
  showSSHDrawer.value = false
}

const createSSHProfile = (): void => {
  editingProfile.value = null
  showSSHProfileModal.value = true
  showSSHDrawer.value = false
}

const createSSHProfileInGroup = (group: SSHGroupWithProfiles): void => {
  editingProfile.value = null
  // Pre-select the group for the new profile
  selectedGroupForNewProfile.value = group
  showSSHProfileModal.value = true
  showSSHDrawer.value = false
}

const saveSSHProfile = async (profileData: Partial<SSHProfile>): Promise<void> => {
  try {
    await window.api.invoke('ssh-profiles.create', profileData)
    showSSHProfileModal.value = false
    selectedGroupForNewProfile.value = null // Reset selected group
    await refreshAllData() // Refresh all data
    showSSHDrawer.value = true // Reopen drawer after saving profile
    // Force refresh the drawer
    if (sshProfileDrawerRef.value) {
      sshProfileDrawerRef.value.refreshProfiles()
    }
    console.log('SSH profile created successfully')
  } catch (error) {
    console.error('Failed to create SSH profile:', error)
  }
}

const updateSSHProfile = async (id: string, updates: Partial<SSHProfile>): Promise<void> => {
  try {
    await window.api.invoke('ssh-profiles.update', id, updates)
    showSSHProfileModal.value = false
    editingProfile.value = null
    await refreshAllData() // Refresh all data
    showSSHDrawer.value = true // Reopen drawer after updating profile
    // Force refresh the drawer
    if (sshProfileDrawerRef.value) {
      sshProfileDrawerRef.value.refreshProfiles()
    }
    console.log('SSH profile updated successfully')
  } catch (error) {
    console.error('Failed to update SSH profile:', error)
  }
}

const handleSSHProfileModalClose = (): void => {
  selectedGroupForNewProfile.value = null // Reset selected group
  showSSHDrawer.value = true // Reopen drawer when profile modal is closed
}

const editSSHGroup = (group: SSHGroupWithProfiles): void => {
  editingGroup.value = group
  showSSHGroupModal.value = true
  showSSHDrawer.value = false
}

const deleteSSHGroup = async (group: SSHGroupWithProfiles): Promise<void> => {
  try {
    await window.api.invoke('ssh-groups.delete', group.id)
    await refreshAllData() // Refresh all data
    // Force refresh the drawer if it's open
    if (showSSHDrawer.value && sshProfileDrawerRef.value) {
      sshProfileDrawerRef.value.refreshProfiles()
    }
    console.log('SSH group deleted successfully')
  } catch (error) {
    console.error('Failed to delete SSH group:', error)
    // Don't throw error to avoid unhandled promise rejections
  }
}

const deleteSSHProfile = async (profile: SSHProfileWithConfig): Promise<void> => {
  try {
    await window.api.invoke('ssh-profiles.delete', profile.id)
    await refreshAllData() // Refresh all data
    // Force refresh the drawer if it's open
    if (showSSHDrawer.value && sshProfileDrawerRef.value) {
      sshProfileDrawerRef.value.refreshProfiles()
    }
    console.log('SSH profile deleted successfully')
  } catch (error) {
    console.error('Failed to delete SSH profile:', error)
    // Don't throw error to avoid unhandled promise rejections
  }
}

// SSH Group related methods
const openCreateSSHGroup = (): void => {
  console.log('Opening SSH Group Modal...')
  editingGroup.value = null
  showSSHGroupModal.value = true
  showSSHDrawer.value = false // Close drawer when opening group modal
  console.log('showSSHGroupModal:', showSSHGroupModal.value)
}

const handleSSHGroupModalClose = (): void => {
  showSSHGroupModal.value = false
  editingGroup.value = null
  showSSHDrawer.value = true // Reopen drawer when group modal is closed via X or Cancel
}

const saveSSHGroup = async (
  groupData: Omit<SSHGroup, 'id' | 'created' | 'updated'>
): Promise<void> => {
  try {
    await window.api.invoke('ssh-groups.create', groupData)
    showSSHGroupModal.value = false
    await refreshAllData() // Refresh all data
    showSSHDrawer.value = true // Reopen drawer after saving group
    // Force refresh the drawer
    if (sshProfileDrawerRef.value) {
      sshProfileDrawerRef.value.refreshProfiles()
    }
    console.log('SSH group created successfully')
  } catch (error) {
    console.error('Failed to create SSH group:', error)
  }
}

const updateSSHGroup = async (id: string, updates: Partial<SSHGroup>): Promise<void> => {
  try {
    await window.api.invoke('ssh-groups.update', id, updates)
    showSSHGroupModal.value = false
    editingGroup.value = null
    await refreshAllData() // Refresh all data
    showSSHDrawer.value = true // Reopen drawer after updating group
    // Force refresh the drawer
    if (sshProfileDrawerRef.value) {
      sshProfileDrawerRef.value.refreshProfiles()
    }
    console.log('SSH group updated successfully')
  } catch (error) {
    console.error('Failed to update SSH group:', error)
  }
}

// Auto create first tab when app starts
onMounted(() => {
  addTab()
  refreshAllData() // Use refreshAllData instead of loadSSHGroups

  // Add global error handler to prevent unhandled promise rejections
  window.addEventListener('unhandledrejection', (event) => {
    console.error('Unhandled promise rejection:', event.reason)
    event.preventDefault() // Prevent the default behavior
  })

  // Listen for terminal title changes
  const unsubscribeTitleChanged = window.api?.on('terminal.titleChanged', (...args: unknown[]) => {
    const data = args[0] as { terminalId: string; title: string }
    updateTabTitle(data.terminalId, data.title)
  })

  // Listen for terminal auto close events
  const unsubscribeAutoClose = window.api?.on('terminal.autoClose', (...args: unknown[]) => {
    const data = args[0] as { terminalId: string; reason: string; exitCode?: number }
    console.log(`Auto closing terminal ${data.terminalId}: ${data.reason}`)

    // Auto close the tab
    closeTab(data.terminalId)
  })

  // Store cleanup functions
  onUnmounted(() => {
    unsubscribeTitleChanged?.()
    unsubscribeAutoClose?.()
    // Remove global error handler
    window.removeEventListener('unhandledrejection', () => {})
  })
})
</script>
