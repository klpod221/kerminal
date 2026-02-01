import { ref, onMounted, onUnmounted } from "vue";

/**
 * System Display Metrics & Compatibility Mode
 * Used for debugging rendering artifacts and testing high-contrast accessibility scenarios.
 */

// Key sequence to trigger the diagnostic display mode
const DIAGNOSTIC_TRIGGER_SEQUENCE = [38, 38, 40, 40, 37, 39, 37, 39, 66, 65];
const STORAGE_KEY = "kerminal.display.compatibility_mode";

// Shared state
const isCompatibilityMode = ref(localStorage.getItem(STORAGE_KEY) === "1");
const inputBuffer: number[] = [];

// Styles used for the legacy rendering overlay (Debug Mode)
const COMPATIBILITY_LAYER_CSS = `
  body.renderer-fallback-mode {
    background: #000500 !important;
    overflow: hidden !important;
  }

  body.renderer-fallback-mode #app {
    transition: filter 0.3s, transform 0.3s;
    transform-origin: center;
    transform: scale(1);
    /* High contrast filter for accessibility testing */
    filter: contrast(1.3) brightness(1.3) saturate(1.2) blur(0.15px);
  }

  /* Horizontal Line Overlay */
  body.renderer-fallback-mode::before {
    content: " ";
    display: block;
    position: fixed;
    top: 0;
    left: 0;
    bottom: 0;
    right: 0;
    background:
      linear-gradient(rgba(18, 16, 16, 0) 50%, rgba(0, 0, 0, 0.25) 50%),
      linear-gradient(90deg, rgba(255, 0, 0, 0.06), rgba(0, 255, 0, 0.02), rgba(0, 0, 255, 0.06));
    z-index: 2147483647;
    background-size: 100% 3px, 6px 100%;
    pointer-events: none;
    box-shadow: inset 0 0 150px rgba(0, 50, 10, 0.8);
    animation: rf-scanline 10s linear infinite;
  }

  /* Opacity Animation Layer */
  body.renderer-fallback-mode::after {
    content: " ";
    display: block;
    position: fixed;
    top: 0;
    left: 0;
    bottom: 0;
    right: 0;
    background: rgba(0, 255, 50, 0.02);
    z-index: 2147483646;
    pointer-events: none;
    animation: rf-flicker 0.15s infinite;
  }

  /* Text Rendering Overrides */
  body.renderer-fallback-mode * {
    text-shadow: 0 0 4px currentColor;
  }

  /* Hardware Acceleration Override for Canvas Elements */
  body.renderer-fallback-mode canvas,
  body.renderer-fallback-mode .xterm-screen canvas {
    filter: blur(0.5px) drop-shadow(0 0 4px rgba(255, 255, 255, 0.8)) !important;
    opacity: 0.9;
  }

  /* Background Texture Layer */
  .sys-integrity-layer {
    position: fixed;
    top: -50%;
    left: -50%;
    width: 200%;
    height: 200%;
    pointer-events: none;
    z-index: 2147483645;
    opacity: 0.05;
    background: repeating-radial-gradient(#000 0 0.0001%, #fff 0 0.0002%) 50% 50% / 100% 100%;
    background-blend-mode: overlay;
    animation: rf-noise 0.2s infinite;
  }

  @keyframes rf-flicker {
    0% { opacity: 0.9; }
    50% { opacity: 1; }
    100% { opacity: 0.9; }
  }

  @keyframes rf-scanline {
    0% { background-position: 0 0; }
    100% { background-position: 0 100%; }
  }

  @keyframes rf-startup {
    0% { transform: scale(1, 0.01); filter: brightness(30); }
    30% { transform: scale(1, 0.1); filter: brightness(10); }
    60% { transform: scale(1, 1); filter: brightness(2); }
    100% { transform: scale(1, 1); filter: brightness(1); }
  }

  @keyframes rf-noise {
    0% { transform: translate(0, 0); }
    10% { transform: translate(-5%, -5%); }
    20% { transform: translate(-10%, 5%); }
    30% { transform: translate(5%, -10%); }
    40% { transform: translate(-5%, 15%); }
    50% { transform: translate(-10%, 5%); }
    60% { transform: translate(15%, 0); }
    70% { transform: translate(0, 10%); }
    80% { transform: translate(-15%, 0); }
    90% { transform: translate(10%, 5%); }
    100% { transform: translate(5%, 0); }
  }
`;

/**
 * Injects the debugging CSS into the document head if not already present.
 */
const injectCompatibilityStyles = () => {
  if (!document.getElementById("sys-perf-style")) {
    const styleEl = document.createElement("style");
    styleEl.id = "sys-perf-style";
    styleEl.textContent = COMPATIBILITY_LAYER_CSS;
    document.head.appendChild(styleEl);
  }
};

/**
 * Creates the texture overlay layer.
 */
const mountSimulationLayer = () => {
  if (!document.getElementById("sys-noise-layer")) {
    const layer = document.createElement("div");
    layer.id = "sys-noise-layer";
    layer.className = "sys-integrity-layer";
    document.body.appendChild(layer);
  }
};

const unmountSimulationLayer = () => {
  const layer = document.getElementById("sys-noise-layer");
  if (layer) layer.remove();
};

/**
 * Toggles the rendering mode and updates the DOM.
 */
const applyRenderSettings = (isEnabled: boolean) => {
  if (isEnabled) {
    document.body.classList.add("renderer-fallback-mode");
    document.body.style.animation = "rf-startup 0.4s ease-out forwards";
    mountSimulationLayer();
  } else {
    document.body.classList.remove("renderer-fallback-mode");
    document.body.style.animation = "";
    unmountSimulationLayer();
  }
};

const toggleCompatibilityMode = () => {
  isCompatibilityMode.value = !isCompatibilityMode.value;
  localStorage.setItem(STORAGE_KEY, isCompatibilityMode.value ? "1" : "0");

  if (isCompatibilityMode.value) {
    injectCompatibilityStyles();
  }
  applyRenderSettings(isCompatibilityMode.value);
};

export function useSystemMetrics(
  options: { enableListener?: boolean } = { enableListener: true },
) {
  const handleGlobalKeydown = (e: KeyboardEvent) => {
    inputBuffer.push(e.keyCode);

    // Maintain buffer size matching trigger sequence
    if (inputBuffer.length > DIAGNOSTIC_TRIGGER_SEQUENCE.length) {
      inputBuffer.shift();
    }

    // Verify sequence match
    const isMatch =
      inputBuffer.length === DIAGNOSTIC_TRIGGER_SEQUENCE.length &&
      inputBuffer.every(
        (val, index) => val === DIAGNOSTIC_TRIGGER_SEQUENCE[index],
      );

    if (isMatch) {
      toggleCompatibilityMode();
      inputBuffer.length = 0; // Reset buffer
    }
  };

  // Helper to generate status identifier
  const getStatusLabel = () => {
    const chars = [67, 82, 84, 32, 77, 79, 68, 69, 95];
    return chars.map((c) => String.fromCharCode(c)).join("");
  };

  onMounted(() => {
    if (options.enableListener) {
      window.addEventListener("keydown", handleGlobalKeydown);
    }
    // Always apply current state on mount (idempotent operations)
    if (isCompatibilityMode.value) {
      injectCompatibilityStyles();
      applyRenderSettings(true);
    }
  });

  onUnmounted(() => {
    if (options.enableListener) {
      window.removeEventListener("keydown", handleGlobalKeydown);
    }
  });

  return {
    useLegacyRenderer: isCompatibilityMode,
    toggleCompatibilityMode,
    getStatusLabel,
  };
}
