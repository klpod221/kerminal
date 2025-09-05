<template>
  <div ref="terminalRef" class="w-full h-full bg-[#171717] terminal-container relative">
    <!-- SSH Connecting Overlay -->
    <div
      v-if="isConnecting"
      class="absolute inset-0 bg-[#171717]/95 flex items-center justify-center z-50"
    >
      <div class="flex flex-col items-center space-y-4">
        <!-- Large spinning icon -->
        <div class="relative">
          <div
            class="animate-spin rounded-full h-12 w-12 border-2 border-gray-600 border-t-blue-400"
          ></div>
          <!-- Pulse effect -->
          <div
            class="absolute inset-0 animate-ping rounded-full h-12 w-12 border border-blue-400/20"
          ></div>
        </div>
        <!-- Loading text -->
        <div class="text-center">
          <p class="text-lg font-medium text-white mb-1">Connecting to SSH...</p>
          <p class="text-sm text-gray-400">Please wait while establishing connection</p>
        </div>
      </div>
    </div>
  </div>
</template>

<script setup lang="ts">
import { onMounted, onBeforeUnmount, ref, nextTick, watch } from 'vue'
import { processClipboardText } from '../utils/clipboard'

import { Terminal } from '@xterm/xterm'
import '@xterm/xterm/css/xterm.css'

import { FitAddon } from '@xterm/addon-fit'
import { SearchAddon } from '@xterm/addon-search'
import { WebLinksAddon } from '@xterm/addon-web-links'
import { Unicode11Addon } from '@xterm/addon-unicode11'
import { WebglAddon } from '@xterm/addon-webgl'
import type { TerminalProps } from '../types/components'

const props = withDefaults(defineProps<TerminalProps>(), {
  terminalId: 'default',
  isVisible: true,
  isConnecting: false
})

const emit = defineEmits<{
  'terminal-ready': [terminalId: string]
}>()

const terminalRef = ref<HTMLElement | null>(null)
let term: Terminal
let fitAddon: FitAddon
let removeListener: (() => void) | null = null
let contextMenuListener: (() => void) | null = null

/**
 * Handles paste operation by reading from clipboard and writing to terminal.
 * Includes proper sanitization and error handling.
 * @returns {Promise<void>}
 */
async function handlePaste(): Promise<void> {
  try {
    const clipboardText = await window.api.invoke('get-clipboard-text')
    if (clipboardText && typeof clipboardText === 'string' && clipboardText.trim()) {
      // Process clipboard content with comprehensive validation and sanitization
      const processedText = processClipboardText(clipboardText, {
        maxLength: 5000, // Limit paste size
        multilineConfirm: true // Log multiline pastes
      })

      if (processedText) {
        // Send the processed text to terminal
        window.api.send('terminal.keystroke', {
          terminalId: props.terminalId,
          data: processedText
        })
      } else {
        console.warn('Clipboard content was rejected by validation')
      }
    }
  } catch (error) {
    console.error('Failed to paste from clipboard:', error)
    // Silently fail - don't show error to user as paste failures are common
  }
}

/**
 * Sends the current terminal size to the main process.
 * @returns {void}
 */
function sendTerminalSize(): void {
  if (term) {
    const { cols, rows } = term
    window.api.send('terminal.resize', { terminalId: props.terminalId, cols, rows })
  }
}

/**
 * Handles window resize event and fits the terminal to the container.
 * @returns {void}
 */
function handleResize(): void {
  if (fitAddon && props.isVisible) {
    fitAddon.fit()
  }
}

/**
 * Force terminal to fit and focus
 * @returns {void}
 */
function fitAndFocus(): void {
  if (fitAddon && term && props.isVisible) {
    nextTick(() => {
      fitAddon.fit()
      term.focus()
    })
  }
}

/**
 * Focus the terminal
 * @returns {void}
 */
function focus(): void {
  if (term) {
    term.focus()
  }
}

// Watch for visibility changes to trigger resize
watch(
  () => props.isVisible,
  (newVisible) => {
    if (newVisible && term && fitAddon) {
      nextTick(() => {
        fitAndFocus()
      })
    }
  },
  { immediate: false }
)

// Expose focus method to parent
defineExpose({
  focus,
  fitAndFocus
})

