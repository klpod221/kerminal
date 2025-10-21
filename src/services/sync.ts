import { api } from "./api";
import type {
  ExternalDatabaseConfig,
  ExternalDatabaseWithDetails,
  SyncLog,
  SyncServiceStatus,
  SyncServiceStatistics,
  ConflictResolutionData,
  Device,
  ConnectionDetails,
  SyncSettings,
} from "../types/sync";

class SyncService {
  async getAllDatabases(): Promise<ExternalDatabaseConfig[]> {
    return api.callRaw("get_external_databases");
  }

  async getDatabaseWithDetails(
    id: string
  ): Promise<ExternalDatabaseWithDetails> {
    return api.callRaw("get_external_database_with_details", { id });
  }

  async addDatabase(
    config: Omit<ExternalDatabaseConfig, "id">,
    connectionDetails: ConnectionDetails,
    syncSettings: SyncSettings
  ): Promise<ExternalDatabaseConfig> {
    return api.call("add_external_database", {
      name: config.name,
      dbType: config.dbType,
      connectionDetails: connectionDetails,
      autoSync: syncSettings.autoSync,
      syncIntervalMinutes: syncSettings.syncIntervalMinutes,
      conflictResolutionStrategy: syncSettings.conflictResolutionStrategy,
    });
  }

  async updateDatabase(
    id: string,
    config: Partial<ExternalDatabaseConfig>
  ): Promise<void> {
    return api.call("update_external_database", { id, ...config });
  }

  async deleteDatabase(id: string): Promise<void> {
    return api.callRaw("delete_external_database", { id });
  }

  async testConnection(
    dbType: string,
    connectionDetails: ConnectionDetails,
    databaseId?: string
  ): Promise<boolean> {
    return api.call("test_external_database_connection", {
      dbType,
      connectionDetails,
      databaseId,
    });
  }

  async connect(id: string): Promise<void> {
    return api.callRaw("connect_to_database", { databaseId: id });
  }

  async disconnect(id: string): Promise<void> {
    return api.callRaw("disconnect_from_database", { databaseId: id });
  }

  async sync(
    id: string,
    direction: "push" | "pull" | "bidirectional"
  ): Promise<SyncLog> {
    // Backend expects lowercase direction values
    return api.callRaw("sync_now", {
      databaseId: id,
      direction: direction, // Already lowercase from parameter
    });
  }

  async getSyncStatus(id: string): Promise<SyncServiceStatus> {
    return api.callRaw("get_sync_status", { databaseId: id });
  }

  async getSyncLogs(id: string, limit?: number): Promise<SyncLog[]> {
    return api.callRaw("get_sync_logs", { databaseId: id, limit });
  }

  async getConflicts(): Promise<ConflictResolutionData[]> {
    return api.callRaw("get_unresolved_conflicts");
  }

  async resolveConflict(
    id: string,
    resolution: "local" | "remote"
  ): Promise<void> {
    return api.callRaw("resolve_conflict", {
      conflictId: id,
      resolution: resolution === "local" ? "UseLocal" : "UseRemote",
    });
  }

  async enableAutoSync(id: string): Promise<void> {
    return api.callRaw("enable_auto_sync", { databaseId: id });
  }

  async disableAutoSync(id: string): Promise<void> {
    return api.callRaw("disable_auto_sync", { databaseId: id });
  }

  async getStatistics(): Promise<SyncServiceStatistics> {
    return api.callRaw("get_sync_service_statistics");
  }

  async getCurrentDevice(): Promise<Device | null> {
    return api.callRaw("get_current_device");
  }

  async getAllDevices(): Promise<Device[]> {
    return api.callRaw("get_all_devices");
  }

  async registerDevice(deviceName: string, deviceType: string): Promise<Device> {
    return api.callRaw("register_device", { deviceName, deviceType });
  }
}

export const syncService = new SyncService();
