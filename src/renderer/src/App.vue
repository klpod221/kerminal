<template>
  <div class="h-screen w-screen flex flex-col bg-[#0D0D0D] overflow-hidden">
    <TopBar
      class="flex-shrink-0"
      :is-dashboard-active="showDashboard"
      :sync-status-refresh="syncStatusRefreshCounter"
      @open-dashboard="openDashboard"
      @toggle-ssh-drawer="toggleSSHDrawer"
      @toggle-saved-commands="toggleSavedCommands"
      @toggle-ssh-tunnels="toggleSSHTunnels"
      @open-sync-settings="openSyncSettings"
    />

    <div class="flex-grow overflow-hidden">
      <PanelManager
        :layout="panelLayout"
        :terminals="terminals"
        :window-width="windowWidth"
        :active-panel-id="activePanelId"
        :show-dashboard="showDashboard"
        @select-tab="selectTab"
        @close-tab="closeTab"
        @add-tab="addTab"
        @split-horizontal="splitHorizontal"
        @split-vertical="splitVertical"
        @close-panel="closePanel"
        @move-tab="moveTab"
        @terminal-ready="onTerminalReady"
        @open-ssh-profiles="toggleSSHDrawer"
        @set-active-panel="setActivePanel"
        @layout-updated="updateLayout"
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

    <!-- Saved Commands Drawer -->
    <SavedCommandDrawer
      v-model:visible="showSavedCommands"
      :active-terminal-id="getCurrentActiveTerminalId()"
    />

    <!-- SSH Tunnel Manager Modal -->
    <Modal
      :visible="showSSHTunnels"
      title="SSH Tunnels"
      :icon="Wifi"
      icon-background="bg-purple-500/20"
      icon-color="text-purple-400"
      size="xl"
      @close="closeSSHTunnels"
    >
      <SSHTunnelManager
        :on-hide-manager="hideSSHTunnelsModal"
        :on-show-manager="showSSHTunnelsModal"
        @create-tunnel="handleCreateTunnel"
        @edit-tunnel="handleEditTunnel"
      />
    </Modal>

    <!-- Sync Settings Modal -->
    <SyncSettingsModal
      :visible="showSyncSettings"
      @close="closeSyncSettings"
      @config-updated="onSyncConfigUpdated"
    />

    <!-- SSH Tunnel Modal -->
    <SSHTunnelModal
      v-model:visible="showSSHTunnelModal"
      :tunnel="selectedTunnel"
      :profiles="sshProfiles"
      @save="handleSaveTunnel"
      @update="handleUpdateTunnel"
      @close="handleCloseTunnelModal"
    />
  </div>
</template>

<script setup lang="ts">
import { ref, onMounted, onUnmounted } from 'vue'
import { Wifi } from 'lucide-vue-next'
import TopBar from './components/TopBar.vue'
import PanelManager from './components/PanelManager.vue'
import SSHProfileDrawer from './components/SSHProfileDrawer.vue'
import SSHProfileModal from './components/SSHProfileModal.vue'
import SSHGroupModal from './components/SSHGroupModal.vue'
import SavedCommandDrawer from './components/SavedCommandDrawer.vue'
import SyncSettingsModal from './components/SyncSettingsModal.vue'
import Modal from './components/ui/Modal.vue'
import SSHTunnelManager from './components/SSHTunnelManager.vue'
import SSHTunnelModal from './components/SSHTunnelModal.vue'
import type {
  SSHProfileWithConfig,
  SSHGroup,
  SSHProfile,
  SSHGroupWithProfiles,
  SSHTunnelWithProfile,
  SSHTunnel
} from './types/ssh'
import type { SyncConfig } from './types/sync'
import type { PanelLayout, Panel, Tab, TerminalInstance } from './types/panel'
import { message } from './utils/message'

const showDashboard = ref(false)
const showSSHDrawer = ref(false)
const showSSHProfileModal = ref(false)
const showSSHGroupModal = ref(false)
const showSavedCommands = ref(false)
const showSyncSettings = ref(false)
const showSSHTunnels = ref(false)
const showSSHTunnelModal = ref(false)
const selectedTunnel = ref<SSHTunnelWithProfile | null>(null)
const sshProfiles = ref<SSHProfile[]>([])
const syncStatusRefreshCounter = ref(0)

