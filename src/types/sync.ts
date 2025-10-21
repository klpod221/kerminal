/**
 * MongoDB sync configuration (Legacy)
 */
export interface SyncConfig {
  mongoUri: string;
  databaseName: string;
  enabled: boolean;
  lastSync?: Date;
  autoSync: boolean;
  syncInterval: number; // seconds
}

/**
 * Sync status for monitoring (Legacy)
 */
export interface SyncStatus {
  isConnected: boolean;
  lastSync?: Date;
  lastError?: string;
  isLoading: boolean;
}

/**
 * Conflict resolution strategies (Legacy)
 */
export type ConflictResolution = "local" | "remote" | "merge" | "ask";

// Multi-database sync types

export type DatabaseType = "mysql" | "postgresql" | "mongodb";

export type ConflictResolutionStrategy =
  | "LastWriteWins"
  | "FirstWriteWins"
  | "Manual"
  | "LocalWins"
  | "RemoteWins";

export interface SyncSettings {
  autoSync: boolean;
  syncIntervalMinutes: number;
  conflictResolutionStrategy: ConflictResolutionStrategy;
}

export interface ConnectionDetails {
  host: string;
  port: number;
  username: string;
  password: string;
  database: string;
}

export interface ExternalDatabaseConfig {
  id: string;
  name: string;
  dbType: DatabaseType;
  connectionDetailsEncrypted: string;
  syncSettings: string;
  isActive: boolean;
  autoSyncEnabled: boolean;
  lastSyncAt?: string;
  createdAt: string;
  updatedAt: string;
  deviceId: string;
  version: number;
  syncStatus: string;
}

export type SyncDirection = "Push" | "Pull" | "Bidirectional";

export type SyncLogStatus = "InProgress" | "Completed" | "Failed" | "Cancelled";

export interface SyncLog {
  id: string;
  databaseId: string;
  deviceId: string;
  direction: SyncDirection;
  status: SyncLogStatus;
  startedAt: string;
  completedAt?: string;
  recordsSynced: number;
  conflictsResolved: number;
  manualConflicts: number;
  errorMessage?: string;
}

export interface ConflictResolutionData {
  id: string;
  entityType: string;
  entityId: string;
  localData: any;
  remoteData: any;
  resolutionStrategy?: ConflictResolutionStrategy;
  resolvedAt?: string;
  createdAt: string;
}

export interface SyncServiceStatus {
  isConnected: boolean;
  lastSync?: SyncLog;
  schedulerEnabled: boolean;
}

export interface SyncServiceStatistics {
  totalConnections: number;
  activeDatabaseIds: string[];
  enabledDatabases: number;
  lastSyncTime?: string;
}

export interface Device {
  deviceId: string;
  deviceName: string;
  deviceType: "Desktop" | "Laptop" | "Mobile" | "Server" | "Unknown";
  osInfo: {
    osType: string;
    osVersion: string;
    arch: string;
    hostname: string;
  };
  appVersion: string;
  createdAt: string;
  lastSeen: string;
  isCurrent: boolean;
}
