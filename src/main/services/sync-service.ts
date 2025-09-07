import {
  SyncConfig,
  SyncStatus,
  ConflictResolution,
  SyncOperationResult,
  SyncRecord,
  TombstoneRecord,
  DeviceInfo
} from '../interfaces/sync.interface'
import { ISyncableStorage } from '../interfaces/syncable-storage.interface'
import { MongoDBService } from './mongodb-service'
import { ConsoleLogger } from '../utils/logger'
import { Document } from 'mongodb'
import { BrowserWindow } from 'electron'
import crypto from 'crypto'
import os from 'os'

/**
 * Sync service for managing data synchronization between local files and MongoDB
 * with improved version control, tombstone handling, and conflict resolution
 */
export class SyncService {
  private readonly mongoService: MongoDBService
  private readonly logger: ConsoleLogger
  private config: SyncConfig | null = null
  private syncInterval: NodeJS.Timeout | null = null
  private storageMap: Array<{ name: string; storage: ISyncableStorage }> = []
  private readonly deviceInfo: DeviceInfo
  private onConfigUpdate?: (config: SyncConfig) => Promise<void>
  private readonly status: SyncStatus = {
    isEnabled: false,
    isConnected: false,
    lastSync: undefined,
    lastSyncStatus: 'never',
    lastError: '',
    isLoading: false,
    totalItems: 0,
    syncedItems: 0,
    conflictCount: 0,
    tombstoneCount: 0,
    syncInProgress: false
  }

  constructor() {
    this.mongoService = new MongoDBService()
    this.logger = new ConsoleLogger('Sync')
    this.deviceInfo = this.generateDeviceInfo()
  }

  /**
   * Set callback for config updates (used by SyncManager to persist config changes)
   */
  setConfigUpdateCallback(callback: (config: SyncConfig) => Promise<void>): void {
    this.onConfigUpdate = callback
  }

  /**
   * Generate unique device information
   */
  private generateDeviceInfo(): DeviceInfo {
    const hostname = os.hostname()
    const platform = os.platform()
    const machineId = this.generateMachineId()

    return {
      id: machineId,
      deviceId: machineId,
      name: hostname,
      platform,
      lastActivity: new Date(),
      syncVersion: process.env.APP_VERSION || '1.0.0'
    }
  }

  /**
   * Generate unique machine ID
   */
  private generateMachineId(): string {
    const networkInterfaces = os.networkInterfaces()
    const macAddresses = Object.values(networkInterfaces)
      .flat()
      .filter((iface) => iface && !iface.internal && iface.mac !== '00:00:00:00:00:00')
      .map((iface) => iface!.mac)

    const uniqueString = `${os.hostname()}-${macAddresses.join('-')}-${os.platform()}`
    return crypto.createHash('sha256').update(uniqueString).digest('hex').substring(0, 32)
  }

  /**
   * Initialize sync service with configuration
   */
  async initialize(
    config: SyncConfig,
    storageMap?: Array<{ name: string; storage: ISyncableStorage }>
  ): Promise<boolean> {
    this.config = config
    this.mongoService.setConfig(config)

    if (storageMap) {
      this.storageMap = storageMap
    }

    if (!config.enabled) {
      this.logger.info('Sync is disabled')
      this.status.isConnected = false
      this.status.lastError = ''
      return true
    }

    try {
      this.status.isLoading = true
      this.status.lastError = ''

      const connected = await this.mongoService.connect()

      if (connected) {
        this.status.isConnected = true
        this.status.lastSync = new Date()

        // Register device info
        await this.registerDevice()

        if (config.autoSync && this.storageMap.length > 0) {
          this.startAutoSync(this.storageMap)
        }

        this.logger.info('Sync service initialized successfully')
        return true
      } else {
        this.status.isConnected = false
        this.status.lastError = 'Failed to connect to MongoDB'
        return false
      }
    } catch (error) {
      this.status.isConnected = false
      this.status.lastError = (error as Error).message
      this.logger.error('Failed to initialize Sync service:', error as Error)
      return false
    } finally {
      this.status.isLoading = false
    }
  }

