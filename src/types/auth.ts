/**
 * Master password setup configuration
 */
export interface MasterPasswordSetup {
  password: string;
  confirmPassword: string;
  deviceName: string;
  autoUnlock: boolean;
  useKeychain: boolean;
  autoLockTimeout: number; // in minutes (0 = never)
}

/**
 * Master password verification request
 */
export interface MasterPasswordVerification {
  password: string;
}

/**
 * Master password status information
 */
export interface MasterPasswordStatus {
  isSetup: boolean;
  isUnlocked: boolean;
  autoUnlockEnabled: boolean;
  keychainAvailable: boolean;
  sessionActive: boolean;
  sessionExpiresAt?: string;
  loadedDeviceCount: number;
}

/**
 * Master password change request
 */
export interface MasterPasswordChange {
  oldPassword: string;
  newPassword: string;
  confirmNewPassword: string;
}

/**
 * Security settings for master password
 */
export interface SecuritySettings {
  autoLockTimeout: number; // in minutes (0 = never)
}

/**
 * Master password configuration
 */
export interface MasterPasswordConfig {
  autoUnlock: boolean;
  autoLockTimeout: number; // in minutes (0 = never)
}

/**
 * Master password configuration update request
 */
export interface MasterPasswordConfigUpdate extends MasterPasswordConfig {
  password?: string; // Required when enabling auto-unlock
}

/**
 * Device information for master password
 */
export interface CurrentDevice {
  deviceId: string;
  deviceName: string;
  deviceType: string;
  osName: string;
  osVersion: string;
  createdAt: string;
}
