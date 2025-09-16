import { defineStore } from "pinia";
import { ref, computed } from "vue";
import type {
  MasterPasswordStatus,
  MasterPasswordSetup,
  MasterPasswordVerification,
  MasterPasswordChange,
  SecuritySettings
} from "../types/auth";
import { MasterPasswordFormState } from "../types/auth";
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

  const formState = ref<MasterPasswordFormState>(MasterPasswordFormState.IDLE);
  const lastError = ref<string>("");
  const securitySettings = ref<SecuritySettings>({
    requirePasswordOnStart: true,
    autoLockTimeout: 15,
    sessionTimeoutMinutes: 60,
    useBiometrics: false,
  });

  // Auto-lock timer
  let autoLockTimer: ReturnType<typeof setTimeout> | null = null;

  // Computed properties
  const isAuthenticated = computed(() => status.value.isUnlocked);
  const requiresSetup = computed(() => !status.value.isSetup);
  const requiresUnlock = computed(() => status.value.isSetup && !status.value.isUnlocked);
  const isLoading = computed(() => formState.value === MasterPasswordFormState.LOADING);

  /**
   * Check master password status from backend
   */
  const checkStatus = async (): Promise<void> => {
    try {
      formState.value = MasterPasswordFormState.LOADING;

      const result = await masterPasswordService.getStatus();
      status.value = result;

      formState.value = MasterPasswordFormState.IDLE;
    } catch (error) {
      console.error("Failed to check master password status:", error);
      lastError.value = "Failed to check authentication status";
      formState.value = MasterPasswordFormState.ERROR;
    }
  };

  /**
   * Setup master password for the first time
   * @param setup - Master password setup configuration
   */
  const setupMasterPassword = async (setup: MasterPasswordSetup): Promise<boolean> => {
    try {
      formState.value = MasterPasswordFormState.LOADING;
      lastError.value = "";

      await masterPasswordService.setup(setup);

      // Refresh status after setup
      await checkStatus();

      // Setup auto-lock timer if enabled
      setupAutoLockTimer();

      formState.value = MasterPasswordFormState.SUCCESS;
      return true;
    } catch (error) {
      console.error("Failed to setup master password:", error);
      lastError.value = "Failed to setup master password";
      formState.value = MasterPasswordFormState.ERROR;
      return false;
    }
  };

  /**
   * Unlock with master password
   * @param verification - Master password verification data
   */
  const unlock = async (verification: MasterPasswordVerification): Promise<boolean> => {
    try {
      formState.value = MasterPasswordFormState.LOADING;
      lastError.value = "";

      const isValid = await masterPasswordService.verify(verification);

      if (isValid) {
        // Refresh status after successful unlock
        await checkStatus();

        // Setup auto-lock timer
        setupAutoLockTimer();

        formState.value = MasterPasswordFormState.SUCCESS;
        return true;
      } else {
        lastError.value = "Invalid password";
        formState.value = MasterPasswordFormState.ERROR;
        return false;
      }
    } catch (error) {
      console.error("Failed to unlock:", error);
      lastError.value = "Invalid password or verification failed";
      formState.value = MasterPasswordFormState.ERROR;
      return false;
    }
  };

  /**
   * Lock the application
   */
  const lock = async (): Promise<void> => {
    try {
      formState.value = MasterPasswordFormState.LOADING;

      // Clear auto-lock timer
      clearAutoLockTimer();

      await masterPasswordService.lock();

      // Refresh status after lock
      await checkStatus();

      formState.value = MasterPasswordFormState.IDLE;
    } catch (error) {
      console.error("Failed to lock application:", error);
      lastError.value = "Failed to lock application";
      formState.value = MasterPasswordFormState.ERROR;
    }
  };

  /**
   * Change master password
   * @param changeData - Password change data
   */
  const changeMasterPassword = async (changeData: MasterPasswordChange): Promise<boolean> => {
    try {
      formState.value = MasterPasswordFormState.LOADING;
      lastError.value = "";

      await masterPasswordService.change(changeData);

      formState.value = MasterPasswordFormState.SUCCESS;
      return true;
    } catch (error) {
      console.error("Failed to change master password:", error);
      lastError.value = "Failed to change password";
      formState.value = MasterPasswordFormState.ERROR;
      return false;
    }
  };

  /**
   * Try auto-unlock using keychain
   */
  const tryAutoUnlock = async (): Promise<boolean> => {
    try {
      formState.value = MasterPasswordFormState.LOADING;

      const success = await masterPasswordService.tryAutoUnlock();

      if (success) {
        // Refresh status after successful auto-unlock
        await checkStatus();
        setupAutoLockTimer();
        formState.value = MasterPasswordFormState.SUCCESS;
      } else {
        formState.value = MasterPasswordFormState.IDLE;
      }

      return success;
    } catch (error) {
      console.error("Failed to auto-unlock:", error);
      formState.value = MasterPasswordFormState.IDLE;
      return false;
    }
  };

  /**
   * Check if master password is setup
   */
  const checkSetup = async (): Promise<boolean> => {
    try {
      return await masterPasswordService.isSetup();
    } catch (error) {
      console.error("Failed to check setup status:", error);
      return false;
    }
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
      autoLockTimer = setTimeout(() => {
        console.log("Auto-lock triggered");
        lock();
      }, timeoutMinutes * 60 * 1000);
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
   * Get current security settings
   */
  const getSecuritySettings = async (): Promise<SecuritySettings | null> => {
    try {
      // Note: Backend command not yet implemented
      // const settings = await masterPasswordService.getSecuritySettings();
      // return settings;

      return securitySettings.value;
    } catch (error) {
      console.error("Failed to get security settings:", error);
      return null;
    }
  };

  /**
   * Reset master password (removes all encrypted data)
   */
  const resetMasterPassword = async (): Promise<boolean> => {
    try {
      formState.value = MasterPasswordFormState.LOADING;
      lastError.value = "";

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

      formState.value = MasterPasswordFormState.SUCCESS;
      return true;
    } catch (error) {
      console.error("Failed to reset master password:", error);
      lastError.value = "Failed to reset master password";
      formState.value = MasterPasswordFormState.ERROR;
      return false;
    }
  };

  /**
   * Disable auto-unlock feature
   */
  const disableAutoUnlock = async (): Promise<boolean> => {
    try {
      formState.value = MasterPasswordFormState.LOADING;
      lastError.value = "";

      await masterPasswordService.disableAutoUnlock();

      // Update status
      status.value.autoUnlockEnabled = false;

      formState.value = MasterPasswordFormState.SUCCESS;
      return true;
    } catch (error) {
      console.error("Failed to disable auto-unlock:", error);
      lastError.value = "Failed to disable auto-unlock";
      formState.value = MasterPasswordFormState.ERROR;
      return false;
    }
  };

  /**
   * Get current device information
   */
  const getCurrentDevice = async () => {
    try {
      return await masterPasswordService.getCurrentDevice();
    } catch (error) {
      console.error("Failed to get current device:", error);
      return null;
    }
  };
  const clearError = (): void => {
    lastError.value = "";
    if (formState.value === MasterPasswordFormState.ERROR) {
      formState.value = MasterPasswordFormState.IDLE;
    }
  };

  /**
   * Initialize auth store
   */
  const initialize = async (): Promise<void> => {
    await checkStatus();

    // If already unlocked, setup auto-lock timer
    if (status.value.isUnlocked) {
      setupAutoLockTimer();
    }
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
    formState,
    lastError,
    securitySettings,

    // Computed
    isAuthenticated,
    requiresSetup,
    requiresUnlock,
    isLoading,

    // Actions
    checkStatus,
    checkSetup,
    setupMasterPassword,
    unlock,
    lock,
    tryAutoUnlock,
    changeMasterPassword,
    resetMasterPassword,
    disableAutoUnlock,
    resetAutoLockTimer,
    getSecuritySettings,
    getCurrentDevice,
    clearError,
    initialize,
    cleanup,
  };
});