  /**
   * Register this device in the database
   */
  private async registerDevice(): Promise<void> {
    try {
      const devicesCollection = 'sync_devices'
      const existingDevice = await this.mongoService.findOne(devicesCollection, {
        deviceId: this.deviceInfo.deviceId || this.deviceInfo.id
      })

      if (existingDevice) {
        await this.mongoService.replaceOne(
          devicesCollection,
          this.deviceInfo.deviceId || this.deviceInfo.id,
          {
            ...this.deviceInfo,
            lastSeen: new Date()
          }
        )
      } else {
        await this.mongoService.insertOne(devicesCollection, {
          _id: this.deviceInfo.deviceId || this.deviceInfo.id,
          ...this.deviceInfo
        })
      }
    } catch (error) {
      this.logger.error('Failed to register device:', error as Error)
    }
  }

  /**
   * Test MongoDB connection
   */
  async testConnection(mongoUri: string, databaseName: string): Promise<boolean> {
    const tempConfig: SyncConfig = {
      id: 'temp-test-config',
      provider: 'mongodb',
      mongoUri,
      databaseName,
      enabled: true,
      autoSync: false,
      syncInterval: 5,
      created: new Date(),
      updated: new Date()
    }

    this.mongoService.setConfig(tempConfig)
    return await this.mongoService.testConnection()
  }

  /**
   * Perform full sync for all storage types with enhanced conflict resolution
   */
  async performFullSync(
    storageMap: Array<{ name: string; storage: ISyncableStorage }>
  ): Promise<SyncOperationResult> {
    if (!this.config?.enabled) {
      throw new Error('Sync is not configured or enabled')
    }

    if (this.status.syncInProgress) {
      this.logger.warn('Sync operation already in progress, skipping...')
      return {
        success: false,
        recordsProcessed: 0,
        itemsProcessed: 0,
        conflictsResolved: 0,
        tombstonesProcessed: 0,
        errors: ['Sync already in progress'],
        lastSync: new Date()
      }
    }

    this.status.syncInProgress = true
    this.status.isLoading = true

    const result: SyncOperationResult = {
      success: true,
      recordsProcessed: 0,
      itemsProcessed: 0,
      conflictsResolved: 0,
      tombstonesProcessed: 0,
      errors: [],
      lastSync: new Date()
    }

    try {
      for (const { name, storage } of storageMap) {
        const storageResult = await this.syncStorage(name, storage)

        result.itemsProcessed += storageResult.itemsProcessed
        result.conflictsResolved += storageResult.conflictsResolved
        result.tombstonesProcessed += storageResult.tombstonesProcessed
        result.errors.push(...storageResult.errors)
      }

      // Cleanup old tombstones if configured
      if (this.config.retainTombstoneDays) {
        await this.cleanupOldTombstones(storageMap)
      }

      // Update sync timestamps
      const now = new Date()
      this.status.lastSync = now
      this.status.syncedItems = result.itemsProcessed
      this.status.conflictCount = result.conflictsResolved
      this.status.tombstoneCount = result.tombstonesProcessed

      // Update config lastSync for next sync
      if (this.config) {
        this.config.lastSync = now
        // Persist config changes if callback is available
        if (this.onConfigUpdate) {
          try {
            await this.onConfigUpdate(this.config)
            this.logger.debug('Config lastSync updated and persisted')
          } catch (error) {
            this.logger.warn('Failed to persist config update:', error as Error)
          }
        }
      }

      this.logger.info(
        `Sync completed: ${result.itemsProcessed} items processed, ${result.conflictsResolved} conflicts resolved`
      )

      // Emit sync completion event
      this.emitSyncEvent('sync.completed')
    } catch (error) {
      result.success = false
      result.errors.push((error as Error).message)
      this.status.lastError = (error as Error).message
      this.logger.error('Sync failed:', error as Error)
    } finally {
      this.status.syncInProgress = false
      this.status.isLoading = false
    }

    return result
  }

