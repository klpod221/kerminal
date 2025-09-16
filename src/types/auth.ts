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
  requirePasswordOnStart: boolean;
  autoLockTimeout: number; // in minutes (0 = never)
  sessionTimeoutMinutes?: number;
  useBiometrics?: boolean; // for future implementation
}

/**
 * Master password configuration
 */
export interface MasterPasswordConfig {
  autoUnlock: boolean;
  useKeychain: boolean;
  sessionTimeoutMinutes?: number;
  securitySettings: SecuritySettings;
}

/**
 * Device information for master password
 */
export interface DeviceInfo {
  id: string;
  name: string;
  isCurrentDevice: boolean;
  lastVerified?: string;
  created: string;
}
