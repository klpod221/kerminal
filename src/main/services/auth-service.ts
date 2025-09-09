import { app } from 'electron'
import * as keytar from 'keytar'
import * as path from 'path'
import * as fs from 'fs/promises'
import { CryptoService } from './crypto-service'
import { ConsoleLogger } from '../utils/logger'
import { MongoDBService } from './mongodb-service'
import { SyncManager } from './sync-manager'
import { SyncService } from './sync-service'
import { SyncConfig } from '../interfaces/sync.interface'

// ============================================================================
// INTERFACES
// ============================================================================

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

// ============================================================================
// AUTH SERVICE CLASS
// ============================================================================

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

  // ============================================================================
  // MASTER PASSWORD MANAGEMENT
  // ============================================================================

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
   * Connect to MongoDB and verify master password
   * @param mongoUri - MongoDB connection URI
   * @param databaseName - Database name
   * @param masterPassword - The master password from MongoDB
   * @returns True if connection and password are valid
   */
  async connectToMongoMasterPassword(
    mongoUri: string,
    databaseName: string,
    masterPassword: string
  ): Promise<boolean> {
    try {
      this.logger.info('Attempting to connect to MongoDB and verify master password')

      // Use imported MongoDBService directly
      const mongoService = new MongoDBService()

      // Set temporary config
      const tempConfig = {
        id: 'temp-mongo-test',
        provider: 'mongodb' as const,
        mongoUri,
        databaseName,
        enabled: true,
        autoSync: false,
        syncInterval: 30,
        created: new Date(),
        updated: new Date()
      }

      mongoService.setConfig(tempConfig)

      // Test connection
      const connected = await mongoService.connect()
      if (!connected) {
        this.logger.error('Failed to connect to MongoDB')
        return false
      }

      // Check if master-password collection exists and verify password
      const masterPasswordValid = await this.verifyMongoMasterPassword(mongoService, masterPassword)

      await mongoService.disconnect()

      if (masterPasswordValid) {
        // If verification successful, create local master password with MongoDB key
        await this.createMasterPasswordFromMongo(masterPassword, mongoUri, databaseName)
        return true
      }

      return false
    } catch (error) {
      this.logger.error('Failed to connect to MongoDB master password:', error as Error)
      return false
    }
  }

  /**
   * Create new master password for empty MongoDB database
   * @param mongoUri - MongoDB connection URI
   * @param databaseName - Database name
   * @param masterPassword - The new master password to create
   * @returns True if creation was successful
   */
  async createNewMongoMasterPassword(
    mongoUri: string,
    databaseName: string,
    masterPassword: string
  ): Promise<boolean> {
    try {
      this.logger.info('Creating new master password for MongoDB database')

      // Use imported MongoDBService directly
      const mongoService = new MongoDBService()

      // Set temporary config
      const tempConfig = {
        id: 'temp-mongo-create',
        provider: 'mongodb' as const,
        mongoUri,
        databaseName,
        enabled: true,
        autoSync: false,
        syncInterval: 30,
        created: new Date(),
        updated: new Date()
      }

      mongoService.setConfig(tempConfig)

      // Test connection
      const connected = await mongoService.connect()
      if (!connected) {
        this.logger.error('Failed to connect to MongoDB')
        return false
      }

      // Create master password in MongoDB
      await this.createMasterPasswordInMongo(mongoService, masterPassword)

      await mongoService.disconnect()

      // Create local master password configuration
      await this.createMasterPasswordFromMongo(masterPassword, mongoUri, databaseName)

      this.logger.info('New MongoDB master password created successfully')
      return true
    } catch (error) {
      this.logger.error('Failed to create new MongoDB master password:', error as Error)
      return false
    }
  }

  /**
   * Create master password document in MongoDB
   * @param mongoService - MongoDB service instance
   * @param password - Master password to store
   */
  private async createMasterPasswordInMongo(
    mongoService: MongoDBService,
    password: string
  ): Promise<void> {
    try {
      // Derive key and create verification hash
      const { salt } = this.cryptoService.deriveKey(password)
      const verificationHash = this.cryptoService.createVerificationHash(password, salt)

      // Store in MongoDB master-password collection
      const db = mongoService.getDatabase()
      const collection = db.collection('master-password')

      const masterPasswordDoc = {
        salt: salt.toString('base64'),
        verificationHash,
        createdAt: new Date(),
        updatedAt: new Date()
      }

      await collection.insertOne(masterPasswordDoc)

      this.logger.info('Master password created in MongoDB successfully')
    } catch (error) {
      this.logger.error('Failed to create master password in MongoDB:', error as Error)
      throw error
    }
  }

  /**
   * Verify master password against MongoDB data
   * @param mongoService - MongoDB service instance
   * @param password - Password to verify
   * @returns True if password is valid
   */
  private async verifyMongoMasterPassword(
    mongoService: MongoDBService,
    password: string
  ): Promise<boolean> {
    try {
      // Check if master-password collection exists
      const db = mongoService.getDatabase()
      const collection = db.collection('master-password')

      const masterPasswordDoc = await collection.findOne({})

      if (!masterPasswordDoc) {
        this.logger.info('No master password found in MongoDB')
        return false
      }

      // Verify password using stored salt and verification hash
      const salt = Buffer.from(masterPasswordDoc.salt, 'base64')
      const storedHash = masterPasswordDoc.verificationHash

      return this.cryptoService.verifyMasterPassword(password, storedHash, salt)
    } catch (error) {
      this.logger.error('Error verifying MongoDB master password:', error as Error)
      return false
    }
  }

  // ============================================================================
  // MONGODB PASSWORD VERIFICATION
  // ============================================================================

  /**
   * Verify master password against MongoDB database
   * @param mongoUri - MongoDB URI
   * @param databaseName - Database name
   * @param password - Password to verify
   * @returns True if password is valid
   */
  async verifyMongoPassword(
    mongoUri: string,
    databaseName: string,
    password: string
  ): Promise<boolean> {
    try {
      this.logger.info('Verifying master password against MongoDB')

      // Create MongoDB service instance
      const mongoService = new MongoDBService()

      // Set temporary config for verification
      const tempConfig: SyncConfig = {
        id: 'temp-mongo-verify',
        provider: 'mongodb' as const,
        mongoUri,
        databaseName,
        enabled: true,
        autoSync: false,
        syncInterval: 30,
        created: new Date(),
        updated: new Date()
      }

      // Connect to MongoDB
      mongoService.setConfig(tempConfig)
      const connected = await mongoService.connect()

      if (!connected) {
        throw new Error('Failed to connect to MongoDB')
      }

      // Verify password
      const isValid = await this.verifyMongoMasterPassword(mongoService, password)

      // Clean up
      await mongoService.disconnect()

      return isValid
    } catch (error) {
      this.logger.error('Error verifying MongoDB master password:', error as Error)
      return false
    }
  }

  /**
   * Create local master password configuration from MongoDB data
   * @param password - The verified master password
   * @param mongoUri - MongoDB URI for sync config
   * @param databaseName - Database name for sync config
   */
  private async createMasterPasswordFromMongo(
    password: string,
    mongoUri: string,
    databaseName: string
  ): Promise<void> {
    try {
      // Derive key from password (same as MongoDB)
      const { key, salt } = this.cryptoService.deriveKey(password)
      const verificationHash = this.cryptoService.createVerificationHash(password, salt)

      // Create default security settings
      const settings: SecuritySettings = {
        requirePasswordOnStart: true,
        autoLockTimeout: 15
      }

      // Store local configuration
      const masterPasswordData: MasterPasswordData = {
        salt: salt.toString('base64'),
        verificationHash,
        settings
      }

      await fs.writeFile(this.configFile, JSON.stringify(masterPasswordData, null, 2))

      // Set internal state
      this.derivedKey = key
      this.isUnlockedState = true

      // Setup sync configuration
      await this.setupSyncConfiguration(mongoUri, databaseName, password)

      this.logger.info('Master password created from MongoDB successfully')
    } catch (error) {
      this.logger.error('Failed to create master password from MongoDB:', error as Error)
      throw error
    }
  }

  /**
   * Setup sync configuration after MongoDB connection
   * @param mongoUri - MongoDB URI
   * @param databaseName - Database name
   * @param masterPassword - Master password for encryption
   */
  private async setupSyncConfiguration(
    mongoUri: string,
    databaseName: string,
    masterPassword: string
  ): Promise<void> {
    try {
      // Use imported SyncManager and SyncService directly
      const syncManager = new SyncManager()

      // Create proper sync config using SyncService helper
      const syncConfig = SyncService.createSyncConfigFromMongo({
        mongoUri,
        databaseName,
        masterPassword
      })

      await syncManager.setupSync(syncConfig)
      this.logger.info('Sync configuration setup completed with auto-sync enabled')
    } catch (error) {
      this.logger.error('Failed to setup sync configuration:', error as Error)
      // Don't throw error here as master password creation was successful
    }
  }

  /**
   * Check if MongoDB has existing master password data
   * @param mongoUri - MongoDB connection URI
   * @param databaseName - Database name
   * @returns True if master password data exists
   */
  async checkMongoMasterPasswordExists(mongoUri: string, databaseName: string): Promise<boolean> {
    try {
      const mongoService = new MongoDBService()

      const tempConfig = {
        id: 'temp-check-master-password',
        provider: 'mongodb' as const,
        mongoUri,
        databaseName,
        enabled: true,
        autoSync: false,
        syncInterval: 30,
        created: new Date(),
        updated: new Date()
      }

      mongoService.setConfig(tempConfig)

      const connected = await mongoService.connect()
      if (!connected) {
        return false
      }

      const db = mongoService.getDatabase()
      const collection = db.collection('master-password')
      const doc = await collection.findOne({})

      await mongoService.disconnect()

      return doc !== null
    } catch (error) {
      this.logger.error('Error checking MongoDB master password:', error as Error)
      return false
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

  /**
   * Sync local master password to MongoDB (when setting up sync with new MongoDB)
   * @param mongoUri - MongoDB connection URI
   * @param databaseName - Database name
   * @returns True if sync was successful
   */
  async syncMasterPasswordToMongo(mongoUri: string, databaseName: string): Promise<boolean> {
    try {
      // Check if we have local master password data
      if (!(await this.hasMasterPassword())) {
        this.logger.warn('No local master password found to sync')
        return false
      }

      // Read local master password data
      const data = await fs.readFile(this.configFile, 'utf8')
      const masterPasswordData: MasterPasswordData = JSON.parse(data)

      // Connect to MongoDB
      const mongoService = new MongoDBService()
      const tempConfig = {
        id: 'temp-sync-master-password',
        provider: 'mongodb' as const,
        mongoUri,
        databaseName,
        enabled: true,
        autoSync: false,
        syncInterval: 30,
        created: new Date(),
        updated: new Date()
      }

      mongoService.setConfig(tempConfig)
      const connected = await mongoService.connect()
      if (!connected) {
        this.logger.error('Failed to connect to MongoDB for master password sync')
        return false
      }

      // Check if master password already exists in MongoDB
      const db = mongoService.getDatabase()
      const collection = db.collection('master-password')
      const existingDoc = await collection.findOne({})

      if (existingDoc) {
        this.logger.info('Master password already exists in MongoDB, skipping sync')
        await mongoService.disconnect()
        return true
      }

      // Sync local master password to MongoDB
      const masterPasswordDoc = {
        salt: masterPasswordData.salt,
        verificationHash: masterPasswordData.verificationHash,
        createdAt: new Date(),
        updatedAt: new Date(),
        syncedFromLocal: true // Flag to indicate this was synced from local
      }

      await collection.insertOne(masterPasswordDoc)
      await mongoService.disconnect()

      this.logger.info('Local master password synced to MongoDB successfully')
      return true
    } catch (error) {
      this.logger.error('Failed to sync master password to MongoDB:', error as Error)
      return false
    }
  }
}