  /**
   * Sync individual storage with enhanced features
   */
  private async syncStorage(name: string, storage: ISyncableStorage): Promise<SyncOperationResult> {
    const collectionName = this.getCollectionName(name)
    const result: SyncOperationResult = {
      success: true,
      recordsProcessed: 0,
      itemsProcessed: 0,
      conflictsResolved: 0,
      tombstonesProcessed: 0,
      errors: [],
      lastSync: new Date()
    }

    try {
      // Check if storage supports Sync features
      const hasEnhancedSync = this.hasEnhancedSyncSupport(storage)

      if (hasEnhancedSync) {
        // Use Sync with version control and tombstones
        await this.performEnhancedSync(name, storage, collectionName, result)
      } else {
        // Fallback to basic sync
        await this.performBasicSync(name, storage, collectionName, result)
      }
    } catch (error) {
      result.success = false
      result.errors.push(`${name}: ${(error as Error).message}`)
      this.logger.error(`Failed to sync ${name}:`, error as Error)
    }

    return result
  }

  /**
   * Check if storage supports Sync features
   */
  private hasEnhancedSyncSupport(storage: ISyncableStorage): boolean {
    return !!(
      storage.getSyncMetadata &&
      storage.getTombstones &&
      storage.getModifiedSince &&
      storage.generateHash
    )
  }

  /**
   * Perform Sync with version control and tombstones
   */
  private async performEnhancedSync(
    name: string,
    storage: ISyncableStorage,
    collectionName: string,
    result: SyncOperationResult
  ): Promise<void> {
    const lastSyncTime = this.config?.lastSync || new Date(0)
    const isFirstSync = !this.config?.lastSync

    this.logger.debug(
      `Enhanced sync for ${name}: isFirstSync=${isFirstSync}, lastSyncTime=${lastSyncTime.toISOString()}`
    )

    // Get local changes since last sync
    const localChanges = await storage.getModifiedSince!(lastSyncTime)

    // Special handling for first sync on new machine:
    // If local is empty and this is first sync, pull all remote data
    if (isFirstSync && localChanges.length === 0) {
      const localData = await storage.readData<Record<string, unknown>>()
      if (localData.length === 0) {
        this.logger.info(`First sync on new machine for ${name}, pulling all remote data`)
        await this.performInitialPullSync(storage, collectionName, result)
        return
      }
    }

    // Get remote changes since last sync
    // For first sync, get all remote data instead of filtered changes
    const remoteChanges = isFirstSync
      ? await this.mongoService.findAll(collectionName)
      : await this.getRemoteChangesSince(collectionName, lastSyncTime)

    this.logger.debug(
      `Sync ${name}: ${localChanges.length} local changes, ${remoteChanges.length} remote changes`
    )

    // Process tombstones first
    const tombstones = await storage.getTombstones!()
    for (const tombstone of tombstones) {
      if (tombstone.deletedAt > lastSyncTime) {
        await this.processTombstone(tombstone, collectionName)
        result.tombstonesProcessed++
      }
    }

    // Merge and resolve conflicts
    const mergedChanges = await this.mergeChanges(localChanges, remoteChanges)
    result.conflictsResolved = mergedChanges.conflicts

    // Apply merged changes
    for (const change of mergedChanges.records) {
      await this.processDataChange(change, storage, collectionName)
      result.itemsProcessed++
    }

    this.logger.debug(
      `Sync ${name}: ${result.itemsProcessed} items, ${result.tombstonesProcessed} tombstones`
    )
  }

