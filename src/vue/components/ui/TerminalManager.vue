<!--
  - Copyright (C) 2026 Bùi Thanh Xuân (klpod221)
  - SPDX-License-Identifier: GPL-3.0-or-later
-->

<template>
  <div class="w-full h-full relative">
    <!-- Mount point for teleported terminal -->
    <div
      ref="mountPointRef"
      class="terminal-mount-point w-full h-full absolute inset-0"
    />

    <!-- Latency Badge -->
    <div
      v-if="
        currentTerminal?.latency !== undefined &&
        !isConnecting &&
        !showDisconnectedOverlay &&
        !showErrorOverlay
      "
      class="absolute top-2 right-2 z-10 bg-black/50 backdrop-blur-sm rounded-md px-2 py-1 flex items-center gap-1.5 border border-white/10 transition-all duration-300 hover:bg-black/70 group select-none pointer-events-none"
    >
      <Wifi :size="14" :class="getLatencyColor(currentTerminal.latency)" />
      <span
        class="text-xs font-mono font-medium"
        :class="getLatencyColor(currentTerminal.latency)"
      >
        {{ currentTerminal.latency }}ms
      </span>
    </div>

    <!-- SSH Connecting Overlay -->
    <div
      v-if="isConnecting"
      class="absolute inset-0 bg-bg-secondary/95 flex items-center justify-center z-50"
    >
      <div class="flex flex-col items-center space-y-4">
        <div class="relative">
          <div
            class="animate-spin rounded-full h-12 w-12 border-2 border-gray-600 border-t-blue-400"
          ></div>
          <div
            class="absolute inset-0 animate-ping rounded-full h-12 w-12 border border-blue-400/20"
          ></div>
        </div>
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
      class="absolute inset-0 bg-bg-secondary/95 flex items-center justify-center z-50"
    >
      <div class="flex flex-col items-center space-y-6 max-w-md px-4">
        <div class="relative">
          <div
            class="rounded-full h-16 w-16 border-2 border-red-500/50 bg-red-500/10 flex items-center justify-center"
          >
            <XCircle class="h-8 w-8 text-red-400" />
          </div>
        </div>
        <div class="text-center">
          <p class="text-lg font-medium text-white mb-2">
            {{ disconnectMessage.title }}
          </p>
          <p class="text-sm text-gray-400">
            {{ disconnectMessage.message }}
          </p>
        </div>
        <div class="flex gap-3">
          <Button
            v-if="canReconnect"
            variant="primary"
            size="md"
            :icon="RefreshCw"
            @click="handleReconnect"
          >
            Reconnect
          </Button>
          <Button
            variant="secondary"
            size="md"
            :icon="X"
            @click="handleCloseTab"
          >
            Close Tab
          </Button>
        </div>
      </div>
    </div>

    <!-- Error Overlay for SSH Connection Errors -->
    <div
      v-if="showErrorOverlay"
      class="absolute inset-0 bg-bg-secondary/95 flex items-center justify-center z-50"
    >
      <div class="flex flex-col items-center space-y-6 max-w-lg px-4">
        <div class="relative">
          <div
            class="rounded-full h-16 w-16 border-2 border-red-500/50 bg-red-500/10 flex items-center justify-center"
          >
            <XCircle class="h-8 w-8 text-red-400" />
          </div>
        </div>
        <div class="text-center">
          <p class="text-lg font-medium text-white mb-2">Connection Failed</p>
          <p class="text-sm text-gray-400 mb-4">
            {{ formattedErrorMessage }}
          </p>
        </div>
        <div class="flex gap-3">
          <Button
            v-if="canReconnect"
            variant="primary"
            size="md"
            :icon="RefreshCw"
            @click="handleReconnect"
          >
            Reconnect
          </Button>
          <Button
            variant="secondary"
            size="md"
            :icon="X"
            @click="handleCloseTab"
          >
            Close Tab
          </Button>
        </div>
      </div>
    </div>
  </div>
</template>

