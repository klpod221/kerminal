<template>
  <div class="w-full h-full relative">
    <!-- Mount point for teleported terminal -->
    <div
      ref="mountPointRef"
      class="terminal-mount-point w-full h-full absolute inset-0"
    />
  </div>
</template>

<script setup lang="ts">
import { ref, watch, nextTick, onMounted, onBeforeUnmount } from "vue";
import { bytesToString, debounce } from "../../utils/helpers";
import type { TerminalInstance } from "../../types/panel";
import { useWorkspaceStore } from "../../stores/workspace";
import { TerminalRegistry, InputBatcher } from "../../core";

// Import Terminal component dynamically for creating instances
import { Terminal } from "@xterm/xterm";
import "@xterm/xterm/css/xterm.css";
import { FitAddon } from "@xterm/addon-fit";
import { SearchAddon } from "@xterm/addon-search";
import { WebLinksAddon } from "@xterm/addon-web-links";
import { Unicode11Addon } from "@xterm/addon-unicode11";
import { WebglAddon } from "@xterm/addon-webgl";
import { ImageAddon } from "@xterm/addon-image";
import { openUrl } from "@tauri-apps/plugin-opener";
import { writeText, readText } from "@tauri-apps/plugin-clipboard-manager";

import { getTerminalTheme } from "../../utils/terminalTheme";
import { useSettingsStore } from "../../stores/settings";

interface TerminalManagerProps {
  terminals: TerminalInstance[];
  activeTerminalId?: string;
  focusedTerminalId?: string | null;
}

const props = defineProps<TerminalManagerProps>();

const workspaceStore = useWorkspaceStore();
const settingsStore = useSettingsStore();

const mountPointRef = ref<HTMLElement | null>(null);

let outputUnlisten: (() => void) | null = null;
const inputBatcher = InputBatcher.getInstance();

// Track currently mounted terminal
const currentMountedId = ref<string | null>(null);

const emit = defineEmits(["terminal-ready"]);

/**
 * Create xterm instance for a terminal
 */
const createTerminalInstance = async (
  _terminalId: string,
  container: HTMLDivElement,
): Promise<{ term: Terminal; fitAddon: FitAddon }> => {
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

  const webglAddon = new WebglAddon();
  term.loadAddon(webglAddon);

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

  // Handle paste
  term.attachCustomKeyEventHandler((arg: KeyboardEvent): boolean => {
    if (
      (arg.ctrlKey || arg.metaKey) &&
      arg.shiftKey &&
      arg.key === "v" &&
      arg.type === "keydown"
    ) {
      (async () => {
        const clipboardText = await readText();
        if (clipboardText) {
          term.write(clipboardText);
        }
      })();
      return false;
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

  // Note: Input handler will be attached later via setInputHandler
  // after backendTerminalId is available

  return { term, fitAddon };
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
  const { term, fitAddon } = await createTerminalInstance(
    terminalId,
    container,
  );

  // Update registry with term and fitAddon
  TerminalRegistry.updateTerminalInstance(terminalId, term, fitAddon);

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

      // Update input handler if backendTerminalId is set (uses safe replacement)
      if (terminal.backendTerminalId) {
        TerminalRegistry.setInputHandler(terminal.id, (data) => {
          if (terminal.backendTerminalId) {
            inputBatcher.batchInput(terminal.backendTerminalId, data);
          }
        });
      }
    }
  },
  { deep: true },
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
        if (managed?.term) {
          const output = bytesToString(terminalData.data);
          managed.term.write(output);
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
  contain: layout size;
}

.terminal-mount-point :deep(.terminal-container) {
  width: 100%;
  height: 100%;
}

.terminal-mount-point :deep(.xterm) {
  width: 100%;
  height: 100%;
  padding: 4px;
}

.terminal-mount-point :deep(.xterm-viewport) {
  width: 100% !important;
  height: 100% !important;
}

.terminal-mount-point :deep(.xterm-screen) {
  width: 100%;
  height: 100%;
  user-select: text;
}

/* Terminal cursor blink enhancement */
.terminal-mount-point :deep(.xterm-cursor) {
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

/* Terminal selection styling */
.terminal-mount-point :deep(.xterm-selection) {
  background-color: rgba(255, 255, 255, 0.2) !important;
}
</style>
