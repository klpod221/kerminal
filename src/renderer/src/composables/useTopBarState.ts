import { ref, computed, type Ref } from 'vue'

export type TopBarPage = 'dashboard' | 'workspace'
export type TopBarModal = 'ssh-drawer' | 'saved-commands' | 'ssh-tunnels' | 'sync-settings'

interface TopBarState {
  currentPage: Ref<TopBarPage>
  activeModal: Ref<TopBarModal | null>
}

/**
 * Composable for managing TopBar active states in a centralized way
 * Pages: dashboard, workspace (main content areas)
 * Modals: ssh-drawer, saved-commands, ssh-tunnels, sync-settings (overlays on top of pages)
 */
export function useTopBarState(): TopBarState & {
  // Computed states for each view
  isDashboardActive: Ref<boolean>
  isWorkspaceActive: Ref<boolean>
  isSSHDrawerActive: Ref<boolean>
  isSavedCommandsActive: Ref<boolean>
  isSSHTunnelsActive: Ref<boolean>
  isSyncSettingsActive: Ref<boolean>

  // Actions to switch views
  setPage: (page: TopBarPage) => void
  openModal: (modal: TopBarModal) => void
  closeModal: () => void
  toggleModal: (modal: TopBarModal) => void
} {
  // Internal state
  const currentPage = ref<TopBarPage>('workspace')
  const activeModal = ref<TopBarModal | null>(null)

  // Computed states for each page
  const isDashboardActive = computed(() => currentPage.value === 'dashboard')
  const isWorkspaceActive = computed(() => currentPage.value === 'workspace')

  // Computed states for each modal
  const isSSHDrawerActive = computed(() => activeModal.value === 'ssh-drawer')
  const isSavedCommandsActive = computed(() => activeModal.value === 'saved-commands')
  const isSSHTunnelsActive = computed(() => activeModal.value === 'ssh-tunnels')
  const isSyncSettingsActive = computed(() => activeModal.value === 'sync-settings')

  /**
   * Set the current page (dashboard or workspace)
   */
  const setPage = (page: TopBarPage): void => {
    currentPage.value = page
  }

  /**
   * Open a modal/drawer
   */
  const openModal = (modal: TopBarModal): void => {
    activeModal.value = modal
  }

  /**
   * Close any active modal/drawer
   */
  const closeModal = (): void => {
    activeModal.value = null
  }

  /**
   * Toggle a modal view - open if closed, close if open
   */
  const toggleModal = (modal: TopBarModal): void => {
    if (activeModal.value === modal) {
      // Currently showing this modal, close it
      closeModal()
    } else {
      // Switch to this modal
      openModal(modal)
    }
  }

  return {
    currentPage,
    activeModal,
    isDashboardActive,
    isWorkspaceActive,
    isSSHDrawerActive,
    isSavedCommandsActive,
    isSSHTunnelsActive,
    isSyncSettingsActive,
    setPage,
    openModal,
    closeModal,
    toggleModal
  }
}
