import { BaseStorage } from './base-storage'
import { SSHGroup } from '../types/ssh'
import { v4 as uuidv4 } from 'uuid'

/**
 * Storage service for SSH Groups
 */
export class SSHGroupStorage extends BaseStorage {
  constructor() {
    super('ssh-groups.json')
  }

  /**
   * Get all SSH groups
   */
  async getAll(): Promise<SSHGroup[]> {
    const groups = await this.readData<Record<string, unknown>>()
    return groups.map(this.deserializeGroup)
  }

  /**
   * Get SSH group by ID
   */
  async getById(id: string): Promise<SSHGroup | null> {
    const groups = await this.getAll()
    return groups.find((group) => group.id === id) || null
  }

  /**
   * Create a new SSH group
   */
  async create(groupData: Omit<SSHGroup, 'id' | 'created' | 'updated'>): Promise<SSHGroup> {
    const groups = await this.getAll()
    const now = new Date()

    const newGroup: SSHGroup = {
      id: uuidv4(),
      ...groupData,
      created: now,
      updated: now
    }

    groups.push(newGroup)
    await this.writeData(groups.map(this.serializeGroup))

    return newGroup
  }

  /**
   * Update an existing SSH group
   */
  async update(
    id: string,
    updates: Partial<Omit<SSHGroup, 'id' | 'created'>>
  ): Promise<SSHGroup | null> {
    const groups = await this.getAll()
    const index = groups.findIndex((group) => group.id === id)

    if (index === -1) {
      return null
    }

    const updatedGroup: SSHGroup = {
      ...groups[index],
      ...updates,
      updated: new Date()
    }

    groups[index] = updatedGroup
    await this.writeData(groups.map(this.serializeGroup))

    return updatedGroup
  }

  /**
   * Delete an SSH group
   */
  async delete(id: string): Promise<boolean> {
    const groups = await this.getAll()
    const initialLength = groups.length
    const filteredGroups = groups.filter((group) => group.id !== id)

    if (filteredGroups.length === initialLength) {
      return false
    }

    await this.writeData(filteredGroups.map(this.serializeGroup))
    return true
  }

  /**
   * Check if a group exists
   */
  async groupExists(id: string): Promise<boolean> {
    const group = await this.getById(id)
    return group !== null
  }

  /**
   * Serialize group for storage (convert dates to strings)
   */
  private serializeGroup(group: SSHGroup): Record<string, unknown> {
    return {
      ...group,
      created: group.created.toISOString(),
      updated: group.updated.toISOString()
    }
  }

  /**
   * Deserialize group from storage (convert strings to dates)
   */
  private deserializeGroup(data: Record<string, unknown>): SSHGroup {
    return {
      ...data,
      created: new Date(data.created as string),
      updated: new Date(data.updated as string)
    } as SSHGroup
  }
}
