import { ipcMain, shell, clipboard, dialog } from 'electron'
import { TerminalManager } from './services/terminal-manager'
import { WindowManager } from './services/window-manager'
import { SystemInfoService } from './services/system-info'
import { SSHProfileService } from './services/ssh-profile-service'
import { SSHConnectionService } from './services/ssh-connection-service'
import { SavedCommandService } from './services/saved-command-service'
import { SyncManager } from './services/sync-manager'
import { SSHTunnelService } from './services/ssh-tunnel-service'

/**
 * Sets up all IPC (Inter-Process Communication) handlers for the application.
 * @param windowManager - The window manager instance.
 * @param terminalManager - The terminal manager instance.
 */
export function setupIpcHandlers(
  windowManager: WindowManager,
  terminalManager: TerminalManager
): void {
  // Create service instances
  const sshProfileService = new SSHProfileService()
  const sshConnectionService = new SSHConnectionService()
  const savedCommandService = new SavedCommandService()
  const syncManager = new SyncManager()
  const sshTunnelService = new SSHTunnelService()

  // Initialize sync manager
  syncManager.initialize().catch(console.error)

  // Terminal-related IPC handlers
  setupTerminalHandlers(terminalManager, sshConnectionService, sshProfileService)

  // Window control IPC handlers
  setupWindowHandlers(windowManager)

  // System information IPC handlers
  setupSystemHandlers()

  // SSH-related IPC handlers
  setupSSHHandlers(sshProfileService, sshConnectionService)

  // SSH Tunnel IPC handlers
  setupSSHTunnelHandlers(sshTunnelService)

  // Saved Commands IPC handlers
  setupSavedCommandHandlers(savedCommandService, terminalManager)

  // Sync IPC handlers
  setupSyncHandlers(syncManager)

  // Utility IPC handlers
  setupUtilityHandlers()

  // Test IPC handler
  ipcMain.on('ping', () => console.log('pong'))
}

/**
 * Sets up terminal-related IPC handlers.
 * @param terminalManager - The terminal manager instance.
 * @param sshConnectionService - The SSH connection service instance.
 */
function setupTerminalHandlers(
  terminalManager: TerminalManager,
  sshConnectionService: SSHConnectionService,
  sshProfileService: SSHProfileService
): void {
  // Handle data from the renderer
  ipcMain.on('terminal.keystroke', (_event, { terminalId, data }) => {
    // Check if this is an SSH connection
    const sshConnection = sshConnectionService.getConnection(terminalId)
    if (sshConnection?.stdin) {
      sshConnection.stdin.write(data)
    } else {
      terminalManager.writeToTerminal(terminalId, data)
    }
  })

  // Handle terminal resize events
  ipcMain.on('terminal.resize', (_event, { terminalId, cols, rows }) => {
    terminalManager.resizeTerminal(terminalId, cols, rows)
  })

  // Handle terminal ready event
  ipcMain.on('terminal.ready', (_event, { terminalId }) => {
    terminalManager.handleTerminalReady(terminalId)
  })

  // Handle new terminal creation
  ipcMain.on('terminal.create', (_event, { terminalId }) => {
    terminalManager.createTerminal(terminalId)
  })

  // Handle SSH terminal creation
  ipcMain.on('terminal.createSSH', async (_event, { terminalId, profileId }) => {
    try {
      // Get profile with resolved configuration
      const profile = await sshProfileService.getProfileById(profileId)
      if (!profile) {
        throw new Error(`SSH Profile with ID ${profileId} not found`)
      }

      // Validate configuration
      const validation = sshProfileService.validateConnectionConfig(profile.resolvedConfig)
      if (!validation.valid) {
        throw new Error(`Invalid SSH configuration: ${validation.errors.join(', ')}`)
      }

      // Create SSH terminal with TerminalManager
      await terminalManager.createSSHTerminal(
        terminalId,
        profile.resolvedConfig,
        profileId,
        profile.name
      )

      // Record connection attempt
      await sshProfileService.recordConnection(profileId, 'connected')

      console.log(`SSH terminal created for profile ${profileId} with terminal ID ${terminalId}`)
    } catch (error) {
      console.error(`Failed to create SSH terminal:`, error)
      _event.reply('terminal.sshError', {
        terminalId,
        error: error instanceof Error ? error.message : 'Unknown error'
      })

      // Record failed connection
      try {
        await sshProfileService.recordConnection(profileId, 'failed')
      } catch (recordError) {
        console.error('Failed to record connection failure:', recordError)
      }
    }
  })

  // Handle terminal destruction
  ipcMain.on('terminal.destroy', (_event, { terminalId }) => {
    // Close SSH connection if exists
    sshConnectionService.closeConnection(terminalId)
    // Destroy regular terminal
    terminalManager.destroyTerminal(terminalId)
  })
}

