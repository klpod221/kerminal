import { ref, computed, type Ref } from 'vue'
import type {
  KeyboardShortcut,
  KeyboardShortcutAction,
  KeyboardEventHandler
} from '../types/keyboard'
import { DEFAULT_KEYBOARD_SHORTCUTS } from '../types/keyboard'

/**
 * Keyboard shortcut service for managing keyboard shortcuts
 */
class KeyboardShortcutService {
  private readonly shortcuts: Ref<KeyboardShortcut[]> = ref([...DEFAULT_KEYBOARD_SHORTCUTS])
  private readonly actionHandlers: Map<KeyboardShortcutAction, KeyboardEventHandler> = new Map()
  private isListening = false

  /**
   * Get all shortcuts
   */
  getShortcuts = computed(() => this.shortcuts.value)

  /**
   * Get shortcuts by category
   */
  getShortcutsByCategory = computed(() => {
    return this.shortcuts.value.reduce(
      (acc, shortcut) => {
        if (!acc[shortcut.category]) {
          acc[shortcut.category] = []
        }
        acc[shortcut.category].push(shortcut)
        return acc
      },
      {} as Record<string, KeyboardShortcut[]>
    )
  })

  /**
   * Register action handler
   */
  registerActionHandler(action: KeyboardShortcutAction, handler: KeyboardEventHandler): void {
    this.actionHandlers.set(action, handler)
  }

  /**
   * Unregister action handler
   */
  unregisterActionHandler(action: KeyboardShortcutAction): void {
    this.actionHandlers.delete(action)
  }

  /**
   * Start listening for keyboard events
   */
  startListening(): void {
    if (this.isListening) return

    document.addEventListener('keydown', this.handleKeyDown, true)
    this.isListening = true
  }

  /**
   * Stop listening for keyboard events
   */
  stopListening(): void {
    if (!this.isListening) return

    document.removeEventListener('keydown', this.handleKeyDown, true)
    this.isListening = false
  }

  /**
   * Handle keydown events
   */
  private readonly handleKeyDown = (event: KeyboardEvent): void => {
    // Don't handle shortcuts when typing in inputs
    const target = event.target as HTMLElement
    if (
      target.tagName === 'INPUT' ||
      target.tagName === 'TEXTAREA' ||
      target.contentEditable === 'true'
    ) {
      return
    }

    const pressedKeys = this.getEventKeys(event)
    const shortcut = this.findMatchingShortcut(pressedKeys)

    if (shortcut?.enabled) {
      const handler = this.actionHandlers.get(shortcut.action as KeyboardShortcutAction)
      if (handler) {
        // Always prevent default for Tab key combinations to avoid browser tab switching
        if (event.key === 'Tab') {
          event.preventDefault()
          event.stopPropagation()
        }

        const shouldPreventDefault = handler(event)
        if (shouldPreventDefault !== false) {
          event.preventDefault()
          event.stopPropagation()
        }
      }
    }
  }

  /**
   * Get keys from keyboard event
   */
  private getEventKeys(event: KeyboardEvent): string[] {
    const keys: string[] = []

    if (event.ctrlKey) keys.push('Ctrl')
    if (event.altKey) keys.push('Alt')
    if (event.shiftKey) keys.push('Shift')
    if (event.metaKey) keys.push('Meta')

    // Handle special keys
    const specialKeys: Record<string, string> = {
      ' ': 'Space',
      ArrowUp: 'ArrowUp',
      ArrowDown: 'ArrowDown',
      ArrowLeft: 'ArrowLeft',
      ArrowRight: 'ArrowRight',
      PageUp: 'PageUp',
      PageDown: 'PageDown',
      Home: 'Home',
      End: 'End',
      Insert: 'Insert',
      Delete: 'Delete',
      Backspace: 'Backspace',
      Tab: 'Tab',
      Enter: 'Enter',
      Escape: 'Escape',
      F1: 'F1',
      F2: 'F2',
      F3: 'F3',
      F4: 'F4',
      F5: 'F5',
      F6: 'F6',
      F7: 'F7',
      F8: 'F8',
      F9: 'F9',
      F10: 'F10',
      F11: 'F11',
      F12: 'F12',
      '-': '-',
      '=': '=',
      '\\': '\\',
      '/': '/',
      '?': '?'
    }

    const key = specialKeys[event.key] || event.key
    if (key && !['Control', 'Alt', 'Shift', 'Meta'].includes(key)) {
      keys.push(key)
    }

    return keys
  }

