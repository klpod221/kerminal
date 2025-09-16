/**
 * Master password setup configuration
 */
export interface MasterPasswordSetup {
  password: string
  confirmPassword: string
  deviceName: string
  autoUnlock: boolean
  useKeychain: boolean
}

/**
 * Master password verification request
 */
export interface MasterPasswordVerification {
  password: string
  deviceId?: string
}

/**
 * Master password status information
 */
export interface MasterPasswordStatus {
  isSetup: boolean
  isUnlocked: boolean
  autoUnlockEnabled: boolean
  keychainAvailable: boolean
  sessionActive: boolean
  sessionExpiresAt?: string
  loadedDeviceCount: number
}

/**
 * Master password change request
 */
export interface MasterPasswordChange {
  oldPassword: string
  newPassword: string
  confirmNewPassword: string
}

/**
 * Security settings for master password
 */
export interface SecuritySettings {
  requirePasswordOnStart: boolean
  autoLockTimeout: number // in minutes (0 = never)
  sessionTimeoutMinutes?: number
  useBiometrics?: boolean // for future implementation
}

/**
 * Master password configuration
 */
export interface MasterPasswordConfig {
  autoUnlock: boolean
  useKeychain: boolean
  sessionTimeoutMinutes?: number
  securitySettings: SecuritySettings
}

/**
 * Device information for master password
 */
export interface DeviceInfo {
  id: string
  name: string
  isCurrentDevice: boolean
  lastVerified?: string
  created: string
}

/**
 * Master password validation errors
 */
export enum MasterPasswordError {
  PASSWORD_TOO_SHORT = 'PASSWORD_TOO_SHORT',
  PASSWORD_TOO_LONG = 'PASSWORD_TOO_LONG',
  PASSWORD_MISMATCH = 'PASSWORD_MISMATCH',
  PASSWORD_TOO_COMMON = 'PASSWORD_TOO_COMMON',
  PASSWORD_MISSING_REQUIREMENTS = 'PASSWORD_MISSING_REQUIREMENTS',
  DEVICE_NAME_EMPTY = 'DEVICE_NAME_EMPTY',
  KEYCHAIN_NOT_AVAILABLE = 'KEYCHAIN_NOT_AVAILABLE',
  VERIFICATION_FAILED = 'VERIFICATION_FAILED',
  SESSION_EXPIRED = 'SESSION_EXPIRED',
  NOT_SETUP = 'NOT_SETUP'
}

/**
 * Master password validation result
 */
export interface PasswordValidation {
  isValid: boolean
  errors: MasterPasswordError[]
  score: number // 0-4 (weak to strong)
  suggestions: string[]
}

/**
 * Master password form states
 */
export enum MasterPasswordFormState {
  IDLE = 'IDLE',
  LOADING = 'LOADING',
  SUCCESS = 'SUCCESS',
  ERROR = 'ERROR'
}

/**
 * Master password events
 */
export interface MasterPasswordEvents {
  onSetup?: () => void
  onUnlock?: () => void
  onLock?: () => void
  onChange?: () => void
  onError?: (error: string) => void
  onSessionExpired?: () => void
}
