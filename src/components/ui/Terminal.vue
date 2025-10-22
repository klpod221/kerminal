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

    <!-- Connection Lost Overlay with Reconnect -->
    <div
      v-if="showDisconnectedOverlay"
      class="absolute inset-0 bg-[#171717]/95 flex items-center justify-center z-50"
    >
      <div class="flex flex-col items-center space-y-6 max-w-md px-4">
        <!-- Error icon -->
        <div class="relative">
          <div
            class="rounded-full h-16 w-16 border-2 border-red-500/50 bg-red-500/10 flex items-center justify-center"
          >
            <component :is="XCircle" class="h-8 w-8 text-red-400" />
          </div>
        </div>

        <!-- Message text -->
        <div class="text-center">
          <p class="text-lg font-medium text-white mb-2">Connection Lost</p>
          <p class="text-sm text-gray-400">
            The terminal connection was unexpectedly closed
          </p>
        </div>

        <!-- Action buttons -->
        <div class="flex gap-3">
          <Button
            v-if="canReconnect"
            variant="primary"
            size="md"
            :icon="RefreshCw"
            text="Reconnect"
            @click="handleReconnect"
          />
          <Button
            variant="secondary"
            size="md"
            :icon="X"
            text="Close Tab"
            @click="handleCloseTab"
          />
        </div>
      </div>
    </div>

    <!-- Error Overlay for SSH Connection Errors -->
    <div
      v-if="showErrorOverlay"
      class="absolute inset-0 bg-[#171717]/95 flex items-center justify-center z-50"
    >
      <div class="flex flex-col items-center space-y-6 max-w-lg px-4">
        <!-- Error icon -->
        <div class="relative">
          <div
            class="rounded-full h-16 w-16 border-2 border-red-500/50 bg-red-500/10 flex items-center justify-center"
          >
            <component :is="XCircle" class="h-8 w-8 text-red-400" />
          </div>
        </div>

        <!-- Message text -->
        <div class="text-center">
          <p class="text-lg font-medium text-white mb-2">Connection Failed</p>
          <p class="text-sm text-gray-400 mb-4">
            {{ formattedErrorMessage }}
          </p>
          <!-- Show additional error details if available -->
          <div
            v-if="currentTerminal?.errorMessage"
            class="text-xs text-gray-500 bg-gray-800 rounded p-2 font-mono max-w-full overflow-x-auto whitespace-pre-wrap"
          >
            {{ formattedErrorMessage }}
          </div>
        </div>

        <!-- Action buttons -->
        <div class="flex gap-3">
          <Button
            v-if="canReconnect"
            variant="primary"
            size="md"
            :icon="RefreshCw"
            text="Try Again"
            @click="handleReconnect"
          />
          <Button
            variant="secondary"
            size="md"
            :icon="X"
            text="Close Tab"
            @click="handleCloseTab"
          />
        </div>
      </div>
    </div>
  </div>
</template>

<script setup lang="ts">
import {
  onMounted,
  ref,
  nextTick,
  onBeforeUnmount,
  watch,
  computed,
} from "vue";
import { debounce, getErrorMessage } from "../../utils/helpers";
import { resizeTerminal } from "../../services/terminal";
import { TerminalBufferManager, InputBatcher } from "../../core";
import type { SimpleTerminal } from "../../core";
import { openUrl } from "@tauri-apps/plugin-opener";
import { useWorkspaceStore } from "../../stores/workspace";
import { XCircle, RefreshCw, X } from "lucide-vue-next";
import Button from "./Button.vue";

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

const workspaceStore = useWorkspaceStore();

const currentTerminal = computed(() =>
  workspaceStore.terminals.find((t) => t.id === props.terminalId),
);

const showDisconnectedOverlay = computed(
  () =>
    currentTerminal.value?.disconnectReason === "connection-lost" &&
    !currentTerminal.value?.hasError,
);

const showErrorOverlay = computed(
  () =>
    currentTerminal.value?.hasError &&
    currentTerminal.value?.errorMessage &&
    !props.isConnecting,
);

const formattedErrorMessage = computed(() => {
  const errorMsg = currentTerminal.value?.errorMessage;
  return getErrorMessage(errorMsg, "Connection error occurred");
});

const canReconnect = computed(
  () =>
    currentTerminal.value?.canReconnect && currentTerminal.value?.sshProfileId,
);

const handleReconnect = () => {
  console.log(currentTerminal);

  if (currentTerminal.value?.sshProfileId) {
    if (currentTerminal.value.hasError) {
      clearTerminal();
    }
    workspaceStore.reconnectSSH(
      props.terminalId,
      currentTerminal.value.sshProfileId,
    );
  }
};

