/**
 * Master password setup configuration
 */
export interface MasterPasswordSetup {
  password: string;
  confirmPassword: string;
  deviceName: string;
  autoUnlock: boolean;
  useKeychain: boolean;
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
  useBiometrics?: boolean; // for future implementation
}

/**
 * Master password configuration
 */
export interface MasterPasswordConfig {
  autoUnlock: boolean;
}

/**
 * Device information for master password
 */
export interface CurrentDevice {
  device_id: string;
  device_name: string;
  device_type: string;
  os_name: string;
  os_version: string;
  created_at: string;
}
