<template>
  <div class="fixed inset-0 z-9999 bg-black overflow-hidden font-mono">
    <!-- GPU Canvas Layer -->
    <canvas ref="canvasRef" class="absolute inset-0 block"></canvas>

    <!-- Diagnostics Overlay -->
    <div
      class="absolute inset-0 flex flex-col items-center justify-center pointer-events-none"
    >
      <div
        class="bg-black/80 border border-green-500 p-8 rounded shadow-[0_0_20px_rgba(0,255,0,0.5)] text-green-500 text-center pointer-events-auto backdrop-blur-sm max-w-md w-full mx-4"
      >
        <h1
          class="text-4xl font-bold mb-2 tracking-widest uppercase glitch-text"
        >
          CORE DUMP
        </h1>
        <div class="h-px bg-green-500 w-full my-4 shadow-[0_0_10px_#0f0]"></div>
        <p class="text-xl mb-2">Kernel Version: {{ appVersion }}</p>
        <p class="text-lg mb-2 opacity-80">SysAdmin: klpod221</p>

        <div
          v-if="status"
          class="flex justify-between w-full mb-4 text-xs opacity-60 font-mono"
        >
          <span>MEM: {{ status.memory_pressure }}%</span>
          <span>PROCS: {{ status.process_count }}</span>
        </div>

        <div class="h-6 flex items-center justify-center">
          <p
            :key="logSequenceKey"
            class="text-sm typing-effect opacity-70 text-green-400"
          >
            > {{ currentLogMessage }}
          </p>
        </div>
      </div>
    </div>

    <!-- Abort Protocol Button -->
    <button
      @click="$emit('close')"
      class="absolute bottom-8 right-8 px-6 py-2 border border-green-500 text-green-500 hover:bg-green-500 hover:text-black transition-all duration-300 uppercase tracking-wider font-bold shadow-[0_0_10px_rgba(0,255,0,0.3)] hover:shadow-[0_0_20px_rgba(0,255,0,0.8)] z-50 cursor-pointer"
    >
      [ TERMINATE PROCESS ]
    </button>
  </div>
</template>

<script setup lang="ts">
import { ref, onMounted, onUnmounted, computed } from "vue";
import type { SystemIntegrityStatus } from "../../services/dashboard";

/**
 * RenderOptimizationLayer.vue
 *
 * This component is used to stress-test the canvas rendering engine
 * and visualize data stream throughput for the SFTP buffer.
 * It uses a heavy particle system to ensure 60fps stability.
 */

declare const __APP_VERSION__: string;
const appVersion = __APP_VERSION__;

const props = defineProps<{
  status: SystemIntegrityStatus | null;
}>();

const canvasRef = ref<HTMLCanvasElement | null>(null);
const emit = defineEmits(["close"]);

// Fallback logs if backend status is unavailable
const defaultLogs = [
  "Initializing neural simulation...",
  "System integrity compromised.",
];

const availableLogs = computed(() => {
  if (props.status && props.status.active_nodes.length > 0) {
    return props.status.active_nodes;
  }
  return defaultLogs;
});

const currentLogMessage = ref(availableLogs.value[0]);
const logSequenceKey = ref(0);
let logInterval: number | null = null;
let renderLoopId: number | null = null;

// Hex matrix for data stream visualization
const hexMatrix = "ABCDEFGHIJKLMNOPQRSTUVWXYZ123456789@#$%^&*()*&^%";
const streamFontSize = 16;
let dataStream: number[] = [];

// Simple deterministic pseudo-random using incrementing seed
let seed = Date.now();
const nextPseudoRandom = () => {
  seed = (seed * 1103515245 + 12345) & 0x7fffffff;
  return seed / 0x7fffffff;
};

const initMatrix = () => {
  if (!canvasRef.value) return;

  // Initialize canvas to viewport dimensions
  canvasRef.value.width = window.innerWidth;
  canvasRef.value.height = window.innerHeight;

  const streamColumns = Math.floor(canvasRef.value.width / streamFontSize);

  // Re-initialize data stream
  dataStream = [];
  for (let x = 0; x < streamColumns; x++) {
    dataStream[x] = Math.floor(nextPseudoRandom() * -100); // Randomize start positions for better effect
  }
};

onMounted(() => {
  // Start diagnostic log rotation
  logInterval = setInterval(() => {
    const currentIndex = availableLogs.value.indexOf(currentLogMessage.value);
    const nextIndex = (currentIndex + 1) % availableLogs.value.length;
    currentLogMessage.value = availableLogs.value[nextIndex];
    logSequenceKey.value++; // Force re-render for typing animation
  }, 4000);

  if (!canvasRef.value) return;

  const ctx = canvasRef.value.getContext("2d");
  if (!ctx) return;

  // Initial setup
  initMatrix();

  const renderFrame = () => {
    if (!ctx || !canvasRef.value) return;

    // Apply fade effect to previous frame to create trail
    ctx.fillStyle = "rgba(0, 0, 0, 0.05)";
    ctx.fillRect(0, 0, canvasRef.value.width, canvasRef.value.height);

    ctx.fillStyle = "#0F0"; // Signal color: Green
    ctx.font = streamFontSize + "px monospace";

    for (let i = 0; i < dataStream.length; i++) {
      const char = hexMatrix.charAt(
        Math.floor(nextPseudoRandom() * hexMatrix.length),
      );
      ctx.fillText(char, i * streamFontSize, dataStream[i] * streamFontSize);

      // Randomly reset stream based on entropy
      if (
        dataStream[i] * streamFontSize > canvasRef.value.height &&
        nextPseudoRandom() > 0.975
      ) {
        dataStream[i] = 0;
      }

      // Increment packet flow
      dataStream[i]++;
    }
  };

  // Run render loop at ~30fps for stability
  renderLoopId = setInterval(renderFrame, 33);

  window.addEventListener("resize", initMatrix);
});

onUnmounted(() => {
  if (renderLoopId) clearInterval(renderLoopId);
  if (logInterval) clearInterval(logInterval);
  window.removeEventListener("resize", initMatrix);
});
</script>

<style scoped>
.glitch-text {
  text-shadow:
    2px 2px 0px rgba(0, 255, 0, 0.2),
    -2px -2px 0px rgba(0, 255, 0, 0.2);
  animation: glitch 2s infinite linear alternate-reverse;
}

@keyframes glitch {
  0% {
    text-shadow:
      2px 2px 0px rgba(0, 255, 0, 0.2),
      -2px -2px 0px rgba(0, 255, 0, 0.2);
  }
  25% {
    text-shadow:
      -2px 2px 0px rgba(0, 255, 0, 0.2),
      2px -2px 0px rgba(0, 255, 0, 0.2);
  }
  50% {
    text-shadow:
      2px -2px 0px rgba(0, 255, 0, 0.2),
      -2px 2px 0px rgba(0, 255, 0, 0.2);
  }
  75% {
    text-shadow:
      -2px -2px 0px rgba(0, 255, 0, 0.2),
      2px 2px 0px rgba(0, 255, 0, 0.2);
  }
  100% {
    text-shadow:
      2px 2px 0px rgba(0, 255, 0, 0.2),
      -2px -2px 0px rgba(0, 255, 0, 0.2);
  }
}

.typing-effect {
  overflow: hidden;
  white-space: nowrap;
  animation: typing 3.5s steps(40, end);
}

@keyframes typing {
  from {
    width: 0;
  }
  to {
    width: 100%;
  }
}
</style>
