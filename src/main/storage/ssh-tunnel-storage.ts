import { BaseStorage } from './base-storage'
import { SSHTunnel } from '../types/ssh'
import { v4 as uuidv4 } from 'uuid'

/**
 * Storage service for SSH Tunnels
 */
export class SSHTunnelStorage extends BaseStorage {
  constructor() {
    super('ssh-tunnels.json')
  }

  /**
   * Get all SSH tunnels
   */
  async getAll(): Promise<SSHTunnel[]> {
    const tunnels = await this.readData<Record<string, unknown>>()
    return tunnels.map(this.deserializeTunnel)
  }

  /**
   * Get SSH tunnel by ID
   */
  async getById(id: string): Promise<SSHTunnel | null> {
    const tunnels = await this.getAll()
    return tunnels.find((tunnel) => tunnel.id === id) || null
  }

  /**
   * Get tunnels by profile ID
   */
  async getByProfileId(profileId: string): Promise<SSHTunnel[]> {
    const tunnels = await this.getAll()
    return tunnels.filter((tunnel) => tunnel.profileId === profileId)
  }

  /**
   * Get running tunnels
   */
  async getRunning(): Promise<SSHTunnel[]> {
    const tunnels = await this.getAll()
    return tunnels.filter((tunnel) => tunnel.status === 'running')
  }

  /**
   * Get auto-start tunnels
   */
  async getAutoStart(): Promise<SSHTunnel[]> {
    const tunnels = await this.getAll()
    return tunnels.filter((tunnel) => tunnel.autoStart)
  }

  /**
   * Create a new SSH tunnel
   */
  async create(tunnelData: Omit<SSHTunnel, 'id' | 'created' | 'updated'>): Promise<SSHTunnel> {
    const tunnel: SSHTunnel = {
      ...tunnelData,
      id: uuidv4(),
      created: new Date(),
      updated: new Date()
    }

    const tunnels = await this.getAll()
    tunnels.push(tunnel)
    await this.writeData(tunnels.map(this.serializeTunnel))

    return tunnel
  }

  /**
   * Update an existing SSH tunnel
   */
  async update(
    id: string,
    updates: Partial<Omit<SSHTunnel, 'id' | 'created'>>
  ): Promise<SSHTunnel | null> {
    const tunnels = await this.getAll()
    const index = tunnels.findIndex((tunnel) => tunnel.id === id)

    if (index === -1) {
      return null
    }

    const updatedTunnel: SSHTunnel = {
      ...tunnels[index],
      ...updates,
      updated: new Date()
    }

    tunnels[index] = updatedTunnel
    await this.writeData(tunnels.map(this.serializeTunnel))

    return updatedTunnel
  }

  /**
   * Delete an SSH tunnel
   */
  async delete(id: string): Promise<boolean> {
    const tunnels = await this.getAll()
    const filteredTunnels = tunnels.filter((tunnel) => tunnel.id !== id)

    if (filteredTunnels.length === tunnels.length) {
      return false
    }

    await this.writeData(filteredTunnels.map(this.serializeTunnel))
    return true
  }

  /**
   * Delete tunnels by profile ID
   */
  async deleteByProfileId(profileId: string): Promise<number> {
    const tunnels = await this.getAll()
    const filteredTunnels = tunnels.filter((tunnel) => tunnel.profileId !== profileId)
    const deletedCount = tunnels.length - filteredTunnels.length

    if (deletedCount > 0) {
      await this.writeData(filteredTunnels.map(this.serializeTunnel))
    }

    return deletedCount
  }

  /**
   * Update tunnel status
   */
  async updateStatus(
    id: string,
    status: SSHTunnel['status'],
    error?: string
  ): Promise<SSHTunnel | null> {
    const updates: Partial<SSHTunnel> = { status }

    if (status === 'running') {
      updates.lastStarted = new Date()
      updates.lastError = undefined
    } else if (status === 'error' && error) {
      updates.lastError = error
    }

    return this.update(id, updates)
  }

  /**
   * Check if port is already in use by another tunnel
   */
  async isPortInUse(port: number, excludeId?: string): Promise<boolean> {
    const tunnels = await this.getAll()
    return tunnels.some(
      (tunnel) =>
        tunnel.localPort === port &&
        tunnel.status !== 'stopped' &&
        tunnel.status !== 'error' &&
        tunnel.id !== excludeId
    )
  }

  /**
   * Serialize tunnel for storage
   */
  private serializeTunnel(tunnel: SSHTunnel): Record<string, unknown> {
    return {
      ...tunnel,
      created: tunnel.created.toISOString(),
      updated: tunnel.updated.toISOString(),
      lastStarted: tunnel.lastStarted?.toISOString()
    }
  }

  /**
   * Deserialize tunnel from storage
   */
  private deserializeTunnel(data: Record<string, unknown>): SSHTunnel {
    return {
      ...data,
      created: new Date(data.created as string),
      updated: new Date(data.updated as string),
      lastStarted: data.lastStarted ? new Date(data.lastStarted as string) : undefined
    } as SSHTunnel
  }
}
