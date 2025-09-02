import { SyncConfig, SyncStatus, ConflictResolution } from '../interfaces/sync.interface'
import { ISyncableStorage } from '../interfaces/syncable-storage.interface'
import { MongoDBService } from './mongodb-service'
import { ConsoleLogger } from '../utils/logger'
import { Document } from 'mongodb'

/**
 * Sync service for managing data synchronization between local files and MongoDB
 */
export class SyncService {
  private readonly mongoService: MongoDBService
  private readonly logger: ConsoleLogger
  private config: SyncConfig | null = null
  private syncInterval: NodeJS.Timeout | null = null
  private readonly status: SyncStatus = {
    isConnected: false,
    lastSync: undefined,
    lastError: '',
    isLoading: false
  }

  constructor() {
    this.mongoService = new MongoDBService()
    this.logger = new ConsoleLogger('Sync')
  }

  /**
   * Initialize sync service with configuration
   */
  async initialize(config: SyncConfig): Promise<boolean> {
    this.config = config
    this.mongoService.setConfig(config)

    if (!config.enabled) {
      this.logger.info('Sync is disabled')
      return true
    }

    try {
      this.status.isLoading = true
      const connected = await this.mongoService.connect()

      if (connected) {
        this.status.isConnected = true
        this.status.lastSync = new Date()

        if (config.autoSync) {
          this.startAutoSync()
        }

        this.logger.info('Sync service initialized successfully')
        return true
      } else {
        this.status.lastError = 'Failed to connect to MongoDB'
        return false
      }
    } catch (error) {
      this.status.lastError = (error as Error).message
      this.logger.error('Failed to initialize sync service:', error as Error)
      return false
    } finally {
      this.status.isLoading = false
    }
  }

  /**
   * Test MongoDB connection
   */
  async testConnection(mongoUri: string, databaseName: string): Promise<boolean> {
    const tempConfig: SyncConfig = {
      mongoUri,
      databaseName,
      enabled: true,
      autoSync: false,
      syncInterval: 5
    }

    this.mongoService.setConfig(tempConfig)
    return await this.mongoService.testConnection()
  }

  /**
   * Perform full sync between local and remote data
   */
  async performFullSync(
    storageMap: Array<{ name: string; storage: ISyncableStorage }>
  ): Promise<void> {
    if (!this.config?.enabled || !this.status.isConnected) {
      throw new Error('Sync service not properly initialized')
    }

    this.status.isLoading = true

    try {
      for (const { name, storage } of storageMap) {
        await this.syncStorage(name, storage)
      }

      this.status.lastSync = new Date()
      this.logger.info('Full sync completed successfully')
    } catch (error) {
      this.status.lastError = (error as Error).message
      this.logger.error('Full sync failed:', error as Error)
      throw error
    } finally {
      this.status.isLoading = false
    }
  }

  /**
   * Sync individual storage instance
   */
  private async syncStorage(collectionName: string, storage: ISyncableStorage): Promise<void> {
    // Get local data
    const localData = await storage.readData<Record<string, unknown>>()

    // Get remote data
    const remoteData = await this.mongoService.findAll(collectionName)

    // Perform bidirectional sync
    await this.performBidirectionalSync(storage, localData, remoteData, collectionName)
  }

  /**
   * Perform bidirectional sync between local and remote data
   */
  private async performBidirectionalSync(
    storage: ISyncableStorage,
    localData: Record<string, unknown>[],
    remoteData: Document[],
    collectionName: string
  ): Promise<void> {
    const localMap = new Map(localData.map((item) => [item.id, item]))
    const remoteMap = new Map(remoteData.map((item) => [item._id, item]))

    const allIds = new Set([...localMap.keys(), ...remoteMap.keys()])
    const updatedLocalData = [...localData]

    for (const id of allIds) {
      const localItem = localMap.get(id)
      const remoteItem = remoteMap.get(id)

      await this.syncItem(localItem, remoteItem, id, updatedLocalData, collectionName)
    }

    // Save updated local data
    await storage.writeData(updatedLocalData)
  }

  /**
   * Sync individual item
   */
  private async syncItem(
    localItem: Record<string, unknown> | undefined,
    remoteItem: Document | undefined,
    id: unknown,
    updatedLocalData: Record<string, unknown>[],
    collectionName: string
  ): Promise<void> {
    if (!localItem && remoteItem) {
      // Remote item doesn't exist locally - add to local
      const localFormat = this.convertFromRemoteFormat(remoteItem)
      updatedLocalData.push(localFormat)
      this.logger.debug(`Added remote item to local: ${id}`)
    } else if (localItem && !remoteItem) {
      // Local item doesn't exist remotely - add to remote
      const remoteFormat = this.convertToRemoteFormat(localItem)
      await this.mongoService.insertOne(collectionName, remoteFormat)
      this.logger.debug(`Added local item to remote: ${id}`)
    } else if (localItem && remoteItem) {
      // Both exist - resolve conflict
      await this.handleConflict(localItem, remoteItem, id, updatedLocalData, collectionName)
    }
  }

