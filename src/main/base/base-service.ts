/**
 * Base service implementation with common functionality
 */
import { IService, ILogger } from '../interfaces/application.interface'

export abstract class BaseService implements IService {
  public readonly name: string
  protected isServiceInitialized = false
  protected logger?: ILogger

  constructor(name: string, logger?: ILogger) {
    this.name = name
    this.logger = logger
  }

  public isInitialized(): boolean {
    return this.isServiceInitialized
  }

  public async initialize(): Promise<void> {
    if (this.isServiceInitialized) {
      this.logger?.warn(`Service ${this.name} is already initialized`)
      return
    }

    try {
      await this.onInitialize()
      this.isServiceInitialized = true
      this.logger?.info(`Service ${this.name} initialized successfully`)
    } catch (error) {
      this.logger?.error(`Failed to initialize service ${this.name}`, error as Error)
      throw error
    }
  }

  public cleanup(): void {
    if (!this.isServiceInitialized) {
      return
    }

    try {
      this.onCleanup()
      this.isServiceInitialized = false
      this.logger?.info(`Service ${this.name} cleaned up successfully`)
    } catch (error) {
      this.logger?.error(`Failed to cleanup service ${this.name}`, error as Error)
    }
  }

  protected abstract onInitialize(): Promise<void>
  protected abstract onCleanup(): void
}
