import { app } from 'electron'
import * as keytar from 'keytar'
import * as path from 'path'
import * as fs from 'fs/promises'
import { CryptoService } from './crypto-service'
import { ConsoleLogger } from '../utils/logger'

export interface SecuritySettings {
  requirePasswordOnStart: boolean
  autoLockTimeout: number // in minutes (0 = never)
  useBiometrics?: boolean // for future implementation
}

export interface MasterPasswordData {
  salt: string // base64 encoded salt
  verificationHash: string // hash for password verification
  settings: SecuritySettings
}

/**
 * Service for managing authentication state and master password
 * Handles locked/unlocked state and keychain integration
 */
export class AuthService {
  private readonly logger = new ConsoleLogger('AuthService')
  private readonly cryptoService: CryptoService
  private readonly serviceName = 'kerminal'
  private readonly accountName = 'master-key'
  private readonly configFile: string

  // In-memory state
  private derivedKey: Buffer | null = null
  private isUnlockedState = false
  private autoLockTimer: NodeJS.Timeout | null = null

  constructor() {
    this.cryptoService = new CryptoService()
    this.configFile = path.join(app.getPath('userData'), 'master-password.json')
  }

  /**
   * Check if a master password has been set
   */
  async hasMasterPassword(): Promise<boolean> {
    try {
      await fs.access(this.configFile)
      return true
    } catch {
      return false
    }
  }

  /**
   * Create a new master password
   * @param password - The master password
   * @param settings - Security settings
   */
  async createMasterPassword(password: string, settings: SecuritySettings): Promise<void> {
    try {
      this.logger.info('Creating new master password')

      // Derive key and create verification hash
      const { key, salt } = this.cryptoService.deriveKey(password)
      const verificationHash = this.cryptoService.createVerificationHash(password, salt)

      // Store configuration
      const masterPasswordData: MasterPasswordData = {
        salt: salt.toString('base64'),
        verificationHash,
        settings
      }

      await fs.writeFile(this.configFile, JSON.stringify(masterPasswordData, null, 2))

      // Set internal state
      this.derivedKey = key
      this.isUnlockedState = true

      // Store in keychain if auto-unlock is enabled
      if (!settings.requirePasswordOnStart) {
        await this.storeKeyInKeychain(key)
      }

      // Setup auto-lock timer
      this.setupAutoLockTimer(settings.autoLockTimeout)

      this.logger.info('Master password created successfully')
    } catch (error) {
      this.logger.error('Failed to create master password:', error as Error)
      throw new Error('Failed to create master password')
    }
  }

  /**
   * Unlock the application with master password
   * @param password - The master password
   * @returns True if unlock was successful
   */
  async unlockWithMasterPassword(password: string): Promise<boolean> {
    try {
      this.logger.info('Attempting to unlock with master password')

      const config = await this.loadMasterPasswordConfig()
      if (!config) {
        throw new Error('No master password configuration found')
      }

      // Verify password
      const salt = Buffer.from(config.salt, 'base64')
      const isValid = this.cryptoService.verifyMasterPassword(
        password,
        config.verificationHash,
        salt
      )

      if (!isValid) {
        this.logger.warn('Invalid master password provided')
        return false
      }

      // Derive key
      const { key } = this.cryptoService.deriveKey(password, salt)

      // Set internal state
      this.derivedKey = key
      this.isUnlockedState = true

      // Store in keychain if auto-unlock is enabled
      if (!config.settings.requirePasswordOnStart) {
        await this.storeKeyInKeychain(key)
      }

      // Setup auto-lock timer
      this.setupAutoLockTimer(config.settings.autoLockTimeout)

      this.logger.info('Successfully unlocked with master password')
      return true
    } catch (error) {
      this.logger.error('Failed to unlock with master password:', error as Error)
      return false
    }
  }

  /**
   * Attempt to unlock using stored keychain key
   * @returns True if unlock was successful
   */
  async unlockWithKeychain(): Promise<boolean> {
    try {
      this.logger.info('Attempting to unlock with keychain')

      const config = await this.loadMasterPasswordConfig()
      if (!config) {
        this.logger.warn('No master password configuration found')
        return false
      }

      // Check if auto-unlock is disabled
      if (config.settings.requirePasswordOnStart) {
        this.logger.info('Auto-unlock is disabled by user settings')
        return false
      }

      // Try to get key from keychain
      const storedKeyString = await keytar.getPassword(this.serviceName, this.accountName)
      if (!storedKeyString) {
        this.logger.info('No key found in keychain')
        return false
      }

      const derivedKey = Buffer.from(storedKeyString, 'base64')

      // Set internal state
      this.derivedKey = derivedKey
      this.isUnlockedState = true

      // Setup auto-lock timer
      this.setupAutoLockTimer(config.settings.autoLockTimeout)

      this.logger.info('Successfully unlocked with keychain')
      return true
    } catch (error) {
      this.logger.error('Failed to unlock with keychain:', error as Error)
      return false
    }
  }

  /**
   * Lock the application
   */
  async lock(): Promise<void> {
    try {
      this.logger.info('Locking application')

      // Clear in-memory state
      this.derivedKey = null
      this.isUnlockedState = false

      // Clear auto-lock timer
      if (this.autoLockTimer) {
        clearTimeout(this.autoLockTimer)
        this.autoLockTimer = null
      }

      // Remove key from keychain
      await this.removeKeyFromKeychain()

      this.logger.info('Application locked successfully')
    } catch (error) {
      this.logger.error('Failed to lock application:', error as Error)
      throw new Error('Failed to lock application')
    }
  }

