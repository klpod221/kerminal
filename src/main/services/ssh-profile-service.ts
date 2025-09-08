import {
  SSHGroup,
  SSHProfile,
  SSHProfileWithConfig,
  ResolvedSSHConfig,
  SSHProxy
} from '../types/ssh'
import { EncryptedSSHGroupStorage } from '../storage/encrypted-ssh-group-storage'
import { EncryptedSSHProfileStorage } from '../storage/encrypted-ssh-profile-storage'
import { SSHConnectionStorage } from '../storage/ssh-connection-storage'
import { AuthService } from './auth-service'
import { CryptoService } from './crypto-service'

/**
 * Service for managing SSH profiles and groups
 */
export class SSHProfileService {
  private readonly groupStorage: EncryptedSSHGroupStorage
  private readonly profileStorage: EncryptedSSHProfileStorage
  private readonly connectionStorage: SSHConnectionStorage

  constructor(authService: AuthService, cryptoService: CryptoService) {
    this.groupStorage = new EncryptedSSHGroupStorage(authService, cryptoService)
    this.profileStorage = new EncryptedSSHProfileStorage(authService, cryptoService)
    this.connectionStorage = new SSHConnectionStorage()
  }

  // Group management methods

  /**
   * Get all SSH groups
   */
  async getAllGroups(): Promise<SSHGroup[]> {
    return this.groupStorage.getAllGroups()
  }

  /**
   * Get SSH group by ID
   */
  async getGroupById(id: string): Promise<SSHGroup | null> {
    return this.groupStorage.getGroupById(id)
  }

  /**
   * Create a new SSH group
   */
  async createGroup(groupData: Omit<SSHGroup, 'id' | 'created' | 'updated'>): Promise<SSHGroup> {
    return this.groupStorage.createGroup(groupData)
  }

  /**
   * Update an existing SSH group
   */
  async updateGroup(
    id: string,
    updates: Partial<Omit<SSHGroup, 'id' | 'created'>>
  ): Promise<SSHGroup | null> {
    await this.groupStorage.updateGroup(id, updates)
    return this.groupStorage.getGroupById(id)
  }

  /**
   * Delete an SSH group and move its profiles to ungrouped
   */
  async deleteGroup(id: string): Promise<boolean> {
    // First, get all profiles in this group
    const profiles = await this.profileStorage.getByGroupId(id)

    // Update all profiles to remove group association (make them ungrouped)
    for (const profile of profiles) {
      await this.profileStorage.updateProfile(profile.id, { groupId: undefined })
    }

    // Finally delete the group itself
    await this.groupStorage.deleteGroup(id)
    return true
  }

  // Profile management methods

  /**
   * Get all SSH profiles with resolved configurations
   */
  async getAllProfiles(): Promise<SSHProfile[]> {
    const profiles = await this.profileStorage.getAll()
    return Promise.all(
      profiles.map((profile) => this.resolveProfileConfig(profile as unknown as SSHProfile))
    )
  }

  /**
   * Get SSH profile by ID with resolved configuration
   */
  async getProfileById(id: string): Promise<SSHProfileWithConfig | null> {
    const profile = await this.profileStorage.getById(id)
    if (!profile) {
      return null
    }
    return this.resolveProfileConfig(profile as unknown as SSHProfile)
  }

  /**
   * Get profiles by group ID with resolved configurations
   */
  async getProfilesByGroupId(groupId: string): Promise<SSHProfileWithConfig[]> {
    const profiles = await this.profileStorage.getByGroupId(groupId)
    return Promise.all(
      profiles.map((profile) => this.resolveProfileConfig(profile as unknown as SSHProfile))
    )
  }

  /**
   * Get favorite profiles with resolved configurations
   */
  async getFavoriteProfiles(): Promise<SSHProfileWithConfig[]> {
    const profiles = await this.profileStorage.getFavorites()
    return Promise.all(
      profiles.map((profile) => this.resolveProfileConfig(profile as unknown as SSHProfile))
    )
  }

  /**
   * Get recently connected profiles with resolved configurations
   */
  async getRecentlyConnectedProfiles(limit = 10): Promise<SSHProfileWithConfig[]> {
    const profiles = await this.profileStorage.getRecentlyConnected(limit)
    return Promise.all(
      profiles.map((profile) => this.resolveProfileConfig(profile as unknown as SSHProfile))
    )
  }

  /**
   * Create a new SSH profile
   */
  async createProfile(
    profileData: Omit<SSHProfile, 'id' | 'created' | 'updated'>
  ): Promise<SSHProfile> {
    return this.profileStorage.create(profileData) as unknown as Promise<SSHProfile>
  }

  /**
   * Update an existing SSH profile
   */
  async updateProfile(
    id: string,
    updates: Partial<Omit<SSHProfile, 'id' | 'created'>>
  ): Promise<SSHProfile | null> {
    return this.profileStorage.update(id, updates) as unknown as Promise<SSHProfile | null>
  }

  /**
   * Toggle favorite status of a profile
   */
  async toggleProfileFavorite(id: string): Promise<SSHProfile | null> {
    return this.profileStorage.toggleFavorite(id)
  }

  /**
   * Delete an SSH profile
   */
  async deleteProfile(id: string): Promise<boolean> {
    // Delete connection records for this profile
    await this.connectionStorage.deleteByProfileId(id)
    // Delete the profile itself
    return this.profileStorage.delete(id)
  }

