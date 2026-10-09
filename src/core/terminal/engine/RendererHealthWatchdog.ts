// Copyright (C) 2026 Bùi Thanh Xuân (klpod221)
// SPDX-License-Identifier: GPL-3.0-or-later



export type TerminalRendererHealthSignal = "healthy" | "canvas-detached" | "canvas-zero-sized" | "webgl-context-lost";

export interface TerminalRendererHealthWatchdogScheduler {
  cancel(handle: number): void;
  schedule(callback: () => void, delayMs: number): number;
}

export interface TerminalRendererHealthWatchdog {
  check(): void;
  dispose(): void;
}

export interface CreateTerminalRendererHealthWatchdogOptions {
  container: HTMLElement;
  intervalMs?: number;
  renderer: {
    backend: "dom" | "canvas" | "webgl";
    getTrackedCanvases(): HTMLCanvasElement[];
    reportHealth(signal: TerminalRendererHealthSignal): void;
  };
  scheduler?: TerminalRendererHealthWatchdogScheduler;
}

const DEFAULT_HEALTH_WATCHDOG_INTERVAL_MS = 2000;

export function createTerminalRendererHealthWatchdog({
  container,
  intervalMs = DEFAULT_HEALTH_WATCHDOG_INTERVAL_MS,
  renderer,
  scheduler = browserWatchdogScheduler,
}: CreateTerminalRendererHealthWatchdogOptions): TerminalRendererHealthWatchdog {
  const resolvedIntervalMs = Math.max(250, Math.floor(intervalMs));
  let disposed = false;
  let timerHandle: number | null = null;
  let contextLostListener: ((e: Event) => void) | null = null;

  const scheduleNext = () => {
    if (disposed || timerHandle !== null) return;
    timerHandle = scheduler.schedule(() => {
      timerHandle = null;
      check();
      scheduleNext();
    }, resolvedIntervalMs);
  };

  const check = () => {
    if (disposed) return;
    if (renderer.backend !== "webgl") return;
    
    const canvases = renderer.getTrackedCanvases();
    const signal = resolveCanvasHealthSignal(container, canvases);
    
    if (signal !== "healthy") {
      renderer.reportHealth(signal);
    } else {
      // Re-attach context lost listener to ensure we catch it natively
      attachContextLostListeners(canvases);
    }
  };

  const attachContextLostListeners = (canvases: HTMLCanvasElement[]) => {
    if (!contextLostListener) {
      contextLostListener = (e: Event) => {
        e.preventDefault();
        renderer.reportHealth("webgl-context-lost");
      };
    }
    for (const canvas of canvases) {
      canvas.removeEventListener("webglcontextlost", contextLostListener);
      canvas.addEventListener("webglcontextlost", contextLostListener, false);
    }
  };

  scheduleNext();

  return {
    check,
    dispose() {
      if (disposed) return;
      disposed = true;
      if (timerHandle !== null) {
        scheduler.cancel(timerHandle);
        timerHandle = null;
      }
      if (contextLostListener) {
        const canvases = renderer.getTrackedCanvases();
        for (const canvas of canvases) {
          canvas.removeEventListener("webglcontextlost", contextLostListener);
        }
      }
    },
  };
}

function resolveCanvasHealthSignal(
  container: HTMLElement,
  rendererCanvases: readonly HTMLCanvasElement[],
): TerminalRendererHealthSignal {
  if (
    rendererCanvases.length === 0 ||
    rendererCanvases.some(
      (canvas) => !canvas.isConnected || !container.contains(canvas),
    )
  ) {
    return "canvas-detached";
  }
  if (
    rendererCanvases.some((canvas) => {
      const rect = canvas.getBoundingClientRect();
      return (
        canvas.width <= 0 ||
        canvas.height <= 0 ||
        rect.width <= 0 ||
        rect.height <= 0
      );
    })
  ) {
    return "canvas-zero-sized";
  }
  return "healthy";
}

const browserWatchdogScheduler: TerminalRendererHealthWatchdogScheduler = {
  cancel(handle) {
    window.clearTimeout(handle);
  },
  schedule(callback, delayMs) {
    return window.setTimeout(callback, delayMs);
  },
};
