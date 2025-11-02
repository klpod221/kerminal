import { defineStore } from "pinia";
import { ref, computed } from "vue";
import type {
  ExternalDatabaseConfig,
  SyncLog,
  SyncServiceStatus,
  SyncServiceStatistics,
  ConflictResolutionData,
  ConnectionDetails,
  DatabaseSyncSettings,
  ConflictResolutionStrategy,
  SyncDirection,
  SyncSettings,
  Device,
} from "../types/sync";
import { syncService } from "../services/sync";
import { api } from "../services/api";
import {
  withRetry,
  handleError,
  type ErrorContext,
} from "../utils/errorHandler";
import { message } from "../utils/message";

/**
 * External Database Sync Store
 * Manages external database connections, sync operations, and conflict resolution
 */
export const useSyncStore = defineStore("sync", () => {
  const databases = ref<ExternalDatabaseConfig[]>([]);
  const currentDatabaseId = ref<string | null>(null);
  const syncStatus = ref<SyncServiceStatus | null>(null);
  const syncLogs = ref<SyncLog[]>([]);
  const conflicts = ref<ConflictResolutionData[]>([]);
  const statistics = ref<SyncServiceStatistics | null>(null);
  const isLoading = ref(false);
  const isSyncing = ref(false);

  /**
   * Get currently selected database
   */
  const currentDatabase = computed(() => {
    if (!currentDatabaseId.value) return null;
    return databases.value.find(
      (db: ExternalDatabaseConfig) => db.id === currentDatabaseId.value,
    );
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

  /**
   * Load all external databases with error handling
   */
  async function loadDatabases(): Promise<void> {
    isLoading.value = true;
    const context: ErrorContext = {
      operation: "Load Databases",
    };

    try {
      databases.value = await withRetry(
        () => syncService.getAllDatabases(),
        { maxRetries: 2 },
        context,
      );
    } catch (error) {
      const errorMessage = handleError(error, context);
      message.error(errorMessage);
      databases.value = [];
    } finally {
      isLoading.value = false;
    }
  }

  /**
   * Get database with details including connection info and sync settings
   * @param id - Database ID
   * @returns Database with details
   */
  async function getDatabaseWithDetails(id: string): Promise<{
    config: ExternalDatabaseConfig;
    connectionDetails: ConnectionDetails;
    syncSettings: DatabaseSyncSettings;
  }> {
    const context: ErrorContext = {
      operation: "Load Database Details",
      context: { databaseId: id },
    };

    try {
      const result = await withRetry(
        () => syncService.getDatabaseWithDetails(id),
        { maxRetries: 1 },
        context,
      );
      const syncSettingsStr = result.config?.syncSettings;
      let syncSettings: DatabaseSyncSettings = {
        autoSync: result.config?.autoSyncEnabled || false,
        syncIntervalMinutes: 60,
        conflictResolutionStrategy: "Manual",
        syncDirection: "Bidirectional",
      };
      if (syncSettingsStr) {
        try {
          const parsed = JSON.parse(syncSettingsStr);
          syncSettings = { ...syncSettings, ...parsed };
        } catch (e) {
          console.warn("Failed to parse syncSettings:", e);
        }
      }
      return {
        config: result.config,
        connectionDetails: result.connectionDetails,
        syncSettings,
      };
    } catch (error) {
      const errorMessage = handleError(error, context);
      message.error(errorMessage);
      throw new Error(errorMessage);
    }
  }

  /**
   * Add new external database with error handling
   * @param config - Database configuration
   * @param connectionDetails - Connection details
   * @param syncSettings - Sync settings
   * @returns Created database
   */
  async function addDatabase(
    config: Omit<ExternalDatabaseConfig, "id">,
    connectionDetails: ConnectionDetails,
    syncSettings: DatabaseSyncSettings,
  ): Promise<ExternalDatabaseConfig> {
    isLoading.value = true;
    const context: ErrorContext = {
      operation: "Add Database",
      context: { name: config.name, dbType: config.dbType },
    };

    try {
      const newDb = await withRetry(
        () => syncService.addDatabase(config, connectionDetails, syncSettings),
        { maxRetries: 1 },
        context,
      );
      databases.value.push(newDb);
      return newDb;
    } catch (error) {
      const errorMessage = handleError(error, context);
      message.error(errorMessage);
      throw new Error(errorMessage);
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
    },
  ): Promise<void> {
    isLoading.value = true;
    try {
      await syncService.updateDatabase(id, config);
      const index = databases.value.findIndex(
        (db: ExternalDatabaseConfig) => db.id === id,
      );
      if (index !== -1) {
        databases.value[index] = { ...databases.value[index], ...config };
      }
    } finally {
      isLoading.value = false;
    }
  }

  /**
   * Delete external database with error handling
   * @param id - Database ID to delete
   */
  async function deleteDatabase(id: string): Promise<void> {
    isLoading.value = true;
    const context: ErrorContext = {
      operation: "Delete Database",
      context: { databaseId: id },
    };

    try {
      await syncService.deleteDatabase(id);
      databases.value = databases.value.filter(
        (db: ExternalDatabaseConfig) => db.id !== id,
      );
      if (currentDatabaseId.value === id) {
        currentDatabaseId.value = null;
      }
    } catch (error) {
      const errorMessage = handleError(error, context);
      message.error(errorMessage);
      throw new Error(errorMessage);
    } finally {
      isLoading.value = false;
    }
  }

  /**
   * Test database connection with retry logic
   * @param dbType - Database type
   * @param connectionDetails - Connection details
   * @param databaseId - Optional database ID
   * @returns True if connection successful
   */
  async function testConnection(
    dbType: string,
    connectionDetails: ConnectionDetails,
    databaseId?: string,
  ): Promise<boolean> {
    isLoading.value = true;
    const context: ErrorContext = {
      operation: "Test Database Connection",
      context: { dbType, host: connectionDetails.host },
    };

    try {
      return await withRetry(
        () => syncService.testConnection(dbType, connectionDetails, databaseId),
        { maxRetries: 1 },
        context,
      );
    } catch (error) {
      const errorMessage = handleError(error, context);
      message.error(errorMessage);
      throw new Error(errorMessage);
    } finally {
      isLoading.value = false;
    }
  }

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

  /**
   * Perform sync operation with retry logic
   * @param id - Database ID
   * @param direction - Sync direction
   * @returns Sync log
   */
  async function sync(
    id: string,
    direction: "push" | "pull" | "bidirectional",
  ): Promise<SyncLog> {
    isSyncing.value = true;
    const context: ErrorContext = {
      operation: "Sync Database",
      context: { databaseId: id, direction },
    };

    try {
      const log = await withRetry(
        () => syncService.sync(id, direction),
        { maxRetries: 1 },
        context,
      );
      syncLogs.value.unshift(log);
      await loadSyncStatus(id);
      return log;
    } catch (error) {
      const errorMessage = handleError(error, context);
      message.error(errorMessage);
      throw new Error(errorMessage);
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
      syncLogs.value = await syncService.loadSyncLogs(id, limit);
    } finally {
      isLoading.value = false;
    }
  }

  /**
   * Load all conflicts
   */
  async function loadConflicts(): Promise<void> {
    isLoading.value = true;
    try {
      conflicts.value = await syncService.getUnresolvedConflictResolutions();
    } finally {
      isLoading.value = false;
    }
  }

  /**
   * Resolve conflict
   */
  async function resolveConflict(
    id: string,
    resolution: "local" | "remote",
  ): Promise<void> {
    isLoading.value = true;
    try {
      const strategy: ConflictResolutionStrategy =
        resolution === "local" ? "LocalWins" : "RemoteWins";
      await syncService.resolveConflictResolution(id, strategy);
      conflicts.value = conflicts.value.filter(
        (c: ConflictResolutionData) => c.id !== id,
      );
    } finally {
      isLoading.value = false;
    }
  }

  /**
   * Enable auto-sync for database
   */
  async function enableAutoSync(id: string): Promise<void> {
    await syncService.enableAutoSync(id);
    await loadDatabases();
  }

  /**
   * Disable auto-sync for database
   */
  async function disableAutoSync(id: string): Promise<void> {
    await syncService.disableAutoSync(id);
    await loadDatabases();
  }

  /**
   * Load sync statistics
   */
  async function loadStatistics(): Promise<void> {
    statistics.value = await syncService.getStatistics();
  }

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

  const upsertDatabase = (db: ExternalDatabaseConfig) => {
    if (!db?.id) return;
    const i = databases.value.findIndex((x) => x?.id === db.id);
    if (i === -1) {
      databases.value = [...databases.value, db];
    } else {
      databases.value[i] = { ...databases.value[i]!, ...db };
    }
  };

  const removeDatabase = (id: string) => {
    databases.value = databases.value.filter((db) => db?.id !== id);
    if (currentDatabaseId.value === id) {
      currentDatabaseId.value = null;
      syncStatus.value = null;
    }
  };

  let unsubscribeDatabaseRealtime: (() => void) | null = null;
  let unsubscribeSyncRealtime: (() => void) | null = null;
  let unsubscribeConflictRealtime: (() => void) | null = null;

  const startRealtime = async (): Promise<void> => {
    if (
      unsubscribeDatabaseRealtime &&
      unsubscribeSyncRealtime &&
      unsubscribeConflictRealtime
    )
      return;

    try {
      if (!unsubscribeDatabaseRealtime) {
        const u1 = await api.listen<ExternalDatabaseConfig>(
          "sync_database_created",
          upsertDatabase,
        );
        const u2 = await api.listen<ExternalDatabaseConfig>(
          "sync_database_updated",
          upsertDatabase,
        );
        const u3 = await api.listen<{ id: string }>(
          "sync_database_deleted",
          ({ id }) => removeDatabase(id),
        );
        unsubscribeDatabaseRealtime = () => {
          u1();
          u2();
          u3();
        };
      }

      if (!unsubscribeSyncRealtime) {
        const s1 = await api.listen<SyncServiceStatus>(
          "sync_status_updated",
          (status) => {
            if (
              currentDatabaseId.value &&
              status.lastSync?.databaseId === currentDatabaseId.value
            ) {
              syncStatus.value = status;
            }
          },
        );
        const s2 = await api.listen<SyncLog>("sync_log_added", (log) => {
          syncLogs.value.unshift(log);
        });
        unsubscribeSyncRealtime = () => {
          s1();
          s2();
        };
      }

      if (!unsubscribeConflictRealtime) {
        const c1 = await api.listen<ConflictResolutionData>(
          "sync_conflict_created",
          (conflict) => {
            conflicts.value.push(conflict);
          },
        );
        const c2 = await api.listen<{ id: string }>(
          "sync_conflict_resolved",
          ({ id }) => {
            conflicts.value = conflicts.value.filter((c) => c.id !== id);
          },
        );
        unsubscribeConflictRealtime = () => {
          c1();
          c2();
        };
      }
    } catch (e) {
      console.error("Failed to subscribe sync realtime events:", e);
    }
  };

  const stopRealtime = (): void => {
    if (unsubscribeDatabaseRealtime) {
      unsubscribeDatabaseRealtime();
      unsubscribeDatabaseRealtime = null;
    }
    if (unsubscribeSyncRealtime) {
      unsubscribeSyncRealtime();
      unsubscribeSyncRealtime = null;
    }
    if (unsubscribeConflictRealtime) {
      unsubscribeConflictRealtime();
      unsubscribeConflictRealtime = null;
    }
  };

  /**
   * Get global sync settings
   * @returns Global sync settings
   */
  async function getGlobalSyncSettings(): Promise<{
    selectedDatabaseId?: string;
    autoSyncEnabled?: boolean;
    conflictStrategy?: ConflictResolutionStrategy;
    syncDirection?: "push" | "pull" | "bidirectional";
    isActive?: boolean;
    syncIntervalMinutes?: number;
  } | null> {
    const context: ErrorContext = {
      operation: "Load Global Sync Settings",
    };

    try {
      const result = await withRetry(
        () => syncService.getGlobalSyncSettings(),
        { maxRetries: 1 },
        context,
      );
      if (!result) return null;
      const syncDirectionMap: Record<
        SyncDirection,
        "push" | "pull" | "bidirectional"
      > = {
        Push: "push",
        Pull: "pull",
        Both: "bidirectional",
        Bidirectional: "bidirectional",
      };
      return {
        selectedDatabaseId: result.selectedDatabaseId,
        autoSyncEnabled: result.autoSyncEnabled,
        conflictStrategy: result.conflictStrategy,
        syncDirection: result.syncDirection
          ? syncDirectionMap[result.syncDirection] || "bidirectional"
          : undefined,
        isActive: result.isActive,
        syncIntervalMinutes: result.syncIntervalMinutes,
      };
    } catch (error) {
      const errorMessage = handleError(error, context);
      message.error(errorMessage);
      throw new Error(errorMessage);
    }
  }

  /**
   * Update global sync settings
   * @param settings - Settings to update
   */
  async function updateGlobalSyncSettings(settings: {
    selectedDatabaseId?: string;
    autoSyncEnabled?: boolean;
    conflictStrategy?: ConflictResolutionStrategy;
    syncDirection?: "push" | "pull" | "bidirectional";
  }): Promise<void> {
    const context: ErrorContext = {
      operation: "Update Global Sync Settings",
    };

    try {
      const syncDirectionMap: Record<
        "push" | "pull" | "bidirectional",
        SyncDirection
      > = {
        push: "Push",
        pull: "Pull",
        bidirectional: "Bidirectional",
      };
      const convertedSettings: Partial<SyncSettings> = {
        ...settings,
        syncDirection: settings.syncDirection
          ? syncDirectionMap[settings.syncDirection]
          : undefined,
      };
      await withRetry(
        () => syncService.updateGlobalSyncSettings(convertedSettings),
        { maxRetries: 1 },
        context,
      );
    } catch (error) {
      const errorMessage = handleError(error, context);
      message.error(errorMessage);
      throw new Error(errorMessage);
    }
  }

  /**
   * Get all devices
   * @returns Array of devices
   */
  async function getAllDevices(): Promise<
    Array<{
      id: string;
      name: string;
      deviceType: string;
      registeredAt: string;
      lastSeenAt: string;
    }>
  > {
    const context: ErrorContext = {
      operation: "Load All Devices",
    };

    try {
      const devices = await withRetry(
        () => syncService.getAllDevices(),
        { maxRetries: 1 },
        context,
      );
      return devices.map((device: Device) => ({
        id: device.deviceId,
        name: device.deviceName,
        deviceType: device.deviceType,
        registeredAt: device.createdAt,
        lastSeenAt: device.lastSeen,
      }));
    } catch (error) {
      const errorMessage = handleError(error, context);
      message.error(errorMessage);
      throw new Error(errorMessage);
    }
  }

  /**
   * Get current device
   * @returns Current device or null
   */
  async function getCurrentDevice(): Promise<{
    id: string;
    name: string;
    deviceType: string;
    registeredAt: string;
    lastSeenAt: string;
  } | null> {
    const context: ErrorContext = {
      operation: "Load Current Device",
    };

    try {
      const device = await withRetry(
        () => syncService.getCurrentDevice(),
        { maxRetries: 1 },
        context,
      );
      if (!device) return null;
      return {
        id: device.deviceId,
        name: device.deviceName,
        deviceType: device.deviceType,
        registeredAt: device.createdAt,
        lastSeenAt: device.lastSeen,
      };
    } catch (error) {
      const errorMessage = handleError(error, context);
      message.error(errorMessage);
      throw new Error(errorMessage);
    }
  }

  /**
   * Register current device
   * @param deviceName - Device name
   * @param deviceType - Device type
   * @returns Registered device
   */
  async function registerDevice(
    deviceName: string,
    deviceType: string,
  ): Promise<{
    id: string;
    name: string;
    deviceType: string;
    registeredAt: string;
    lastSeenAt: string;
  }> {
    const context: ErrorContext = {
      operation: "Register Device",
      context: { deviceName, deviceType },
    };

    try {
      const device = await withRetry(
        () => syncService.registerDevice(deviceName, deviceType),
        { maxRetries: 1 },
        context,
      );
      return {
        id: device.deviceId,
        name: device.deviceName,
        deviceType: device.deviceType,
        registeredAt: device.createdAt,
        lastSeenAt: device.lastSeen,
      };
    } catch (error) {
      const errorMessage = handleError(error, context);
      message.error(errorMessage);
      throw new Error(errorMessage);
    }
  }

  /**
   * Resolve conflict resolution (direct service call)
   * @param id - Conflict ID
   * @param strategy - Resolution strategy
   */
  async function resolveConflictResolution(
    id: string,
    strategy: ConflictResolutionStrategy,
  ): Promise<void> {
    const context: ErrorContext = {
      operation: "Resolve Conflict Resolution",
      context: { conflictId: id, strategy },
    };

    try {
      await withRetry(
        () => syncService.resolveConflictResolution(id, strategy),
        { maxRetries: 1 },
        context,
      );
    } catch (error) {
      const errorMessage = handleError(error, context);
      message.error(errorMessage);
      throw new Error(errorMessage);
    }
  }

  return {
    databases,
    currentDatabaseId,
    syncStatus,
    syncLogs,
    conflicts,
    statistics,
    isLoading,
    isSyncing,
    currentDatabase,
    hasActiveDatabases,
    pendingConflicts,
    loadDatabases,
    getDatabaseWithDetails,
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
    resolveConflictResolution,
    enableAutoSync,
    disableAutoSync,
    loadStatistics,
    setCurrentDatabase,
    getGlobalSyncSettings,
    updateGlobalSyncSettings,
    getAllDevices,
    getCurrentDevice,
    registerDevice,
    reset,

    startRealtime,
    stopRealtime,
  };
});
