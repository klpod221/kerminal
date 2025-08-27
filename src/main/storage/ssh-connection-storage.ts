import { BaseStorage } from './base-storage'
import { SSHConnection } from '../types/ssh'
import { v4 as uuidv4 } from 'uuid'

/**
 * Storage service for SSH Connection history
 */
export class SSHConnectionStorage extends BaseStorage {
  constructor() {
    super('ssh-connections.json')
  }

  /**
   * Get all SSH connections
   */
  async getAll(): Promise<SSHConnection[]> {
    const connections = await this.readData<Record<string, unknown>>()
    return connections.map(this.deserializeConnection)
  }

  /**
   * Get recent connections (one per profile)
   */
  async getRecent(limit = 20): Promise<SSHConnection[]> {
    const connections = await this.getAll()

    // Sort by most recent first
    const sortedConnections = connections.toSorted(
      (a, b) => b.connectedAt.getTime() - a.connectedAt.getTime()
    )

    // Get unique connections by profile ID (most recent for each profile)
    const uniqueConnections: SSHConnection[] = []
    const seenProfileIds = new Set<string>()

    for (const connection of sortedConnections) {
      if (!seenProfileIds.has(connection.profileId)) {
        uniqueConnections.push(connection)
        seenProfileIds.add(connection.profileId)
      }
    }

    return uniqueConnections.slice(0, limit)
  }

  /**
   * Get connections by profile ID
   */
  async getByProfileId(profileId: string): Promise<SSHConnection[]> {
    const connections = await this.getAll()
    return connections
      .filter((connection) => connection.profileId === profileId)
      .sort((a, b) => b.connectedAt.getTime() - a.connectedAt.getTime())
  }

  /**
   * Get connections by status
   */
  async getByStatus(status: SSHConnection['status']): Promise<SSHConnection[]> {
    const connections = await this.getAll()
    return connections
      .filter((connection) => connection.status === status)
      .sort((a, b) => b.connectedAt.getTime() - a.connectedAt.getTime())
  }

  /**
   * Add a new connection record or update existing one for the same profile
   */
  async addConnection(connectionData: Omit<SSHConnection, 'id'>): Promise<SSHConnection> {
    const connections = await this.getAll()

    // Find existing connection for the same profile
    const existingConnectionIndex = connections.findIndex(
      (conn) => conn.profileId === connectionData.profileId
    )

    let updatedConnection: SSHConnection

    if (existingConnectionIndex !== -1) {
      // Update existing connection with new timestamp and status
      updatedConnection = {
        ...connections[existingConnectionIndex],
        connectedAt: connectionData.connectedAt,
        status: connectionData.status,
        duration: connectionData.duration
      }
      connections[existingConnectionIndex] = updatedConnection
    } else {
      // Create new connection if none exists for this profile
      updatedConnection = {
        id: uuidv4(),
        ...connectionData
      }
      connections.push(updatedConnection)
    }

    // Keep only the last 1000 connections to prevent storage bloat
    const sortedConnections = connections.toSorted(
      (a, b) => b.connectedAt.getTime() - a.connectedAt.getTime()
    )
    const recentConnections = sortedConnections.slice(0, 1000)

    await this.writeData(recentConnections.map(this.serializeConnection))

    return updatedConnection
  }

  /**
   * Update connection status and duration
   */
  async updateConnection(
    id: string,
    updates: Partial<Pick<SSHConnection, 'status' | 'duration'>>
  ): Promise<SSHConnection | null> {
    const connections = await this.getAll()
    const index = connections.findIndex((connection) => connection.id === id)

    if (index === -1) {
      return null
    }

    const updatedConnection: SSHConnection = {
      ...connections[index],
      ...updates
    }

    connections[index] = updatedConnection
    await this.writeData(connections.map(this.serializeConnection))

    return updatedConnection
  }

  /**
   * Delete connection records by profile ID
   */
  async deleteByProfileId(profileId: string): Promise<number> {
    const connections = await this.getAll()
    const filteredConnections = connections.filter(
      (connection) => connection.profileId !== profileId
    )
    const deletedCount = connections.length - filteredConnections.length

    if (deletedCount > 0) {
      await this.writeData(filteredConnections.map(this.serializeConnection))
    }

    return deletedCount
  }

  /**
   * Clear old connection records (older than specified days)
   */
  async clearOldConnections(daysOld = 30): Promise<number> {
    const connections = await this.getAll()
    const cutoffDate = new Date()
    cutoffDate.setDate(cutoffDate.getDate() - daysOld)

    const recentConnections = connections.filter(
      (connection) => connection.connectedAt > cutoffDate
    )
    const deletedCount = connections.length - recentConnections.length

    if (deletedCount > 0) {
      await this.writeData(recentConnections.map(this.serializeConnection))
    }

    return deletedCount
  }

  /**
   * Get connection statistics
   */
  async getStats(): Promise<{
    totalConnections: number
    uniqueProfiles: number
    successfulConnections: number
    failedConnections: number
    averageDuration: number
  }> {
    const connections = await this.getAll()
    const uniqueProfiles = new Set(connections.map((c) => c.profileId)).size
    const successfulConnections = connections.filter((c) => c.status === 'connected').length
    const failedConnections = connections.filter((c) => c.status === 'failed').length
    const connectionsWithDuration = connections.filter((c) => c.duration && c.duration > 0)
    const averageDuration =
      connectionsWithDuration.length > 0
        ? connectionsWithDuration.reduce((sum, c) => sum + (c.duration || 0), 0) /
          connectionsWithDuration.length
        : 0

    return {
      totalConnections: connections.length,
      uniqueProfiles,
      successfulConnections,
      failedConnections,
      averageDuration: Math.round(averageDuration)
    }
  }

  /**
   * Serialize connection for storage (convert dates to strings)
   */
  private serializeConnection(connection: SSHConnection): Record<string, unknown> {
    return {
      ...connection,
      connectedAt: connection.connectedAt.toISOString()
    }
  }

  /**
   * Deserialize connection from storage (convert strings to dates)
   */
  private deserializeConnection(data: Record<string, unknown>): SSHConnection {
    return {
      ...data,
      connectedAt: new Date(data.connectedAt as string)
    } as SSHConnection
  }
}