onMounted(async () => {
  if (!terminalRef.value) return

  term = new Terminal({
    cursorBlink: true,
    fontFamily: "'Fira Code', monospace",
    fontSize: 14,
    lineHeight: 1,
    theme: {
      background: '#171717',
      foreground: '#d4d4d4'
    },
    allowProposedApi: true
  })

  // --- Load terminal addons ---

  // 1. WebGL (load before open for better performance)
  const webglAddon = new WebglAddon()
  term.loadAddon(webglAddon)

  // 2. Fit Addon (fit terminal to window)
  fitAddon = new FitAddon()
  term.loadAddon(fitAddon)

  // 3. Web Links Addon (enable clickable links)
  term.loadAddon(
    new WebLinksAddon((event, uri) => {
      event.preventDefault()
      window.api.send('open-external-link', uri)
    })
  )

  // 4. Search Addon (enable text search)
  const searchAddon = new SearchAddon()
  term.loadAddon(searchAddon)

  // 5. Unicode 11 Addon (support wide characters, emoji)
  const unicode11Addon = new Unicode11Addon()
  term.loadAddon(unicode11Addon)
  // Activate Unicode 11 addon after loading
  term.unicode.activeVersion = '11'

  // Open terminal in DOM
  term.open(terminalRef.value)

  // Wait for DOM to be ready then fit
  await nextTick()

  // Add a small delay for all terminals to ensure proper sizing
  setTimeout(() => {
    fitAddon.fit()
    sendTerminalSize()
    term.focus()
  }, 150)

  // Handle terminal resize events
  term.onResize(() => {
    sendTerminalSize()
  })

  // Receive data from main process and display in terminal
  removeListener = window.api.on('terminal.incomingData', (...args: unknown[]) => {
    const data = args[0] as string
    const terminalId = args[1] as string
    // Only process data for this terminal instance
    if (terminalId === props.terminalId) {
      term.write(data)
    }
  })

  // Handle context menu clicks
  contextMenuListener = window.api.on('context-menu-click', (...args: unknown[]) => {
    const action = args[0] as string
    switch (action) {
      case 'Copy':
        if (term.hasSelection()) {
          const selectedText = term.getSelection()
          if (selectedText.trim()) {
            window.api.send('copy-to-clipboard', selectedText)
          }
        }
        break
      case 'Paste':
        handlePaste()
        break
      case 'Select All':
        term.selectAll()
        break
    }
  })

  // Send user input from terminal to main process
  term.onData((key) => {
    window.api.send('terminal.keystroke', { terminalId: props.terminalId, data: key })
  })

  // Handle keyboard shortcuts and selection
  let isManualCopy = false
  let lastCopyTime = 0
  const COPY_DEBOUNCE_MS = 100 // Prevent excessive copying

  /**
   * Handles copy operation when text is selected
   * @returns {boolean} - Whether to prevent default behavior
   */
  function handleCopyKeyboard(): boolean {
    if (term.hasSelection()) {
      isManualCopy = true
      const selectedText = term.getSelection()
      if (selectedText.trim()) {
        window.api.send('copy-to-clipboard', selectedText)
        return false // Prevent default terminal behavior
      }
    }
    // If no selection, let terminal handle Ctrl+C (interrupt)
    return true
  }

  /**
   * Handles paste operation
   * @param event - Keyboard event
   * @returns {boolean} - Whether to prevent default behavior
   */
  function handlePasteKeyboard(event: KeyboardEvent): boolean {
    if (event.type === 'keydown') {
      event.preventDefault()
      handlePaste()
      return false // Prevent default terminal behavior
    }
    return true
  }

  /**
   * Handles select all operation
   * @param event - Keyboard event
   * @returns {boolean} - Whether to prevent default behavior
   */
  function handleSelectAllKeyboard(event: KeyboardEvent): boolean {
    if (event.type === 'keydown') {
      event.preventDefault()
      term.selectAll()
      return false
    }
    return true
  }

  term.onSelectionChange(() => {
    // Only auto-copy if it's a manual copy operation (like Ctrl+C)
    if (term.hasSelection() && isManualCopy) {
      const now = Date.now()
      if (now - lastCopyTime > COPY_DEBOUNCE_MS) {
        const selectedText = term.getSelection()
        if (selectedText.trim()) {
          window.api.send('copy-to-clipboard', selectedText)
          lastCopyTime = now
        }
      }
      isManualCopy = false
    }
  })

  // Handle keyboard shortcuts
  term.attachCustomKeyEventHandler((event) => {
    if (event.ctrlKey || event.metaKey) {
      switch (event.key) {
        case 'c':
        case 'C':
          return handleCopyKeyboard()
        case 'v':
        case 'V':
          return handlePasteKeyboard(event)
        case 'a':
        case 'A':
          return handleSelectAllKeyboard(event)
      }
    }
    return true // Allow other keys to be processed normally
  })

  // Handle right-click context menu
  terminalRef.value.addEventListener('contextmenu', (event) => {
    event.preventDefault()

    // Create simple context menu
    const hasSelection = term.hasSelection()
    const menu = [
      {
        label: 'Copy',
        enabled: hasSelection,
        action: () => {
          if (hasSelection) {
            const selectedText = term.getSelection()
            if (selectedText.trim()) {
              window.api.send('copy-to-clipboard', selectedText)
            }
          }
        }
      },
      {
        label: 'Paste',
        enabled: true,
        action: () => handlePaste()
      },
      {
        label: 'Select All',
        enabled: true,
        action: () => term.selectAll()
      }
    ]

    // Show native context menu via main process
    window.api.send('show-context-menu', {
      items: menu,
      x: event.clientX,
      y: event.clientY
    })
  })

  window.api.send('terminal.ready', { terminalId: props.terminalId })

  // Emit ready event to parent
  emit('terminal-ready', props.terminalId)
})

onBeforeUnmount(() => {
  if (removeListener) {
    removeListener()
  }
  if (contextMenuListener) {
    contextMenuListener()
  }
  window.removeEventListener('resize', handleResize)
  term?.dispose()
})
</script>

<style scoped>
.terminal-container {
  animation: terminalFadeIn 0.5s ease-out;
}

@keyframes terminalFadeIn {
  from {
    opacity: 0;
    transform: translateY(10px);
  }
  to {
    opacity: 1;
    transform: translateY(0);
  }
}

/* Terminal cursor blink enhancement */
:deep(.xterm-cursor) {
  animation: terminalCursor 1s infinite;
}

@keyframes terminalCursor {
  0%,
  50% {
    opacity: 1;
  }
  51%,
  100% {
    opacity: 0;
  }
}
</style>
