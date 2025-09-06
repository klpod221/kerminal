import { app } from 'electron'
import * as fs from 'fs/promises'
import * as path from 'path'
import * as crypto from 'crypto'
import { ISyncableStorage } from '../interfaces/syncable-storage.interface'
import {
  SyncMetadata,
  TombstoneRecord,
  DataVersion,
  SyncRecord
} from '../interfaces/sync.interface'

/**
 * Base storage service for handling file operations with sync support
 */
export class BaseStorage implements ISyncableStorage {
  protected readonly dataPath: string
  protected readonly metadataFile: string
  protected readonly tombstoneFile: string
  protected readonly filename: string

  constructor(fileName: string) {
    this.filename = fileName
    this.dataPath = path.join(app.getPath('userData'), fileName)
    const baseName = path.basename(fileName, '.json')
    this.metadataFile = path.join(app.getPath('userData'), `${baseName}.metadata.json`)
    this.tombstoneFile = path.join(app.getPath('userData'), `${baseName}.tombstones.json`)
  }

  /**
   * Ensure the data directory exists
   */
  private async ensureDataDirectory(): Promise<void> {
    const dir = path.dirname(this.dataPath)
    try {
      await fs.access(dir)
    } catch {
      await fs.mkdir(dir, { recursive: true })
    }
  }

  /**
   * Read data from file
   */
  public async readData<T>(): Promise<T[]> {
    try {
      await this.ensureDataDirectory()
      const data = await fs.readFile(this.dataPath, 'utf-8')
      const parsed = JSON.parse(data)
      // Ensure we always return an array
      return Array.isArray(parsed) ? parsed : []
    } catch {
      // File doesn't exist or is invalid, return empty array
      return []
    }
  }

  /**
   * Write data to file with auto-versioning support
   */
  public async writeData<T>(data: T[]): Promise<void> {
    try {
      await this.ensureDataDirectory()
      await fs.writeFile(this.dataPath, JSON.stringify(data, null, 2), 'utf-8')

      // Auto-update metadata for items that have IDs
      const currentTime = new Date()
      const deviceId = process.env.DEVICE_ID || 'unknown'

      for (const item of data) {
        const recordItem = item as Record<string, unknown>
        if (recordItem.id) {
          const existingMeta = await this.getItemMetadata(recordItem.id as string)
          const newVersion = (existingMeta?.version?.version || 0) + 1

          const metadata: SyncMetadata = {
            id: recordItem.id as string,
            version: {
              version: newVersion,
              timestamp: currentTime,
              deviceId,
              hash: this.generateHash(recordItem)
            },
            isDeleted: false
          }

          await this.updateItemMetadata(recordItem.id as string, metadata)
        }
      }
    } catch (error) {
      console.error('Failed to write data to file:', error)
      throw new Error('Failed to save data')
    }
  }

  /**
   * Check if storage file exists
   */
  protected async exists(): Promise<boolean> {
    try {
      await fs.access(this.dataPath)
      return true
    } catch {
      return false
    }
  }

  /**
   * Delete the storage file
   */
  protected async deleteFile(): Promise<void> {
    try {
      await fs.unlink(this.dataPath)
    } catch {
      // File might not exist, ignore error
    }
  }

  /**
   * Read file content
   */
  protected async readFile(filePath: string): Promise<string | null> {
    try {
      return await fs.readFile(filePath, 'utf-8')
    } catch {
      return null
    }
  }

  /**
   * Write file content
   */
  protected async writeFile(filePath: string, content: string): Promise<void> {
    const dir = path.dirname(filePath)

    try {
      await fs.access(dir)
    } catch {
      await fs.mkdir(dir, { recursive: true })
    }

    await fs.writeFile(filePath, content, 'utf-8')
  }

  /**
   * Get sync metadata for all items
   */
  async getSyncMetadata(): Promise<SyncMetadata[]> {
    try {
      const content = await this.readFile(this.metadataFile)
      return content ? JSON.parse(content) : []
    } catch {
      return []
    }
  }

  /**
   * Get sync metadata for specific item
   */
  async getItemMetadata(id: string): Promise<SyncMetadata | null> {
    const allMetadata = await this.getSyncMetadata()
    return allMetadata.find((meta) => meta.id === id) || null
  }

  /**
   * Update sync metadata for item
   */
  async updateItemMetadata(id: string, metadata: SyncMetadata): Promise<void> {
    const allMetadata = await this.getSyncMetadata()
    const existingIndex = allMetadata.findIndex((meta) => meta.id === id)

    if (existingIndex >= 0) {
      allMetadata[existingIndex] = metadata
    } else {
      allMetadata.push(metadata)
    }

    await this.writeFile(this.metadataFile, JSON.stringify(allMetadata, null, 2))
  }

