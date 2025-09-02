import { app } from 'electron'
import * as fs from 'fs/promises'
import * as path from 'path'
import { ISyncableStorage } from '../interfaces/syncable-storage.interface'

/**
 * Base storage service for handling file operations
 */
export class BaseStorage implements ISyncableStorage {
  protected readonly dataPath: string

  constructor(fileName: string) {
    this.dataPath = path.join(app.getPath('userData'), fileName)
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
   * Write data to file
   */
  public async writeData<T>(data: T[]): Promise<void> {
    try {
      await this.ensureDataDirectory()
      await fs.writeFile(this.dataPath, JSON.stringify(data, null, 2), 'utf-8')
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
}