const terminals = ref<TerminalInstance[]>([])
const sshProfileDrawerRef = ref()
const editingProfile = ref<SSHProfileWithConfig | null>(null)
const editingGroup = ref<SSHGroup | null>(null)
const sshGroups = ref<SSHGroup[]>([])
const selectedGroupForNewProfile = ref<SSHGroupWithProfiles | null>(null)

// Panel system state
const panelLayout = ref<PanelLayout>({
  type: 'panel',
  id: 'panel-1',
  panel: {
    id: 'panel-1',
    activeTabId: '',
    tabs: []
  }
})
const activePanelId = ref('panel-1')
const windowWidth = ref(window.innerWidth)

let tabCounter = 1
let panelCounter = 2 // Start from 2 since panel-1 is already created

const findPanelInLayout = (layout: PanelLayout, panelId: string): Panel | null => {
  if (layout.type === 'panel' && layout.panel?.id === panelId) {
    return layout.panel
  }
  if (layout.type === 'split' && layout.children) {
    for (const child of layout.children) {
      const found = findPanelInLayout(child, panelId)
      if (found) return found
    }
  }
  return null
}

const removePanelFromLayout = (layout: PanelLayout, panelId: string): PanelLayout | null => {
  if (layout.type === 'panel' && layout.panel?.id === panelId) {
    return null // This panel should be removed
  }

  if (layout.type === 'split' && layout.children) {
    const filteredChildren = layout.children
      .map((child) => removePanelFromLayout(child, panelId))
      .filter((child) => child !== null) as PanelLayout[]

    if (filteredChildren.length === 0) {
      return null
    }

    if (filteredChildren.length === 1) {
      // Collapse split with only one child
      return filteredChildren[0]
    }

    // Update sizes proportionally when a child is removed
    const newSizes = layout.sizes
      ? filteredChildren.map(() => 1 / filteredChildren.length)
      : undefined

    return {
      ...layout,
      children: filteredChildren,
      sizes: newSizes
    }
  }

  return layout
}

const splitPanelInLayout = (
  layout: PanelLayout,
  panelId: string,
  newPanel: Panel,
  direction: 'horizontal' | 'vertical'
): boolean => {
  if (layout.type === 'panel' && layout.panel?.id === panelId) {
    // This is the panel we want to split
    // We need to replace this layout with a split layout
    const originalPanel = layout.panel

    // Update layout properties in-place for better reactivity
    layout.type = 'split'
    layout.direction = direction
    layout.children = [
      {
        type: 'panel',
        id: originalPanel.id,
        panel: originalPanel
      },
      {
        type: 'panel',
        id: newPanel.id,
        panel: newPanel
      }
    ]
    layout.sizes = [0.5, 0.5]

    // Clear panel property since we're now a split
    delete (layout as PanelLayout & { panel?: Panel }).panel

    return true
  }

  if (layout.type === 'split' && layout.children) {
    for (const child of layout.children) {
      if (splitPanelInLayout(child, panelId, newPanel, direction)) {
        return true
      }
    }
  }

  return false
}

const openDashboard = (): void => {
  showDashboard.value = true
}

const setActivePanel = (panelId: string): void => {
  activePanelId.value = panelId
}

const selectTab = (panelId: string, tabId: string): void => {
  const panel = findPanelInLayout(panelLayout.value, panelId)
  if (panel) {
    panel.activeTabId = tabId
    activePanelId.value = panelId
    showDashboard.value = false
  }
}

const addTab = (panelId: string): void => {
  const panel = findPanelInLayout(panelLayout.value, panelId)
  if (!panel) return

  const newTabId = tabCounter.toString()
  const newTab: Tab = {
    id: newTabId,
    title: 'Terminal' // Default title, will be updated by terminal process
  }

  const newTerminal: TerminalInstance = {
    id: newTabId,
    ready: false
  }

  // Add tab to panel
  panel.tabs.push(newTab)
  panel.activeTabId = newTabId

  // Add terminal instance
  terminals.value.push(newTerminal)

  // Set active panel and hide dashboard
  activePanelId.value = panelId
  showDashboard.value = false

  // Request new terminal from main process
  window.api?.send('terminal.create', { terminalId: newTabId })

  tabCounter++
}

