<template>
  <div class="w-full h-full relative">
    <Terminal
      v-for="terminal in terminals"
      :key="terminal.id"
      :ref="(el) => setTerminalRef(terminal.id, el)"
      :terminal-id="terminal.id"
      :backend-terminal-id="terminal.backendTerminalId"
      :is-connecting="terminal.isSSHConnecting || false"
      :class="{ hidden: terminal.id !== activeTerminalId }"
      class="w-full h-full absolute inset-0"
      @terminal-ready="onTerminalReady"
    />
  </div>
</template>

<script setup lang="ts">
import { ref, watch, nextTick, onMounted, onBeforeUnmount } from "vue";
import Terminal from "./Terminal.vue";
import type { ComponentPublicInstance } from "vue";
import type { TerminalInstance } from "../../types/panel";
import { listenToTerminalOutput, bytesToString } from "../../services/terminal";
import { TerminalBufferManager } from "../../utils/terminalBufferManager";

interface TerminalManagerProps {
  terminals: TerminalInstance[];
  activeTerminalId?: string;
}

interface TerminalComponent extends ComponentPublicInstance {
  focus: () => void;
  fitAndFocus: () => void;
  writeOutput: (data: string) => void;
  restoreBuffer: () => Promise<boolean>;
  clearTerminal: () => Promise<void>;
}

const props = defineProps<TerminalManagerProps>();

const terminalRefs = ref<Record<string, ComponentPublicInstance | null>>({});
let outputUnlisten: (() => void) | null = null;

// Get buffer manager instance
const bufferManager = TerminalBufferManager.getInstance();

const emit = defineEmits(["terminal-ready"]);

/**
 * Focus the active terminal
 */
const focusActiveTerminal = (): void => {
  if (props.activeTerminalId && terminalRefs.value[props.activeTerminalId]) {
    const terminalInstance = terminalRefs.value[props.activeTerminalId];
    if (terminalInstance && "fitAndFocus" in terminalInstance) {
      (terminalInstance as TerminalComponent).fitAndFocus();
    }
  }
};

/**
 * Expose methods to parent component
 */
defineExpose({
  focusActiveTerminal,
});

/**
 * Set the ref for a terminal instance.
 * Only store Vue component instances, not DOM elements.
 * @param {string} terminalId - The terminal id.
 * @param {any} el - The ref value (component instance or DOM element).
 */
const setTerminalRef = (
  terminalId: string,
  el: ComponentPublicInstance | Element | null,
): void => {
  // Check if el is a Vue component instance (has $el property)
  if (el && typeof el === "object" && "$el" in el) {
    terminalRefs.value[terminalId] = el as ComponentPublicInstance;
  } else {
    delete terminalRefs.value[terminalId];
  }
};

const onTerminalReady = async (terminalId: string): Promise<void> => {
  emit("terminal-ready", terminalId);

  // Try to restore buffer when terminal is ready
  const terminalInstance = terminalRefs.value[terminalId];
  if (terminalInstance && "restoreBuffer" in terminalInstance) {
    const matchingTerminal = props.terminals.find((t) => t.id === terminalId);
    if (matchingTerminal && matchingTerminal.backendTerminalId) {
      try {
        await (terminalInstance as TerminalComponent).restoreBuffer();
      } catch (error) {
        console.error(
          `Failed to restore buffer for terminal ${terminalId}:`,
          error,
        );
      }
    }
  }

  // Check if this terminal should be focused when ready (e.g., from split operation)
  const matchingTerminal = props.terminals.find((t) => t.id === terminalId);
  const shouldFocusOnReady =
    matchingTerminal?.shouldFocusOnReady ||
    terminalId === props.activeTerminalId;

  if (shouldFocusOnReady) {
    await nextTick();
    setTimeout(() => {
      if (terminalInstance && "fitAndFocus" in terminalInstance) {
        (terminalInstance as TerminalComponent).fitAndFocus();
      }
    }, 200); // Longer delay for new terminals to ensure proper initialization
  }
};

// Watch for active terminal changes to ensure proper focus
watch(
  () => props.activeTerminalId,
  async (newActiveId) => {
    if (newActiveId && terminalRefs.value[newActiveId]) {
      await nextTick();
      // Add a small delay to ensure the terminal is fully visible
      setTimeout(() => {
        const terminalInstance = terminalRefs.value[newActiveId];
        if (terminalInstance && "fitAndFocus" in terminalInstance) {
          (terminalInstance as TerminalComponent).fitAndFocus();
        }
      }, 100);
    }
  },
  { immediate: true },
);

// Listen to terminal output from backend
onMounted(async () => {
  try {
    // Listen to terminal output from backend
    outputUnlisten = await listenToTerminalOutput((terminalData) => {
      const backendTerminalId = terminalData.terminal_id;

      // Find the terminal instance that matches this backend terminal ID
      const matchingTerminal = props.terminals.find(
        (t) => t.backendTerminalId === backendTerminalId,
      );
      if (!matchingTerminal) return;

      const terminalRef = terminalRefs.value[matchingTerminal.id];

      if (terminalRef && "writeOutput" in terminalRef) {
        const output = bytesToString(terminalData.data);
        (terminalRef as TerminalComponent).writeOutput(output);
      }
    });

    // Listen for window focus events to auto-focus active terminal
    const handleWindowFocus = (): void => {
      // Small delay to ensure proper focus
      setTimeout(() => {
        focusActiveTerminal();
      }, 100);
    };

    window.addEventListener("focus", handleWindowFocus);

    // Store the cleanup function
    const originalUnlisten = outputUnlisten;
    outputUnlisten = () => {
      if (originalUnlisten) originalUnlisten();
      window.removeEventListener("focus", handleWindowFocus);
    };
  } catch (error) {
    console.error("Failed to listen to terminal output:", error);
  }
});

// Cleanup output listener
onBeforeUnmount(() => {
  if (outputUnlisten) {
    outputUnlisten();
  }

  // Cleanup local buffers for terminals that are being unmounted
  Object.keys(terminalRefs.value).forEach((terminalId) => {
    const matchingTerminal = props.terminals.find((t) => t.id === terminalId);
    if (matchingTerminal && matchingTerminal.backendTerminalId) {
      bufferManager.clearLocalBuffer(matchingTerminal.backendTerminalId);
    }
  });
});
</script>
