import { defineStore } from "pinia";
import { ref, computed } from "vue";
import type {
  MasterPasswordStatus,
  MasterPasswordSetup,
  MasterPasswordVerification,
  MasterPasswordChange,
  MasterPasswordConfig,
  MasterPasswordConfigUpdate,
  SecuritySettings,
  CurrentDevice,
} from "../types/auth";
import * as masterPasswordService from "../services/auth";
import { api } from "../services/api";

/**
 * Authentication Store
 * Manages authentication state, master password operations, and security settings
 */
export const useAuthStore = defineStore("auth", () => {
  const status = ref<MasterPasswordStatus>({
    isSetup: false,
    isUnlocked: false,
    autoUnlockEnabled: false,
    keychainAvailable: false,
    sessionActive: false,
    sessionExpiresAt: undefined,
    loadedDeviceCount: 0,
  });

  const securitySettings = ref<SecuritySettings>({
    autoLockTimeout: 0,
  });

  const currentDevice = ref<CurrentDevice | null>({
    deviceId: "",
    deviceName: "",
    deviceType: "",
    osName: "",
    osVersion: "",
    createdAt: "",
  });

  let autoLockTimer: ReturnType<typeof setTimeout> | null = null;
  // Removed backend polling; realtime only
  let sessionCountdownInterval: ReturnType<typeof setInterval> | null = null;
  let unsubscribeAuthRealtime: (() => void) | null = null;

  const sessionRemainingMs = ref<number>(0);

  const isAuthenticated = computed(() => status.value.isUnlocked);
  const requiresSetup = computed(() => !status.value.isSetup);
  const requiresUnlock = computed(
    () => status.value.isSetup && !status.value.isUnlocked,
  );

  /**
   * Check master password status from backend
   */
  const checkStatus = async (): Promise<void> => {
    const result = await masterPasswordService.getStatus();
    status.value = result;
  };

  /**
   * Load security settings from backend
   */
  const loadSecuritySettings = async (): Promise<void> => {
    const config = await masterPasswordService.getConfig();
    if (config && config.sessionTimeoutMinutes !== undefined) {
      securitySettings.value.autoLockTimeout =
        config.sessionTimeoutMinutes || 0;
    }
  };

  /**
   * Check if current session is valid (not expired)
   */
  // Realtime-only: no periodic validity polling

  /**
   * Setup master password for the first time
   * @param setup - Master password setup configuration
   */
  const setupMasterPassword = async (
    setup: MasterPasswordSetup,
  ): Promise<boolean> => {
    await masterPasswordService.setup(setup);

    await checkStatus();

    await getCurrentDevice();

    if (setup.autoUnlock && setup.useKeychain) {
      await tryAutoUnlock();

      if (status.value.isUnlocked) {
        setupAutoLockTimer();
      }
    } else {
      await lock();
    }

    return true;
  };

  /**
   * Unlock with master password
   * @param verification - Master password verification data
   */
  const unlock = async (
    verification: MasterPasswordVerification,
  ): Promise<boolean> => {
    try {
      const isValid = await masterPasswordService.verify(verification);

      await checkStatus();

      if (isValid) {
        setupAutoLockTimer();

        return true;
      } else {
        return false;
      }
    } catch (error) {
      await checkStatus();
      throw error;
    }
  };

  /**
   * Lock the application
   */
  const lock = async (): Promise<void> => {
    clearAutoLockTimer();

    await masterPasswordService.lock();

    await checkStatus();
  };

  /**
   * Change master password
   * @param changeData - Password change data
   */
  const changeMasterPassword = async (
    changeData: MasterPasswordChange,
  ): Promise<boolean> => {
    await masterPasswordService.change(changeData);

    clearAutoLockTimer();

    await checkStatus();

    return true;
  };

  /**
   * Update master password configuration
   * @param config - New configuration values
   */
  const updateMasterPasswordConfig = async (
    config: MasterPasswordConfig | MasterPasswordConfigUpdate,
  ): Promise<boolean> => {
    const configData = {
      ...config,
      ...(config.autoLockTimeout !== undefined && {
        autoLockTimeout: Number(config.autoLockTimeout),
      }),
    };

    await masterPasswordService.updateConfig(configData);

    await loadSecuritySettings();
    await checkStatus();

    setupAutoLockTimer();

    return true;
  };

  /**
   * Try auto-unlock using keychain
   */
  const tryAutoUnlock = async (): Promise<boolean> => {
    await masterPasswordService.tryAutoUnlock();

    await checkStatus();

    if (status.value.isUnlocked) {
      setupAutoLockTimer();
      return true; // Return true if we are actually unlocked
    }

    return false; // Return false if we are still locked
  };

  /**
   * Reset auto-lock timer (called on user activity)
   */
  const resetAutoLockTimer = (): void => {
    if (!status.value.isUnlocked) return;

    setupAutoLockTimer();
  };

  /**
   * Clear auto-lock timer
   */
  const clearAutoLockTimer = (): void => {
    if (autoLockTimer) {
      clearTimeout(autoLockTimer);
      autoLockTimer = null;
    }
  };

  /**
   * Start periodic session validity check
   */
  // Removed periodic session validity check

  const startSessionCountdown = (): void => {
    stopSessionCountdown();
    if (!status.value.sessionExpiresAt) return;

    const update = () => {
      if (!status.value.sessionExpiresAt) {
        sessionRemainingMs.value = 0;
        return;
      }
      const expires = new Date(status.value.sessionExpiresAt).getTime();
      const now = Date.now();
      sessionRemainingMs.value = Math.max(0, expires - now);
    };

    update();
    sessionCountdownInterval = setInterval(update, 1000);
  };

  const stopSessionCountdown = (): void => {
    if (sessionCountdownInterval) {
      clearInterval(sessionCountdownInterval);
      sessionCountdownInterval = null;
    }
    sessionRemainingMs.value = 0;
  };

  const startAuthRealtime = async (): Promise<void> => {
    if (unsubscribeAuthRealtime) return;
    try {
      const u1 = await api.listen<any>("auth_session_unlocked", async () => {
        await checkStatus();
        startSessionCountdown();
      });
      const u2 = await api.listen<any>("auth_session_locked", async () => {
        await checkStatus();
        startSessionCountdown();
      });
      const u3 = await api.listen<any>("auth_session_updated", async () => {
        await checkStatus();
        startSessionCountdown();
      });
      unsubscribeAuthRealtime = () => { u1(); u2(); u3(); };
    } catch (e) {
      console.error("Failed to subscribe to auth realtime events:", e);
    }
  };
  const stopAuthRealtime = (): void => {
    if (unsubscribeAuthRealtime) {
      unsubscribeAuthRealtime();
      unsubscribeAuthRealtime = null;
    }
  };

  /**
   * Reset master password (removes all encrypted data)
   */
  const resetMasterPassword = async (): Promise<boolean> => {
    await masterPasswordService.reset();

    status.value = {
      isSetup: false,
      isUnlocked: false,
      autoUnlockEnabled: false,
      keychainAvailable: false,
      sessionActive: false,
      sessionExpiresAt: undefined,
      loadedDeviceCount: 0,
    };

    clearAutoLockTimer();
    return true;
  };

  /**
   * Initialize auth store
   */
  const initialize = async (): Promise<void> => {
    await checkStatus();
    await loadSecuritySettings();
    await getCurrentDevice();

    if (status.value.isUnlocked) {
      setupAutoLockTimer();
      startSessionCountdown();
    }

    await startAuthRealtime();
  };

  /**
   * Setup auto-lock timer based on current settings
   */
  const setupAutoLockTimer = (): void => {
    clearAutoLockTimer();

    const timeoutMinutes = securitySettings.value.autoLockTimeout;
    if (timeoutMinutes > 0 && status.value.isUnlocked) {
      autoLockTimer = setTimeout(
        () => {
          console.log("Auto-lock triggered");
          lock();
        },
        timeoutMinutes * 60 * 1000,
      );
    }
  };

  /**
   * Get current device information
   */
  const getCurrentDevice = async (): Promise<void> => {
    const deviceInfo = await masterPasswordService.getCurrentDevice();
    currentDevice.value = deviceInfo;
  };

  /**
   * Cleanup auth store
   */
  const cleanup = (): void => {
    clearAutoLockTimer();
    stopSessionCountdown();
    stopAuthRealtime();
  };

  return {
    status,
    securitySettings,
    currentDevice,
    sessionRemainingMs,

    isAuthenticated,
    requiresSetup,
    requiresUnlock,

    checkStatus,
    loadSecuritySettings,
    setupMasterPassword,
    unlock,
    lock,
    tryAutoUnlock,
    changeMasterPassword,
    resetMasterPassword,
    updateMasterPasswordConfig,
    resetAutoLockTimer,
    getCurrentDevice,
    initialize,
    cleanup,
  };
});