const closeTab = (panelId: string, tabId: string): void => {
  const panel = findPanelInLayout(panelLayout.value, panelId)
  if (!panel) return

  const tabIndex = panel.tabs.findIndex((tab) => tab.id === tabId)
  const terminalIndex = terminals.value.findIndex((terminal) => terminal.id === tabId)

  if (tabIndex !== -1) {
    // Remove terminal instance
    if (terminalIndex !== -1) {
      // Request terminal destruction from main process
      window.api?.send('terminal.destroy', { terminalId: tabId })
      terminals.value.splice(terminalIndex, 1)
    }

    // Remove the tab
    const wasActive = panel.activeTabId === tabId
    panel.tabs.splice(tabIndex, 1)

    // AUTO-CLOSE PANEL: If this was the last tab in the panel, auto-close the panel
    if (panel.tabs.length === 0) {
      autoClosePanel(panelId)
      return
    }

    // If closed tab was active, activate another tab
    if (wasActive) {
      const newActiveIndex = Math.min(tabIndex, panel.tabs.length - 1)
      panel.activeTabId = panel.tabs[newActiveIndex].id
    }
  }
}

// Auto close panel when it has no tabs left
const autoClosePanel = (panelId: string): void => {
  // Remove panel from layout
  const newLayout = removePanelFromLayout(panelLayout.value, panelId)
  if (newLayout) {
    panelLayout.value = newLayout
    // Find a new active panel if the closed panel was active
    if (activePanelId.value === panelId) {
      const firstPanel = findFirstPanel(panelLayout.value)
      if (firstPanel) {
        activePanelId.value = firstPanel.id
      } else {
        // No panels left, show dashboard
        showDashboard.value = true
      }
    }
  } else {
    // All panels closed, show dashboard
    showDashboard.value = true
  }
}

const splitHorizontal = (panelId: string): void => {
  const panel = findPanelInLayout(panelLayout.value, panelId)
  if (!panel) return

  // Clone current active tab or create default tab
  let newTab: Tab
  if (panel.activeTabId && panel.tabs.length > 0) {
    const activeTab = panel.tabs.find((tab) => tab.id === panel.activeTabId)
    if (activeTab) {
      const newTabId = tabCounter.toString()
      newTab = {
        id: newTabId,
        title: activeTab.title,
        color: activeTab.color,
        profileId: activeTab.profileId,
        groupId: activeTab.groupId
      }
      // Create terminal for cloned tab
      const newTerminal: TerminalInstance = { id: newTabId, ready: false }
      terminals.value.push(newTerminal)

      if (activeTab.profileId) {
        // Clone SSH connection
        window.api?.send('terminal.createSSH', {
          terminalId: newTabId,
          profileId: activeTab.profileId
        })
      } else {
        // Clone regular terminal
        window.api?.send('terminal.create', { terminalId: newTabId })
      }
    } else {
      newTab = { id: tabCounter.toString(), title: 'Terminal' }
    }
  } else {
    newTab = { id: tabCounter.toString(), title: 'Terminal' }
  }

  // Create new panel
  const newPanelId = `panel-${panelCounter++}`
  const newPanel: Panel = {
    id: newPanelId,
    activeTabId: newTab.id,
    tabs: [newTab]
  }

  // Split the specific panel in the layout
  splitPanelInLayout(panelLayout.value, panelId, newPanel, 'horizontal')
  activePanelId.value = newPanelId
  tabCounter++
}