<script setup lang="ts">
import { ref, computed, watch, nextTick, onMounted, onBeforeUnmount } from "vue";
import { Wifi, XCircle, RefreshCw, X } from "lucide-vue-next";
import Button from "./Button.vue";
import { extractErrorMessage } from "../../utils/errorHandler";
import { debounce } from "../../utils/helpers";
import type { TerminalInstance } from "../../types/panel";
import { useWorkspaceStore } from "../../stores/workspace";
import { TerminalRegistry, createTerminalOutputWriter, createTerminalRendererHealthWatchdog } from "../../core";
import { getTerminalMetrics } from "../../services/terminal";

(window as any).getTerminalMetrics = async (id: string) => {
  try { return await getTerminalMetrics(id); } catch (err) { console.error('Metrics error:', err); }
};

// Import Terminal component dynamically for creating instances
import { Terminal } from "@xterm/xterm";
import "@xterm/xterm/css/xterm.css";
import { FitAddon } from "@xterm/addon-fit";
import { SearchAddon } from "@xterm/addon-search";
import { WebLinksAddon } from "@xterm/addon-web-links";
import { Unicode11Addon } from "@xterm/addon-unicode11";
import { ImageAddon } from "@xterm/addon-image";
import { openUrl } from "@tauri-apps/plugin-opener";
import { writeText, readText } from "@tauri-apps/plugin-clipboard-manager";

import { getTerminalTheme } from "../../utils/terminalTheme";
import { loadCanvasRenderer } from "../../utils/terminalRenderer";
import { useSettingsStore } from "../../stores/settings";

interface TerminalManagerProps {
  terminals: TerminalInstance[];
  activeTerminalId?: string;
  focusedTerminalId?: string | null;
}

const props = defineProps<TerminalManagerProps>();

const workspaceStore = useWorkspaceStore();
const settingsStore = useSettingsStore();

import { useTerminalInputSecurity } from "../../composables/terminal/useTerminalInputSecurity";
import { useBroadcastStore } from "../../stores/broadcast";
const { handleTerminalInput } = useTerminalInputSecurity();
const broadcastStore = useBroadcastStore();

const mountPointRef = ref<HTMLElement | null>(null);

let outputUnlisten: (() => void) | null = null;

// Track currently mounted terminal
const currentMountedId = ref<string | null>(null);

const currentTerminal = computed(() =>
  props.terminals.find((t) => t.id === props.activeTerminalId),
);

const isConnecting = computed(() => !!currentTerminal.value?.isSSHConnecting);

const showDisconnectedOverlay = computed(
  () =>
    (currentTerminal.value?.disconnectReason === "connection-lost" ||
      currentTerminal.value?.disconnectReason === "server-disconnect" ||
      currentTerminal.value?.disconnectReason === "connection-error") &&
    !currentTerminal.value?.hasError,
);

const disconnectMessage = computed(() => {
  switch (currentTerminal.value?.disconnectReason) {
    case "server-disconnect":
      return {
        title: "Server Disconnected",
        message: "The server has closed the connection",
      };
    case "connection-error":
      return {
        title: "Connection Error",
        message: "Connection timeout - No response from server",
      };
    case "connection-lost":
    default:
      return {
        title: "Connection Lost",
        message: "The terminal connection was unexpectedly closed",
      };
  }
});

const showErrorOverlay = computed(
  () =>
    Boolean(
      currentTerminal.value?.hasError &&
        currentTerminal.value?.errorMessage &&
        !isConnecting.value,
    ),
);

const formattedErrorMessage = computed(() => {
  const errorMsg = currentTerminal.value?.errorMessage;
  return errorMsg ? extractErrorMessage(errorMsg) : "";
});

const canReconnect = computed(
  () =>
    Boolean(
      currentTerminal.value?.canReconnect &&
        (currentTerminal.value?.sshProfileId ||
          currentTerminal.value?.sshConfigHost),
    ),
);

