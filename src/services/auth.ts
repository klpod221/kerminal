import { api } from "./api";
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
  return await api.call<void>("setup_master_password", {
    ...setup,
    autoLockTimeout: Number(setup.autoLockTimeout),
    useKeychain: setup.useKeychain || false,
  });
}

/**
 * Verify master password
 * @param verification - Master password verification request
 * @returns True if password is valid, false otherwise
 */
export async function verify(
  verification: MasterPasswordVerification,
): Promise<boolean> {
  return await api.call<boolean>("verify_master_password", verification);
}

/**
 * Try to auto-unlock using keychain
 * @returns True if auto-unlock was successful, false otherwise
 */
export async function tryAutoUnlock(): Promise<boolean> {
  return await api.callRaw<boolean>("try_auto_unlock");
}

/**
 * Lock the current session
 */
export async function lock(): Promise<void> {
  return await api.callRaw<void>("lock_session");
}

/**
 * Change master password
 * @param change - Master password change request
 */
export async function change(change: MasterPasswordChange): Promise<void> {
  return await api.call<void>("change_master_password", change);
}

/**
 * Get master password status
 * @returns Current master password status
 */
export async function getStatus(): Promise<MasterPasswordStatus> {
  return await api.callRaw<MasterPasswordStatus>("get_master_password_status");
}

/**
 * Reset master password (removes all encrypted data)
 */
export async function reset(): Promise<void> {
  return await api.callRaw<void>("reset_master_password");
}

/**
 * Update master password configuration
 * @param config - New configuration
 */
export async function updateConfig(
  config: Partial<MasterPasswordConfig>,
): Promise<void> {
  return await api.call<void>("update_master_password_config", config);
}

/**
 * Get current device information
 * @returns Current device information
 */
export async function getCurrentDevice(): Promise<any> {
  return await api.callRaw<any>("get_current_device");
}

/**
 * Check if session is valid (not expired)
 * @returns True if session is valid, false otherwise
 */
export async function isSessionValid(): Promise<boolean> {
  return await api.callRaw<boolean>("is_session_valid");
}

/**
 * Get master password configuration
 * @returns Master password configuration
 */
export async function getConfig(): Promise<any> {
  return await api.callRaw<any>("get_master_password_config");
}
