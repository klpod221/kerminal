/**
 * Sync configuration and status types for frontend
 */

/**
 * MongoDB sync configuration
 */
export interface SyncConfig {
  mongoUri: string
  databaseName: string
  enabled: boolean
  lastSync?: Date
  autoSync: boolean
  syncInterval: number // seconds
}

/**
 * Sync status for monitoring
 */
export interface SyncStatus {
  isConnected: boolean
  lastSync?: Date
  lastError?: string
  isLoading: boolean
}

/**
 * Conflict resolution strategies
 */
export type ConflictResolution = 'local' | 'remote' | 'merge' | 'ask'
