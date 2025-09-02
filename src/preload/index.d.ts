import { ElectronAPI } from '@electron-toolkit/preload'

interface SystemInfo {
  platform: string
  release: string
  arch: string
  hostname: string
  uptime: number
  totalMemory: number
  freeMemory: number
  loadAverage: number[]
  cpus: Array<{
    model: string
    speed: number
    times: {
      user: number
      nice: number
      sys: number
      idle: number
      irq: number
    }
  }>
  osRelease?: string
  cpuInfo?: string
  memInfo?: string
  gpuInfo?: string
  resolution?: string
}

interface NetworkInterface {
  name: string
  address: string
  netmask: string
  mac: string
  isConnected?: boolean
}

interface NetworkStatus {
  isConnected: boolean
  primaryInterface: NetworkInterface | null
  interfaces: NetworkInterface[]
}

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
