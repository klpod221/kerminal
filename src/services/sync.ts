import { api } from "./api";
import type {
  ExternalDatabaseConfig,
  SyncLog,
  SyncServiceStatus,
  SyncServiceStatistics,
  ConflictResolutionData,
  Device,
} from "../types/sync";

class SyncService {
  async getAllDatabases(): Promise<ExternalDatabaseConfig[]> {
    return api.callRaw("get_external_databases");
  }

  async addDatabase(
    config: Omit<ExternalDatabaseConfig, "id">
  ): Promise<ExternalDatabaseConfig> {
    return api.call("add_external_database", {
      name: config.name,
      dbType: config.dbType,
      connectionDetails: config.connectionDetailsEncrypted,
      syncSettings: config.syncSettings,
    });
  }

  async updateDatabase(
    id: string,
    config: Partial<ExternalDatabaseConfig>
  ): Promise<void> {
    return api.call("update_external_database", { id, config });
  }

  async deleteDatabase(id: string): Promise<void> {
    return api.call("delete_external_database", { id });
  }

  async testConnection(id: string): Promise<boolean> {
    return api.call("test_external_database_connection", { id });
  }

  async connect(id: string): Promise<void> {
    return api.call("connect_to_database", { databaseId: id });
  }

  async disconnect(id: string): Promise<void> {
    return api.call("disconnect_from_database", { databaseId: id });
  }

  async sync(
    id: string,
    direction: "push" | "pull" | "bidirectional"
  ): Promise<SyncLog> {
    const directionMap = {
      push: "Push",
      pull: "Pull",
      bidirectional: "Bidirectional",
    };
    return api.call("sync_now", {
      databaseId: id,
      direction: directionMap[direction],
    });
  }

  async getSyncStatus(id: string): Promise<SyncServiceStatus> {
    return api.call("get_sync_status", { databaseId: id });
  }

  async getSyncLogs(id: string, limit?: number): Promise<SyncLog[]> {
    return api.call("get_sync_logs", { databaseId: id, limit });
  }

  async getConflicts(): Promise<ConflictResolutionData[]> {
    return api.callRaw("get_unresolved_conflicts");
  }

  async resolveConflict(
    id: string,
    resolution: "local" | "remote"
  ): Promise<void> {
    return api.call("resolve_conflict", {
      conflictId: id,
      resolution: resolution === "local" ? "UseLocal" : "UseRemote",
    });
  }

  async enableAutoSync(id: string): Promise<void> {
    return api.call("enable_auto_sync", { databaseId: id });
  }

  async disableAutoSync(id: string): Promise<void> {
    return api.call("disable_auto_sync", { databaseId: id });
  }

  async getStatistics(): Promise<SyncServiceStatistics> {
    return api.callRaw("get_sync_statistics");
  }

  async getCurrentDevice(): Promise<Device | null> {
    return api.callRaw("get_current_device");
  }

  async getAllDevices(): Promise<Device[]> {
    return api.callRaw("get_all_devices");
  }

  async registerDevice(device: Partial<Device>): Promise<Device> {
    return api.call("register_device", { device });
  }
}

export const syncService = new SyncService();
