import { SSHGroup, SSHProfile, SSHProfileWithConfig, ResolvedSSHConfig } from '../types/ssh'
import { SSHGroupStorage } from '../storage/ssh-group-storage'
import { SSHProfileStorage } from '../storage/ssh-profile-storage'
import { SSHConnectionStorage } from '../storage/ssh-connection-storage'

/**
 * Service for managing SSH profiles and groups
 */
export class SSHProfileService {
  private readonly groupStorage: SSHGroupStorage
  private readonly profileStorage: SSHProfileStorage
  private readonly connectionStorage: SSHConnectionStorage

  constructor() {
    this.groupStorage = new SSHGroupStorage()
    this.profileStorage = new SSHProfileStorage()
    this.connectionStorage = new SSHConnectionStorage()
  }

  // Group management methods

  /**
   * Get all SSH groups
   */
  async getAllGroups(): Promise<SSHGroup[]> {
    return this.groupStorage.getAll()
  }

  /**
   * Get SSH group by ID
   */
  async getGroupById(id: string): Promise<SSHGroup | null> {
    return this.groupStorage.getById(id)
  }

  /**
   * Create a new SSH group
   */
  async createGroup(groupData: Omit<SSHGroup, 'id' | 'created' | 'updated'>): Promise<SSHGroup> {
    return this.groupStorage.create(groupData)
  }

  /**
   * Update an existing SSH group
   */
  async updateGroup(
    id: string,
    updates: Partial<Omit<SSHGroup, 'id' | 'created'>>
  ): Promise<SSHGroup | null> {
    return this.groupStorage.update(id, updates)
  }

  /**
   * Delete an SSH group and move its profiles to ungrouped
   */
  async deleteGroup(id: string): Promise<boolean> {
    // First, get all profiles in this group
    const profiles = await this.profileStorage.getByGroupId(id)

    // Update all profiles to remove group association (make them ungrouped)
    for (const profile of profiles) {
      await this.profileStorage.update(profile.id, { groupId: undefined })
    }

    // Finally delete the group itself
    return this.groupStorage.delete(id)
  }

  // Profile management methods

  /**
   * Get all SSH profiles with resolved configurations
   */
  async getAllProfiles(): Promise<SSHProfileWithConfig[]> {
    const profiles = await this.profileStorage.getAll()
    return Promise.all(profiles.map((profile) => this.resolveProfileConfig(profile)))
  }

  /**
   * Get SSH profile by ID with resolved configuration
   */
  async getProfileById(id: string): Promise<SSHProfileWithConfig | null> {
    const profile = await this.profileStorage.getById(id)
    if (!profile) {
      return null
    }
    return this.resolveProfileConfig(profile)
  }

  /**
   * Get profiles by group ID with resolved configurations
   */
  async getProfilesByGroupId(groupId: string): Promise<SSHProfileWithConfig[]> {
    const profiles = await this.profileStorage.getByGroupId(groupId)
    return Promise.all(profiles.map((profile) => this.resolveProfileConfig(profile)))
  }

  /**
   * Get favorite profiles with resolved configurations
   */
  async getFavoriteProfiles(): Promise<SSHProfileWithConfig[]> {
    const profiles = await this.profileStorage.getFavorites()
    return Promise.all(profiles.map((profile) => this.resolveProfileConfig(profile)))
  }

  /**
   * Get recently connected profiles with resolved configurations
   */
  async getRecentlyConnectedProfiles(limit = 10): Promise<SSHProfileWithConfig[]> {
    const profiles = await this.profileStorage.getRecentlyConnected(limit)
    return Promise.all(profiles.map((profile) => this.resolveProfileConfig(profile)))
  }

  /**
   * Create a new SSH profile
   */
  async createProfile(
    profileData: Omit<SSHProfile, 'id' | 'created' | 'updated'>
  ): Promise<SSHProfile> {
    return this.profileStorage.create(profileData)
  }

  /**
   * Update an existing SSH profile
   */
  async updateProfile(
    id: string,
    updates: Partial<Omit<SSHProfile, 'id' | 'created'>>
  ): Promise<SSHProfile | null> {
    return this.profileStorage.update(id, updates)
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

    // Record or update the connection (addConnection now handles updating existing ones)
    await this.connectionStorage.addConnection({
      profileId,
      profileName: profile.name,
      host: profile.host,
      user: profile.user,
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
      group = (await this.groupStorage.getById(profile.groupId)) || undefined
    }

    // Resolve configuration with group defaults
    const resolvedConfig: ResolvedSSHConfig = {
      host: profile.host,
      port: profile.port || group?.defaultPort || 22,
      user: profile.user || group?.defaultUser || 'root',
      keyPath: profile.keyPath || group?.defaultKeyPath,
      password: profile.password || group?.defaultPassword,
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
    const ungroupedProfiles = allProfiles.filter((profile) => !profile.groupId)
    return Promise.all(ungroupedProfiles.map((profile) => this.resolveProfileConfig(profile)))
  }
}
