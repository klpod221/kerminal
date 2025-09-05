<template>
  <div ref="terminalRef" class="w-full h-full bg-[#171717] terminal-container"></div>
</template>

<script setup lang="ts">
import { onMounted, onBeforeUnmount, ref, nextTick, watch } from 'vue'

import { Terminal } from '@xterm/xterm'
import '@xterm/xterm/css/xterm.css'

import { FitAddon } from '@xterm/addon-fit'
import { SearchAddon } from '@xterm/addon-search'
import { WebLinksAddon } from '@xterm/addon-web-links'
import { Unicode11Addon } from '@xterm/addon-unicode11'
import { WebglAddon } from '@xterm/addon-webgl'

interface Props {
  terminalId?: string
  isVisible?: boolean
}

const props = withDefaults(defineProps<Props>(), {
  terminalId: 'default',
  isVisible: true
})

const emit = defineEmits<{
  'terminal-ready': [terminalId: string]
}>()

const terminalRef = ref<HTMLElement | null>(null)
let term: Terminal
let fitAddon: FitAddon
let removeListener: (() => void) | null = null

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

  // Send user input from terminal to main process
  term.onData((key) => {
    window.api.send('terminal.keystroke', { terminalId: props.terminalId, data: key })
  })

  term.onSelectionChange(() => {
    if (term.hasSelection()) {
      const selectedText = term.getSelection()
      window.api.send('copy-to-clipboard', selectedText)
    }
  })

  window.addEventListener('resize', handleResize)

  window.api.send('terminal.ready', { terminalId: props.terminalId })

  // Emit ready event to parent
  emit('terminal-ready', props.terminalId)
})

onBeforeUnmount(() => {
  if (removeListener) {
    removeListener()
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
