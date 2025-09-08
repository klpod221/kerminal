import { SyncService } from './sync-service'
import { SyncConfigStorage } from '../storage/sync-config-storage'
import { SSHProfileStorage } from '../storage/ssh-profile-storage'
import { SSHGroupStorage } from '../storage/ssh-group-storage'
import { SSHTunnelStorage } from '../storage/ssh-tunnel-storage'
import { SavedCommandStorage } from '../storage/saved-command-storage'
import { SyncableStorageRegistry } from '../interfaces/syncable-storage.interface'
import { SyncConfig, SyncStatus } from '../interfaces/sync.interface'
import { ConsoleLogger } from '../utils/logger'
import { AuthService } from './auth-service'

/**
 * Main sync manager that coordinates all sync operations
 */
export class SyncManager {
  private readonly syncService: SyncService
  private readonly syncConfigStorage: SyncConfigStorage
  private readonly storageRegistry: SyncableStorageRegistry
  private readonly logger: ConsoleLogger
  private isInitialized = false

  constructor() {
    this.syncService = new SyncService()
    this.syncConfigStorage = new SyncConfigStorage()
    this.storageRegistry = new SyncableStorageRegistry()
    this.logger = new ConsoleLogger('SyncManager')

    // Set callback for config updates
    this.syncService.setConfigUpdateCallback(async (config: SyncConfig) => {
      await this.syncConfigStorage.saveConfig(config)
    })

    this.initializeStorageRegistry()
  }

  /**
   * Initialize storage registry with all available storages
   */
  private initializeStorageRegistry(): void {
    this.storageRegistry.register('ssh-profiles', new SSHProfileStorage())
    this.storageRegistry.register('ssh-groups', new SSHGroupStorage())
    this.storageRegistry.register('ssh-tunnels', new SSHTunnelStorage())
    this.storageRegistry.register('saved-commands', new SavedCommandStorage())
  }

  /**
   * Initialize sync manager
   */
  async initialize(): Promise<void> {
    if (this.isInitialized) {
      return
    }

    try {
      // Load sync configuration
      const config = await this.syncConfigStorage.getConfig()

      if (config?.enabled) {
        const storages = this.storageRegistry.getAll()
        const success = await this.syncService.initialize(config, storages)
        if (success) {
          this.logger.info('Sync manager initialized with existing configuration')
        } else {
          this.logger.warn('Failed to initialize sync with existing configuration')
        }
      } else {
        this.logger.info('Sync manager initialized without sync configuration')
      }

      this.isInitialized = true
    } catch (error) {
      this.logger.error('Failed to initialize sync manager:', error as Error)
      throw error
    }
  }

  /**
   * Setup sync with MongoDB URI (first time or reconfigure)
   */
  async setupSync(config: SyncConfig): Promise<boolean> {
    try {
      // Test connection first
      const connectionTest = await this.syncService.testConnection(
        config.connectionString || config.mongoUri!,
        config.databaseName || 'kerminal'
      )

      if (!connectionTest) {
        throw new Error('Failed to connect to MongoDB')
      }

      // Check if have existing local data to migrate
      const hasLocalData = await this.hasExistingLocalData()

      // Initialize sync service
      const storages = this.storageRegistry.getAll()
      const initialized = await this.syncService.initialize(config, storages)

      if (!initialized) {
        throw new Error('Failed to initialize sync service')
      }

      // Verify that the connection is actually established
      const status = this.syncService.getStatus()
      if (!status.isConnected) {
        throw new Error('Sync service initialized but not connected')
      }

      // Migrate existing data if needed
      if (hasLocalData) {
        await this.migrateExistingData()
        this.logger.info('Existing local data migrated to MongoDB')
      }

      // Sync master password to MongoDB if not exists
      await this.syncMasterPasswordToMongo(config)

      // Save configuration
      await this.syncConfigStorage.saveConfig(config)

      this.logger.info('Sync setup completed successfully')
      return true
    } catch (error) {
      this.logger.error('Sync setup failed:', error as Error)
      return false
    }
  }

  /**
   * Setup sync with master password verification (for existing data scenario)
   */
  async setupSyncWithPassword(config: SyncConfig, masterPassword: string): Promise<boolean> {
    try {
      // Create auth service instance
      const authService = new AuthService()

      // First verify the provided master password
      const isValidPassword = await authService.unlockWithMasterPassword(masterPassword)
      if (!isValidPassword) {
        throw new Error('Incorrect master password')
      }

      // Test connection first
      const connectionTest = await this.syncService.testConnection(
        config.connectionString || config.mongoUri!,
        config.databaseName || 'kerminal'
      )

      if (!connectionTest) {
        throw new Error('Failed to connect to MongoDB')
      }

      // Check if have existing local data to migrate
      const hasLocalData = await this.hasExistingLocalData()

      // Initialize sync service
      const storages = this.storageRegistry.getAll()
      const initialized = await this.syncService.initialize(config, storages)

      if (!initialized) {
        throw new Error('Failed to initialize sync service')
      }

      // Verify that the connection is actually established
      const status = this.syncService.getStatus()
      if (!status.isConnected) {
        throw new Error('Sync service initialized but not connected')
      }

      // Migrate existing data if needed
      if (hasLocalData) {
        await this.migrateExistingData()
        this.logger.info('Existing local data migrated to MongoDB')
      }

      // Connect to MongoDB master password (this will re-encrypt data with MongoDB password)
      const connectSuccess = await authService.connectToMongoMasterPassword(
        config.mongoUri!,
        config.databaseName || 'kerminal',
        masterPassword
      )

      if (!connectSuccess) {
        throw new Error('Failed to connect with MongoDB master password')
      }

      // Save configuration
      await this.syncConfigStorage.saveConfig(config)

      this.logger.info('Sync setup with password completed successfully')
      return true
    } catch (error) {
      this.logger.error('Sync setup with password failed:', error as Error)
      throw error
    }
  }

