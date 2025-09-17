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
    useBiometrics: false,
  });

  const currentDevice = ref<CurrentDevice | null>({
    device_id: "",
    device_name: "",
    device_type: "",
    os_name: "",
    os_version: "",
    created_at: "",
  });

  // Auto-lock timer
  let autoLockTimer: ReturnType<typeof setTimeout> | null = null;

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

    // Setup auto-lock timer if enabled
    setupAutoLockTimer();
    return true;
  };

  /**
   * Unlock with master password
   * @param verification - Master password verification data
   */
  const unlock = async (
    verification: MasterPasswordVerification,
  ): Promise<boolean> => {
    const isValid = await masterPasswordService.verify(verification);

    if (isValid) {
      // Refresh status after successful unlock
      await checkStatus();

      // Setup auto-lock timer
      setupAutoLockTimer();

      return true;
    } else {
      return false;
    }
  };

  /**
   * Lock the application
   */
  const lock = async (): Promise<void> => {
    // Clear auto-lock timer
    clearAutoLockTimer();

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
    return true;
  };

  /**
   * Update master password configuration
   * @param config - New configuration values
   */
  const updateMasterPasswordConfig = async (
    config: MasterPasswordConfig,
  ): Promise<boolean> => {
    await masterPasswordService.updateConfig(config);

    await checkStatus();
    return true;
  };

  /**
   * Try auto-unlock using keychain
   */
  const tryAutoUnlock = async (): Promise<boolean> => {
    const success = await masterPasswordService.tryAutoUnlock();

    await checkStatus();
    setupAutoLockTimer();

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
   * Clear auto-lock timer
   */
  const clearAutoLockTimer = (): void => {
    if (autoLockTimer) {
      clearTimeout(autoLockTimer);
      autoLockTimer = null;
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
    await getCurrentDevice();

    // If already unlocked, setup auto-lock timer
    if (status.value.isUnlocked) {
      setupAutoLockTimer();
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
