<template>
  <div class="h-screen w-screen flex flex-col bg-[#0D0D0D] overflow-hidden">
    <TopBar
      class="flex-shrink-0"
      :top-bar-state="topBarState"
      :sync-status-refresh="syncStatusRefreshCounter"
      @open-dashboard="openDashboard"
      @open-workspace="openWorkspace"
      @toggle-ssh-drawer="toggleSSHDrawer"
      @toggle-saved-commands="toggleSavedCommands"
      @toggle-ssh-tunnels="toggleSSHTunnels"
      @open-sync-settings="openSyncSettings"
    />

    <div class="flex-grow overflow-hidden">
      <!-- Dashboard Page -->
      <Dashboard
        v-show="topBarState.isDashboardActive.value"
        class="h-full"
        @create-terminal="createTerminalFromDashboard"
        @open-ssh-profiles="toggleSSHDrawer"
      />

      <!-- Workspace with Panels -->
      <PanelManager
        v-show="topBarState.isWorkspaceActive.value"
        class="h-full"
        :layout="panelLayout"
        :terminals="terminals"
        :window-width="windowWidth"
        :active-panel-id="activePanelId"
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
      :visible="topBarState.isSSHDrawerActive.value"
      @update:visible="handleSSHDrawerVisibilityChange"
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
      :visible="topBarState.isSavedCommandsActive.value"
      :active-terminal-id="getCurrentActiveTerminalId()"
      @update:visible="handleSavedCommandsVisibilityChange"
    />

    <!-- SSH Tunnel Manager Modal -->
    <Modal
      :visible="topBarState.isSSHTunnelsActive.value"
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
      :visible="topBarState.isSyncSettingsActive.value"
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
import { ref, onMounted, onUnmounted, nextTick, watch } from 'vue'
import { Wifi } from 'lucide-vue-next'
import TopBar from './components/TopBar.vue'
import Dashboard from './components/Dashboard.vue'
import PanelManager from './components/PanelManager.vue'
import SSHProfileDrawer from './components/SSHProfileDrawer.vue'
import SSHProfileModal from './components/SSHProfileModal.vue'
import SSHGroupModal from './components/SSHGroupModal.vue'
import SavedCommandDrawer from './components/SavedCommandDrawer.vue'
import SyncSettingsModal from './components/SyncSettingsModal.vue'
import Modal from './components/ui/Modal.vue'
import SSHTunnelManager from './components/SSHTunnelManager.vue'
import SSHTunnelModal from './components/SSHTunnelModal.vue'
import { useTopBarState } from './composables/useTopBarState'
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
import { debounce } from './utils/debounce'

// Initialize TopBar state management
const topBarState = useTopBarState()

// Modal states - using topBarState for main navigation
const showSSHProfileModal = ref(false)
const showSSHGroupModal = ref(false)
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

/**
 * Triggers resize for all visible terminals (debounced)
 */
const triggerTerminalResize = debounce((): void => {
  // Use a small delay to ensure DOM has updated
  nextTick(() => {
    setTimeout(() => {
      // Trigger resize event for all terminals to adjust to new layout
      window.dispatchEvent(new Event('resize'))
    }, 50) // Reduced delay since we're already debouncing
  })
}, 150) // Debounce for 150ms to prevent excessive resize calls

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
  topBarState.setPage('dashboard')
}

const openWorkspace = (): void => {
  topBarState.setPage('workspace')
}

const createTerminalFromDashboard = (): void => {
  // Switch to workspace and create a new terminal in the first available panel
  topBarState.setPage('workspace')
  const firstPanel = findFirstPanel(panelLayout.value)
  if (firstPanel) {
    addTab(firstPanel.id)
  }
}

const setActivePanel = (panelId: string): void => {
  activePanelId.value = panelId
}

const selectTab = (panelId: string, tabId: string): void => {
  const panel = findPanelInLayout(panelLayout.value, panelId)
  if (panel) {
    panel.activeTabId = tabId
    activePanelId.value = panelId
    topBarState.setPage('workspace')
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

  // Set active panel and switch to workspace
  activePanelId.value = panelId
  topBarState.setPage('workspace')

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
        topBarState.setPage('dashboard')
      }
    }
  } else {
    // All panels closed, show dashboard
    topBarState.setPage('dashboard')
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
        topBarState.setPage('dashboard')
      }
    }
  } else {
    // All panels closed, show dashboard
    topBarState.setPage('dashboard')
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
  // Trigger terminal resize after layout update
  triggerTerminalResize()
}

// Watch for window width changes to trigger terminal resize
watch(windowWidth, () => {
  triggerTerminalResize()
})

// Watch for active panel changes to trigger terminal resize for visibility
watch(activePanelId, () => {
  // Small delay to ensure new active terminal is visible
  setTimeout(() => {
    triggerTerminalResize()
  }, 100)
})

// Watch for panel layout changes to trigger terminal resize
watch(
  panelLayout,
  () => {
    triggerTerminalResize()
  },
  { deep: true }
)

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
  topBarState.toggleModal('ssh-drawer')
}

/**
 * Generic handler for modal/drawer visibility changes
 */
const handleModalVisibilityChange = (visible: boolean): void => {
  if (!visible) {
    topBarState.closeModal()
  }
}

const handleSSHDrawerVisibilityChange = handleModalVisibilityChange

// Saved Commands methods
const toggleSavedCommands = (): void => {
  topBarState.toggleModal('saved-commands')
}

const handleSavedCommandsVisibilityChange = handleModalVisibilityChange

// SSH Tunnels methods
const toggleSSHTunnels = (): void => {
  topBarState.toggleModal('ssh-tunnels')
}

const closeSSHTunnels = (): void => {
  topBarState.closeModal()
}

