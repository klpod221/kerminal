import { EncryptedBaseStorage } from './encrypted-base-storage'
import { AuthService } from '../services/auth-service'
import { CryptoService } from '../services/crypto-service'
import { SSHGroup } from '../types/ssh'

/**
 * Encrypted storage service for SSH Groups
 * Automatically encrypts sensitive fields like passwords and private keys
 */
export class EncryptedSSHGroupStorage extends EncryptedBaseStorage {
  constructor(authService: AuthService, cryptoService: CryptoService) {
    super('ssh-groups.json', authService, cryptoService)

    // Define which fields should be encrypted
    this.setEncryptedFields([
      'defaultPassword', // Default SSH password for group
      'defaultKeyPath', // Private key path (might contain sensitive info)
      'defaultProxy.password', // Proxy password
      'defaultProxy.jumpPassword' // Jump host password
    ])
  }

  /**
   * Get all SSH groups
   */
  async getAllGroups(): Promise<SSHGroup[]> {
    const groups = await this.getAll<Record<string, unknown>>()
    return groups.map((group) => this.deserializeGroup(group as Partial<SSHGroup>))
  }

  /**
   * Get group by ID
   */
  async getGroupById(id: string): Promise<SSHGroup | null> {
    const group = await this.getById<Record<string, unknown>>(id)
    return group ? this.deserializeGroup(group as Partial<SSHGroup>) : null
  }

  /**
   * Create a new SSH group
   */
  async createGroup(groupData: Omit<SSHGroup, 'id' | 'created' | 'updated'>): Promise<SSHGroup> {
    const group: SSHGroup = {
      ...groupData,
      id: this.generateId(),
      created: new Date(),
      updated: new Date()
    }

    await this.create(group as unknown as Record<string, unknown>)
    return group
  }

  /**
   * Update an existing SSH group
   */
  async updateGroup(id: string, updates: Partial<Omit<SSHGroup, 'id' | 'created'>>): Promise<void> {
    const updateData = {
      ...updates,
      updated: new Date()
    }
    await this.update(id, updateData as Record<string, unknown>)
  }

  /**
   * Delete SSH group
   */
  async deleteGroup(id: string): Promise<void> {
    await this.delete(id)
  }

  /**
   * Get groups with their profile counts
   */
  async getGroupsWithProfileCounts(): Promise<Array<SSHGroup & { profileCount: number }>> {
    const groups = await this.getAllGroups()
    // Note: This would need to be implemented with profile counting logic
    // For now, return groups with profileCount = 0
    return groups.map((group) => ({ ...group, profileCount: 0 }))
  }

  /**
   * Custom encryption for nested proxy fields
   */
  protected override encryptItem<T extends Record<string, unknown>>(
    item: T,
    encryptionKey: Buffer
  ): T {
    const encryptedItem = { ...item } as Record<string, unknown>

    try {
      // Handle nested proxy fields
      if (encryptedItem.defaultProxy && typeof encryptedItem.defaultProxy === 'object') {
        const proxy = encryptedItem.defaultProxy as Record<string, unknown>
        const encryptedProxy = { ...proxy }

        // Encrypt proxy password fields
        for (const field of ['password', 'jumpPassword']) {
          if (proxy[field] && typeof proxy[field] === 'string') {
            try {
              encryptedProxy[field] = this.cryptoService.encrypt(
                proxy[field] as string,
                encryptionKey
              )
            } catch (error) {
              this.encLogger.error(`Failed to encrypt nested field ${field}:`, error as Error)
            }
          }
        }

        encryptedItem.defaultProxy = encryptedProxy
      }

      // Encrypt top-level fields
      for (const field of this.encryptedFields) {
        if (field.includes('.')) continue // Skip nested fields (already handled above)

        if (encryptedItem[field] && typeof encryptedItem[field] === 'string') {
          try {
            encryptedItem[field] = this.cryptoService.encrypt(
              encryptedItem[field] as string,
              encryptionKey
            )
          } catch (error) {
            this.encLogger.error(`Failed to encrypt field ${field}:`, error as Error)
          }
        }
      }

      return encryptedItem as T
    } catch (error) {
      this.encLogger.error('Failed to encrypt group item:', error as Error)
      return item
    }
  }

  /**
   * Custom decryption for nested proxy fields
   */
  protected override decryptItem<T extends Record<string, unknown>>(
    item: T,
    encryptionKey: Buffer
  ): T {
    const decryptedItem = { ...item } as Record<string, unknown>

    try {
      // Handle nested proxy fields
      if (decryptedItem.defaultProxy && typeof decryptedItem.defaultProxy === 'object') {
        const proxy = decryptedItem.defaultProxy as Record<string, unknown>
        const decryptedProxy = { ...proxy }

        // Decrypt proxy password fields
        for (const field of ['password', 'jumpPassword']) {
          if (proxy[field] && typeof proxy[field] === 'string') {
            try {
              decryptedProxy[field] = this.cryptoService.decrypt(
                proxy[field] as string,
                encryptionKey
              )
            } catch (error) {
              this.encLogger.error(`Failed to decrypt nested field ${field}:`, error as Error)
            }
          }
        }

        decryptedItem.defaultProxy = decryptedProxy
      }

      // Decrypt top-level fields
      for (const field of this.encryptedFields) {
        if (field.includes('.')) continue // Skip nested fields (already handled above)

        if (decryptedItem[field] && typeof decryptedItem[field] === 'string') {
          try {
            decryptedItem[field] = this.cryptoService.decrypt(
              decryptedItem[field] as string,
              encryptionKey
            )
          } catch (error) {
            this.encLogger.error(`Failed to decrypt field ${field}:`, error as Error)
          }
        }
      }

      return decryptedItem as T
    } catch (error) {
      this.encLogger.error('Failed to decrypt group item:', error as Error)
      return item
    }
  }

  /**
   * Deserialize group data from storage format
   */
  private deserializeGroup(data: Partial<SSHGroup>): SSHGroup {
    return {
      id: data.id || '',
      name: data.name || '',
      description: data.description,
      defaultUser: data.defaultUser,
      defaultHost: data.defaultHost,
      defaultPort: data.defaultPort,
      defaultKeyPath: data.defaultKeyPath,
      defaultPassword: data.defaultPassword,
      defaultProxy: data.defaultProxy,
      color: data.color,
      created: data.created ? new Date(data.created) : new Date(),
      updated: data.updated ? new Date(data.updated) : new Date()
    }
  }
}
