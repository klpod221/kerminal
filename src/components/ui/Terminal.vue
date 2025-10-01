<template>
  <div
    ref="terminalRef"
    class="w-full h-full bg-[#171717] terminal-container relative"
  >
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
          <p class="text-lg font-medium text-white mb-1">
            Connecting to SSH...
          </p>
          <p class="text-sm text-gray-400">
            Please wait while establishing connection
          </p>
        </div>
      </div>
    </div>
  </div>
</template>

<script setup lang="ts">
import { onMounted, ref, nextTick, onBeforeUnmount, watch } from "vue";
import { debounce } from "../../utils/helpers";
import { resizeTerminal } from "../../services/terminal";
import { TerminalBufferManager, InputBatcher } from "../../core";
import type { SimpleTerminal } from "../../core";
import { openUrl } from '@tauri-apps/plugin-opener';

import { Terminal } from "@xterm/xterm";
import "@xterm/xterm/css/xterm.css";

import { FitAddon } from "@xterm/addon-fit";
import { SearchAddon } from "@xterm/addon-search";
import { WebLinksAddon } from "@xterm/addon-web-links";
import { Unicode11Addon } from "@xterm/addon-unicode11";
import { WebglAddon } from "@xterm/addon-webgl";

interface TerminalProps {
  terminalId?: string;
  backendTerminalId?: string;
  isVisible?: boolean;
  isConnecting?: boolean;
}

const props = withDefaults(defineProps<TerminalProps>(), {
  terminalId: "default",
  backendTerminalId: "",
  isVisible: true,
  isConnecting: false,
});

const emit = defineEmits<{
  "terminal-ready": [terminalId: string];
  "terminal-output": [terminalId: string, data: string];
}>();

const terminalRef = ref<HTMLElement | null>(null);
let term: Terminal;
let fitAddon: FitAddon;

// Get buffer manager instance
const bufferManager = TerminalBufferManager.getInstance();

// Get input batcher instance
const inputBatcher = InputBatcher.getInstance();

// Handle terminal input using batching for better performance
const handleTerminalInput = (data: string): void => {
  if (!props.backendTerminalId) return;

  try {
    // Use InputBatcher to batch rapid input and reduce API calls
    inputBatcher.batchInput(props.backendTerminalId, data);
  } catch (error) {
    console.error("Failed to batch input for terminal:", error);
  }
};

// Handle terminal resize and notify backend
const handleTerminalResize = async (): Promise<void> => {
  if (!fitAddon || !props.backendTerminalId) return;

  try {
    const dimensions = fitAddon.proposeDimensions();
    if (dimensions) {
      await resizeTerminal({
        terminalId: props.backendTerminalId,
        cols: dimensions.cols,
        rows: dimensions.rows,
      });
    }
  } catch (error) {
    console.error("Failed to resize terminal:", error);
  }
};

const handleCopy = async (): Promise<void> => {
  if (term && term.hasSelection()) {
    const selection = term.getSelection();
    try {
      await navigator.clipboard.writeText(selection);
    } catch (err) {
      console.warn('Failed to copy to clipboard:', err);
    }
  }
};

const handlePaste = async (): Promise<void> => {
  try {
    const text = await navigator.clipboard.readText();
    if (term) {
      term.paste(text);
    }
  } catch (err) {
    console.warn('Failed to read clipboard:', err);
  }
};

const handleResize = debounce(async () => {
  if (fitAddon && props.isVisible) {
    fitAddon.fit();
    await handleTerminalResize();
  }
}, 100);

// Expose methods for parent components
const focus = (): void => {
  if (term) {
    term.focus();
  }
};

const fitAndFocus = debounce((): void => {
  if (fitAddon && term && props.isVisible) {
    fitAddon.fit();
    term.focus();
    handleTerminalResize();
  }
}, 50);

// Method to write output to terminal (called from parent)
const writeOutput = (data: string): void => {
  if (term) {
    term.write(data);

    // Save output to local buffer for immediate access
    if (props.backendTerminalId) {
      bufferManager.saveToLocalBuffer(props.backendTerminalId, data);
    }
  }
};

// Method to restore buffer from backend
const restoreBuffer = async (): Promise<boolean> => {
  if (!term || !props.backendTerminalId) return false;

  try {
    const simpleTerminal: SimpleTerminal = {
      clear: () => term.clear(),
      write: (data: string) => term.write(data),
    };
    return await bufferManager.restoreBuffer(props.backendTerminalId, simpleTerminal);
  } catch (error) {
    console.error("Failed to restore buffer:", error);
    return false;
  }
};

// Method to clear terminal and buffer
const clearTerminal = async (): Promise<void> => {
  if (term) {
    term.clear();
  }

  if (props.backendTerminalId) {
    bufferManager.clearLocalBuffer(props.backendTerminalId);
  }
};