const splitVertical = (panelId: string): void => {
  const panel = findPanelInLayout(panelLayout.value, panelId)
  if (!panel) return

  // Clone current active tab or create default tab
  let newTab: Tab
  if (panel.activeTabId && panel.tabs.length > 0) {
    const activeTab = panel.tabs.find((tab) => tab.id === panel.activeTabId)
    if (activeTab) {
      const newTabId = tabCounter.toString()
      newTab = {
        id: newTabId,
        title: activeTab.title,
        color: activeTab.color,
        profileId: activeTab.profileId,
        groupId: activeTab.groupId
      }
      // Create terminal for cloned tab
      const newTerminal: TerminalInstance = { id: newTabId, ready: false }
      terminals.value.push(newTerminal)

      if (activeTab.profileId) {
        // Clone SSH connection
        window.api?.send('terminal.createSSH', {
          terminalId: newTabId,
          profileId: activeTab.profileId
        })
      } else {
        // Clone regular terminal
        window.api?.send('terminal.create', { terminalId: newTabId })
      }
    } else {
      newTab = { id: tabCounter.toString(), title: 'Terminal' }
    }
  } else {
    newTab = { id: tabCounter.toString(), title: 'Terminal' }
  }

  // Create new panel
  const newPanelId = `panel-${panelCounter++}`
  const newPanel: Panel = {
    id: newPanelId,
    activeTabId: newTab.id,
    tabs: [newTab]
  }

  // Split the specific panel in the layout
  splitPanelInLayout(panelLayout.value, panelId, newPanel, 'vertical')
  activePanelId.value = newPanelId
  tabCounter++
}

const closePanel = (panelId: string): void => {
  const panel = findPanelInLayout(panelLayout.value, panelId)
  if (!panel) return

  // Close all tabs in the panel (this will also destroy their terminals)
  const tabIds = [...panel.tabs.map((tab) => tab.id)] // Create a copy to avoid mutation during iteration

  for (const tabId of tabIds) {
    const terminalIndex = terminals.value.findIndex((terminal) => terminal.id === tabId)
    if (terminalIndex !== -1) {
      // Request terminal destruction from main process
      window.api?.send('terminal.destroy', { terminalId: tabId })
      terminals.value.splice(terminalIndex, 1)
    }
  }

  // Remove panel from layout
  const newLayout = removePanelFromLayout(panelLayout.value, panelId)
  if (newLayout) {
    panelLayout.value = newLayout
    // Find a new active panel if the closed panel was active
    if (activePanelId.value === panelId) {
      const firstPanel = findFirstPanel(panelLayout.value)
      if (firstPanel) {
        activePanelId.value = firstPanel.id
      } else {
        // No panels left, show dashboard
        showDashboard.value = true
      }
    }
  } else {
    // All panels closed, show dashboard
    showDashboard.value = true
  }
}

const findFirstPanel = (layout: PanelLayout): Panel | null => {
  if (layout.type === 'panel') {
    return layout.panel || null
  }
  if (layout.type === 'split' && layout.children) {
    for (const child of layout.children) {
      const found = findFirstPanel(child)
      if (found) return found
    }
  }
  return null
}

/**
 * Move a tab between panels or reorder within the same panel.
 * @param {string} fromPanelId - Source panel ID.
 * @param {string} toPanelId - Destination panel ID.
 * @param {string} tabId - Tab ID to move.
 * @param {string} [targetTabId] - Target tab ID for reordering.
 */
const moveTab = (
  fromPanelId: string,
  toPanelId: string,
  tabId: string,
  targetTabId?: string
): void => {
  if (fromPanelId === toPanelId) {
    reorderTabWithinPanel(fromPanelId, tabId, targetTabId)
  } else {
    moveTabBetweenPanels(fromPanelId, toPanelId, tabId, targetTabId)
  }
}

/**
 * Reorder a tab within the same panel.
 * @param {string} panelId
 * @param {string} tabId
 * @param {string} [targetTabId]
 */
function reorderTabWithinPanel(panelId: string, tabId: string, targetTabId?: string): void {
  const panel = findPanelInLayout(panelLayout.value, panelId)
  if (!panel || !targetTabId) return

  const draggedIndex = panel.tabs.findIndex((tab) => tab.id === tabId)
  const targetIndex = panel.tabs.findIndex((tab) => tab.id === targetTabId)

  if (draggedIndex === -1 || targetIndex === -1) return

  const [draggedTab] = panel.tabs.splice(draggedIndex, 1)
  panel.tabs.splice(targetIndex, 0, draggedTab)
}