  /**
   * Handle conflict between local and remote items
   */
  private async handleConflict(
    localItem: Record<string, unknown>,
    remoteItem: Document,
    id: unknown,
    updatedLocalData: Record<string, unknown>[],
    collectionName: string
  ): Promise<void> {
    const resolution = await this.resolveConflict(localItem, remoteItem)

    if (resolution === 'local') {
      // Update remote with local data
      const remoteFormat = this.convertToRemoteFormat(localItem)
      await this.mongoService.replaceOne(collectionName, id as string, remoteFormat)
      this.logger.debug(`Updated remote with local data: ${id}`)
    } else if (resolution === 'remote') {
      // Update local with remote data
      const localFormat = this.convertFromRemoteFormat(remoteItem)
      const index = updatedLocalData.findIndex((item) => item.id === id)
      if (index !== -1) {
        updatedLocalData[index] = localFormat
      }
      this.logger.debug(`Updated local with remote data: ${id}`)
    }
  }

  /**
   * Resolve conflicts between local and remote data
   */
  private async resolveConflict(
    localItem: Record<string, unknown>,
    remoteItem: Document
  ): Promise<ConflictResolution> {
    // Simple timestamp-based resolution - newer wins
    const localUpdated = new Date((localItem.updated || localItem.created) as string)
    const remoteUpdated = new Date(
      (remoteItem._syncedAt || remoteItem.updated || remoteItem.created) as string
    )

    return localUpdated > remoteUpdated ? 'local' : 'remote'
  }

  /**
   * Convert local data format to remote format
   */
  private convertToRemoteFormat(localItem: Record<string, unknown>): Document {
    return {
      _id: localItem.id,
      ...localItem,
      _syncedAt: new Date(),
      _deviceId: this.mongoService.getDeviceId()
    }
  }

  /**
   * Convert remote data format to local format
   */
  private convertFromRemoteFormat(remoteItem: Document): Record<string, unknown> {
    // eslint-disable-next-line @typescript-eslint/no-unused-vars
    const { _id, _syncedAt, _deviceId, ...localData } = remoteItem
    return {
      id: _id,
      ...localData
    }
  }

  /**
   * Get collection name from storage name
   */
  private getCollectionName(name: string): string {
    return name.toLowerCase().replace(/storage$/, '') + 's'
  }

  /**
   * Start automatic sync
   */
  private startAutoSync(): void {
    if (!this.config?.autoSync) return

    this.stopAutoSync()

    const intervalMs = this.config.syncInterval * 1000 // Convert seconds to ms
    this.syncInterval = setInterval(async () => {
      try {
        // Note: We would need to inject storage instances or maintain a registry
        this.logger.info('Performing automatic sync...')
        // await this.performFullSync(storageInstances)
      } catch (error) {
        this.logger.error('Automatic sync failed:', error as Error)
      }
    }, intervalMs)

    this.logger.info(`Auto sync started with interval: ${this.config.syncInterval} seconds`)
  }

  /**
   * Stop automatic sync
   */
  private stopAutoSync(): void {
    if (this.syncInterval) {
      clearInterval(this.syncInterval)
      this.syncInterval = null
      this.logger.info('Auto sync stopped')
    }
  }

  /**
   * Enable sync
   */
  async enableSync(config: SyncConfig): Promise<boolean> {
    return await this.initialize(config)
  }

  /**
   * Disable sync
   */
  async disableSync(): Promise<void> {
    this.stopAutoSync()
    await this.mongoService.disconnect()
    this.status.isConnected = false
    this.config = null
    this.logger.info('Sync disabled')
  }

  /**
   * Get current sync status
   */
  getStatus(): SyncStatus {
    return { ...this.status }
  }

  /**
   * Get sync configuration
   */
  getConfig(): SyncConfig | null {
    return this.config ? { ...this.config } : null
  }

  /**
   * Update sync configuration
   */
  async updateConfig(config: SyncConfig): Promise<boolean> {
    if (this.status.isConnected) {
      await this.disableSync()
    }

    return await this.initialize(config)
  }

  /**
   * Force immediate sync
   */
  async forceSyncNow(
    storageMap: Array<{ name: string; storage: ISyncableStorage }>
  ): Promise<void> {
    if (!this.config?.enabled) {
      throw new Error('Sync is not enabled')
    }

    await this.performFullSync(storageMap)
  }

  /**
   * Migrate existing local data to MongoDB (first-time setup)
   */
  async migrateLocalDataToMongo(
    storageMap: Array<{ name: string; storage: ISyncableStorage }>
  ): Promise<void> {
    if (!this.config?.enabled || !this.status.isConnected) {
      throw new Error('Sync service not properly initialized')
    }

    this.status.isLoading = true

    try {
      for (const { name, storage } of storageMap) {
        const collectionName = this.getCollectionName(name)
        const localData = await storage.readData<Record<string, unknown>>()

        // Check if remote collection is empty (first migration)
        const remoteData = await this.mongoService.findAll(collectionName)

        if (remoteData.length === 0 && localData.length > 0) {
          // Migrate all local data to remote
          for (const item of localData) {
            const remoteFormat = this.convertToRemoteFormat(item)
            await this.mongoService.insertOne(collectionName, remoteFormat)
          }

          this.logger.info(`Migrated ${localData.length} items from ${collectionName} to MongoDB`)
        } else {
          this.logger.info(`Collection ${collectionName} already has data, performing normal sync`)
          await this.syncStorage(collectionName, storage)
        }
      }

      this.status.lastSync = new Date()
      this.logger.info('Data migration completed successfully')
    } catch (error) {
      this.status.lastError = (error as Error).message
      this.logger.error('Data migration failed:', error as Error)
      throw error
    } finally {
      this.status.isLoading = false
    }
  }
}
