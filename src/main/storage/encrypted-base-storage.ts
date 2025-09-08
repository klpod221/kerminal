import { BaseStorage } from './base-storage'
import { AuthService } from '../services/auth-service'
import { CryptoService } from '../services/crypto-service'
import { ConsoleLogger } from '../utils/logger'
import { v4 as uuidv4 } from 'uuid'

/**
 * Extended base storage with encryption support
 * Automatically encrypts/decrypts sensitive fields in stored data
 */
export class EncryptedBaseStorage extends BaseStorage {
  protected readonly encLogger = new ConsoleLogger('EncryptedBaseStorage')
  protected readonly authService: AuthService
  protected readonly cryptoService: CryptoService

  // Fields to encrypt (can be overridden by child classes)
  protected encryptedFields: string[] = []

  constructor(fileName: string, authService: AuthService, cryptoService: CryptoService) {
    super(fileName)
    this.authService = authService
    this.cryptoService = cryptoService
  }

  /**
   * Read and decrypt data from storage
   */
  public async readData<T>(): Promise<T[]> {
    const data = await super.readData<
      T extends Record<string, unknown> ? T : Record<string, unknown>
    >()

    // If app is not unlocked, return empty array (security)
    if (!this.authService.isUnlocked()) {
      this.encLogger.warn('Attempted to read encrypted data while app is locked')
      return []
    }

    const encryptionKey = this.authService.getEncryptionKey()
    if (!encryptionKey) {
      this.encLogger.warn('No encryption key available for reading data')
      return []
    }

    // Decrypt sensitive fields
    return data.map((item) =>
      this.decryptItem(item as T & Record<string, unknown>, encryptionKey)
    ) as T[]
  }

  /**
   * Write data to file after encrypting sensitive fields
   */
  public async writeData<T>(data: T[]): Promise<void> {
    // If app is not unlocked, refuse to write data (security)
    this.requireUnlocked()

    const encryptionKey = this.authService.getEncryptionKey()
    if (!encryptionKey) {
      throw new Error('No encryption key available for writing data')
    }

    // Encrypt sensitive fields
    const encryptedData = data.map((item) =>
      this.encryptItem(item as T & Record<string, unknown>, encryptionKey)
    )

    await super.writeData(encryptedData)
  }

  /**
   * Encrypt sensitive fields in an item
   */
  protected encryptItem<T extends Record<string, unknown>>(item: T, encryptionKey: Buffer): T {
    if (this.encryptedFields.length === 0) {
      return item
    }

    const encrypted = { ...item } as Record<string, unknown>

    for (const field of this.encryptedFields) {
      if (encrypted[field] != null && encrypted[field] !== '') {
        try {
          const plaintext = String(encrypted[field])
          encrypted[field] = this.cryptoService.encrypt(plaintext, encryptionKey)
        } catch (error) {
          this.encLogger.error(`Failed to encrypt field ${field}:`, error as Error)
          // Don't save the item if encryption fails
          throw new Error(`Failed to encrypt sensitive field: ${field}`)
        }
      }
    }

    return encrypted as T
  }

  /**
   * Decrypt sensitive fields in an item
   */
  protected decryptItem<T extends Record<string, unknown>>(item: T, encryptionKey: Buffer): T {
    if (this.encryptedFields.length === 0) {
      return item
    }

    const decrypted = { ...item } as Record<string, unknown>

    for (const field of this.encryptedFields) {
      if (decrypted[field] != null && decrypted[field] !== '') {
        try {
          const encryptedValue = String(decrypted[field])
          // Check if the value looks like encrypted data (contains colons)
          if (encryptedValue.includes(':') && encryptedValue.split(':').length === 4) {
            decrypted[field] = this.cryptoService.decrypt(encryptedValue, encryptionKey)
          }
          // If it doesn't look encrypted, leave it as is (might be legacy data)
        } catch (error) {
          this.encLogger.error(`Failed to decrypt field ${field}:`, error as Error)
          // For security, set field to empty string if decryption fails
          decrypted[field] = ''
        }
      }
    }

    return decrypted as T
  }

  /**
   * Check if the application is unlocked before performing operations
   */
  protected requireUnlocked(): void {
    if (!this.authService.isUnlocked()) {
      throw new Error('Application must be unlocked to access encrypted data')
    }
  }

  /**
   * Safe read that returns empty array if app is locked
   */
  protected async safeReadData<T extends Record<string, unknown>>(): Promise<T[]> {
    try {
      return await this.readData<T>()
    } catch (error) {
      this.encLogger.error('Failed to read encrypted data:', error as Error)
      return []
    }
  }

  /**
   * Create a new record with encryption
   */
  public async create<T extends Record<string, unknown>>(
    data: Omit<T, 'id' | 'created' | 'updated'>
  ): Promise<T> {
    this.requireUnlocked()

    const now = new Date()
    const newRecord = {
      ...data,
      id: this.generateId(),
      created: now,
      updated: now
    } as unknown as T

    const allData = await this.readData<T>()
    allData.push(newRecord)
    await this.writeData(allData)

    return newRecord
  }

  /**
   * Update an existing record with encryption
   */
  public async update<T extends Record<string, unknown>>(
    id: string,
    updates: Partial<Omit<T, 'id' | 'created'>>
  ): Promise<T | null> {
    this.requireUnlocked()

    const allData = await this.readData<T>()
    const index = allData.findIndex((item) => item.id === id)

    if (index === -1) {
      return null
    }

    const updated = {
      ...allData[index],
      ...updates,
      updated: new Date()
    } as T

    allData[index] = updated
    await this.writeData(allData)

    return updated
  }

  /**
   * Delete a record by ID
   */
  public async delete(id: string): Promise<boolean> {
    this.requireUnlocked()

    const allData = await this.readData<Record<string, unknown>>()
    const initialLength = allData.length
    const filtered = allData.filter((item) => item.id !== id)

    if (filtered.length === initialLength) {
      return false
    }

    await this.writeData(filtered)
    await this.markAsDeleted(id, process.env.DEVICE_ID || 'unknown')

    return true
  }

  /**
   * Get a single record by ID
   */
  public async getById<T extends Record<string, unknown>>(id: string): Promise<T | null> {
    if (!this.authService.isUnlocked()) {
      return null
    }

    const allData = await this.readData<T>()
    return allData.find((item) => item.id === id) || null
  }

  /**
   * Get all records
   */
  public async getAll<T extends Record<string, unknown>>(): Promise<T[]> {
    return await this.safeReadData<T>()
  }

  /**
   * Generate a unique ID for records
   */
  protected generateId(): string {
    return uuidv4()
  }

  /**
   * Set which fields should be encrypted (to be called by child classes)
   */
  protected setEncryptedFields(fields: string[]): void {
    this.encryptedFields = fields
  }
}