const getLatencyColor = (latency: number) => {
  if (latency < 100) return "text-green-400";
  if (latency < 300) return "text-yellow-400";
  return "text-red-400";
};

const handleReconnect = () => {
  if (!currentTerminal.value || !props.activeTerminalId) return;

  if (currentTerminal.value.hasError) {
    const managed = TerminalRegistry.getTerminal(props.activeTerminalId);
    managed?.term?.clear();
  }

  if (currentTerminal.value.sshProfileId) {
    workspaceStore.reconnectSSH(
      props.activeTerminalId,
      currentTerminal.value.sshProfileId,
    );
  } else if (currentTerminal.value.sshConfigHost) {
    workspaceStore.reconnectSSHConfig(
      props.activeTerminalId,
      currentTerminal.value.sshConfigHost,
      currentTerminal.value.sshConfigPassword,
    );
  }
};

const handleCloseTab = () => {
  if (props.activeTerminalId) {
    workspaceStore.closeTerminal(props.activeTerminalId);
  }
};

const emit = defineEmits(["terminal-ready"]);

/**
 * Create xterm instance for a terminal
 */
const createTerminalInstance = async (
  _terminalId: string,
  container: HTMLDivElement,
): Promise<{
  term: Terminal;
  fitAddon: FitAddon;
  outputWriter: any;
}> => {
  const customTheme = settingsStore.getCustomTheme(settingsStore.terminalTheme);
  const theme = customTheme
    ? customTheme.colors
    : getTerminalTheme(settingsStore.terminalTheme as any);

  const term = new Terminal({
    allowProposedApi: true,
    allowTransparency: false,
    rightClickSelectsWord: true,
    altClickMovesCursor: true,
    scrollback: 10000,
    customGlyphs: true,
    cursorBlink: true,
    cols: 110,
    rows: 30,
    fontFamily: `'${settingsStore.fontFamily}', monospace`,
    fontSize: settingsStore.fontSize,
    theme: theme,
  });

  const canvasAddon = await loadCanvasRenderer(term);
  if (canvasAddon) {
    createTerminalRendererHealthWatchdog({
      container: container,
      renderer: {
        backend: "canvas",
        getTrackedCanvases: () => Array.from(container.querySelectorAll("canvas")),
        reportHealth: (signal) => {
          if (signal !== "healthy") {
            try { canvasAddon.dispose(); } catch (e) {}
          }
        }
      }
    });
  }

  const fitAddon = new FitAddon();
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

  const imageAddon = new ImageAddon({
    sixelSupport: true,
    sixelScrolling: true,
    sixelPaletteLimit: 256,
  });
  term.loadAddon(imageAddon);

  term.open(container);

  // Handle custom key events
  term.attachCustomKeyEventHandler((arg: KeyboardEvent): boolean => {
    if (
      (arg.ctrlKey || arg.metaKey) &&
      arg.shiftKey &&
      arg.type === "keydown"
    ) {
      if (arg.key === "v") {
        (async () => {
          const clipboardText = await readText();
          if (clipboardText) {
            term.paste(clipboardText);
          }
        })();
        return false;
      }
      if (arg.key === "b" || arg.key === "B") {
        broadcastStore.toggleBroadcast();
        return false;
      }
    }
    return true;
  });

  // Handle selection copy
  term.onSelectionChange(async () => {
    if (term.hasSelection()) {
      const selectedText = term.getSelection();
      await writeText(selectedText);
    }
  });

  // Initialize AdaptiveOutputWriter
  const outputWriter = createTerminalOutputWriter(term, {
    adaptive: true,
    cadence: "focused",
  });
  outputWriter.setTuiCursorProtection(true);

  return { term, fitAddon, outputWriter };
};

/**
 * Ensure terminal exists in registry and create if needed
 */