/**
 * Sets up window control IPC handlers.
 * @param windowManager - The window manager instance.
 */
function setupWindowHandlers(windowManager: WindowManager): void {
  ipcMain.on('window-minimize', () => {
    windowManager.minimize()
  })

  ipcMain.on('window-maximize', () => {
    windowManager.toggleMaximize()
  })

  ipcMain.on('window-close', () => {
    windowManager.close()
  })
}

/**
 * Sets up system information IPC handlers.
 */
function setupSystemHandlers(): void {
  // Handle system info requests
  ipcMain.handle('get-system-info', async () => {
    return await SystemInfoService.getSystemInfo()
  })

  // Handle network info requests
  ipcMain.handle('get-network-info', async () => {
    return await SystemInfoService.getNetworkInfo()
  })

  // Handle network status requests (includes connectivity check)
  ipcMain.handle('get-network-status', async () => {
    return await SystemInfoService.getNetworkStatus()
  })
}

/**
 * Sets up utility IPC handlers.
 */
function setupUtilityHandlers(): void {
  // Handle external links
  ipcMain.on('open-external-link', (_event, uri) => {
    shell.openExternal(uri)
  })

  // Handle copy to clipboard
  ipcMain.on('copy-to-clipboard', (_event, text) => {
    clipboard.writeText(text)
  })

  // Handle file dialog for SSH key selection
  ipcMain.handle('dialog.selectFile', async (_event, options = {}) => {
    const result = await dialog.showOpenDialog({
      title: 'Select SSH Private Key',
      defaultPath: '~/.ssh/',
      filters: [
        { name: 'SSH Key Files', extensions: ['', 'pem', 'key', 'rsa', 'dsa', 'ecdsa', 'ed25519'] },
        { name: 'All Files', extensions: ['*'] }
      ],
      properties: ['openFile'],
      ...options
    })

    if (result.canceled) {
      return null
    }

    return result.filePaths[0] || null
  })
}

/**
 * Sets up SSH-related IPC handlers.
 * @param sshProfileService - The SSH profile service instance.
 * @param sshConnectionService - The SSH connection service instance.
 */