  /**
   * Find matching shortcut by pressed keys
   */
  private findMatchingShortcut(pressedKeys: string[]): KeyboardShortcut | undefined {
    return this.shortcuts.value.find((shortcut) => {
      if (shortcut.keys.length !== pressedKeys.length) return false
      return shortcut.keys.every((key) => pressedKeys.includes(key))
    })
  }

  /**
   * Format keys for display
   */
  formatKeys(keys: string[]): string {
    const keyMap: Record<string, string> = {
      Ctrl: '⌃',
      Alt: '⌥',
      Shift: '⇧',
      Meta: '⌘',
      ArrowUp: '↑',
      ArrowDown: '↓',
      ArrowLeft: '←',
      ArrowRight: '→',
      Space: '␣',
      Enter: '⏎',
      Tab: '⇥',
      Backspace: '⌫',
      Delete: '⌦',
      Escape: '⎋',
      '-': '-',
      '\\': '\\',
      '/': '/',
      '?': '?'
    }

    return keys.map((key) => keyMap[key] || key).join(' + ')
  }

  /**
   * Update shortcut
   */
  updateShortcut(id: string, updates: Partial<KeyboardShortcut>): void {
    const index = this.shortcuts.value.findIndex((s) => s.id === id)
    if (index !== -1) {
      this.shortcuts.value[index] = { ...this.shortcuts.value[index], ...updates }
    }
  }

  /**
   * Enable shortcut
   */
  enableShortcut(id: string): void {
    this.updateShortcut(id, { enabled: true })
  }

  /**
   * Disable shortcut
   */
  disableShortcut(id: string): void {
    this.updateShortcut(id, { enabled: false })
  }

  /**
   * Reset shortcuts to default
   */
  resetToDefaults(): void {
    this.shortcuts.value = [...DEFAULT_KEYBOARD_SHORTCUTS]
  }
}

// Create singleton instance
export const keyboardShortcutService = new KeyboardShortcutService()

/**
 * Composable for using keyboard shortcuts
 */
export function useKeyboardShortcuts(): {
  shortcuts: typeof keyboardShortcutService.getShortcuts
  shortcutsByCategory: typeof keyboardShortcutService.getShortcutsByCategory
  registerActionHandler: typeof keyboardShortcutService.registerActionHandler
  unregisterActionHandler: typeof keyboardShortcutService.unregisterActionHandler
  startListening: typeof keyboardShortcutService.startListening
  stopListening: typeof keyboardShortcutService.stopListening
  formatKeys: typeof keyboardShortcutService.formatKeys
  enableShortcut: typeof keyboardShortcutService.enableShortcut
  disableShortcut: typeof keyboardShortcutService.disableShortcut
  resetToDefaults: typeof keyboardShortcutService.resetToDefaults
} {
  return {
    shortcuts: keyboardShortcutService.getShortcuts,
    shortcutsByCategory: keyboardShortcutService.getShortcutsByCategory,
    registerActionHandler:
      keyboardShortcutService.registerActionHandler.bind(keyboardShortcutService),
    unregisterActionHandler:
      keyboardShortcutService.unregisterActionHandler.bind(keyboardShortcutService),
    startListening: keyboardShortcutService.startListening.bind(keyboardShortcutService),
    stopListening: keyboardShortcutService.stopListening.bind(keyboardShortcutService),
    formatKeys: keyboardShortcutService.formatKeys.bind(keyboardShortcutService),
    enableShortcut: keyboardShortcutService.enableShortcut.bind(keyboardShortcutService),
    disableShortcut: keyboardShortcutService.disableShortcut.bind(keyboardShortcutService),
    resetToDefaults: keyboardShortcutService.resetToDefaults.bind(keyboardShortcutService)
  }
}