const ensureTerminalExists = async (terminalId: string): Promise<void> => {
  if (TerminalRegistry.hasTerminal(terminalId)) {
    return;
  }

  // Create container for terminal
  const container = document.createElement("div");
  container.className = "terminal-container w-full h-full bg-bg-secondary";
  container.dataset.terminalId = terminalId;

  // Register before creating xterm (container first)
  TerminalRegistry.registerTerminal(terminalId, container);

  // Create xterm instance
  const { term, fitAddon, outputWriter } = await createTerminalInstance(
    terminalId,
    container,
  );

  // Update registry with term and fitAddon
  TerminalRegistry.updateTerminalInstance(
    terminalId,
    term,
    fitAddon,
    outputWriter,
  );

  // Attach input handler
  TerminalRegistry.setInputHandler(terminalId, (data) => {
    handleTerminalInput(terminalId, data, {
      activeTerminalId: props.activeTerminalId || terminalId,
      panels: workspaceStore.getAllPanels(),
      allTerminals: workspaceStore.terminals,
    });
  });

  // Note: Fit will be done after mounting to panel (hidden host has no dimensions)

  // Emit ready event
  emit("terminal-ready", terminalId);
};

/**
 * Mount active terminal to this panel
 */
const mountActiveTerminal = async (): Promise<void> => {
  const activeId = props.activeTerminalId;
  if (!activeId || !mountPointRef.value) return;

  // Unmount previous if different
  if (currentMountedId.value && currentMountedId.value !== activeId) {
    TerminalRegistry.unmountFromPanel(currentMountedId.value);
  }

  // Ensure terminal exists
  await ensureTerminalExists(activeId);

  // Mount to this panel
  TerminalRegistry.mountToPanel(activeId, mountPointRef.value);
  currentMountedId.value = activeId;

  await nextTick();

  const managed = TerminalRegistry.getTerminal(activeId);
  if (!managed) return;

  const performFitAndResize = (): void => {
    TerminalRegistry.fitTerminal(activeId);

    const terminal = props.terminals.find((t) => t.id === activeId);
    if (managed.fitAddon && terminal?.backendTerminalId) {
      const dimensions = managed.fitAddon.proposeDimensions();
      if (dimensions) {
        workspaceStore.resizeTerminal({
          terminalId: terminal.backendTerminalId,
          cols: dimensions.cols,
          rows: dimensions.rows,
        });
      }
    }
  };

  // Fit multiple times with delays to handle slow layouts
  performFitAndResize();
  requestAnimationFrame(performFitAndResize);
  [50, 150, 300].forEach((delay) => setTimeout(performFitAndResize, delay));
};

/**
 * Focus the active terminal
 */
const focusActiveTerminal = (): void => {
  if (!props.activeTerminalId) return;

  const managed = TerminalRegistry.getTerminal(props.activeTerminalId);
  if (managed?.term) {
    managed.term.focus();
    managed.fitAddon?.fit();
  }
};

/**
 * Expose methods to parent component
 */
defineExpose({
  focusActiveTerminal,
});

// Handle resize
const handleResize = debounce(() => {
  if (props.activeTerminalId) {
    TerminalRegistry.fitTerminal(props.activeTerminalId);

    const managed = TerminalRegistry.getTerminal(props.activeTerminalId);
    const terminal = props.terminals.find(
      (t) => t.id === props.activeTerminalId,
    );
    if (managed?.fitAddon && terminal?.backendTerminalId) {
      const dimensions = managed.fitAddon.proposeDimensions();
      if (dimensions) {
        workspaceStore.resizeTerminal({
          terminalId: terminal.backendTerminalId,
          cols: dimensions.cols,
          rows: dimensions.rows,
        });
      }
    }
  }
}, 100);

// Watch for active terminal changes
watch(
  () => props.activeTerminalId,
  async (newId, oldId) => {
    if (newId !== oldId && newId) {
      await mountActiveTerminal();
      await nextTick();
      setTimeout(focusActiveTerminal, 100);
    }
  },
  { immediate: true },
);

