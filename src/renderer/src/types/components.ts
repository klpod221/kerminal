import type { ComponentPublicInstance } from 'vue'
import type { TerminalInstance } from './panel'

/**
 * Props interface for TerminalManager component
 */
export interface TerminalManagerProps {
  terminals: TerminalInstance[]
  activeTerminalId?: string
}

/**
 * Interface for Terminal component reference
 */
export interface TerminalComponent extends ComponentPublicInstance {
  focus: () => void
  fitAndFocus: () => void
}

/**
 * Props interface for Terminal component
 */
export interface TerminalProps {
  terminalId?: string
  isVisible?: boolean
  isConnecting?: boolean
}

/**
 * Props interface for PanelManager component
 */
export interface PanelManagerProps {
  layout: import('./panel').PanelLayout
  terminals: TerminalInstance[]
  windowWidth: number
  activePanelId: string
}

/**
 * PanelManager component emits
 */
export interface PanelManagerEmits {
  selectTab: [panelId: string, tabId: string]
  closeTab: [panelId: string, tabId: string]
  addTab: [panelId: string]
  splitHorizontal: [panelId: string]
  splitVertical: [panelId: string]
  closePanel: [panelId: string]
  moveTab: [fromPanelId: string, toPanelId: string, tabId: string, targetTabId?: string]
  terminalReady: [terminalId: string]
  setActivePanel: [panelId: string]
  layoutUpdated: [layout: import('./panel').PanelLayout]
  duplicateTab: [panelId: string, tabId: string]
  moveTabToNewPanel: [panelId: string, tabId: string]
}

/**
 * Props interface for Panel component
 */
export interface PanelProps {
  panel: import('./panel').Panel
  terminals: TerminalInstance[]
  windowWidth: number
  isActive: boolean
}

/**
 * Panel component emits
 */
export interface PanelEmits {
  selectTab: [panelId: string, tabId: string]
  closeTab: [panelId: string, tabId: string]
  addTab: [panelId: string]
  splitHorizontal: [panelId: string]
  splitVertical: [panelId: string]
  closePanel: [panelId: string]
  moveTab: [fromPanelId: string, toPanelId: string, tabId: string, targetTabId?: string]
  terminalReady: [terminalId: string]
  panelClick: [panelId: string]
  duplicateTab: [panelId: string, tabId: string]
  moveTabToNewPanel: [panelId: string, tabId: string]
}

/**
 * Props interface for TabBar component
 */
export interface TabBarProps {
  panel: import('./panel').Panel
  windowWidth: number
  isActive: boolean
  terminals?: TerminalInstance[]
}

/**
 * TabBar component emits
 */
export interface TabBarEmits {
  selectTab: [panelId: string, tabId: string]
  closeTab: [panelId: string, tabId: string]
  addTab: [panelId: string]
  splitHorizontal: [panelId: string]
  splitVertical: [panelId: string]
  closePanel: [panelId: string]
  moveTab: [fromPanelId: string, toPanelId: string, tabId: string, targetTabId?: string]
  duplicateTab: [panelId: string, tabId: string]
  moveTabToNewPanel: [panelId: string, tabId: string]
}

/**
 * TopBar page types
 */
export type TopBarPage = 'dashboard' | 'workspace'

/**
 * TopBar modal types
 */
export type TopBarModal = 'ssh-drawer' | 'saved-commands' | 'ssh-tunnels' | 'sync-settings'

/**
 * TopBar state interface
 */
export interface TopBarState {
  currentPage: import('vue').Ref<TopBarPage>
  activeModal: import('vue').Ref<TopBarModal | null>
}

/**
 * TopBar component props
 */
export interface TopBarProps {
  topBarState: ReturnType<typeof import('../composables/useTopBarState').useTopBarState>
  syncStatusRefresh?: number
  isMasterPasswordModalOpen?: boolean
}

/**
 * SSHTunnelManager component props
 */
export interface SSHTunnelManagerProps {
  onHideManager?: () => void
  onShowManager?: () => void
}

/**
 * SavedCommandDrawer component props
 */
export interface SavedCommandDrawerProps {
  visible?: boolean
  activeTerminalId?: string
}

/**
 * SSHProfileDrawer component props
 */
export interface SSHProfileDrawerProps {
  visible?: boolean
}