  /**
   * Perform initial pull sync for new machines (pull all remote data)
   */
  private async performInitialPullSync(
    storage: ISyncableStorage,
    collectionName: string,
    result: SyncOperationResult
  ): Promise<void> {
    try {
      // Get all remote data
      const remoteData = await this.mongoService.findAll(collectionName)

      if (remoteData.length === 0) {
        this.logger.debug(`No remote data found for ${collectionName}`)
        return
      }

      // Convert all remote documents to local format
      const localData: Record<string, unknown>[] = []
      for (const remoteDoc of remoteData) {
        const localFormat = this.convertFromRemoteFormat(remoteDoc)
        localData.push(localFormat)
      }

      // Save all data to local storage
      await storage.writeData(localData)

      result.itemsProcessed = localData.length

      this.logger.info(
        `Initial sync completed: pulled ${localData.length} items from remote to local`
      )

      // Emit sync event
      this.emitSyncEvent('sync.dataChanged')
    } catch (error) {
      this.logger.error('Initial pull sync failed:', error as Error)
      throw error
    }
  }

  /**
   * Perform basic sync (backward compatibility)
   */
  private async performBasicSync(
    _name: string,
    storage: ISyncableStorage,
    collectionName: string,
    result: SyncOperationResult
  ): Promise<void> {
    // Get local data
    const localData = await storage.readData<Record<string, unknown>>()

    // Get remote data
    const remoteData = await this.mongoService.findAll(collectionName)

    // Perform bidirectional sync
    await this.performBidirectionalSync(storage, localData, remoteData, collectionName)

    result.itemsProcessed = localData.length

    // Emit sync event
    this.emitSyncEvent('sync.dataChanged')
  }

  /**
   * Get remote changes since specific timestamp
   */
  private async getRemoteChangesSince(collectionName: string, since: Date): Promise<Document[]> {
    try {
      return await this.mongoService.find(collectionName, {
        '_syncMeta.version.timestamp': { $gt: since }
      })
    } catch {
      return []
    }
  }

  /**
   * Merge local and remote changes, resolving conflicts
   */
  private async mergeChanges(
    localChanges: SyncRecord[],
    remoteChanges: Document[]
  ): Promise<{ records: SyncRecord[]; conflicts: number }> {
    const mergedChanges = [...localChanges]
    let conflictCount = 0

    const localMap = new Map(localChanges.map((change) => [change.id, change]))

    for (const remoteChange of remoteChanges) {
      const localChange = localMap.get(remoteChange._id)

      if (!localChange) {
        mergedChanges.push(this.convertRemoteToLocalChange(remoteChange))
      } else {
        const resolved = await this.resolveConflict(localChange, remoteChange)

        const index = mergedChanges.findIndex((c) => c.id === resolved.id)
        if (index >= 0) {
          mergedChanges[index] = resolved
          conflictCount++
        }
      }
    }

    return { records: mergedChanges, conflicts: conflictCount }
  }

  /**
   * Resolve conflict between local and remote changes
   */
  private async resolveConflict(
    localChange: SyncRecord,
    remoteChange: Document
  ): Promise<SyncRecord> {
    const strategy = this.config?.conflictResolutionStrategy || 'latest-wins'

    switch (strategy) {
      case 'latest-wins':
        return this.resolveByTimestamp(localChange, remoteChange)

      case 'local-wins':
        return localChange

      case 'remote-wins':
        return this.convertRemoteToLocalChange(remoteChange)

      case 'manual':
        // Emit conflict event and fallback to latest-wins
        this.emitSyncEvent('sync.conflictResolutionRequired')
        return this.resolveByTimestamp(localChange, remoteChange)

      default:
        return this.resolveByVersion(localChange, remoteChange)
    }
  }

  /**
   * Resolve conflict by version number (higher version wins)
   */
  private resolveByVersion(localChange: SyncRecord, remoteChange: Document): SyncRecord {
    const localVersion = localChange.version?.timestamp || 0
    const remoteVersion = remoteChange._syncMeta?.version?.timestamp || 0

    return localVersion > remoteVersion
      ? localChange
      : this.convertRemoteToLocalChange(remoteChange)
  }