function setupSSHHandlers(
  sshProfileService: SSHProfileService,
  sshConnectionService: SSHConnectionService
): void {
  // SSH Groups
  ipcMain.handle('ssh-groups.getAll', async () => {
    return sshProfileService.getAllGroups()
  })

  ipcMain.handle('ssh-groups.getById', async (_event, id: string) => {
    return sshProfileService.getGroupById(id)
  })

  ipcMain.handle('ssh-groups.create', async (_event, groupData) => {
    return sshProfileService.createGroup(groupData)
  })

  ipcMain.handle('ssh-groups.update', async (_event, id: string, updates) => {
    return sshProfileService.updateGroup(id, updates)
  })

  ipcMain.handle('ssh-groups.delete', async (_event, id: string) => {
    return sshProfileService.deleteGroup(id)
  })

  // SSH Profiles
  ipcMain.handle('ssh-profiles.getAll', async () => {
    return sshProfileService.getAllProfiles()
  })

  ipcMain.handle('ssh-profiles.getById', async (_event, id: string) => {
    return sshProfileService.getProfileById(id)
  })

  ipcMain.handle('ssh-profiles.getByGroupId', async (_event, groupId: string) => {
    return sshProfileService.getProfilesByGroupId(groupId)
  })

  ipcMain.handle('ssh-profiles.getFavorites', async () => {
    return sshProfileService.getFavoriteProfiles()
  })

  ipcMain.handle('ssh-profiles.getRecent', async (_event, limit?: number) => {
    return sshProfileService.getRecentlyConnectedProfiles(limit)
  })

  ipcMain.handle('ssh-profiles.create', async (_event, profileData) => {
    return sshProfileService.createProfile(profileData)
  })

  ipcMain.handle('ssh-profiles.update', async (_event, id: string, updates) => {
    return sshProfileService.updateProfile(id, updates)
  })

  ipcMain.handle('ssh-profiles.toggleFavorite', async (_event, id: string) => {
    return sshProfileService.toggleProfileFavorite(id)
  })

  ipcMain.handle('ssh-profiles.delete', async (_event, id: string) => {
    return sshProfileService.deleteProfile(id)
  })

  ipcMain.handle('ssh-profiles.search', async (_event, query: string) => {
    return sshProfileService.searchProfiles(query)
  })

  ipcMain.handle('ssh-profiles.getGroupsWithProfiles', async () => {
    return sshProfileService.getGroupsWithProfiles()
  })

  ipcMain.handle('ssh-profiles.getUngrouped', async () => {
    return sshProfileService.getUngroupedProfiles()
  })

  // SSH Connections
  ipcMain.handle('ssh-connections.getRecent', async (_event, limit?: number) => {
    return sshProfileService.getRecentConnections(limit)
  })

  ipcMain.handle('ssh-connections.getStats', async () => {
    return sshProfileService.getConnectionStats()
  })

  ipcMain.handle('ssh-connections.cleanup', async (_event, daysOld?: number) => {
    return sshProfileService.cleanupOldConnections(daysOld)
  })

  // SSH Connection Management
  ipcMain.handle('ssh.getActiveConnections', async () => {
    return sshConnectionService.getActiveConnectionCount()
  })
}

/**
 * Sets up SSH Tunnel IPC handlers.
 * @param sshTunnelService - The SSH tunnel service instance.
 */
function setupSSHTunnelHandlers(sshTunnelService: SSHTunnelService): void {
  // List all tunnels
  ipcMain.handle('ssh-tunnels.getAll', async () => {
    return sshTunnelService.getAllTunnels()
  })

  // Get tunnel by ID
  ipcMain.handle('ssh-tunnels.getById', async (_event, id: string) => {
    return sshTunnelService.getTunnelById(id)
  })

  // Create a new tunnel
  ipcMain.handle('ssh-tunnels.create', async (_event, tunnelData) => {
    return sshTunnelService.createTunnel(tunnelData)
  })

  // Update an existing tunnel
  ipcMain.handle('ssh-tunnels.update', async (_event, id: string, updates) => {
    return sshTunnelService.updateTunnel(id, updates)
  })

  // Delete a tunnel
  ipcMain.handle('ssh-tunnels.delete', async (_event, id: string) => {
    return sshTunnelService.deleteTunnel(id)
  })

  // Start a tunnel
  ipcMain.handle('ssh-tunnels.start', async (_event, id: string) => {
    return sshTunnelService.startTunnel(id)
  })

  // Stop a tunnel
  ipcMain.handle('ssh-tunnels.stop', async (_event, id: string) => {
    return sshTunnelService.stopTunnel(id)
  })

  // Get tunnel status
  ipcMain.handle('ssh-tunnels.getStatus', async (_event, id: string) => {
    const tunnel = await sshTunnelService.getTunnelById(id)
    return tunnel?.status || 'stopped'
  })

  // Get real tunnel status (check actual process)
  ipcMain.handle('ssh-tunnels.getRealStatus', async (_event, id: string) => {
    return sshTunnelService.getTunnelRealStatus(id)
  })

  // Get all auto-start tunnels
  ipcMain.handle('ssh-tunnels.getAutoStart', async () => {
    const allTunnels = await sshTunnelService.getAllTunnels()
    return allTunnels.filter((tunnel) => tunnel.autoStart)
  })

  // Start all auto-start tunnels
  ipcMain.handle('ssh-tunnels.startAutoStart', async () => {
    return sshTunnelService.startAutoStartTunnels()
  })

  // Stop all tunnels
  ipcMain.handle('ssh-tunnels.stopAll', async () => {
    return sshTunnelService.stopAllTunnels()
  })
}

