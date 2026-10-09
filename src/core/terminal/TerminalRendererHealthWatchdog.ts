import type { Terminal } from "@xterm/xterm";

/**
 * TerminalRendererHealthWatchdog
 * 
 * Monitors the WebGL renderer for context loss events (which can happen
 * when the GPU crashes, memory is exhausted, or display settings change).
 * Automatically falls back to the DOM renderer when WebGL fails.
 */
export class TerminalRendererHealthWatchdog {
  /**
   * Monitor the WebGL addon for context loss and fallback automatically
   */
  public static monitor(
    term: Terminal,
    webglAddon: any,
    fallbackRenderer: () => void
  ) {
    if (!webglAddon) return;

    // Listen using xterm.js native event if available
    if (typeof webglAddon.onContextLoss === 'function') {
      webglAddon.onContextLoss(() => {
        console.warn("[WebGL Watchdog] WebGL context lost detected via addon event! Initiating fallback to DOM renderer...");
        this.triggerFallback(webglAddon, fallbackRenderer);
      });
    } else {
      // Fallback: observe the canvas element if the event is not exposed
      if (term.element) {
        const canvases = term.element.querySelectorAll('canvas');
        canvases.forEach(canvas => {
          canvas.addEventListener('webglcontextlost', (e) => {
            e.preventDefault();
            console.warn("[WebGL Watchdog] WebGL context lost detected via DOM event! Initiating fallback to DOM renderer...");
            this.triggerFallback(webglAddon, fallbackRenderer);
          }, false);
        });
      }
    }
  }

  private static triggerFallback(webglAddon: any, fallbackRenderer: () => void) {
    try {
      // Dispose the broken WebGL addon to allow fallback
      webglAddon.dispose();
    } catch (e) {
      console.error("[WebGL Watchdog] Error disposing WebGL addon:", e);
    }
    // Call the fallback which uses DOM renderer (since Canvas addon is not installed)
    fallbackRenderer();
  }
}