  /**
   * Search profiles by query
   */
  async searchProfiles(query: string): Promise<SSHProfileWithConfig[]> {
    const profiles = await this.profileStorage.search(query)
    return Promise.all(profiles.map((profile) => this.resolveProfileConfig(profile)))
  }

  // Connection management methods

  /**
   * Record a new SSH connection or update existing one for the same profile
   */
  async recordConnection(profileId: string, status: 'connected' | 'failed'): Promise<void> {
    const profile = await this.profileStorage.getById(profileId)
    if (!profile) {
      throw new Error(`Profile with ID ${profileId} not found`)
    }

    const typedProfile = profile as unknown as SSHProfile

    // Record or update the connection (addConnection now handles updating existing ones)
    await this.connectionStorage.addConnection({
      profileId,
      profileName: typedProfile.name,
      host: typedProfile.host,
      user: typedProfile.user,
      connectedAt: new Date(),
      status
    })

    // Update last connected timestamp if successful
    if (status === 'connected') {
      await this.profileStorage.updateLastConnected(profileId)
    }
  }

  /**
   * Update connection status and duration
   */
  async updateConnectionStatus(
    connectionId: string,
    status: 'connected' | 'disconnected' | 'failed',
    duration?: number
  ): Promise<void> {
    await this.connectionStorage.updateConnection(connectionId, { status, duration })
  }

  /**
   * Get recent connections
   */
  async getRecentConnections(
    limit = 20
  ): Promise<Awaited<ReturnType<SSHConnectionStorage['getRecent']>>> {
    return this.connectionStorage.getRecent(limit)
  }

  /**
   * Get connection statistics
   */
  async getConnectionStats(): Promise<Awaited<ReturnType<SSHConnectionStorage['getStats']>>> {
    return this.connectionStorage.getStats()
  }

  /**
   * Clean up old connection records
   */
  async cleanupOldConnections(daysOld = 30): Promise<number> {
    return this.connectionStorage.clearOldConnections(daysOld)
  }

  // Helper methods

  /**
   * Resolve profile configuration by merging with group defaults
   */
  private async resolveProfileConfig(profile: SSHProfile): Promise<SSHProfileWithConfig> {
    let group: SSHGroup | undefined

    // Get group if profile belongs to one
    if (profile.groupId) {
      group =
        ((await this.groupStorage.getById(profile.groupId)) as unknown as SSHGroup) || undefined
    }

    // Resolve configuration with group defaults
    const resolvedConfig: ResolvedSSHConfig = {
      host: profile.host,
      port: profile.port || group?.defaultPort || 22,
      user: profile.user || group?.defaultUser || 'root',
      keyPath: profile.keyPath || group?.defaultKeyPath,
      password: profile.password || group?.defaultPassword,
      proxy: profile.proxy || group?.defaultProxy,
      commands: profile.commands
    }

    return {
      ...profile,
      resolvedConfig,
      group
    }
  }

  /**
   * Validate SSH connection configuration
   */
  validateConnectionConfig(config: ResolvedSSHConfig): { valid: boolean; errors: string[] } {
    const errors: string[] = []

    if (!config.host || config.host.trim() === '') {
      errors.push('Host is required')
    }

    if (!config.user || config.user.trim() === '') {
      errors.push('User is required')
    }

    if (config.port && (config.port < 1 || config.port > 65535)) {
      errors.push('Port must be between 1 and 65535')
    }

    if (!config.keyPath && !config.password) {
      errors.push('Either SSH key path or password is required')
    }

    return {
      valid: errors.length === 0,
      errors
    }
  }

  /**
   * Get profile groups with their profiles
   */
  async getGroupsWithProfiles(): Promise<Array<SSHGroup & { profiles: SSHProfileWithConfig[] }>> {
    const groups = await this.getAllGroups()
    const result: Array<SSHGroup & { profiles: SSHProfileWithConfig[] }> = []

    for (const group of groups) {
      const profiles = await this.getProfilesByGroupId(group.id)
      result.push({
        ...group,
        profiles
      })
    }

    return result
  }

  /**
   * Get ungrouped profiles
   */
  async getUngroupedProfiles(): Promise<SSHProfileWithConfig[]> {
    const allProfiles = await this.profileStorage.getAll()
    const ungroupedProfiles = allProfiles.filter(
      (profile) => !(profile as unknown as SSHProfile).groupId
    )
    return Promise.all(
      ungroupedProfiles.map((profile) =>
        this.resolveProfileConfig(profile as unknown as SSHProfile)
      )
    )
  }

  /**
   * Create a resolved SSH config from profile form data for testing
   */
  createResolvedConfigFromFormData(formData: {
    host: string
    port: number
    user: string
    authType: 'password' | 'key' | 'agent'
    password?: string
    privateKeyPath?: string
    proxy?: SSHProxy
  }): ResolvedSSHConfig {
    return {
      host: formData.host,
      port: formData.port,
      user: formData.user,
      password: formData.authType === 'password' ? formData.password : undefined,
      keyPath: formData.authType === 'key' ? formData.privateKeyPath : undefined,
      proxy: formData.proxy || undefined,
      commands: []
    }
  }
}
