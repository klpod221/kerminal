/**
 * Sync configuration and status interfaces
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
 * Data sync record for tracking changes
 */
export interface SyncRecord {
  id: string
  collection: string
  action: 'create' | 'update' | 'delete'
  data: Record<string, unknown>
  timestamp: Date
  synced: boolean
  deviceId: string
}

/**
 * Conflict resolution strategies
 */
export type ConflictResolution = 'local' | 'remote' | 'merge' | 'ask'

/**
 * Sync conflict information
 */
export interface SyncConflict {
  id: string
  collection: string
  localData: Record<string, unknown>
  remoteData: Record<string, unknown>
  field: string
  resolution?: ConflictResolution
}