  /**
   * Resolve conflict by timestamp (newer wins)
   */
  private resolveByTimestamp(localChange: SyncRecord, remoteChange: Document): SyncRecord {
    const localTime = new Date(localChange.version?.timestamp || 0)
    const remoteTime = new Date(remoteChange._syncMeta?.version?.timestamp || 0)

    return localTime > remoteTime ? localChange : this.convertRemoteToLocalChange(remoteChange)
  }

  /**
   * Convert remote document to local change format
   */
  private convertRemoteToLocalChange(remoteDoc: Document): SyncRecord {
    const { _id, _syncMeta, ...data } = remoteDoc

    return {
      id: _id,
      type: _syncMeta?.type || 'ssh-profile',
      data,
      metadata: {
        version: _syncMeta?.version || {
          deviceId: 'unknown',
          timestamp: Date.now(),
          hash: 'unknown'
        },
        lastModified: new Date(),
        deviceId: _syncMeta?.version?.deviceId || 'unknown',
        checksum: _syncMeta?.version?.hash || 'unknown'
      },
      version: _syncMeta?.version || {
        deviceId: 'unknown',
        timestamp: Date.now(),
        hash: 'unknown'
      }
    }
  }

  /**
   * Process tombstone record
   */
  private async processTombstone(
    tombstone: TombstoneRecord,
    collectionName: string
  ): Promise<void> {
    try {
      const tombstoneCollectionName = `${collectionName}_tombstones`
      const existingTombstone = await this.mongoService.findById(
        tombstoneCollectionName,
        tombstone.id
      )

      if (
        !existingTombstone ||
        (existingTombstone.version &&
          tombstone.version &&
          existingTombstone.version < tombstone.version)
      ) {
        await this.mongoService.replaceOne(tombstoneCollectionName, tombstone.id, {
          _id: tombstone.id,
          ...tombstone,
          _syncedAt: new Date(),
          _deviceId: this.deviceInfo.deviceId
        })
      }

      // Remove the actual document from remote collection
      await this.mongoService.deleteMany(collectionName, {
        _id: tombstone.id as unknown as import('mongodb').ObjectId
      })
    } catch (error) {
      this.logger.error(`Failed to process tombstone ${tombstone.id}:`, error as Error)
    }
  }

  /**
   * Process data change
   */
  private async processDataChange(
    change: SyncRecord,
    storage: ISyncableStorage,
    collectionName: string
  ): Promise<void> {
    try {
      if (change.data && !change.isDeleted) {
        // Save data locally with version info if supported
        if (storage.saveDataWithVersion && change.version) {
          await storage.saveDataWithVersion(change.id, change.data, change.version)
        }

        // Update remote document
        await this.mongoService.replaceOne(collectionName, change.id, {
          _id: change.id,
          ...change.data,
          _syncMeta: {
            version: change.version,
            type: change.type,
            deviceId: this.deviceInfo.deviceId || this.deviceInfo.id,
            updatedAt: new Date()
          }
        })
      }
    } catch (error) {
      this.logger.error(`Failed to process data change ${change.id}:`, error as Error)
    }
  }

  /**
   * Cleanup old tombstones
   */
  private async cleanupOldTombstones(
    storageMap: Array<{ name: string; storage: ISyncableStorage }>
  ): Promise<void> {
    if (!this.config?.retainTombstoneDays) {
      return
    }

    for (const { name, storage } of storageMap) {
      try {
        // Cleanup local tombstones if supported
        if (storage.cleanup) {
          await storage.cleanup(this.config.retainTombstoneDays)
        }

        // Cleanup remote tombstones
        const collectionName = `${this.getCollectionName(name)}_tombstones`
        const cutoffDate = new Date()
        cutoffDate.setDate(cutoffDate.getDate() - this.config.retainTombstoneDays)

        await this.mongoService.deleteMany(collectionName, {
          deletedAt: { $lt: cutoffDate }
        })
      } catch (error) {
        this.logger.error(`Failed to cleanup tombstones for ${name}:`, error as Error)
      }
    }
  }

