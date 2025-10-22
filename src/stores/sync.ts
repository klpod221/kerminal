import { defineStore } from "pinia";
import { ref, computed } from "vue";
import type {
  ExternalDatabaseConfig,
  SyncLog,
  SyncServiceStatus,
  SyncServiceStatistics,
  ConflictResolutionData,
  ConnectionDetails,
  SyncSettings,
  ConflictResolutionStrategy,
} from "../types/sync";
import { syncService } from "../services/sync";

/**
 * External Database Sync Store
 * Manages external database connections, sync operations, and conflict resolution
 */
export const useSyncStore = defineStore("sync", () => {
  // === State ===
  const databases = ref<ExternalDatabaseConfig[]>([]);
  const currentDatabaseId = ref<string | null>(null);
  const syncStatus = ref<SyncServiceStatus | null>(null);
  const syncLogs = ref<SyncLog[]>([]);
  const conflicts = ref<ConflictResolutionData[]>([]);
  const statistics = ref<SyncServiceStatistics | null>(null);
  const isLoading = ref(false);
  const isSyncing = ref(false);

  // === Computed ===

  /**
   * Get currently selected database
   */
  const currentDatabase = computed(() => {
    if (!currentDatabaseId.value) return null;
    return databases.value.find((db: ExternalDatabaseConfig) => db.id === currentDatabaseId.value);
  });

  /**
   * Check if any database is active
   */
  const hasActiveDatabases = computed(() => {
    return databases.value.some((db: ExternalDatabaseConfig) => db.isActive);
  });

  /**
   * Get unresolved conflicts
   */
  const pendingConflicts = computed(() => {
    return conflicts.value.filter((c: ConflictResolutionData) => !c.resolvedAt);
  });

  // === Database Actions ===

  /**
   * Load all external databases
   */
  async function loadDatabases(): Promise<void> {
    isLoading.value = true;
    try {
      databases.value = await syncService.getAllDatabases();
    } finally {
      isLoading.value = false;
    }
  }

  /**
   * Add new external database
   */
  async function addDatabase(
    config: Omit<ExternalDatabaseConfig, "id">,
    connectionDetails: ConnectionDetails,
    syncSettings: SyncSettings
  ): Promise<ExternalDatabaseConfig> {
    isLoading.value = true;
    try {
      const newDb = await syncService.addDatabase(
        config,
        connectionDetails,
        syncSettings
      );
      databases.value.push(newDb);
      return newDb;
    } finally {
      isLoading.value = false;
    }
  }

  /**
   * Update existing database configuration
   */
  async function updateDatabase(
    id: string,
    config: {
      name?: string;
      connectionDetails?: ConnectionDetails;
      autoSync?: boolean;
      syncIntervalMinutes?: number;
      conflictResolutionStrategy?: ConflictResolutionStrategy;
    }
  ): Promise<void> {
    isLoading.value = true;
    try {
      await syncService.updateDatabase(id, config);
      const index = databases.value.findIndex((db: ExternalDatabaseConfig) => db.id === id);
      if (index !== -1) {
        databases.value[index] = { ...databases.value[index], ...config };
      }
    } finally {
      isLoading.value = false;
    }
  }

  /**
   * Delete external database
   */
  async function deleteDatabase(id: string): Promise<void> {
    isLoading.value = true;
    try {
      await syncService.deleteDatabase(id);
      databases.value = databases.value.filter((db: ExternalDatabaseConfig) => db.id !== id);
      if (currentDatabaseId.value === id) {
        currentDatabaseId.value = null;
      }
    } finally {
      isLoading.value = false;
    }
  }

  /**
   * Test database connection
   */
  async function testConnection(
    dbType: string,
    connectionDetails: ConnectionDetails,
    databaseId?: string
  ): Promise<boolean> {
    isLoading.value = true;
    try {
      return await syncService.testConnection(dbType, connectionDetails, databaseId);
    } finally {
      isLoading.value = false;
    }
  }

  // === Connection Actions ===

  /**
   * Connect to external database
   */
  async function connect(id: string): Promise<void> {
    isLoading.value = true;
    try {
      await syncService.connect(id);
      await loadSyncStatus(id);
      await loadDatabases(); // Reload to update isActive status
      currentDatabaseId.value = id;
    } finally {
      isLoading.value = false;
    }
  }

  /**
   * Disconnect from external database
   */
  async function disconnect(id: string): Promise<void> {
    isLoading.value = true;
    try {
      await syncService.disconnect(id);
      if (currentDatabaseId.value === id) {
        currentDatabaseId.value = null;
        syncStatus.value = null;
      }
    } finally {
      isLoading.value = false;
    }
  }

  // === Sync Actions ===

  /**
   * Perform sync operation
   */
  async function sync(id: string, direction: "push" | "pull" | "bidirectional"): Promise<SyncLog> {
    isSyncing.value = true;
    try {
      const log = await syncService.sync(id, direction);
      syncLogs.value.unshift(log);
      await loadSyncStatus(id);
      return log;
    } finally {
      isSyncing.value = false;
    }
  }

  /**
   * Load sync status for database
   */
  async function loadSyncStatus(id: string): Promise<void> {
    syncStatus.value = await syncService.getSyncStatus(id);
  }

  /**
   * Load sync logs
   */
  async function loadSyncLogs(id: string, limit?: number): Promise<void> {
    isLoading.value = true;
    try {
      syncLogs.value = await syncService.getSyncLogs(id, limit);
    } finally {
      isLoading.value = false;
    }
  }

  // === Conflict Actions ===

  /**
   * Load all conflicts
   */
  async function loadConflicts(): Promise<void> {
    isLoading.value = true;
    try {
      conflicts.value = await syncService.getConflicts();
    } finally {
      isLoading.value = false;
    }
  }

  /**
   * Resolve conflict
   */
  async function resolveConflict(
    id: string,
    resolution: "local" | "remote"
  ): Promise<void> {
    isLoading.value = true;
    try {
      await syncService.resolveConflict(id, resolution);
      conflicts.value = conflicts.value.filter((c: ConflictResolutionData) => c.id !== id);
    } finally {
      isLoading.value = false;
    }
  }

  // === Auto-Sync Actions ===

  /**
   * Enable auto-sync for database
   */
  async function enableAutoSync(id: string): Promise<void> {
    await syncService.enableAutoSync(id);
    // Reload to get updated status from backend
    await loadDatabases();
  }

  /**
   * Disable auto-sync for database
   */
  async function disableAutoSync(id: string): Promise<void> {
    await syncService.disableAutoSync(id);
    // Reload to get updated status from backend
    await loadDatabases();
  }

  // === Statistics Actions ===

  /**
   * Load sync statistics
   */
  async function loadStatistics(): Promise<void> {
    statistics.value = await syncService.getStatistics();
  }

  // === Utility Actions ===

  /**
   * Set current database
   */
  function setCurrentDatabase(id: string | null): void {
    currentDatabaseId.value = id;
  }

  /**
   * Reset all state
   */
  function reset(): void {
    databases.value = [];
    currentDatabaseId.value = null;
    syncStatus.value = null;
    syncLogs.value = [];
    conflicts.value = [];
    statistics.value = null;
    isLoading.value = false;
    isSyncing.value = false;
  }

  return {
    // State
    databases,
    currentDatabaseId,
    syncStatus,
    syncLogs,
    conflicts,
    statistics,
    isLoading,
    isSyncing,
    // Computed
    currentDatabase,
    hasActiveDatabases,
    pendingConflicts,
    // Actions
    loadDatabases,
    addDatabase,
    updateDatabase,
    deleteDatabase,
    testConnection,
    connect,
    disconnect,
    sync,
    loadSyncStatus,
    loadSyncLogs,
    loadConflicts,
    resolveConflict,
    enableAutoSync,
    disableAutoSync,
    loadStatistics,
    setCurrentDatabase,
    reset,
  };
});