/**
 * Move a tab from one panel to another.
 * @param {string} fromPanelId
 * @param {string} toPanelId
 * @param {string} tabId
 * @param {string} [targetTabId]
 */
function moveTabBetweenPanels(
  fromPanelId: string,
  toPanelId: string,
  tabId: string,
  targetTabId?: string
): void {
  const fromPanel = findPanelInLayout(panelLayout.value, fromPanelId)
  const toPanel = findPanelInLayout(panelLayout.value, toPanelId)

  if (!fromPanel || !toPanel) return

  const tabIndex = fromPanel.tabs.findIndex((tab) => tab.id === tabId)
  if (tabIndex === -1) return

  const [tab] = fromPanel.tabs.splice(tabIndex, 1)

  insertTabToPanel(toPanel, tab, targetTabId)

  updateActiveTabsAfterMove(fromPanel, toPanel, tabId, tabIndex)
}

/**
 * Insert tab to panel at target position or at the end.
 * @param {Panel} panel
 * @param {Tab} tab
 * @param {string} [targetTabId]
 */
function insertTabToPanel(panel: Panel, tab: Tab, targetTabId?: string): void {
  if (targetTabId) {
    const targetIndex = panel.tabs.findIndex((t) => t.id === targetTabId)
    if (targetIndex !== -1) {
      panel.tabs.splice(targetIndex, 0, tab)
      return
    }
  }
  panel.tabs.push(tab)
}

/**
 * Update activeTabId for panels after moving tab.
 * @param {Panel} fromPanel
 * @param {Panel} toPanel
 * @param {string} tabId
 * @param {number} tabIndex
 */
function updateActiveTabsAfterMove(
  fromPanel: Panel,
  toPanel: Panel,
  tabId: string,
  tabIndex: number
): void {
  if (fromPanel.activeTabId === tabId) {
    if (fromPanel.tabs.length > 0) {
      fromPanel.activeTabId = fromPanel.tabs[Math.min(tabIndex, fromPanel.tabs.length - 1)].id
    } else {
      fromPanel.activeTabId = ''
    }
  }
  toPanel.activeTabId = tabId
  activePanelId.value = toPanel.id
}

const getCurrentActiveTerminalId = (): string => {
  const activePanel = findPanelInLayout(panelLayout.value, activePanelId.value)
  return activePanel?.activeTabId || ''
}

const updateLayout = (newLayout: PanelLayout): void => {
  // Deep clone to ensure reactivity
  panelLayout.value = JSON.parse(JSON.stringify(newLayout))
}

const onTerminalReady = (terminalId: string): void => {
  const terminal = terminals.value.find((t) => t.id === terminalId)
  if (terminal) {
    terminal.ready = true
  }
}

