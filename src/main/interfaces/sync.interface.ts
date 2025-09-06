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
  conflictResolutionStrategy?: ConflictResolutionStrategy
  retainTombstoneDays?: number // How long to keep tombstone records
  enableVersionHistory?: boolean
  maxVersionHistory?: number // Maximum versions to keep
}

/**
 * Sync status for monitoring
 */
export interface SyncStatus {
  isConnected: boolean
  lastSync?: Date
  lastError?: string
  isLoading: boolean
  totalItems?: number
  syncedItems?: number
  conflictCount?: number
  tombstoneCount?: number
  syncInProgress?: boolean
}

/**
 * Data version information
 */
export interface DataVersion {
  version: number
  timestamp: Date
  deviceId: string
  hash?: string // Content hash for integrity check
}

/**
 * Tombstone record for tracking deletions
 */
export interface TombstoneRecord {
  id: string
  collection: string
  deletedAt: Date
  deletedBy: string // deviceId
  version: number
}

/**
 * Sync metadata for each data item
 */
export interface SyncMetadata {
  id: string
  version: DataVersion
  isDeleted: boolean
  tombstone?: TombstoneRecord
}

/**
 * Data sync record for tracking changes
 */
export interface SyncRecord {
  id: string
  collection: string
  action: 'create' | 'update' | 'delete'
  data?: Record<string, unknown> // undefined for delete operations
  version: DataVersion
  previousVersion?: number
  isTombstone: boolean
  timestamp: Date
  synced: boolean
  deviceId: string
}

/**
 * Conflict resolution strategies
 */
export type ConflictResolutionStrategy =
  | 'timestamp' // Newer timestamp wins
  | 'version' // Higher version number wins
  | 'manual' // Ask user
  | 'merge' // Attempt to merge changes
  | 'last-writer-wins' // Always use the latest change

/**
 * Legacy conflict resolution for backward compatibility
 */
export type ConflictResolution = 'local' | 'remote' | 'merge' | 'ask'

/**
 * Sync operation result
 */
export interface SyncOperationResult {
  success: boolean
  itemsProcessed: number
  conflictsResolved: number
  tombstonesProcessed: number
  errors: string[]
}

/**
 * Device information for tracking sync sources
 */
export interface DeviceInfo {
  deviceId: string
  deviceName: string
  platform: string
  lastSeen: Date
  version: string
}

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
