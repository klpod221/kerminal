import { SyncService } from './sync-service'
import { SyncConfigStorage } from '../storage/sync-config-storage'
import { SSHProfileStorage } from '../storage/ssh-profile-storage'
import { SSHGroupStorage } from '../storage/ssh-group-storage'
import { SavedCommandStorage } from '../storage/saved-command-storage'
import { SyncableStorageRegistry } from '../interfaces/syncable-storage.interface'
import { SyncConfig, SyncStatus } from '../interfaces/sync.interface'
import { ConsoleLogger } from '../utils/logger'

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

    this.initializeStorageRegistry()
  }

  /**
   * Initialize storage registry with all available storages
   */
  private initializeStorageRegistry(): void {
    this.storageRegistry.register('ssh-profiles', new SSHProfileStorage())
    this.storageRegistry.register('ssh-groups', new SSHGroupStorage())
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
        const success = await this.syncService.initialize(config)
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
        config.mongoUri,
        config.databaseName
      )

      if (!connectionTest) {
        throw new Error('Failed to connect to MongoDB')
      }

      // Check if we have existing local data to migrate
      const hasLocalData = await this.hasExistingLocalData()

      // Initialize sync service
      const initialized = await this.syncService.initialize(config)

      if (!initialized) {
        throw new Error('Failed to initialize sync service')
      }

      // Migrate existing data if needed
      if (hasLocalData) {
        await this.migrateExistingData()
        this.logger.info('Existing local data migrated to MongoDB')
      }

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
    const success = await this.syncService.enableSync(config)
    if (success) {
      await this.syncConfigStorage.saveConfig(config)
    }
    return success
  }

  /**
   * Disable sync
   */
  async disableSync(): Promise<void> {
    await this.syncService.disableSync()

    // Update config to disabled but keep connection details
    const config = await this.syncConfigStorage.getConfig()
    if (config) {
      config.enabled = false
      await this.syncConfigStorage.saveConfig(config)
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
    const success = await this.syncService.updateConfig(config)
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
    await this.syncService.forceSyncNow(storages)
  }

  /**
   * Delete sync configuration and disable sync
   */
  async deleteSyncConfig(): Promise<void> {
    await this.disableSync()
    await this.syncConfigStorage.deleteConfig()
    this.logger.info('Sync configuration deleted')
  }
}
