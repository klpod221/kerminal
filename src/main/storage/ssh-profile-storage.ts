import { BaseStorage } from './base-storage'
import { SSHProfile } from '../types/ssh'
import { v4 as uuidv4 } from 'uuid'

/**
 * Storage service for SSH Profiles
 */
export class SSHProfileStorage extends BaseStorage {
  constructor() {
    super('ssh-profiles.json')
  }

  /**
   * Get all SSH profiles
   */
  async getAll(): Promise<SSHProfile[]> {
    const profiles = await this.readData<Record<string, unknown>>()
    return profiles.map(this.deserializeProfile)
  }

  /**
   * Get SSH profile by ID
   */
  async getById(id: string): Promise<SSHProfile | null> {
    const profiles = await this.getAll()
    return profiles.find((profile) => profile.id === id) || null
  }

  /**
   * Get profiles by group ID
   */
  async getByGroupId(groupId: string): Promise<SSHProfile[]> {
    const profiles = await this.getAll()
    return profiles.filter((profile) => profile.groupId === groupId)
  }

  /**
   * Get favorite profiles
   */
  async getFavorites(): Promise<SSHProfile[]> {
    const profiles = await this.getAll()
    return profiles.filter((profile) => profile.favorite)
  }

  /**
   * Get recently connected profiles
   */
  async getRecentlyConnected(limit = 10): Promise<SSHProfile[]> {
    const profiles = await this.getAll()
    return profiles
      .filter((profile) => profile.lastConnected)
      .sort((a, b) => {
        const aTime = a.lastConnected?.getTime() || 0
        const bTime = b.lastConnected?.getTime() || 0
        return bTime - aTime
      })
      .slice(0, limit)
  }

  /**
   * Create a new SSH profile
   */
  async create(profileData: Omit<SSHProfile, 'id' | 'created' | 'updated'>): Promise<SSHProfile> {
    const profiles = await this.getAll()
    const now = new Date()

    const newProfile: SSHProfile = {
      id: uuidv4(),
      ...profileData,
      created: now,
      updated: now
    }

    profiles.push(newProfile)
    await this.writeData(profiles.map(this.serializeProfile))

    return newProfile
  }

  /**
   * Update an existing SSH profile
   */
  async update(
    id: string,
    updates: Partial<Omit<SSHProfile, 'id' | 'created'>>
  ): Promise<SSHProfile | null> {
    const profiles = await this.getAll()
    const index = profiles.findIndex((profile) => profile.id === id)

    if (index === -1) {
      return null
    }

    const updatedProfile: SSHProfile = {
      ...profiles[index],
      ...updates,
      updated: new Date()
    }

    profiles[index] = updatedProfile
    await this.writeData(profiles.map(this.serializeProfile))

    return updatedProfile
  }

  /**
   * Update last connected timestamp
   */
  async updateLastConnected(id: string): Promise<SSHProfile | null> {
    return this.update(id, { lastConnected: new Date() })
  }

  /**
   * Toggle favorite status
   */
  async toggleFavorite(id: string): Promise<SSHProfile | null> {
    const profile = await this.getById(id)
    if (!profile) {
      return null
    }

    return this.update(id, { favorite: !profile.favorite })
  }

  /**
   * Delete an SSH profile
   */
  async delete(id: string): Promise<boolean> {
    const profiles = await this.getAll()
    const initialLength = profiles.length
    const filteredProfiles = profiles.filter((profile) => profile.id !== id)

    if (filteredProfiles.length === initialLength) {
      return false
    }

    await this.writeData(filteredProfiles.map(this.serializeProfile))
    return true
  }

  /**
   * Delete all profiles in a group
   */
  async deleteByGroupId(groupId: string): Promise<number> {
    const profiles = await this.getAll()
    const filteredProfiles = profiles.filter((profile) => profile.groupId !== groupId)
    const deletedCount = profiles.length - filteredProfiles.length

    if (deletedCount > 0) {
      await this.writeData(filteredProfiles.map(this.serializeProfile))
    }

    return deletedCount
  }

  /**
   * Search profiles by name or host
   */
  async search(query: string): Promise<SSHProfile[]> {
    const profiles = await this.getAll()
    const searchTerm = query.toLowerCase()

    return profiles.filter(
      (profile) =>
        profile.name.toLowerCase().includes(searchTerm) ||
        profile.host.toLowerCase().includes(searchTerm) ||
        profile.user.toLowerCase().includes(searchTerm) ||
        profile.description?.toLowerCase().includes(searchTerm)
    )
  }

  /**
   * Serialize profile for storage (convert dates to strings)
   */
  private serializeProfile(profile: SSHProfile): Record<string, unknown> {
    return {
      ...profile,
      created: profile.created.toISOString(),
      updated: profile.updated.toISOString(),
      lastConnected: profile.lastConnected?.toISOString() || null
    }
  }

  /**
   * Deserialize profile from storage (convert strings to dates)
   */
  private deserializeProfile(data: Record<string, unknown>): SSHProfile {
    return {
      ...data,
      created: new Date(data.created as string),
      updated: new Date(data.updated as string),
      lastConnected: data.lastConnected ? new Date(data.lastConnected as string) : undefined
    } as SSHProfile
  }
}
