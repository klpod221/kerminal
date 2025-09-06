/**
 * Shared sync types between main and renderer processes
 * This file contains common sync-related types to avoid duplication
 */

/**
 * Conflict resolution strategy type
 */
export type ConflictResolutionStrategy = 'local-wins' | 'remote-wins' | 'latest-wins' | 'manual'

/**
 * Conflict resolution type
 */
export type ConflictResolution = 'local' | 'remote' | 'merge' | 'ask'

/**
 * Sync configuration interface
 */
export interface SyncConfig {
  id: string
  provider: 'mongodb'
  enabled: boolean
  connectionString?: string
  mongoUri?: string
  databaseName?: string
  collectionPrefix?: string
  encryptionKey?: string
  lastSync?: Date
  autoSync?: boolean
  syncInterval?: number
  deviceName?: string
  deviceId?: string
  conflictResolutionStrategy?: ConflictResolutionStrategy
  retainTombstoneDays?: number
  created: Date
  updated: Date
}

/**
 * Sync status interface
 */
export interface SyncStatus {
  isEnabled: boolean
  isConnected?: boolean
  isLoading?: boolean
  syncInProgress?: boolean
  lastSync?: Date
  lastSyncStatus: 'success' | 'error' | 'in-progress' | 'never'
  lastSyncError?: string
  lastError?: string
  nextSync?: Date
  deviceCount?: number
  lastActivity?: Date
  lastPull?: Date
  lastPush?: Date
  pendingChanges?: number
  totalItems?: number
  syncedItems?: number
  conflictCount?: number
  tombstoneCount?: number
}

/**
 * Data version interface for tracking changes
 */
export interface DataVersion {
  deviceId: string
  timestamp: number
  hash: string
}

/**
 * Tombstone record interface for handling deletions
 */
export interface TombstoneRecord {
  id: string
  deviceId: string
  deletedAt: Date
  type: string
  collection?: string
  deletedBy?: string
  version?: number
}

/**
 * Sync metadata interface
 */
export interface SyncMetadata {
  version: DataVersion
  lastModified: Date
  deviceId: string
  checksum: string
  id?: string
  isDeleted?: boolean
  tombstone?: TombstoneRecord
}

/**
 * Sync record interface
 */
export interface SyncRecord {
  id: string
  type: 'ssh-profile' | 'ssh-group' | 'saved-command' | 'ssh-tunnel'
  data: Record<string, unknown>
  metadata: SyncMetadata
  version?: DataVersion
  isDeleted?: boolean
}

/**
 * Interface for storage classes that support synchronization
 */
export interface ISyncableStorage {
  /**
   * Read data from storage
   */
  readData<T>(): Promise<T[]>

  /**
   * Write data to storage
   */
  writeData<T>(data: T[]): Promise<void>

  /**
   * Get sync metadata for all items
   */
  getSyncMetadata?(): Promise<SyncMetadata[]>

  /**
   * Get sync metadata for specific item
   */
  getItemMetadata?(id: string): Promise<SyncMetadata | null>

  /**
   * Update sync metadata for item
   */
  updateItemMetadata?(id: string, metadata: SyncMetadata): Promise<void>

  /**
   * Mark item as deleted (create tombstone)
   */
  markAsDeleted?(id: string, deletedBy: string): Promise<void>

  /**
   * Get all tombstone records
   */
  getTombstones?(): Promise<TombstoneRecord[]>

  /**
   * Remove tombstone record
   */
  removeTombstone?(id: string): Promise<void>

  /**
   * Get data with version information
   */
  getDataWithVersion?(
    id: string
  ): Promise<{ data: Record<string, unknown>; version: DataVersion } | null>

  /**
   * Save data with version information
   */
  saveDataWithVersion?(
    id: string,
    data: Record<string, unknown>,
    version: DataVersion
  ): Promise<void>

  /**
   * Get items modified after specific timestamp
   */
  getModifiedSince?(timestamp: Date): Promise<SyncRecord[]>

  /**
   * Generate content hash for data integrity
   */
  generateHash?(data: Record<string, unknown>): string

  /**
   * Cleanup old tombstones and version history
   */
  cleanup?(retainDays: number): Promise<void>
}

/**
 * Storage registry for sync service
 */
export class SyncableStorageRegistry {
  private readonly storages: Map<string, ISyncableStorage> = new Map()

  /**
   * Register a storage instance for syncing
   */
  register(name: string, storage: ISyncableStorage): void {
    this.storages.set(name, storage)
  }

  /**
   * Get a storage instance by name
   */
  get(name: string): ISyncableStorage | undefined {
    return this.storages.get(name)
  }

  /**
   * Get all registered storages
   */
  getAll(): Array<{ name: string; storage: ISyncableStorage }> {
    return Array.from(this.storages.entries()).map(([name, storage]) => ({
      name,
      storage
    }))
  }

  /**
   * Check if storage is registered
   */
  has(name: string): boolean {
    return this.storages.has(name)
  }

  /**
   * Remove storage from registry
   */
  unregister(name: string): boolean {
    return this.storages.delete(name)
  }

  /**
   * Clear all registered storages
   */
  clear(): void {
    this.storages.clear()
  }
}

/**
 * Sync operation result interface
 */
export interface SyncOperationResult {
  success: boolean
  recordsProcessed: number
  itemsProcessed: number
  tombstonesProcessed: number
  conflictsResolved: number
  errors: string[]
  lastSync: Date
}

/**
 * Device info interface
 */
export interface DeviceInfo {
  id: string
  deviceId?: string
  name: string
  platform: string
  lastActivity: Date
  syncVersion: string
}

/**
 * Sync conflict interface
 */
export interface SyncConflict {
  recordId: string
  type: string
  localData: Record<string, unknown>
  remoteData: Record<string, unknown>
  localVersion: DataVersion
  remoteVersion: DataVersion
}
