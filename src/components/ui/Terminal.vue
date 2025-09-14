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
import { onMounted, ref, nextTick } from "vue";
import { debounce } from "../../utils/helpers";

import { Terminal } from "@xterm/xterm";

import { FitAddon } from "@xterm/addon-fit";
import { SearchAddon } from "@xterm/addon-search";
import { WebLinksAddon } from "@xterm/addon-web-links";
import { Unicode11Addon } from "@xterm/addon-unicode11";
import { WebglAddon } from "@xterm/addon-webgl";

interface TerminalProps {
  terminalId?: string;
  isVisible?: boolean;
  isConnecting?: boolean;
}

const props = withDefaults(defineProps<TerminalProps>(), {
  terminalId: "default",
  isVisible: true,
  isConnecting: false,
});

const emit = defineEmits<{
  "terminal-ready": [terminalId: string];
}>();

const terminalRef = ref<HTMLElement | null>(null);
let term: Terminal;
let fitAddon: FitAddon;

const handleResize = debounce(() => {
  if (fitAddon && props.isVisible) {
    fitAddon.fit();
  }
}, 100);

onMounted(async () => {
  if (!terminalRef.value) return;

  term = new Terminal({
    cursorBlink: true,
    fontFamily: "'Fira Code', monospace",
    fontSize: 14,
    lineHeight: 1,
    theme: {
      background: "#171717",
      foreground: "#d4d4d4",
    },
    allowProposedApi: true,
  });

  // --- Load terminal addons ---

  // 1. WebGL (load before open for better performance)
  const webglAddon = new WebglAddon();
  term.loadAddon(webglAddon);

  // 2. Fit Addon (fit terminal to window)
  fitAddon = new FitAddon();
  term.loadAddon(fitAddon);

  // 3. Web Links Addon (enable clickable links)
  term.loadAddon(
    new WebLinksAddon((event, _uri) => {
      event.preventDefault();
      // window.api.send('open-external-link', uri)
    })
  );

  // 4. Search Addon (enable text search)
  const searchAddon = new SearchAddon();
  term.loadAddon(searchAddon);

  // 5. Unicode 11 Addon (support wide characters, emoji)
  const unicode11Addon = new Unicode11Addon();
  term.loadAddon(unicode11Addon);
  // Activate Unicode 11 addon after loading
  term.unicode.activeVersion = "11";

  // Open terminal in DOM
  term.open(terminalRef.value);

  // Wait for DOM to be ready
  await nextTick();

  // Notify parent that terminal is ready
  emit("terminal-ready", props.terminalId || "default");

  // Handle window resize
  window.addEventListener("resize", handleResize);

  // Initial fit
  handleResize();
});
</script>

<style scoped>
@import "@xterm/xterm/css/xterm.css";

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
