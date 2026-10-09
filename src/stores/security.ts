// Copyright (C) 2026 Bùi Thanh Xuân (klpod221)
// SPDX-License-Identifier: GPL-3.0-or-later

import { defineStore } from "pinia";
import { ref } from "vue";
import { Store } from "@tauri-apps/plugin-store";
import type { DangerousCommandPattern } from "../services/security/dangerousCommands";
import { message } from "../utils/message";

export interface TargetTerminalInfo {
  id: string;
  backendTerminalId?: string;
  title: string;
}

export interface PendingCommandConfirmation {
  command: string;
  pattern: DangerousCommandPattern;
  targets: TargetTerminalInfo[];
  isBroadcast: boolean;
  resolve: (confirmed: boolean) => void;
}

let store: Store | null = null;
const initStore = async () => {
  store ??= await Store.load("settings.json");
  return store;
};

export const useSecurityStore = defineStore("security", () => {
  const isProtectionEnabled = ref(true);
  const pendingConfirmation = ref<PendingCommandConfirmation | null>(null);

  /**
   * Load protection setting from persistent store
   */
  const loadProtectionSetting = async () => {
    try {
      const storeInstance = await initStore();
      const saved = await storeInstance.get<boolean>("security-protection-enabled");
      if (typeof saved === "boolean") {
        isProtectionEnabled.value = saved;
      }
    } catch (e) {
      console.error("Failed to load security setting:", e);
    }
  };

  /**
   * Save protection setting to persistent store
   */
  const saveProtectionSetting = async () => {
    try {
      const storeInstance = await initStore();
      await storeInstance.set("security-protection-enabled", isProtectionEnabled.value);
      await storeInstance.save();
    } catch (e) {
      console.error("Failed to save security setting:", e);
    }
  };

  /**
   * Toggle protection mode
   */
  const toggleProtection = async () => {
    isProtectionEnabled.value = !isProtectionEnabled.value;
    if (isProtectionEnabled.value) {
      message.success("Dangerous Command Protection ENABLED: Risky commands will be blocked for confirmation.");
    } else {
      message.warning("Dangerous Command Protection DISABLED: Commands will execute without security prompt.");
    }
    await saveProtectionSetting();
  };

  /**
   * Set protection mode explicitly
   */
  const setProtectionEnabled = async (enabled: boolean) => {
    isProtectionEnabled.value = enabled;
    await saveProtectionSetting();
  };

  // Load saved preference on initialize
  loadProtectionSetting();

  const requestConfirmation = (
    command: string,
    pattern: DangerousCommandPattern,
    targets: TargetTerminalInfo[],
    isBroadcast: boolean,
  ): Promise<boolean> => {
    return new Promise((resolve) => {
      pendingConfirmation.value = {
        command,
        pattern,
        targets,
        isBroadcast,
        resolve,
      };
    });
  };

  const confirm = () => {
    if (pendingConfirmation.value) {
      pendingConfirmation.value.resolve(true);
      pendingConfirmation.value = null;
    }
  };

  const cancel = () => {
    if (pendingConfirmation.value) {
      pendingConfirmation.value.resolve(false);
      pendingConfirmation.value = null;
    }
  };

  return {
    isProtectionEnabled,
    pendingConfirmation,
    toggleProtection,
    setProtectionEnabled,
    requestConfirmation,
    confirm,
    cancel,
  };
});
