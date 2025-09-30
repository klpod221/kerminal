import { defineStore } from "pinia";
import { ref, computed } from "vue";
import type {
  MasterPasswordStatus,
  MasterPasswordSetup,
  MasterPasswordVerification,
  MasterPasswordChange,
  MasterPasswordConfig,
  SecuritySettings,
  CurrentDevice,
} from "../types/auth";
import * as masterPasswordService from "../services/auth";

/**
 * Authentication Store
 * Manages authentication state, master password operations, and security settings
 */
export const useAuthStore = defineStore("auth", () => {
  // State variables
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

  // Auto-lock timer
  let autoLockTimer: ReturnType<typeof setTimeout> | null = null;

  // Session validity check interval
  let sessionCheckInterval: ReturnType<typeof setInterval> | null = null;

  // Computed properties
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
      // Session was expired and locked, update local status
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

    // Refresh status after setup
    await checkStatus();

    // Load current device information
    await getCurrentDevice();

    // If auto-unlock is enabled, try to unlock
    if (setup.autoUnlock && setup.useKeychain) {
      await tryAutoUnlock();

      // Setup auto-lock timer and session check if unlocked
      if (status.value.isUnlocked) {
        setupAutoLockTimer();
        startSessionValidityCheck();
      }
    } else {
      // If auto-unlock is not enabled, lock the session
      // User will need to manually unlock
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

      // Always refresh status after verify attempt
      await checkStatus();

      if (isValid) {
        // Setup auto-lock timer
        setupAutoLockTimer();

        // Start session validity check
        startSessionValidityCheck();

        return true;
      } else {
        return false;
      }
    } catch (error) {
      // Refresh status even on error to get current state
      await checkStatus();
      throw error;
    }
  };

  /**
   * Lock the application
   */
  const lock = async (): Promise<void> => {
    // Clear auto-lock timer and session check
    clearAutoLockTimer();
    stopSessionValidityCheck();

    await masterPasswordService.lock();

    // Refresh status after lock
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

    // Clear current auto-lock timer
    clearAutoLockTimer();

    // Refresh status after successful password change (backend will lock the session)
    await checkStatus();

    return true;
  };

  /**
   * Update master password configuration
   * @param config - New configuration values
   */
  const updateMasterPasswordConfig = async (
    config: MasterPasswordConfig,
  ): Promise<boolean> => {
    // Ensure autoLockTimeout is a number if provided
    const configData = {
      ...config,
      ...(config.autoLockTimeout !== undefined && {
        autoLockTimeout: Number(config.autoLockTimeout),
      }),
    };

    await masterPasswordService.updateConfig(configData);

    // Reload security settings and status after config update
    await loadSecuritySettings();
    await checkStatus();

    // Restart auto-lock timer with new settings
    setupAutoLockTimer();

    // Restart session validity check with new settings
    startSessionValidityCheck();

    return true;
  };

  /**
   * Try auto-unlock using keychain
   */
  const tryAutoUnlock = async (): Promise<boolean> => {
    const success = await masterPasswordService.tryAutoUnlock();

    await checkStatus();

    if (status.value.isUnlocked) {
      setupAutoLockTimer();
      startSessionValidityCheck();
    }

    return success;
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
    // Clear existing interval if any
    stopSessionValidityCheck();

    // Only start if authenticated
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

    // Clear all state after reset
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

    // If already unlocked, setup auto-lock timer and session check
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
    // State
    status,
    securitySettings,
    currentDevice,

    // Computed
    isAuthenticated,
    requiresSetup,
    requiresUnlock,

    // Actions
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
