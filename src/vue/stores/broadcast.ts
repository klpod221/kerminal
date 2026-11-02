// Copyright (C) 2026 Bùi Thanh Xuân (klpod221)
// SPDX-License-Identifier: GPL-3.0-or-later

import { defineStore } from "pinia";
import { ref } from "vue";
import type { Panel, TerminalInstance } from "../types/panel";
import { message } from "../utils/message";

export type BroadcastScope = "workspace" | "panel";

export const useBroadcastStore = defineStore("broadcast", () => {
  const isBroadcastEnabled = ref(false);
  const broadcastScope = ref<BroadcastScope>("workspace");

  const toggleBroadcast = () => {
    isBroadcastEnabled.value = !isBroadcastEnabled.value;
    if (isBroadcastEnabled.value) {
      message.warning(
        `Broadcast Input ON: Keystrokes will be sent to ${
          broadcastScope.value === "workspace" ? "ALL terminals" : "active panel terminals"
        }.`,
      );
    } else {
      message.info("Broadcast Input OFF: Single terminal input restored.");
    }
  };

  const setBroadcastEnabled = (enabled: boolean) => {
    isBroadcastEnabled.value = enabled;
  };

  const setScope = (scope: BroadcastScope) => {
    broadcastScope.value = scope;
  };

  /**
   * Determine target terminals that should receive replicated input.
   * If broadcast is disabled, returns only the active terminal.
   */
  const getTargetTerminals = (
    activeTerminalId: string,
    currentPanelId: string,
    panels: Panel[],
    allTerminals: TerminalInstance[],
  ): TerminalInstance[] => {
    const activeTerminal = allTerminals.find((t) => t.id === activeTerminalId);
    if (!isBroadcastEnabled.value) {
      return activeTerminal ? [activeTerminal] : [];
    }

    if (broadcastScope.value === "panel") {
      const panel = panels.find((p) => p.id === currentPanelId);
      if (!panel) return activeTerminal ? [activeTerminal] : [];
      const tabIds = new Set(panel.tabs.map((tab) => tab.id));
      return allTerminals.filter(
        (t) => tabIds.has(t.id) && Boolean(t.backendTerminalId),
      );
    }

    // Default 'workspace': broadcast to currently active tabs in all open panels
    const activeTabIds = new Set(panels.map((p) => p.activeTabId).filter(Boolean));
    const targetList = allTerminals.filter(
      (t) => activeTabIds.has(t.id) && Boolean(t.backendTerminalId),
    );

    return targetList.length > 0 ? targetList : (activeTerminal ? [activeTerminal] : []);
  };

  const isTerminalBroadcasting = (_terminalId?: string): boolean => {
    return isBroadcastEnabled.value;
  };

  return {
    isBroadcastEnabled,
    broadcastScope,
    toggleBroadcast,
    setBroadcastEnabled,
    setScope,
    getTargetTerminals,
    isTerminalBroadcasting,
  };
});