  /**
   * Mark item as deleted (create tombstone)
   */
  async markAsDeleted(id: string, deletedBy: string): Promise<void> {
    const metadata = await this.getItemMetadata(id)
    const currentVersion = metadata?.version?.version || 0

    const tombstone: TombstoneRecord = {
      id,
      collection: this.getCollectionName(),
      deletedAt: new Date(),
      deletedBy,
      version: currentVersion + 1
    }

    // Add to tombstones
    const tombstones = await this.getTombstones()
    const existingIndex = tombstones.findIndex((t) => t.id === id)

    if (existingIndex >= 0) {
      tombstones[existingIndex] = tombstone
    } else {
      tombstones.push(tombstone)
    }

    await this.writeFile(this.tombstoneFile, JSON.stringify(tombstones, null, 2))

    // Update metadata to mark as deleted
    if (metadata) {
      metadata.isDeleted = true
      metadata.tombstone = tombstone
      metadata.version = {
        version: currentVersion + 1,
        timestamp: new Date(),
        deviceId: deletedBy
      }
      await this.updateItemMetadata(id, metadata)
    }

    // Remove from actual data
    const data = await this.readData<Record<string, unknown>>()
    const filteredData = data.filter((item) => item.id !== id)
    await this.writeData(filteredData)
  }

  /**
   * Get all tombstone records
   */
  async getTombstones(): Promise<TombstoneRecord[]> {
    try {
      const content = await this.readFile(this.tombstoneFile)
      return content ? JSON.parse(content) : []
    } catch {
      return []
    }
  }

  /**
   * Remove tombstone record
   */
  async removeTombstone(id: string): Promise<void> {
    const tombstones = await this.getTombstones()
    const filtered = tombstones.filter((t) => t.id !== id)
    await this.writeFile(this.tombstoneFile, JSON.stringify(filtered, null, 2))
  }

  /**
   * Get data with version information
   */
  async getDataWithVersion(
    id: string
  ): Promise<{ data: Record<string, unknown>; version: DataVersion } | null> {
    const data = await this.readData<Record<string, unknown>>()
    const item = data.find((d) => d.id === id)

    if (!item) {
      return null
    }

    const metadata = await this.getItemMetadata(id)
    const version = metadata?.version || {
      version: 1,
      timestamp: new Date(),
      deviceId: 'unknown'
    }

    return { data: item, version }
  }

  /**
   * Save data with version information
   */
  async saveDataWithVersion(
    id: string,
    data: Record<string, unknown>,
    version: DataVersion
  ): Promise<void> {
    const allData = await this.readData<Record<string, unknown>>()
    const existingIndex = allData.findIndex((item) => item.id === id)

    const itemWithId = { ...data, id }

    if (existingIndex >= 0) {
      allData[existingIndex] = itemWithId
    } else {
      allData.push(itemWithId)
    }

    await this.writeData(allData)

    // Update metadata
    const metadata: SyncMetadata = {
      id,
      version: {
        ...version,
        hash: this.generateHash(itemWithId)
      },
      isDeleted: false
    }

    await this.updateItemMetadata(id, metadata)
  }

  /**
   * Get items modified since specific timestamp
   */
  async getModifiedSince(timestamp: Date): Promise<SyncRecord[]> {
    const metadata = await this.getSyncMetadata()
    const data = await this.readData<Record<string, unknown>>()
    const records: SyncRecord[] = []

    // Check modified items
    for (const meta of metadata) {
      if (meta.version.timestamp > timestamp) {
        const item = data.find((d) => d.id === meta.id)

        let action: 'create' | 'update' | 'delete' = 'create'
        if (meta.isDeleted) {
          action = 'delete'
        } else if (item) {
          action = 'update'
        }

        records.push({
          id: meta.id,
          collection: this.getCollectionName(),
          action,
          data: item,
          version: meta.version,
          previousVersion: meta.version.version - 1,
          isTombstone: meta.isDeleted,
          timestamp: meta.version.timestamp,
          synced: true,
          deviceId: meta.version.deviceId
        })
      }
    }

    // Check tombstones
    const tombstones = await this.getTombstones()
    for (const tombstone of tombstones) {
      if (tombstone.deletedAt > timestamp) {
        records.push({
          id: tombstone.id,
          collection: this.getCollectionName(),
          action: 'delete',
          version: {
            version: tombstone.version,
            timestamp: tombstone.deletedAt,
            deviceId: tombstone.deletedBy
          },
          isTombstone: true,
          timestamp: tombstone.deletedAt,
          synced: true,
          deviceId: tombstone.deletedBy
        })
      }
    }

    return records
  }

  /**
   * Generate content hash for data integrity
   */
  generateHash(data: Record<string, unknown>): string {
    const content = JSON.stringify(
      data,
      Object.keys(data).sort((a, b) => a.localeCompare(b))
    )
    return crypto.createHash('sha256').update(content).digest('hex')
  }

  /**
   * Cleanup old tombstones and version history
   */
  async cleanup(retainDays: number): Promise<void> {
    const cutoffDate = new Date()
    cutoffDate.setDate(cutoffDate.getDate() - retainDays)

    // Cleanup old tombstones
    const tombstones = await this.getTombstones()
    const recentTombstones = tombstones.filter((t) => t.deletedAt > cutoffDate)
    await this.writeFile(this.tombstoneFile, JSON.stringify(recentTombstones, null, 2))

    // Cleanup metadata for deleted items older than cutoff
    const metadata = await this.getSyncMetadata()
    const cleanedMetadata = metadata.filter((meta) => {
      if (meta.isDeleted && meta.version.timestamp < cutoffDate) {
        return false // Remove old deleted items
      }
      return true
    })

    await this.writeFile(this.metadataFile, JSON.stringify(cleanedMetadata, null, 2))
  }

  /**
   * Get collection name from filename
   */
  private getCollectionName(): string {
    return this.filename.replace(/\.json$/, '').replace(/-/g, '_')
  }
}