const hideSSHTunnelsModal = (): void => {
  topBarState.closeModal()
}

const showSSHTunnelsModal = (): void => {
  topBarState.openModal('ssh-tunnels')
}

// Sync Settings methods
const openSyncSettings = (): void => {
  topBarState.openModal('sync-settings')
}

const closeSyncSettings = (): void => {
  topBarState.closeModal()
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
    ready: false,
    isSSHConnecting: true
  }

  // Add tab to active panel
  activePanel.tabs.push(newTab)
  activePanel.activeTabId = newTabId
  terminals.value.push(newTerminal)

  // Switch to terminal and close modals
  topBarState.setPage('workspace')
  topBarState.closeModal()

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
  topBarState.closeModal() // Close drawer when opening modal
}

const createSSHProfile = (): void => {
  editingProfile.value = null
  showSSHProfileModal.value = true
  topBarState.closeModal() // Close drawer when opening modal
}

const createSSHProfileInGroup = (group: SSHGroupWithProfiles): void => {
  editingProfile.value = null
  // Pre-select the group for the new profile
  selectedGroupForNewProfile.value = group
  showSSHProfileModal.value = true
  topBarState.closeModal() // Close drawer when opening modal
}

const saveSSHProfile = async (profileData: Partial<SSHProfile>): Promise<void> => {
  try {
    await window.api.invoke('ssh-profiles.create', profileData)
    showSSHProfileModal.value = false
    selectedGroupForNewProfile.value = null // Reset selected group
    await refreshAllData() // Refresh all data
    topBarState.openModal('ssh-drawer') // Reopen drawer after saving profile
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
    topBarState.openModal('ssh-drawer') // Reopen drawer after updating profile
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
  topBarState.openModal('ssh-drawer') // Reopen drawer when profile modal is closed
}

const editSSHGroup = (group: SSHGroupWithProfiles): void => {
  editingGroup.value = group
  showSSHGroupModal.value = true
  topBarState.closeModal() // Close drawer when opening modal
}

const deleteSSHGroup = async (group: SSHGroupWithProfiles): Promise<void> => {
  try {
    await window.api.invoke('ssh-groups.delete', group.id)
    await refreshAllData() // Refresh all data
    // Force refresh the drawer if it's open
    if (topBarState.isSSHDrawerActive.value && sshProfileDrawerRef.value) {
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
    if (topBarState.isSSHDrawerActive.value && sshProfileDrawerRef.value) {
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
  topBarState.closeModal() // Close drawer when opening group modal
  console.log('showSSHGroupModal:', showSSHGroupModal.value)
}

const handleSSHGroupModalClose = (): void => {
  showSSHGroupModal.value = false
  editingGroup.value = null
  topBarState.openModal('ssh-drawer') // Reopen drawer when group modal is closed via X or Cancel
}

const saveSSHGroup = async (
  groupData: Omit<SSHGroup, 'id' | 'created' | 'updated'>
): Promise<void> => {
  try {
    await window.api.invoke('ssh-groups.create', groupData)
    showSSHGroupModal.value = false
    await refreshAllData() // Refresh all data
    topBarState.openModal('ssh-drawer') // Reopen drawer after saving group
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
    topBarState.openModal('ssh-drawer') // Reopen drawer after updating group
    // Force refresh the drawer
    if (sshProfileDrawerRef.value) {
      sshProfileDrawerRef.value.refreshProfiles()
    }
    console.log('SSH group updated successfully')
  } catch (error) {
    console.error('Failed to update SSH group:', error)
  }
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

/**
 * Find panel containing a specific terminal ID
 */
const findPanelForTerminal = (layout: PanelLayout, terminalId: string): string | null => {
  if (layout.type === 'panel' && layout.panel) {
    const hasTab = layout.panel.tabs.some((tab) => tab.id === terminalId)
    if (hasTab) return layout.panel.id
  }
  if (layout.type === 'split' && layout.children) {
    for (const child of layout.children) {
      const found = findPanelForTerminal(child, terminalId)
      if (found) return found
    }
  }
  return null
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

  // Listen for SSH connecting state
  const unsubscribeSSHConnecting = window.api?.on(
    'terminal.sshConnecting',
    (...args: unknown[]) => {
      const data = args[0] as { terminalId: string }
      const terminal = terminals.value.find((t) => t.id === data.terminalId)
      if (terminal) {
        terminal.isSSHConnecting = true
      }
    }
  )

  // Listen for SSH connected state
  const unsubscribeSSHConnected = window.api?.on('terminal.sshConnected', (...args: unknown[]) => {
    const data = args[0] as { terminalId: string }
    const terminal = terminals.value.find((t) => t.id === data.terminalId)
    if (terminal) {
      terminal.isSSHConnecting = false
    }
  })

  // Listen for SSH connection errors
  const unsubscribeSSHError = window.api?.on('terminal.sshError', (...args: unknown[]) => {
    const data = args[0] as { terminalId: string; error: string }

    // Remove connecting state
    const terminal = terminals.value.find((t) => t.id === data.terminalId)
    if (terminal) {
      terminal.isSSHConnecting = false
    }

    // Show error message
    message.error(`SSH Connection Failed: ${data.error}`)
    console.error('SSH connection error:', data.error)

    // Auto close the tab with error
    const panelId = findPanelForTerminal(panelLayout.value, data.terminalId)
    if (panelId) {
      setTimeout(() => {
        closeTab(panelId, data.terminalId)
      }, 1000) // Give user time to see the error
    }
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
    unsubscribeSSHConnecting?.()
    unsubscribeSSHConnected?.()
    unsubscribeSSHError?.()
    window.removeEventListener('resize', updateWindowWidth)
    // Remove global error handler
    window.removeEventListener('unhandledrejection', () => {})
  })
})
</script>
