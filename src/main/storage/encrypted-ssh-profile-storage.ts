import { EncryptedBaseStorage } from './encrypted-base-storage'
import { AuthService } from '../services/auth-service'
import { CryptoService } from '../services/crypto-service'
import { SSHProfile } from '../types/ssh'

/**
 * Encrypted storage service for SSH Profiles
 * Automatically encrypts sensitive fields like passwords and private keys
 */
export class EncryptedSSHProfileStorage extends EncryptedBaseStorage {
  constructor(authService: AuthService, cryptoService: CryptoService) {
    super('ssh-profiles.json', authService, cryptoService)

    // Define which fields should be encrypted
    this.setEncryptedFields([
      'password', // SSH password
      'keyPath', // Private key path (might contain sensitive info)
      'proxy.password', // Proxy password
      'proxy.jumpPassword' // Jump host password
    ])
  }

  /**
   * Get all SSH profiles
   */
  async getAllProfiles(): Promise<SSHProfile[]> {
    const profiles = await this.getAll<Record<string, unknown>>()
    return profiles.map((profile) => this.deserializeProfile(profile as Partial<SSHProfile>))
  }

  /**
   * Get SSH profile by ID
   */
  async getProfileById(id: string): Promise<SSHProfile | null> {
    const profile = await this.getById<Record<string, unknown>>(id)
    return profile ? this.deserializeProfile(profile as Partial<SSHProfile>) : null
  }

  /**
   * Get profiles by group ID
   */
  async getByGroupId(groupId: string): Promise<SSHProfile[]> {
    const profiles = await this.getAllProfiles()
    return profiles.filter((profile) => profile.groupId === groupId)
  }

  /**
   * Get favorite profiles
   */
  async getFavorites(): Promise<SSHProfile[]> {
    const profiles = await this.getAllProfiles()
    return profiles.filter((profile) => profile.favorite)
  }

  /**
   * Get recently connected profiles
   */
  async getRecentlyConnected(limit = 10): Promise<SSHProfile[]> {
    const profiles = await this.getAllProfiles()
    return profiles
      .filter((profile) => profile.lastConnected)
      .sort((a, b) => {
        const aTime = a.lastConnected ? new Date(a.lastConnected).getTime() : 0
        const bTime = b.lastConnected ? new Date(b.lastConnected).getTime() : 0
        return bTime - aTime
      })
      .slice(0, limit)
  }

  /**
   * Create a new SSH profile
   */
  async createProfile(
    profileData: Omit<SSHProfile, 'id' | 'created' | 'updated'>
  ): Promise<SSHProfile> {
    const created = await this.create<Record<string, unknown>>(
      profileData as Record<string, unknown>
    )
    return this.deserializeProfile(created as Partial<SSHProfile>)
  }

  /**
   * Update an existing SSH profile
   */
  async updateProfile(
    id: string,
    updates: Partial<Omit<SSHProfile, 'id' | 'created'>>
  ): Promise<SSHProfile | null> {
    const updated = await this.update<Record<string, unknown>>(
      id,
      updates as Record<string, unknown>
    )
    return updated ? this.deserializeProfile(updated as Partial<SSHProfile>) : null
  }

  /**
   * Toggle favorite status of a profile
   */
  async toggleFavorite(id: string): Promise<SSHProfile | null> {
    const profile = await this.getProfileById(id)
    if (!profile) {
      return null
    }

    return this.updateProfile(id, { favorite: !profile.favorite })
  }

  /**
   * Update last connected timestamp
   */
  async updateLastConnected(id: string): Promise<SSHProfile | null> {
    return this.updateProfile(id, { lastConnected: new Date() })
  }

  /**
   * Search profiles by query
   */
  async search(query: string): Promise<SSHProfile[]> {
    const profiles = await this.getAllProfiles()
    const searchTerm = query.toLowerCase()

    return profiles.filter(
      (profile) =>
        profile.name.toLowerCase().includes(searchTerm) ||
        profile.host.toLowerCase().includes(searchTerm) ||
        profile.user.toLowerCase().includes(searchTerm) ||
        (profile.description && profile.description.toLowerCase().includes(searchTerm))
    )
  }

  /**
   * Delete an SSH profile
   */
  async deleteProfile(id: string): Promise<boolean> {
    return this.delete(id)
  }