  /**
   * Perform bidirectional sync (backward compatibility)
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

    await storage.writeData(updatedLocalData)
  }

  /**
   * Sync individual item (backward compatibility)
   */
  private async syncItem(
    localItem: Record<string, unknown> | undefined,
    remoteItem: Document | undefined,
    id: unknown,
    updatedLocalData: Record<string, unknown>[],
    collectionName: string
  ): Promise<void> {
    if (!localItem && remoteItem) {
      const localFormat = this.convertFromRemoteFormat(remoteItem)
      updatedLocalData.push(localFormat)
    } else if (localItem && !remoteItem) {
      const remoteFormat = this.convertToRemoteFormat(localItem)
      await this.mongoService.insertOne(collectionName, remoteFormat)
    } else if (localItem && remoteItem) {
      const resolution = await this.resolveBasicConflict(localItem, remoteItem)

      if (resolution === 'local') {
        const remoteFormat = this.convertToRemoteFormat(localItem)
        await this.mongoService.replaceOne(collectionName, id as string, remoteFormat)
      } else if (resolution === 'remote') {
        const localFormat = this.convertFromRemoteFormat(remoteItem)
        const index = updatedLocalData.findIndex((item) => item.id === id)
        if (index !== -1) {
          updatedLocalData[index] = localFormat
        }
      }
    }
  }

  /**
   * Resolve conflicts for basic sync (backward compatibility)
   */
  private async resolveBasicConflict(
    localItem: Record<string, unknown>,
    remoteItem: Document
  ): Promise<ConflictResolution> {
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
      _deviceId: this.deviceInfo.deviceId
    }
  }

  /**
   * Convert remote data format to local format
   */
  private convertFromRemoteFormat(remoteItem: Document): Record<string, unknown> {
    // eslint-disable-next-line @typescript-eslint/no-unused-vars
    const { _id, _syncedAt, _deviceId, _syncMeta, ...localData } = remoteItem
    return {
      id: _id,
      ...localData
    }
  }

  /**
   * Get collection name from storage name
   */
  private getCollectionName(name: string): string {
    return name.toLowerCase().replace(/storage$/, '')
  }

  /**
   * Emit sync event to renderer processes
   */
  private emitSyncEvent(eventType: string): void {
    const windows = BrowserWindow.getAllWindows()
    windows.forEach((window) => {
      window.webContents.send(eventType, {
        timestamp: new Date(),
        deviceId: this.deviceInfo.deviceId
      })
    })
  }

  /**
   * Start automatic sync
   */
  private startAutoSync(storageMap: Array<{ name: string; storage: ISyncableStorage }>): void {
    if (!this.config?.autoSync) return

    this.stopAutoSync()

    const intervalMs = (this.config.syncInterval || 300) * 1000
    this.syncInterval = setInterval(async () => {
      if (!this.status.isConnected || this.status.syncInProgress) {
        return
      }

      try {
        await this.performFullSync(storageMap)
      } catch (error) {
        this.logger.error('Automatic sync failed:', error as Error)
        this.status.lastError = (error as Error).message

        if (!this.status.isConnected) {
          this.logger.warn('Connection lost, will attempt to reconnect on next sync')
        }
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
   * Get current sync status
   */
  getStatus(): SyncStatus {
    return { ...this.status }
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

        const remoteData = await this.mongoService.findAll(collectionName)

        if (remoteData.length === 0 && localData.length > 0) {
          for (const item of localData) {
            const remoteFormat = this.convertToRemoteFormat(item)
            await this.mongoService.insertOne(collectionName, remoteFormat)
          }

          this.logger.info(`Migrated ${localData.length} items from ${collectionName} to MongoDB`)
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
