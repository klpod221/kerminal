<template>
  <div class="w-full h-full relative">
    <Terminal
      v-for="terminal in terminals"
      :key="terminal.id"
      v-show="terminal.id === activeTerminalId"
      :ref="(el) => setTerminalRef(terminal.id, el)"
      :terminal-id="terminal.id"
      :backend-terminal-id="terminal.backendTerminalId"
      :is-connecting="terminal.isSSHConnecting || false"
      :is-visible="terminal.id === activeTerminalId"
      :is-focused="props.focusedTerminalId === terminal.id"
      class="w-full h-full absolute inset-0"
      @terminal-ready="onTerminalReady"
      @focus-terminal="onFocusTerminal"
    />
  </div>
</template>

<script setup lang="ts">
import { ref, watch, nextTick, onMounted, onBeforeUnmount } from "vue";
import Terminal from "./Terminal.vue";
import type { ComponentPublicInstance } from "vue";
import { bytesToString } from "../../utils/helpers";
import type { TerminalInstance } from "../../types/panel";
import { useWorkspaceStore } from "../../stores/workspace";

interface TerminalManagerProps {
  terminals: TerminalInstance[];
  activeTerminalId?: string;
  focusedTerminalId?: string | null;
}

interface TerminalComponent extends ComponentPublicInstance {
  focus: () => void;
  fitAndFocus: () => void;
  writeOutput: (data: string) => void;
  clearTerminal: () => Promise<void>;
}

const props = defineProps<TerminalManagerProps>();

const workspaceStore = useWorkspaceStore();

const terminalRefs = ref<Record<string, ComponentPublicInstance | null>>({});
let outputUnlisten: (() => void) | null = null;

const emit = defineEmits(["terminal-ready"]);

/**
 * Handle focus terminal event from child terminal
 */
const onFocusTerminal = (terminalId: string): void => {
  workspaceStore.setFocusedTerminal(terminalId);
};

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
  if (el && typeof el === "object" && "$el" in el) {
    terminalRefs.value[terminalId] = el as ComponentPublicInstance;
  } else {
    delete terminalRefs.value[terminalId];
  }
};

const onTerminalReady = async (terminalId: string): Promise<void> => {
  emit("terminal-ready", terminalId);

  const matchingTerminal = props.terminals.find((t) => t.id === terminalId);
  const terminalInstance = terminalRefs.value[terminalId];

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

watch(
  () => props.activeTerminalId,
  async (newActiveId) => {
    if (newActiveId && terminalRefs.value[newActiveId]) {
      await nextTick();
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

onMounted(async () => {
  try {
    const workspaceStore = useWorkspaceStore();
    outputUnlisten = await workspaceStore.listenToTerminalOutput(
      (terminalData) => {
        const backendTerminalId = terminalData.terminalId;

        const matchingTerminal = props.terminals.find(
          (t) => t.backendTerminalId === backendTerminalId,
        );
        if (!matchingTerminal) return;

        const terminalRef = terminalRefs.value[matchingTerminal.id];

        if (terminalRef && "writeOutput" in terminalRef) {
          const output = bytesToString(terminalData.data);
          (terminalRef as TerminalComponent).writeOutput(output);
        }
      },
    );

    const handleWindowFocus = (): void => {
      setTimeout(() => {
        focusActiveTerminal();
      }, 100);
    };

    window.addEventListener("focus", handleWindowFocus);

    const originalUnlisten = outputUnlisten;
    outputUnlisten = () => {
      if (originalUnlisten) originalUnlisten();
      window.removeEventListener("focus", handleWindowFocus);
    };
  } catch (error) {
    console.error("Failed to listen to terminal output:", error);
  }
});

onBeforeUnmount(() => {
  if (outputUnlisten) {
    outputUnlisten();
  }
});
</script>