const updateTabTitle = (terminalId: string, title: string): void => {
  // Find tab in all panels
  const updateInLayout = (layout: PanelLayout): boolean => {
    if (layout.type === 'panel' && layout.panel) {
      const tab = layout.panel.tabs.find((t) => t.id === terminalId)
      if (tab) {
        tab.title = title
        return true
      }
    }
    if (layout.type === 'split' && layout.children) {
      for (const child of layout.children) {
        if (updateInLayout(child)) return true
      }
    }
    return false
  }
  updateInLayout(panelLayout.value)
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
  // Find active panel or default panel
  const activePanel =
    findPanelInLayout(panelLayout.value, activePanelId.value) || findFirstPanel(panelLayout.value)

  if (!activePanel) return

  const newTabId = tabCounter.toString()
  const newTab: Tab = {
    id: newTabId,
    title: profile.name,
    color: profile.color,
    profileId: profile.id,
    groupId: profile.groupId
  }

  const newTerminal: TerminalInstance = {
    id: newTabId,
    ready: false
  }

  // Add tab to active panel
  activePanel.tabs.push(newTab)
  activePanel.activeTabId = newTabId
  terminals.value.push(newTerminal)

  // Switch to terminal and close drawer
  showDashboard.value = false
  showSSHDrawer.value = false

  // Request new SSH terminal from main process
  window.api?.send('terminal.createSSH', {
    terminalId: newTabId,
    profileId: profile.id
  })

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

// Saved Commands methods
const toggleSavedCommands = (): void => {
  showSavedCommands.value = !showSavedCommands.value
}

// SSH Tunnels methods
const toggleSSHTunnels = (): void => {
  showSSHTunnels.value = !showSSHTunnels.value
}

const closeSSHTunnels = (): void => {
  showSSHTunnels.value = false
}

const hideSSHTunnelsModal = (): void => {
  showSSHTunnels.value = false
}

const showSSHTunnelsModal = (): void => {
  showSSHTunnels.value = true
}

// SSH Tunnel Modal handlers
const handleCreateTunnel = (): void => {
  selectedTunnel.value = null
  showSSHTunnelModal.value = true
}

const handleEditTunnel = (tunnel: SSHTunnelWithProfile): void => {
  selectedTunnel.value = tunnel
  showSSHTunnelModal.value = true
}

const handleSaveTunnel = async (tunnelData: Partial<SSHTunnel>): Promise<void> => {
  try {
    await window.api.invoke('ssh-tunnels.create', tunnelData)
    message.success('Tunnel created successfully')
    showSSHTunnelModal.value = false
    selectedTunnel.value = null
    showSSHTunnelsModal() // Show manager again
  } catch (error) {
    console.error('Failed to create tunnel:', error)
    message.error(`Failed to create tunnel: ${error}`)
  }
}

const handleUpdateTunnel = async (id: string, tunnelData: Partial<SSHTunnel>): Promise<void> => {
  try {
    await window.api.invoke('ssh-tunnels.update', id, tunnelData)
    message.success('Tunnel updated successfully')
    showSSHTunnelModal.value = false
    selectedTunnel.value = null
    showSSHTunnelsModal() // Show manager again
  } catch (error) {
    console.error('Failed to update tunnel:', error)
    message.error(`Failed to update tunnel: ${error}`)
  }
}

const handleCloseTunnelModal = (): void => {
  showSSHTunnelModal.value = false
  selectedTunnel.value = null
  showSSHTunnelsModal() // Show manager again
}

// Sync Settings methods
const openSyncSettings = (): void => {
  showSyncSettings.value = true
}

const closeSyncSettings = (): void => {
  showSyncSettings.value = false
}

const onSyncConfigUpdated = (config: SyncConfig | null): void => {
  console.log('Sync config updated:', config)
  // Force refresh sync status in TopBar
  syncStatusRefreshCounter.value++
}

const loadSSHProfiles = async (): Promise<void> => {
  try {
    const result = await window.api.invoke('ssh-profiles.getAll')
    sshProfiles.value = result as SSHProfile[]
  } catch (error) {
    console.error('Failed to load SSH profiles:', error)
  }
}

// Auto create first tab when app starts
onMounted(() => {
  // Initialize window width tracking
  const updateWindowWidth = (): void => {
    windowWidth.value = window.innerWidth
  }
  window.addEventListener('resize', updateWindowWidth)

  // Create first tab in default panel
  addTab('panel-1')

  refreshAllData() // Use refreshAllData instead of loadSSHGroups
  loadSSHProfiles() // Load SSH profiles for tunnel modal

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

    // Find which panel this terminal belongs to
    const findPanelForTerminal = (layout: PanelLayout): string | null => {
      if (layout.type === 'panel' && layout.panel) {
        const hasTab = layout.panel.tabs.some((tab) => tab.id === data.terminalId)
        if (hasTab) return layout.panel.id
      }
      if (layout.type === 'split' && layout.children) {
        for (const child of layout.children) {
          const found = findPanelForTerminal(child)
          if (found) return found
        }
      }
      return null
    }

    const panelId = findPanelForTerminal(panelLayout.value)
    if (panelId) {
      closeTab(panelId, data.terminalId)
    }
  })

  // Store cleanup functions
  onUnmounted(() => {
    unsubscribeTitleChanged?.()
    unsubscribeAutoClose?.()
    window.removeEventListener('resize', updateWindowWidth)
    // Remove global error handler
    window.removeEventListener('unhandledrejection', () => {})
  })
})
</script>