// Watch for visibility changes
watch(
  () => props.isVisible,
  (newVisible) => {
    if (newVisible && term && fitAddon) {
      nextTick(() => {
        fitAndFocus();
      });
    }
  },
);

// Expose methods to parent component
defineExpose({
  focus,
  fitAndFocus,
  writeOutput,
  restoreBuffer,
  clearTerminal,
});

onMounted(async () => {
  if (!terminalRef.value) return;

  term = new Terminal({
    cursorBlink: true,
    fontFamily: "'Fira Code', monospace",
    fontSize: 14,
    lineHeight: 1,
    cols: 110,
    rows: 28,
    theme: {
      background: "#171717",
      foreground: "#d4d4d4",
      cursor: "#ffffff",
    },
    allowProposedApi: true,
    // Enable right-click context menu for copy/paste
    rightClickSelectsWord: true,
    // Allow clipboard operations
    allowTransparency: false,
  });

  // --- Load terminal addons ---

  // 1. WebGL (load before open for better performance)
  const webglAddon = new WebglAddon();
  term.loadAddon(webglAddon);

  // 2. Fit Addon (fit terminal to window)
  fitAddon = new FitAddon();
  term.loadAddon(fitAddon);

  // 3. Web Links Addon (enable clickable links with proper handler)
  const webLinksAddon = new WebLinksAddon(async (event: MouseEvent, uri: string) => {
    event.preventDefault();
    try {
      // Use Tauri's opener plugin for better security in desktop app
      await openUrl(uri);
    } catch (error) {
      console.warn('Failed to open link with Tauri opener, falling back to window.open:', error);
      // Fallback to window.open if Tauri opener fails
      window.open(uri, '_blank');
    }
  });
  term.loadAddon(webLinksAddon);

  // 4. Search Addon (enable text search)
  const searchAddon = new SearchAddon();
  term.loadAddon(searchAddon);

  // 5. Unicode 11 Addon (support wide characters, emoji)
  const unicode11Addon = new Unicode11Addon();
  term.loadAddon(unicode11Addon);
  // Activate Unicode 11 addon after loading
  term.unicode.activeVersion = "11";

  // Add custom keyboard shortcuts for copy/paste
  term.attachCustomKeyEventHandler((event: KeyboardEvent): boolean => {
    // Ctrl+Shift+C for copy
    if (event.ctrlKey && event.shiftKey && event.key === 'C' && event.type === 'keydown') {
      event.preventDefault();
      handleCopy();
      return false;
    }

    // CMD+C for copy on Mac
    if (event.metaKey && event.key === 'c' && event.type === 'keydown') {
      event.preventDefault();
      handleCopy();
      return false;
    }

    // Ctrl+Shift+V for paste
    if (event.ctrlKey && event.shiftKey && event.key === 'V' && event.type === 'keydown') {
      event.preventDefault();
      handlePaste();
      return false;
    }

    // CMD+V for paste on Mac
    if (event.metaKey && event.key === 'v' && event.type === 'keydown') {
      event.preventDefault();
      handlePaste();
      return false;
    }

    return true;
  });

  // Open terminal in DOM
  term.open(terminalRef.value);

  // Handle user input and send to backend via InputBatcher
  term.onData((data) => {
    handleTerminalInput(data);
  });

  // Wait for DOM to be ready
  await nextTick();

  // Notify parent that terminal is ready
  emit("terminal-ready", props.terminalId || "default");

  // Handle window resize
  window.addEventListener("resize", handleResize);

  // Initial fit
  handleResize();
});

// Cleanup on unmount
onBeforeUnmount(async () => {
  window.removeEventListener("resize", handleResize);

  // Flush any pending input before cleanup
  if (props.backendTerminalId) {
    try {
      await inputBatcher.flushInput(props.backendTerminalId);
    } catch (error) {
      console.error("Failed to flush input during cleanup:", error);
    }

    // Clear batcher data for this terminal
    inputBatcher.clearTerminal(props.backendTerminalId);
  }

  if (term) {
    term.dispose();
  }
});
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

/* Context menu styles */
:deep(.terminal-context-menu) {
  background: #2d2d2d;
  border: 1px solid #404040;
  border-radius: 6px;
  box-shadow: 0 4px 12px rgba(0, 0, 0, 0.3);
  padding: 4px 0;
  min-width: 120px;
  z-index: 1000;
}

:deep(.terminal-context-menu-item) {
  padding: 8px 12px;
  font-size: 13px;
  color: #d4d4d4;
  cursor: pointer;
  transition: background-color 0.1s ease;
}

:deep(.terminal-context-menu-item:hover) {
  background-color: #404040;
}

:deep(.terminal-context-menu-item:active) {
  background-color: #505050;
}

/* Terminal selection styling */
:deep(.xterm-selection) {
  background-color: rgba(255, 255, 255, 0.2) !important;
}

/* Ensure terminal text is selectable */
:deep(.xterm-screen) {
  user-select: text;
}
</style>
