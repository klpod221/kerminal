// Copyright (C) 2026 Bùi Thanh Xuân (klpod221)
// SPDX-License-Identifier: GPL-3.0-or-later

import type { Terminal } from "@xterm/xterm";

/**
 * Loads the Canvas 2D renderer for xterm.
 * The CanvasAddon renders terminal contents to a single 2D HTML5 canvas.
 * It is fast, stable, and avoids the compositor stalls and high DOM node overhead
 * on Linux/Tauri.
 */
export const loadCanvasRenderer = async (
  term: Terminal,
): Promise<any | null> => {
  try {
    const { CanvasAddon } = await import("@xterm/addon-canvas");
    const canvasAddon = new CanvasAddon();
    term.loadAddon(canvasAddon);
    return canvasAddon;
  } catch (error) {
    console.warn("Canvas renderer failed, falling back to DOM", error);
    return null;
  }
};
