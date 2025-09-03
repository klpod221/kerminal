import { contextBridge, ipcRenderer } from 'electron'
import { electronAPI } from '@electron-toolkit/preload'

// Custom APIs for renderer
const api = {
  send: (channel: string, data: unknown) => {
    ipcRenderer.send(channel, data)
  },
  on: (channel: string, func: (...args: unknown[]) => void) => {
    const subscription = (_event: unknown, ...args: unknown[]): void => func(...args)
    ipcRenderer.on(channel, subscription)
    return () => {
      ipcRenderer.removeListener(channel, subscription)
    }
  },
  invoke: (channel: string, ...args: unknown[]) => {
    return ipcRenderer.invoke(channel, ...args)
  },
  // System information APIs
  getSystemInfo: () => ipcRenderer.invoke('get-system-info'),
  getNetworkInfo: () => ipcRenderer.invoke('get-network-info'),
  getNetworkStatus: () => ipcRenderer.invoke('get-network-status'),

  // SSH APIs - Groups
  sshGroups: {
    getAll: () => ipcRenderer.invoke('ssh-groups.getAll'),
    getById: (id: string) => ipcRenderer.invoke('ssh-groups.getById', id),
    create: (groupData: unknown) => ipcRenderer.invoke('ssh-groups.create', groupData),
    update: (id: string, updates: unknown) => ipcRenderer.invoke('ssh-groups.update', id, updates),
    delete: (id: string) => ipcRenderer.invoke('ssh-groups.delete', id)
  },

  // SSH APIs - Profiles
  sshProfiles: {
    getAll: () => ipcRenderer.invoke('ssh-profiles.getAll'),
    getById: (id: string) => ipcRenderer.invoke('ssh-profiles.getById', id),
    getByGroupId: (groupId: string) => ipcRenderer.invoke('ssh-profiles.getByGroupId', groupId),
    getFavorites: () => ipcRenderer.invoke('ssh-profiles.getFavorites'),
    getRecent: (limit?: number) => ipcRenderer.invoke('ssh-profiles.getRecent', limit),
    create: (profileData: unknown) => ipcRenderer.invoke('ssh-profiles.create', profileData),
    update: (id: string, updates: unknown) =>
      ipcRenderer.invoke('ssh-profiles.update', id, updates),
    toggleFavorite: (id: string) => ipcRenderer.invoke('ssh-profiles.toggleFavorite', id),
    delete: (id: string) => ipcRenderer.invoke('ssh-profiles.delete', id),
    search: (query: string) => ipcRenderer.invoke('ssh-profiles.search', query),
    getGroupsWithProfiles: () => ipcRenderer.invoke('ssh-profiles.getGroupsWithProfiles'),
    getUngrouped: () => ipcRenderer.invoke('ssh-profiles.getUngrouped')
  },

  // SSH APIs - Connections
  sshConnections: {
    getRecent: (limit?: number) => ipcRenderer.invoke('ssh-connections.getRecent', limit),
    getStats: () => ipcRenderer.invoke('ssh-connections.getStats'),
    cleanup: (daysOld?: number) => ipcRenderer.invoke('ssh-connections.cleanup', daysOld)
  },

  // SSH Connection Testing and Management
  ssh: {
    testConnection: (config: unknown) => ipcRenderer.invoke('ssh.testConnection', config),
    getActiveConnections: () => ipcRenderer.invoke('ssh.getActiveConnections'),
    // SSH Tunnel APIs
    listTunnels: () => ipcRenderer.invoke('ssh-tunnels.getAll'),
    getTunnel: (id: string) => ipcRenderer.invoke('ssh-tunnels.getById', id),
    createTunnel: (tunnelData: unknown) => ipcRenderer.invoke('ssh-tunnels.create', tunnelData),
    updateTunnel: (id: string, updates: unknown) =>
      ipcRenderer.invoke('ssh-tunnels.update', id, updates),
    deleteTunnel: (id: string) => ipcRenderer.invoke('ssh-tunnels.delete', id),
    startTunnel: (id: string) => ipcRenderer.invoke('ssh-tunnels.start', id),
    stopTunnel: (id: string) => ipcRenderer.invoke('ssh-tunnels.stop', id),
    getTunnelStatus: (id: string) => ipcRenderer.invoke('ssh-tunnels.getStatus', id),
    getAutoStartTunnels: () => ipcRenderer.invoke('ssh-tunnels.getAutoStart'),
    startAutoStartTunnels: () => ipcRenderer.invoke('ssh-tunnels.startAutoStart'),
    stopAllTunnels: () => ipcRenderer.invoke('ssh-tunnels.stopAll'),
    // For compatibility with existing profile APIs
    listProfiles: () => ipcRenderer.invoke('ssh-profiles.getAll')
  }
}

// Use `contextBridge` APIs to expose Electron APIs to
// renderer only if context isolation is enabled, otherwise
// just add to the DOM global.
if (process.contextIsolated) {
  try {
    contextBridge.exposeInMainWorld('electron', electronAPI)
    contextBridge.exposeInMainWorld('api', api)
  } catch (error) {
    console.error(error)
  }
} else {
  // @ts-ignore (define in dts)
  window.electron = electronAPI
  // @ts-ignore (define in dts)
  window.api = api
}