const handleCloseTab = () => {
  const findPanelWithTab = (layout: any): string | null => {
    if (layout.type === "panel" && layout.panel) {
      const hasTab = layout.panel.tabs.some(
        (t: any) => t.id === props.terminalId,
      );
      if (hasTab) return layout.panel.id;
    } else if (layout.type === "split" && layout.children) {
      for (const child of layout.children) {
        const found = findPanelWithTab(child);
        if (found) return found;
      }
    }
    return null;
  };

  const panelId = findPanelWithTab(workspaceStore.panelLayout);
  if (panelId) {
    workspaceStore.closeTab(panelId, props.terminalId);
  }
};

const bufferManager = TerminalBufferManager.getInstance();

const inputBatcher = InputBatcher.getInstance();

const handleTerminalInput = (data: string): void => {
  if (!props.backendTerminalId) return;

  try {
    inputBatcher.batchInput(props.backendTerminalId, data);
  } catch (error) {
    console.error("Failed to batch input for terminal:", error);
  }
};

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
      console.warn("Failed to copy to clipboard:", err);
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
    console.warn("Failed to read clipboard:", err);
  }
};

const handleResize = debounce(async () => {
  if (fitAddon && props.isVisible) {
    fitAddon.fit();
    await handleTerminalResize();
  }
}, 100);

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

const writeOutput = (data: string): void => {
  if (term) {
    term.write(data);

    if (props.backendTerminalId) {
      bufferManager.saveToLocalBuffer(props.backendTerminalId, data);
    }
  }
};

const restoreBuffer = async (): Promise<boolean> => {
  if (!term || !props.backendTerminalId) return false;

  try {
    const simpleTerminal: SimpleTerminal = {
      clear: () => term.clear(),
      write: (data: string) => term.write(data),
    };
    return await bufferManager.restoreBuffer(
      props.backendTerminalId,
      simpleTerminal,
    );
  } catch (error) {
    console.error("Failed to restore buffer:", error);
    return false;
  }
};

const clearTerminal = async (): Promise<void> => {
  if (term) {
    term.clear();
  }

  if (props.backendTerminalId) {
    bufferManager.clearLocalBuffer(props.backendTerminalId);
  }
};

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
    rows: 30,
    theme: {
      background: "#171717",
      foreground: "#d4d4d4",
      cursor: "#ffffff",
    },
    allowProposedApi: true,
    rightClickSelectsWord: true,
    allowTransparency: false,
  });

  const webglAddon = new WebglAddon();
  term.loadAddon(webglAddon);

  fitAddon = new FitAddon();
  term.loadAddon(fitAddon);

  const webLinksAddon = new WebLinksAddon(
    async (event: MouseEvent, uri: string) => {
      event.preventDefault();
      try {
        await openUrl(uri);
      } catch (error) {
        console.warn(
          "Failed to open link with Tauri opener, falling back to window.open:",
          error,
        );
        window.open(uri, "_blank");
      }
    },
  );
  term.loadAddon(webLinksAddon);

  const searchAddon = new SearchAddon();
  term.loadAddon(searchAddon);

  const unicode11Addon = new Unicode11Addon();
  term.loadAddon(unicode11Addon);
  term.unicode.activeVersion = "11";

  term.attachCustomKeyEventHandler((event: KeyboardEvent): boolean => {
    if (
      event.ctrlKey &&
      event.shiftKey &&
      event.key === "C" &&
      event.type === "keydown"
    ) {
      event.preventDefault();
      handleCopy();
      return false;
    }

    if (event.metaKey && event.key === "c" && event.type === "keydown") {
      event.preventDefault();
      handleCopy();
      return false;
    }

    if (
      event.ctrlKey &&
      event.shiftKey &&
      event.key === "V" &&
      event.type === "keydown"
    ) {
      event.preventDefault();
      handlePaste();
      return false;
    }

    if (event.metaKey && event.key === "v" && event.type === "keydown") {
      event.preventDefault();
      handlePaste();
      return false;
    }

    return true;
  });

  term.open(terminalRef.value);

  term.onData((data) => {
    handleTerminalInput(data);
  });

  await nextTick();

  emit("terminal-ready", props.terminalId || "default");

  window.addEventListener("resize", handleResize);

  handleResize();
});

onBeforeUnmount(async () => {
  window.removeEventListener("resize", handleResize);

  if (props.backendTerminalId) {
    try {
      await inputBatcher.flushInput(props.backendTerminalId);
    } catch (error) {
      console.error("Failed to flush input during cleanup:", error);
    }

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
