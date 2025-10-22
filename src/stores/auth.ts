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

  let sessionCheckInterval: ReturnType<typeof setInterval> | null = null;

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
  const checkSessionValidity = async (): Promise<boolean> => {
    const isValid = await masterPasswordService.isSessionValid();
    if (!isValid && status.value.isUnlocked) {
      await checkStatus();
    }
    return isValid;
  };

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
        startSessionValidityCheck();
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

        startSessionValidityCheck();

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
    stopSessionValidityCheck();

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

    startSessionValidityCheck();

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
      startSessionValidityCheck();
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
  const startSessionValidityCheck = (): void => {
    stopSessionValidityCheck();

    if (status.value.isUnlocked) {
      sessionCheckInterval = setInterval(async () => {
        if (status.value.isUnlocked) {
          await checkSessionValidity();
        }
      }, 30000); // Check every 30 seconds
    }
  };

  /**
   * Stop periodic session validity check
   */
  const stopSessionValidityCheck = (): void => {
    if (sessionCheckInterval) {
      clearInterval(sessionCheckInterval);
      sessionCheckInterval = null;
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
      startSessionValidityCheck();
    }
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
    stopSessionValidityCheck();
  };

  return {
    status,
    securitySettings,
    currentDevice,

    isAuthenticated,
    requiresSetup,
    requiresUnlock,

    checkStatus,
    checkSessionValidity,
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
