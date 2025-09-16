import { invoke } from "@tauri-apps/api/core";
import type {
  MasterPasswordSetup,
  MasterPasswordVerification,
  MasterPasswordStatus,
  MasterPasswordChange,
  MasterPasswordConfig,
  DeviceInfo,
} from "../types/auth";

/**
 * Check if master password is already setup
 * @returns True if master password is setup, false otherwise
 */
export async function isSetup(): Promise<boolean> {
  try {
    return await invoke<boolean>("is_master_password_setup");
  } catch (error) {
    console.error("Failed to check master password setup status:", error);
    throw new Error("Failed to check setup status");
  }
}

/**
 * Setup master password for the first time
 * @param setup - Master password setup configuration
 */
export async function setup(setup: MasterPasswordSetup): Promise<void> {
  try {
    await invoke<void>("setup_master_password", {
      password: setup.password,
      confirmPassword: setup.confirmPassword,
      deviceName: setup.deviceName,
      autoUnlock: setup.autoUnlock,
    });
  } catch (error) {
    console.error("Failed to setup master password:", error);
    throw error;
  }
}

/**
 * Verify master password
 * @param verification - Master password verification request
 * @returns True if password is valid, false otherwise
 */
export async function verify(
  verification: MasterPasswordVerification
): Promise<boolean> {
  try {
    return await invoke<boolean>("verify_master_password", {
      password: verification.password,
    });
  } catch (error) {
    console.error("Failed to verify master password:", error);
    throw error;
  }
}

/**
 * Try to auto-unlock using keychain
 * @returns True if auto-unlock was successful, false otherwise
 */
export async function tryAutoUnlock(): Promise<boolean> {
  try {
    return await invoke<boolean>("try_auto_unlock");
  } catch (error) {
    console.error("Failed to auto-unlock:", error);
    return false;
  }
}

/**
 * Lock the current session
 */
export async function lock(): Promise<void> {
  try {
    await invoke<void>("lock_session");
  } catch (error) {
    console.error("Failed to lock session:", error);
    throw error;
  }
}

/**
 * Change master password
 * @param change - Master password change request
 */
export async function change(change: MasterPasswordChange): Promise<void> {
  try {
    // Note: Backend implementation is incomplete (TODO)
    await invoke<void>("change_master_password", {
      oldPassword: change.oldPassword,
      newPassword: change.newPassword,
    });
  } catch (error) {
    console.error("Failed to change master password:", error);
    throw error;
  }
}

/**
 * Get master password status
 * @returns Current master password status
 */
export async function getStatus(): Promise<MasterPasswordStatus> {
  try {
    return await invoke<MasterPasswordStatus>("get_master_password_status");
  } catch (error) {
    console.error("Failed to get master password status:", error);
    throw error;
  }
}

/**
 * Get current device information
 * @returns Current device info
 */
export async function getCurrentDevice(): Promise<DeviceInfo> {
  try {
    return await invoke<DeviceInfo>("get_current_device");
  } catch (error) {
    console.error("Failed to get current device:", error);
    throw error;
  }
}

/**
 * Reset master password (removes all encrypted data)
 */
export async function reset(): Promise<void> {
  try {
    // Note: Backend implementation is incomplete (TODO)
    await invoke<void>("reset_master_password");
  } catch (error) {
    console.error("Failed to reset master password:", error);
    throw error;
  }
}

/**
 * Disable auto-unlock feature
 */
export async function disableAutoUnlock(): Promise<void> {
  try {
    // Note: Backend implementation is incomplete (TODO)
    await invoke<void>("disable_auto_unlock");
  } catch (error) {
    console.error("Failed to disable auto-unlock:", error);
    throw error;
  }
}

/**
 * Update master password configuration
 * @param config - New configuration
 */
export async function updateConfig(
  config: Partial<MasterPasswordConfig>
): Promise<void> {
  try {
    // Note: Backend command not yet implemented
    await invoke<void>("update_master_password_config", { config });
  } catch (error) {
    console.error("Failed to update master password config:", error);
    throw error;
  }
}