  /**
   * Check if there's existing local data to migrate
   */
  private async hasExistingLocalData(): Promise<boolean> {
    const storages = this.storageRegistry.getAll()

    for (const { storage } of storages) {
      const data = await storage.readData()
      if (data.length > 0) {
        return true
      }
    }

    return false
  }

  /**
   * Sync master password to MongoDB during setup
   */
  private async syncMasterPasswordToMongo(config: SyncConfig): Promise<void> {
    try {
      if (!config.mongoUri || !config.databaseName) {
        this.logger.warn('MongoDB URI or database name not provided for master password sync')
        return
      }

      // Import AuthService to sync master password
      const authService = new AuthService()

      const success = await authService.syncMasterPasswordToMongo(
        config.mongoUri,
        config.databaseName
      )
      if (success) {
        this.logger.info('Master password synced to MongoDB successfully')
      } else {
        this.logger.warn('Master password sync to MongoDB skipped or failed')
      }
    } catch (error) {
      this.logger.error('Failed to sync master password to MongoDB:', error as Error)
      // Don't throw error as this is not critical for sync setup
    }
  }

  /**
   * Check if there's existing local data (public method)
   */
  async hasExistingData(): Promise<boolean> {
    return await this.hasExistingLocalData()
  }

  /**
   * Migrate existing local data to MongoDB
   */
  async migrateExistingData(): Promise<void> {
    const storages = this.storageRegistry.getAll()
    await this.syncService.migrateLocalDataToMongo(storages)
  }

  /**
   * Perform manual sync
   */
  async performSync(): Promise<void> {
    const storages = this.storageRegistry.getAll()
    await this.syncService.performFullSync(storages)
  }

  /**
   * Test MongoDB connection
   */
  async testConnection(mongoUri: string, databaseName: string): Promise<boolean> {
    return await this.syncService.testConnection(mongoUri, databaseName)
  }

  /**
   * Enable sync
   */
  async enableSync(config: SyncConfig): Promise<boolean> {
    const storages = this.storageRegistry.getAll()
    config.enabled = true
    const success = await this.syncService.initialize(config, storages)
    if (success) {
      await this.syncConfigStorage.saveConfig(config)
    }
    return success
  }

  /**
   * Disable sync
   */
  async disableSync(): Promise<void> {
    // Update config to disabled but keep connection details
    const config = await this.syncConfigStorage.getConfig()
    if (config) {
      config.enabled = false
      config.autoSync = false
      await this.syncConfigStorage.saveConfig(config)

      // Reinitialize with disabled config
      await this.syncService.initialize(config, [])
    }
  }

  /**
   * Get current sync status
   */
  getSyncStatus(): SyncStatus {
    return this.syncService.getStatus()
  }

  /**
   * Get current sync configuration
   */
  async getSyncConfig(): Promise<SyncConfig | null> {
    return await this.syncConfigStorage.getConfig()
  }

  /**
   * Update sync configuration
   */
  async updateSyncConfig(config: SyncConfig): Promise<boolean> {
    const storages = this.storageRegistry.getAll()
    const success = await this.syncService.initialize(config, storages)
    if (success) {
      await this.syncConfigStorage.saveConfig(config)
    }
    return success
  }

  /**
   * Check if sync is configured and enabled
   */
  async isSyncEnabled(): Promise<boolean> {
    const config = await this.syncConfigStorage.getConfig()
    return config?.enabled === true
  }

  /**
   * Get storage registry (for advanced usage)
   */
  getStorageRegistry(): SyncableStorageRegistry {
    return this.storageRegistry
  }

  /**
   * Force immediate sync
   */
  async forceSyncNow(): Promise<void> {
    const storages = this.storageRegistry.getAll()
    await this.syncService.performFullSync(storages)
  }

  /**
   * Delete sync configuration and disable sync
   */
  async deleteSyncConfig(): Promise<void> {
    await this.disableSync()
    await this.syncConfigStorage.deleteConfig()
    this.logger.info('Sync configuration deleted')
  }

  /**
   * Check if auto sync is currently running
   */
  async isAutoSyncRunning(): Promise<boolean> {
    const config = await this.syncConfigStorage.getConfig()
    const status = this.syncService.getStatus()
    return status.isConnected === true && config?.enabled === true && config?.autoSync === true
  }
}