  /**
   * Get the current encryption key if unlocked
   * @returns The derived key if unlocked, null otherwise
   */
  getEncryptionKey(): Buffer | null {
    return this.isUnlockedState ? this.derivedKey : null
  }

  /**
   * Check if the application is currently unlocked
   */
  isUnlocked(): boolean {
    return this.isUnlockedState
  }

  /**
   * Get current security settings
   */
  async getSecuritySettings(): Promise<SecuritySettings | null> {
    const config = await this.loadMasterPasswordConfig()
    return config?.settings || null
  }

  /**
   * Update security settings
   * @param settings - New security settings
   */
  async updateSecuritySettings(settings: SecuritySettings): Promise<void> {
    try {
      this.logger.info('Updating security settings')

      const config = await this.loadMasterPasswordConfig()
      if (!config) {
        throw new Error('No master password configuration found')
      }

      // Update settings
      config.settings = settings

      // Save updated configuration
      await fs.writeFile(this.configFile, JSON.stringify(config, null, 2))

      // Handle keychain storage based on new settings
      if (this.derivedKey) {
        if (settings.requirePasswordOnStart) {
          // Remove key from keychain
          await this.removeKeyFromKeychain()
        } else {
          // Store key in keychain
          await this.storeKeyInKeychain(this.derivedKey)
        }
      }

      // Update auto-lock timer
      this.setupAutoLockTimer(settings.autoLockTimeout)

      this.logger.info('Security settings updated successfully')
    } catch (error) {
      this.logger.error('Failed to update security settings:', error as Error)
      throw new Error('Failed to update security settings')
    }
  }

  /**
   * Change master password
   * @param currentPassword - Current master password
   * @param newPassword - New master password
   * @returns True if password was changed successfully
   */
  async changeMasterPassword(currentPassword: string, newPassword: string): Promise<boolean> {
    try {
      this.logger.info('Attempting to change master password')

      // Verify current password first
      const isCurrentValid = await this.unlockWithMasterPassword(currentPassword)
      if (!isCurrentValid) {
        return false
      }

      const config = await this.loadMasterPasswordConfig()
      if (!config) {
        throw new Error('No master password configuration found')
      }

      // Create new derived key and verification hash
      const { key: newKey, salt: newSalt } = this.cryptoService.deriveKey(newPassword)
      const newVerificationHash = this.cryptoService.createVerificationHash(newPassword, newSalt)

      // Update configuration
      config.salt = newSalt.toString('base64')
      config.verificationHash = newVerificationHash

      await fs.writeFile(this.configFile, JSON.stringify(config, null, 2))

      // Update internal state
      this.derivedKey = newKey

      // Update keychain if needed
      if (!config.settings.requirePasswordOnStart) {
        await this.storeKeyInKeychain(newKey)
      }

      this.logger.info('Master password changed successfully')
      return true
    } catch (error) {
      this.logger.error('Failed to change master password:', error as Error)
      return false
    }
  }

  /**
   * Reset auto-lock timer (called on user activity)
   */
  resetAutoLockTimer(): void {
    if (!this.isUnlockedState) return

    this.loadMasterPasswordConfig().then((config) => {
      if (config?.settings.autoLockTimeout) {
        this.setupAutoLockTimer(config.settings.autoLockTimeout)
      }
    })
  }

  /**
   * Load master password configuration from file
   */
  private async loadMasterPasswordConfig(): Promise<MasterPasswordData | null> {
    try {
      const content = await fs.readFile(this.configFile, 'utf-8')
      return JSON.parse(content)
    } catch {
      return null
    }
  }

  /**
   * Store derived key in keychain
   */
  private async storeKeyInKeychain(key: Buffer): Promise<void> {
    try {
      await keytar.setPassword(this.serviceName, this.accountName, key.toString('base64'))
    } catch (error) {
      this.logger.error('Failed to store key in keychain:', error as Error)
      // Don't throw - keychain failure shouldn't break the app
    }
  }

  /**
   * Remove key from keychain
   */
  private async removeKeyFromKeychain(): Promise<void> {
    try {
      await keytar.deletePassword(this.serviceName, this.accountName)
    } catch (error) {
      this.logger.error('Failed to remove key from keychain:', error as Error)
      // Don't throw - keychain failure shouldn't break the app
    }
  }

  /**
   * Setup auto-lock timer
   * @param timeoutMinutes - Timeout in minutes (0 = disabled)
   */
  private setupAutoLockTimer(timeoutMinutes: number): void {
    // Clear existing timer
    if (this.autoLockTimer) {
      clearTimeout(this.autoLockTimer)
      this.autoLockTimer = null
    }

    // Don't set timer if disabled
    if (timeoutMinutes <= 0) {
      return
    }

    // Set new timer
    this.autoLockTimer = setTimeout(
      () => {
        this.logger.info('Auto-lock triggered')
        this.lock().catch((error) => {
          this.logger.error('Auto-lock failed:', error)
        })
      },
      timeoutMinutes * 60 * 1000
    )
  }
}
