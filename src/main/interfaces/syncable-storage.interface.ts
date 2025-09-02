/**
 * Interface for sync-enabled storage classes
 */
export interface ISyncableStorage {
  readData<T>(): Promise<T[]>
  writeData<T>(data: T[]): Promise<void>
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
   * Unregister a storage instance
   */
  unregister(name: string): void {
    this.storages.delete(name)
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
   * Get storage by name
   */
  get(name: string): ISyncableStorage | undefined {
    return this.storages.get(name)
  }

  /**
   * Clear all registered storages
   */
  clear(): void {
    this.storages.clear()
  }
}
