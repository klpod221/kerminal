import { BaseStorage } from './base-storage'
import { SyncConfig } from '../interfaces/sync.interface'

/**
 * Storage for sync configuration
 */
export class SyncConfigStorage extends BaseStorage {
  constructor() {
    super('sync-config.json')
  }

  /**
   * Get sync configuration
   */
  async getConfig(): Promise<SyncConfig | null> {
    const configs = await this.readData<SyncConfig>()
    return configs.length > 0 ? configs[0] : null
  }

  /**
   * Save sync configuration
   */
  async saveConfig(config: SyncConfig): Promise<void> {
    await this.writeData([config])
  }

  /**
   * Delete sync configuration
   */
  async deleteConfig(): Promise<void> {
    await this.writeData([])
  }

  /**
   * Check if sync is configured
   */
  async isConfigured(): Promise<boolean> {
    const config = await this.getConfig()
    return config !== null && config.enabled
  }
}
