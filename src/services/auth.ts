import { invoke } from "@tauri-apps/api/core";
import type {
  MasterPasswordSetup,
  MasterPasswordVerification,
  MasterPasswordStatus,
  MasterPasswordChange,
  MasterPasswordConfig,
} from "../types/auth";

/**
 * Setup master password for the first time
 * @param setup - Master password setup configuration
 */
export async function setup(setup: MasterPasswordSetup): Promise<void> {
  return await invoke<void>("setup_master_password", {
    password: setup.password,
    confirmPassword: setup.confirmPassword,
    deviceName: setup.deviceName,
    autoUnlock: setup.autoUnlock,
  });
}

/**
 * Verify master password
 * @param verification - Master password verification request
 * @returns True if password is valid, false otherwise
 */
export async function verify(
  verification: MasterPasswordVerification
): Promise<boolean> {
  return await invoke<boolean>("verify_master_password", {
    password: verification.password,
  });
}

/**
 * Try to auto-unlock using keychain
 * @returns True if auto-unlock was successful, false otherwise
 */
export async function tryAutoUnlock(): Promise<boolean> {
  return await invoke<boolean>("try_auto_unlock");
}

/**
 * Lock the current session
 */
export async function lock(): Promise<void> {
  return await invoke<void>("lock_session");
}

/**
 * Change master password
 * @param change - Master password change request
 */
export async function change(change: MasterPasswordChange): Promise<void> {
  // Note: Backend implementation is incomplete (TODO)
  return await invoke<void>("change_master_password", {
    oldPassword: change.oldPassword,
    newPassword: change.newPassword,
  });
}

/**
 * Get master password status
 * @returns Current master password status
 */
export async function getStatus(): Promise<MasterPasswordStatus> {
  return await invoke<MasterPasswordStatus>("get_master_password_status");
}

/**
 * Reset master password (removes all encrypted data)
 */
export async function reset(): Promise<void> {
  // Note: Backend implementation is incomplete (TODO)
  return await invoke<void>("reset_master_password");
}

/**
 * Update master password configuration
 * @param config - New configuration
 */
export async function updateConfig(
  config: Partial<MasterPasswordConfig>
): Promise<void> {
  // Note: Backend command not yet implemented
  return await invoke<void>("update_master_password_config", { config });
}