  /**
   * Deserialize profile data and ensure proper types
   */
  private deserializeProfile(data: Partial<SSHProfile>): SSHProfile {
    return {
      id: data.id || '',
      name: data.name || '',
      description: data.description || undefined,
      host: data.host || '',
      port:
        typeof data.port === 'number' ? data.port : data.port ? parseInt(String(data.port)) : 22,
      user: data.user || '',
      password: data.password || undefined,
      keyPath: data.keyPath || undefined,
      groupId: data.groupId || undefined,
      color: data.color || '#6b7280',
      favorite: Boolean(data.favorite),
      lastConnected: data.lastConnected ? new Date(data.lastConnected) : undefined,
      proxy: data.proxy
        ? {
            type: data.proxy.type || 'none',
            host: data.proxy.host || '',
            port:
              typeof data.proxy.port === 'number'
                ? data.proxy.port
                : data.proxy.port
                  ? parseInt(String(data.proxy.port))
                  : 1080,
            username: data.proxy.username || undefined,
            password: data.proxy.password || undefined,
            jumpHost: data.proxy.jumpHost || undefined,
            jumpPort:
              typeof data.proxy.jumpPort === 'number'
                ? data.proxy.jumpPort
                : data.proxy.jumpPort
                  ? parseInt(String(data.proxy.jumpPort))
                  : 22,
            jumpUser: data.proxy.jumpUser || undefined,
            jumpKeyPath: data.proxy.jumpKeyPath || undefined,
            jumpPassword: data.proxy.jumpPassword || undefined
          }
        : undefined,
      commands: data.commands || [],
      created: data.created ? new Date(data.created) : new Date(),
      updated: data.updated ? new Date(data.updated) : new Date()
    }
  }

  /**
   * Handle deep encryption for nested objects like proxy
   */
  protected encryptItem<T extends Record<string, unknown>>(item: T, encryptionKey: Buffer): T {
    const encrypted = { ...item } as Record<string, unknown>

    for (const field of this.encryptedFields) {
      if (field.includes('.')) {
        // Handle nested fields like 'proxy.password'
        const [parentField, childField] = field.split('.')
        const parentValue = encrypted[parentField]

        if (parentValue && typeof parentValue === 'object' && !Array.isArray(parentValue)) {
          const parentObj = parentValue as Record<string, unknown>
          if (parentObj[childField] != null && parentObj[childField] !== '') {
            try {
              const plaintext = String(parentObj[childField])
              parentObj[childField] = this.cryptoService.encrypt(plaintext, encryptionKey)
            } catch (error) {
              this.encLogger.error(`Failed to encrypt nested field ${field}:`, error as Error)
              throw new Error(`Failed to encrypt sensitive field: ${field}`)
            }
          }
        }
      } else {
        // Handle top-level fields
        if (encrypted[field] != null && encrypted[field] !== '') {
          try {
            const plaintext = String(encrypted[field])
            encrypted[field] = this.cryptoService.encrypt(plaintext, encryptionKey)
          } catch (error) {
            this.encLogger.error(`Failed to encrypt field ${field}:`, error as Error)
            throw new Error(`Failed to encrypt sensitive field: ${field}`)
          }
        }
      }
    }

    return encrypted as T
  }

  /**
   * Handle deep decryption for nested objects like proxy
   */
  protected decryptItem<T extends Record<string, unknown>>(item: T, encryptionKey: Buffer): T {
    const decrypted = { ...item } as Record<string, unknown>

    for (const field of this.encryptedFields) {
      if (field.includes('.')) {
        // Handle nested fields like 'proxy.password'
        const [parentField, childField] = field.split('.')
        const parentValue = decrypted[parentField]

        if (parentValue && typeof parentValue === 'object' && !Array.isArray(parentValue)) {
          const parentObj = parentValue as Record<string, unknown>
          if (parentObj[childField] != null && parentObj[childField] !== '') {
            try {
              const encryptedValue = String(parentObj[childField])
              if (encryptedValue.includes(':') && encryptedValue.split(':').length === 4) {
                parentObj[childField] = this.cryptoService.decrypt(encryptedValue, encryptionKey)
              }
            } catch (error) {
              this.encLogger.error(`Failed to decrypt nested field ${field}:`, error as Error)
              parentObj[childField] = '' // Clear on decryption failure for security
            }
          }
        }
      } else {
        // Handle top-level fields
        if (decrypted[field] != null && decrypted[field] !== '') {
          try {
            const encryptedValue = String(decrypted[field])
            if (encryptedValue.includes(':') && encryptedValue.split(':').length === 4) {
              decrypted[field] = this.cryptoService.decrypt(encryptedValue, encryptionKey)
            }
          } catch (error) {
            this.encLogger.error(`Failed to decrypt field ${field}:`, error as Error)
            decrypted[field] = '' // Clear on decryption failure for security
          }
        }
      }
    }

    return decrypted as T
  }
}