/**
 * Sets up saved command IPC handlers.
 * @param savedCommandService - The saved command service instance.
 * @param terminalManager - The terminal manager instance.
 */
function setupSavedCommandHandlers(
  savedCommandService: SavedCommandService,
  terminalManager: TerminalManager
): void {
  // Get all saved commands
  ipcMain.handle('saved-commands.getAll', async () => {
    return savedCommandService.getAllCommands()
  })

  // Create a new saved command
  ipcMain.handle('saved-commands.create', async (_event, commandData) => {
    return savedCommandService.createCommand(commandData)
  })

  // Update an existing saved command
  ipcMain.handle('saved-commands.update', async (_event, id, updates) => {
    return savedCommandService.updateCommand(id, updates)
  })

  // Delete a saved command
  ipcMain.handle('saved-commands.delete', async (_event, id) => {
    return savedCommandService.deleteCommand(id)
  })

  // Execute a command in terminal
  ipcMain.on('saved-commands.execute', (_event, { terminalId, command }) => {
    terminalManager.writeToTerminal(terminalId, command + '\r')
  })

  // Copy command to clipboard
  ipcMain.handle('saved-commands.copyToClipboard', async (_event, command) => {
    return savedCommandService.copyCommandToClipboard(command)
  })
}

/**
 * Sets up sync-related IPC handlers.
 * @param syncManager - The sync manager instance.
 */
function setupSyncHandlers(syncManager: SyncManager): void {
  // Test MongoDB connection
  ipcMain.handle('sync.testConnection', async (_event, mongoUri, databaseName) => {
    try {
      return await syncManager.testConnection(mongoUri, databaseName)
    } catch (error) {
      console.error('Sync test connection error:', error)
      return false
    }
  })

  // Setup sync (first time or reconfigure)
  ipcMain.handle('sync.setup', async (_event, config) => {
    try {
      return await syncManager.setupSync(config)
    } catch (error) {
      console.error('Sync setup error:', error)
      return false
    }
  })

  // Enable sync
  ipcMain.handle('sync.enable', async (_event, config) => {
    try {
      return await syncManager.enableSync(config)
    } catch (error) {
      console.error('Sync enable error:', error)
      return false
    }
  })

  // Disable sync
  ipcMain.handle('sync.disable', async () => {
    try {
      await syncManager.disableSync()
      return true
    } catch (error) {
      console.error('Sync disable error:', error)
      return false
    }
  })

  // Get sync status
  ipcMain.handle('sync.getStatus', () => {
    return syncManager.getSyncStatus()
  })

  // Get sync configuration
  ipcMain.handle('sync.getConfig', async () => {
    try {
      return await syncManager.getSyncConfig()
    } catch (error) {
      console.error('Get sync config error:', error)
      return null
    }
  })

  // Update sync configuration
  ipcMain.handle('sync.updateConfig', async (_event, config) => {
    try {
      return await syncManager.updateSyncConfig(config)
    } catch (error) {
      console.error('Update sync config error:', error)
      return false
    }
  })

  // Check if sync is enabled
  ipcMain.handle('sync.isEnabled', async () => {
    try {
      return await syncManager.isSyncEnabled()
    } catch (error) {
      console.error('Check sync enabled error:', error)
      return false
    }
  })

  // Perform manual sync
  ipcMain.handle('sync.performSync', async () => {
    try {
      await syncManager.performSync()
      return true
    } catch (error) {
      console.error('Perform sync error:', error)
      return false
    }
  })

  // Force immediate sync
  ipcMain.handle('sync.forceSyncNow', async () => {
    try {
      await syncManager.forceSyncNow()
      return true
    } catch (error) {
      console.error('Force sync error:', error)
      return false
    }
  })

  // Migrate existing data
  ipcMain.handle('sync.migrateData', async () => {
    try {
      await syncManager.migrateExistingData()
      return true
    } catch (error) {
      console.error('Migrate data error:', error)
      return false
    }
  })

  // Delete sync configuration
  ipcMain.handle('sync.deleteConfig', async () => {
    try {
      await syncManager.deleteSyncConfig()
      return true
    } catch (error) {
      console.error('Delete sync config error:', error)
      return false
    }
  })
}
