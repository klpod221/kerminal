/**
 * Syncable storage interfaces
 */

import { SyncMetadata, TombstoneRecord, DataVersion, SyncRecord } from './sync.interface'

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
