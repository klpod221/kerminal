import { ElectronAPI } from '@electron-toolkit/preload'
import type { SystemInfo, NetworkStatus, NetworkInterface } from '../renderer/src/types/system'

declare global {
  interface Window {
    electron: ElectronAPI
    api: {
      send: (channel: string, data: unknown) => void
      on: (channel: string, func: (...args: unknown[]) => void) => () => void
      invoke: (channel: string, ...args: unknown[]) => Promise<unknown>
      getSystemInfo: () => Promise<SystemInfo | null>
      getNetworkInfo: () => Promise<NetworkInterface[]>
      getNetworkStatus: () => Promise<NetworkStatus>

      // SSH APIs
      sshGroups: {
        getAll: () => Promise<unknown[]>
        getById: (id: string) => Promise<unknown>
        create: (groupData: unknown) => Promise<unknown>
        update: (id: string, updates: unknown) => Promise<unknown>
        delete: (id: string) => Promise<boolean>
      }

      sshProfiles: {
        getAll: () => Promise<unknown[]>
        getById: (id: string) => Promise<unknown>
        getByGroupId: (groupId: string) => Promise<unknown[]>
        getFavorites: () => Promise<unknown[]>
        getRecent: (limit?: number) => Promise<unknown[]>
        create: (profileData: unknown) => Promise<unknown>
        update: (id: string, updates: unknown) => Promise<unknown>
        toggleFavorite: (id: string) => Promise<unknown>
        delete: (id: string) => Promise<boolean>
        search: (query: string) => Promise<unknown[]>
        getGroupsWithProfiles: () => Promise<unknown[]>
        getUngrouped: () => Promise<unknown[]>
      }

      sshConnections: {
        getRecent: (limit?: number) => Promise<unknown[]>
        getStats: () => Promise<unknown>
        cleanup: (daysOld?: number) => Promise<number>
      }

      ssh: {
        testConnection: (config: unknown) => Promise<{ success: boolean; error?: string }>
        getActiveConnections: () => Promise<number>
        // SSH Tunnel APIs
        listTunnels: () => Promise<unknown[]>
        getTunnel: (id: string) => Promise<unknown>
        createTunnel: (tunnelData: unknown) => Promise<unknown>
        updateTunnel: (id: string, updates: unknown) => Promise<unknown>
        deleteTunnel: (id: string) => Promise<boolean>
        startTunnel: (id: string) => Promise<boolean>
        stopTunnel: (id: string) => Promise<boolean>
        getTunnelStatus: (id: string) => Promise<string>
        getAutoStartTunnels: () => Promise<unknown[]>
        startAutoStartTunnels: () => Promise<boolean>
        stopAllTunnels: () => Promise<boolean>
        listProfiles: () => Promise<unknown[]>
      }

      sync: {
        testConnection: (mongoUri: string, databaseName: string) => Promise<boolean>
        setup: (config: unknown) => Promise<boolean>
        enable: (config: unknown) => Promise<boolean>
        disable: () => Promise<boolean>
        getStatus: () => Promise<{
          isConnected: boolean
          lastSync?: Date
          lastError?: string
          isLoading: boolean
        }>
        getConfig: () => Promise<unknown>
        updateConfig: (config: unknown) => Promise<boolean>
        isEnabled: () => Promise<boolean>
        performSync: () => Promise<boolean>
        forceSyncNow: () => Promise<boolean>
        migrateData: () => Promise<boolean>
        deleteConfig: () => Promise<boolean>
      }
    }
  }
}
