/**
 * Keyboard shortcut types
 */
export interface KeyboardShortcut {
  id: string
  name: string
  description: string
  keys: string[]
  action: string
  category: KeyboardShortcutCategory
  enabled: boolean
}

/**
 * Keyboard shortcut categories
 */
export type KeyboardShortcutCategory =
  | 'terminal'
  | 'panel'
  | 'navigation'
  | 'window'
  | 'general'
  | 'workspace'

/**
 * Keyboard event handler type
 */
export type KeyboardEventHandler = (event: KeyboardEvent) => void | boolean

/**
 * Keyboard shortcut action type
 */
export type KeyboardShortcutAction =
  | 'terminal:close-tab'
  | 'terminal:new-tab'
  | 'general:show-shortcuts'
  | 'general:toggle-ssh-drawer'
  | 'general:toggle-saved-commands'
  | 'general:show-dashboard'
  | 'general:show-workspace'

/**
 * Default keyboard shortcuts configuration
 */
export const DEFAULT_KEYBOARD_SHORTCUTS: KeyboardShortcut[] = [
  // Terminal shortcuts
  {
    id: 'terminal-close-tab',
    name: 'Close Terminal Tab',
    description: 'Close the current terminal tab',
    keys: ['Ctrl', 'w'],
    action: 'terminal:close-tab',
    category: 'terminal',
    enabled: true
  },
  {
    id: 'terminal-new-tab',
    name: 'New Terminal Tab',
    description: 'Create a new terminal tab',
    keys: ['Ctrl', 't'],
    action: 'terminal:new-tab',
    category: 'terminal',
    enabled: true
  },

  // General shortcuts
  {
    id: 'general-show-shortcuts',
    name: 'Show Keyboard Shortcuts',
    description: 'Display the keyboard shortcuts modal',
    keys: ['Ctrl', 'Shift', '?'],
    action: 'general:show-shortcuts',
    category: 'general',
    enabled: true
  },
  {
    id: 'general-toggle-ssh-drawer',
    name: 'Toggle SSH Drawer',
    description: 'Open or close the SSH profiles drawer',
    keys: ['Ctrl', 'Shift', 'S'],
    action: 'general:toggle-ssh-drawer',
    category: 'general',
    enabled: true
  },
  {
    id: 'general-toggle-saved-commands',
    name: 'Toggle Saved Commands',
    description: 'Open or close the saved commands drawer',
    keys: ['Ctrl', 'Shift', 'C'],
    action: 'general:toggle-saved-commands',
    category: 'general',
    enabled: true
  },
  {
    id: 'general-show-dashboard',
    name: 'Show Dashboard',
    description: 'Open the dashboard',
    keys: ['Ctrl', 'Shift', 'D'],
    action: 'general:show-dashboard',
    category: 'general',
    enabled: true
  },
  {
    id: 'general-show-workspace',
    name: 'Show Workspace',
    description: 'Switch to workspace view',
    keys: ['Ctrl', 'Shift', 'W'],
    action: 'general:show-workspace',
    category: 'general',
    enabled: true
  }
]

/**
 * Keyboard shortcut category labels
 */
export const KEYBOARD_SHORTCUT_CATEGORY_LABELS: Record<KeyboardShortcutCategory, string> = {
  terminal: 'Terminal',
  panel: 'Panel',
  navigation: 'Navigation',
  window: 'Window',
  general: 'General',
  workspace: 'Workspace'
}
