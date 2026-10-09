// Copyright (C) 2026 Bùi Thanh Xuân (klpod221)
// SPDX-License-Identifier: GPL-3.0-or-later

import { useSecurityStore } from "../../stores/security";
import { useBroadcastStore } from "../../stores/broadcast";
import { useWorkspaceStore } from "../../stores/workspace";
import { detectDangerousCommand } from "../../services/security/dangerousCommands";
import { InputBatcher, TerminalRegistry } from "../../core";
import type { Panel, TerminalInstance } from "../../types/panel";

export function useTerminalInputSecurity() {
  const securityStore = useSecurityStore();
  const broadcastStore = useBroadcastStore();
  const workspaceStore = useWorkspaceStore();
  const inputBatcher = InputBatcher.getInstance();

  // Track the current typed line buffer per terminal ID
  const lineBuffers = new Map<string, string>();

  /**
   * Handle incoming raw terminal input from xterm.onData
   */
  const handleTerminalInput = async (
    terminalId: string,
    data: string,
    options: {
      activeTerminalId: string;
      panelId?: string;
      panels: Panel[];
      allTerminals: TerminalInstance[];
    },
  ) => {
    // Find terminal instance reliably from workspaceStore or options
    const currentTerminal =
      workspaceStore.terminals.find((t) => t.id === terminalId) ||
      options.allTerminals.find((t) => t.id === terminalId);
    const backendTerminalId = currentTerminal?.backendTerminalId;
    if (!backendTerminalId) return;

    let currentLine = lineBuffers.get(terminalId) || "";

    // Check for Ctrl+C (\x03) or Ctrl+U (\x15) -> reset line buffer
    if (data.includes("\x03") || data.includes("\x15")) {
      lineBuffers.set(terminalId, "");
      dispatchToTargets(terminalId, backendTerminalId, data, options);
      return;
    }

    // Check for Backspace (\x7f or \x08)
    if (data === "\x7f" || data === "\x08") {
      if (currentLine.length > 0) {
        lineBuffers.set(terminalId, currentLine.slice(0, -1));
      }
      dispatchToTargets(terminalId, backendTerminalId, data, options);
      return;
    }

    // Check for Enter key or multiline execution
    const isEnter = data.includes("\r") || data.includes("\n");
    if (isEnter) {
      const fullCommand = currentLine + data.replace(/[\r\n]+/g, "");
      lineBuffers.set(terminalId, "");

      if (securityStore.isProtectionEnabled) {
        const dangerousMatch = detectDangerousCommand(fullCommand);
        if (dangerousMatch) {
          const allTabs = options.panels.flatMap((p) => p.tabs);
          const allTerminals =
            workspaceStore.terminals.length > 0
              ? workspaceStore.terminals
              : options.allTerminals;

          const targets = broadcastStore.getTargetTerminals(
            terminalId,
            options.panelId || "",
            options.panels,
            allTerminals,
          ).map((t) => {
            const matchingTab = allTabs.find((tab) => tab.id === t.id);
            return {
              id: t.id,
              backendTerminalId: t.backendTerminalId,
              title: matchingTab?.title || t.id,
            };
          });

          const confirmed = await securityStore.requestConfirmation(
            fullCommand,
            dangerousMatch,
            targets,
            broadcastStore.isBroadcastEnabled,
          );

          if (!confirmed) {
            // Abort: Send Ctrl+C (\x03) to cancel the prompt line safely
            dispatchToTargets(terminalId, backendTerminalId, "\x03", options);
            return;
          }
        }
      }

      dispatchToTargets(terminalId, backendTerminalId, data, options);
      return;
    }

    // Regular typing: accumulate printable chars into buffer
    if (data.length < 50 && !data.includes("\x1b")) {
      lineBuffers.set(terminalId, currentLine + data);
    }

    dispatchToTargets(terminalId, backendTerminalId, data, options);
  };

  /**
   * Dispatch input to active terminal or replicate to all broadcast targets
   */
  const dispatchToTargets = (
    currentTerminalId: string,
    currentBackendTerminalId: string,
    data: string,
    options: {
      activeTerminalId: string;
      panelId?: string;
      panels: Panel[];
      allTerminals: TerminalInstance[];
    },
  ) => {
    // 1. Normal mode: 100% direct dispatch to current terminal (never drops keys)
    if (!broadcastStore.isBroadcastEnabled) {
      inputBatcher.batchInput(currentBackendTerminalId, data);
      TerminalRegistry.getTerminal(currentTerminalId)?.outputWriter?.flush();
      return;
    }

    // 2. Broadcast mode: Replicate keystrokes across target terminals
    const allTerminals =
      workspaceStore.terminals.length > 0
        ? workspaceStore.terminals
        : options.allTerminals;

    const targets = broadcastStore.getTargetTerminals(
      currentTerminalId,
      options.panelId || "",
      options.panels,
      allTerminals,
    );

    for (const target of targets) {
      if (target.backendTerminalId) {
        inputBatcher.batchInput(target.backendTerminalId, data);
        TerminalRegistry.getTerminal(target.id)?.outputWriter?.flush();
      }
    }
  };

  return {
    handleTerminalInput,
  };
}