// Watch for terminals array changes (new terminals added)
watch(
  () => props.terminals,
  async (newTerminals) => {
    // Ensure all terminals exist in registry
    for (const terminal of newTerminals) {
      if (!TerminalRegistry.hasTerminal(terminal.id)) {
        await ensureTerminalExists(terminal.id);
      }

      // Update input handler (uses safe replacement)
      TerminalRegistry.setInputHandler(terminal.id, (data) => {
        handleTerminalInput(terminal.id, data, {
          activeTerminalId: props.activeTerminalId || terminal.id,
          panels: workspaceStore.getAllPanels(),
          allTerminals: workspaceStore.terminals,
        });
      });
    }
  },
  { deep: true, immediate: true },
);

// Watch for settings changes
watch(
  () => settingsStore.terminalTheme,
  (newTheme) => {
    props.terminals.forEach((terminal) => {
      const managed = TerminalRegistry.getTerminal(terminal.id);
      if (managed?.term) {
        const customTheme = settingsStore.getCustomTheme(newTheme);
        const theme = customTheme
          ? customTheme.colors
          : getTerminalTheme(newTheme as any);
        managed.term.options.theme = theme;
      }
    });
  },
);

watch(
  () => settingsStore.fontFamily,
  (newFont) => {
    props.terminals.forEach((terminal) => {
      const managed = TerminalRegistry.getTerminal(terminal.id);
      if (managed?.term) {
        managed.term.options.fontFamily = `'${newFont}', monospace`;
        managed.fitAddon?.fit();
      }
    });
  },
);

watch(
  () => settingsStore.fontSize,
  (newSize) => {
    props.terminals.forEach((terminal) => {
      const managed = TerminalRegistry.getTerminal(terminal.id);
      if (managed?.term) {
        managed.term.options.fontSize = newSize;
        managed.fitAddon?.fit();
      }
    });
  },
);

onMounted(async () => {
  try {
    // Listen to terminal output
    outputUnlisten = await workspaceStore.listenToTerminalOutput(
      (terminalData) => {
        const backendTerminalId = terminalData.terminalId;

        const matchingTerminal = props.terminals.find(
          (t) => t.backendTerminalId === backendTerminalId,
        );
        if (!matchingTerminal) return;

        const managed = TerminalRegistry.getTerminal(matchingTerminal.id);
        if (managed?.outputWriter) {
          const strData = typeof terminalData.data === 'string' ? terminalData.data : new TextDecoder().decode(terminalData.data);
          managed.outputWriter.write(strData);
        }
      },
    );

    // Handle window focus
    const handleWindowFocus = (): void => {
      setTimeout(focusActiveTerminal, 100);
    };

    window.addEventListener("focus", handleWindowFocus);
    window.addEventListener("resize", handleResize);

    // Update unlisten to include cleanup
    const originalUnlisten = outputUnlisten;
    outputUnlisten = () => {
      if (originalUnlisten) originalUnlisten();
      window.removeEventListener("focus", handleWindowFocus);
      window.removeEventListener("resize", handleResize);
    };

    // Mount initial terminal if any
    await mountActiveTerminal();
  } catch (error) {
    console.error("Failed to initialize TerminalManager:", error);
  }
});

onBeforeUnmount(() => {
  // Unmount current terminal (move back to host)
  if (currentMountedId.value) {
    TerminalRegistry.unmountFromPanel(currentMountedId.value);
  }

  if (outputUnlisten) {
    outputUnlisten();
  }
});
</script>

<style scoped>
.terminal-mount-point {
  contain: strict;
  transform: translate3d(0, 0, 0);
  will-change: transform;
  isolation: isolate;
}
.terminal-mount-point :deep(.terminal-container) {
  width: 100%;
  height: 100%;
  transform: translate3d(0, 0, 0);
}
.terminal-mount-point :deep(.xterm) { width: 100%; height: 100%; padding: 4px; }
.terminal-mount-point :deep(.xterm-viewport) { width: 100% !important; height: 100% !important; }
.terminal-mount-point :deep(.xterm-selection) { background-color: rgba(255, 255, 255, 0.2) !important; }
</style>
