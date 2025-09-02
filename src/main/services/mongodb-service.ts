import { MongoClient, Db, Collection, Filter, Document } from 'mongodb'
import { hostname } from 'os'
import { SyncConfig } from '../interfaces/sync.interface'
import { ConsoleLogger } from '../utils/logger'

/**
 * MongoDB connection and operations service
 */
export class MongoDBService {
  private client: MongoClient | null = null
  private db: Db | null = null
  private config: SyncConfig | null = null
  private readonly deviceId: string
  private readonly logger: ConsoleLogger

  constructor() {
    this.deviceId = this.generateDeviceId()
    this.logger = new ConsoleLogger('MongoDB')
  }

  /**
   * Generate unique device ID
   */
  private generateDeviceId(): string {
    const timestamp = Date.now().toString(36)
    return `${hostname()}-${timestamp}`
  }

  /**
   * Set MongoDB configuration
   */
  setConfig(config: SyncConfig): void {
    this.config = config
  }

  /**
   * Connect to MongoDB
   */
  async connect(): Promise<boolean> {
    if (!this.config) {
      throw new Error('MongoDB configuration not set')
    }

    try {
      this.client = new MongoClient(this.config.mongoUri, {
        maxPoolSize: 10,
        serverSelectionTimeoutMS: 5000,
        socketTimeoutMS: 45000
      })

      await this.client.connect()
      this.db = this.client.db(this.config.databaseName)

      this.logger.info('Connected to MongoDB successfully')
      return true
    } catch (error) {
      this.logger.error('Failed to connect to MongoDB:', error as Error)
      return false
    }
  }

  /**
   * Disconnect from MongoDB
   */
  async disconnect(): Promise<void> {
    if (this.client) {
      await this.client.close()
      this.client = null
      this.db = null
      this.logger.info('Disconnected from MongoDB')
    }
  }

  /**
   * Test MongoDB connection
   */
  async testConnection(): Promise<boolean> {
    if (!this.config) {
      return false
    }

    try {
      const testClient = new MongoClient(this.config.mongoUri, {
        serverSelectionTimeoutMS: 5000
      })

      await testClient.connect()
      await testClient.db(this.config.databaseName).admin().ping()
      await testClient.close()

      return true
    } catch (error) {
      this.logger.error('MongoDB connection test failed:', error as Error)
      return false
    }
  }

  /**
   * Check if connected to MongoDB
   */
  isConnected(): boolean {
    return this.client !== null && this.db !== null
  }

  /**
   * Get collection by name
   */
  getCollection(name: string): Collection<Document> {
    if (!this.db) {
      throw new Error('Not connected to MongoDB')
    }
    return this.db.collection(name)
  }

  /**
   * Get all documents from collection
   */
  async findAll<T = Document>(collectionName: string): Promise<T[]> {
    const collection = this.getCollection(collectionName)
    const documents = await collection.find({}).toArray()
    return documents as T[]
  }

  /**
   * Find document by ID
   */
  async findById<T = Document>(collectionName: string, id: string): Promise<T | null> {
    const collection = this.getCollection(collectionName)
    const filter = { _id: id } as unknown as Filter<Document>
    const document = await collection.findOne(filter)
    return document as T | null
  }

  /**
   * Insert document
   */
  async insertOne<T = Document>(
    collectionName: string,
    document: T & { _id?: string }
  ): Promise<void> {
    const collection = this.getCollection(collectionName)
    const docWithMeta = {
      ...document,
      _syncedAt: new Date(),
      _deviceId: this.deviceId
    }
    await collection.insertOne(docWithMeta as Document)
  }

  /**
   * Update document
   */
  async updateOne<T = Document>(
    collectionName: string,
    id: string,
    document: Partial<T>
  ): Promise<boolean> {
    const collection = this.getCollection(collectionName)
    const filter = { _id: id } as unknown as Filter<Document>
    const result = await collection.updateOne(filter, {
      $set: {
        ...document,
        _syncedAt: new Date(),
        _deviceId: this.deviceId
      }
    })
    return result.modifiedCount > 0
  }

  /**
   * Replace entire document
   */
  async replaceOne<T = Document>(
    collectionName: string,
    id: string,
    document: T & { _id?: string }
  ): Promise<boolean> {
    const collection = this.getCollection(collectionName)
    const filter = { _id: id } as unknown as Filter<Document>
    const docWithMeta = {
      ...document,
      _id: id,
      _syncedAt: new Date(),
      _deviceId: this.deviceId
    }
    const result = await collection.replaceOne(filter, docWithMeta as Document, { upsert: true })
    return result.modifiedCount > 0 || result.upsertedCount > 0
  }

  /**
   * Delete document by ID
   */
  async deleteOne(collectionName: string, id: string): Promise<boolean> {
    const collection = this.getCollection(collectionName)
    const filter = { _id: id } as unknown as Filter<Document>
    const result = await collection.deleteOne(filter)
    return result.deletedCount > 0
  }

  /**
   * Get documents modified after specific date
   */
  async findModifiedAfter<T = Document>(collectionName: string, date: Date): Promise<T[]> {
    const collection = this.getCollection(collectionName)
    const filter = { _syncedAt: { $gt: date } } as unknown as Filter<Document>
    const documents = await collection.find(filter).toArray()
    return documents as T[]
  }

  /**
   * Get device ID
   */
  getDeviceId(): string {
    return this.deviceId
  }
}
