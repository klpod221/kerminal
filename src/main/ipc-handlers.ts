import { ipcMain, shell, clipboard, dialog, Menu, BrowserWindow } from 'electron'
import { TerminalManager } from './services/terminal-manager'
import { WindowManager } from './services/window-manager'
import { SystemInfoService } from './services/system-info'
import { SSHProfileService } from './services/ssh-profile-service'
import { SSHConnectionService } from './services/ssh-connection-service'
import { SavedCommandService } from './services/saved-command-service'
import { SyncManager } from './services/sync-manager'
import { SSHTunnelService } from './services/ssh-tunnel-service'
import { AuthService } from './services/auth-service'
import { ResolvedSSHConfig, SSHProxy } from './types/ssh'
import { ConsoleLogger } from './utils/logger'

const logger = new ConsoleLogger('IpcHandlers')

/**
 * Sets up all IPC (Inter-Process Communication) handlers for the application.
 * @param windowManager - The window manager instance.
 * @param terminalManager - The terminal manager instance.
 * @param authService - The authentication service instance.
 * @param sshProfileService - The SSH profile service instance.
 */
export function setupIpcHandlers(
  windowManager: WindowManager,
  terminalManager: TerminalManager,
  authService: AuthService,
  sshProfileService: SSHProfileService
): void {
  // Create other service instances
  const sshConnectionService = new SSHConnectionService()
  const savedCommandService = new SavedCommandService()
  const syncManager = new SyncManager()
  const sshTunnelService = new SSHTunnelService(sshProfileService)

  // Initialize sync manager
  ;(async () => {
    try {
      await syncManager.initialize()
    } catch (error) {
      logger.error('Failed to initialize sync manager:', error as Error)
    }
  })()

  // Authentication IPC handlers
  setupAuthHandlers(authService)

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
    } catch (error) {
      logger.error(`Failed to create SSH terminal:`, error as Error)
      _event.reply('terminal.sshError', {
        terminalId,
        error: error instanceof Error ? error.message : 'Unknown error'
      })

      // Record failed connection
      try {
        await sshProfileService.recordConnection(profileId, 'failed')
      } catch (recordError) {
        logger.error('Failed to record connection failure:', recordError as Error)
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

  // Terminal Buffer Management IPC handlers

  // Get terminal buffer
  ipcMain.handle('terminal.buffer.get', (_event, terminalId: string) => {
    try {
      return terminalManager.getTerminalBuffer(terminalId)
    } catch (error) {
      logger.error(`Failed to get buffer for terminal ${terminalId}:`, error as Error)
      return []
    }
  })

  // Get terminal buffer as string
  ipcMain.handle('terminal.buffer.getString', (_event, terminalId: string) => {
    try {
      return terminalManager.getTerminalBufferAsString(terminalId)
    } catch (error) {
      logger.error(`Failed to get buffer string for terminal ${terminalId}:`, error as Error)
      return ''
    }
  })

  // Check if terminal has buffer
  ipcMain.handle('terminal.buffer.has', (_event, terminalId: string) => {
    try {
      return terminalManager.hasTerminalBuffer(terminalId)
    } catch (error) {
      logger.error(`Failed to check buffer for terminal ${terminalId}:`, error as Error)
      return false
    }
  })

  // Get buffer statistics
  ipcMain.handle('terminal.buffer.stats', () => {
    try {
      return terminalManager.getBufferStats()
    } catch (error) {
      logger.error('Failed to get buffer stats:', error as Error)
      return { totalTerminals: 0, totalLines: 0, memoryUsage: 0 }
    }
  })

  // Cleanup orphaned buffers
  ipcMain.on('terminal.buffer.cleanup', () => {
    try {
      terminalManager.cleanupOrphanedBuffers()
    } catch (error) {
      logger.error('Failed to cleanup orphaned buffers:', error as Error)
    }
  })
}

/**
 * Sets up window control IPC handlers.
 * @param windowManager - The window manager instance.
 */
function setupWindowHandlers(windowManager: WindowManager): void {
  ipcMain.on('window.minimize', () => {
    windowManager.minimize()
  })

  ipcMain.on('window.maximize', () => {
    windowManager.toggleMaximize()
  })

  ipcMain.on('window.close', () => {
    windowManager.close()
  })
}

/**
 * Sets up system information IPC handlers.
 */
function setupSystemHandlers(): void {
  // Handle system info requests
  ipcMain.handle('dashboard.get-system-info', async () => {
    return await SystemInfoService.getSystemInfo()
  })

  // Handle network info requests
  ipcMain.handle('dashboard.get-network-info', async () => {
    return await SystemInfoService.getNetworkInfo()
  })

  // Handle network status requests (includes connectivity check)
  ipcMain.handle('dashboard.get-network-status', async () => {
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
  ipcMain.on('clipboard.write', (_event, text) => {
    try {
      if (text && typeof text === 'string') {
        clipboard.writeText(text)
      }
    } catch (error) {
      logger.error('Failed to copy to clipboard:', error as Error)
    }
  })

  // Handle get clipboard text
  ipcMain.handle('clipboard.read', async () => {
    try {
      return clipboard.readText()
    } catch (error) {
      logger.error('Failed to read from clipboard:', error as Error)
      return ''
    }
  })

  // Handle context menu
  ipcMain.on('show-context-menu', (event, { items }) => {
    const menuTemplate = (items as import('./types/main').MenuItem[]).map((item) => ({
      label: item.label,
      enabled: item.enabled,
      click: () => {
        // Send click event back to renderer
        event.reply('context-menu-click', item.label)
      }
    }))

    const menu = Menu.buildFromTemplate(menuTemplate)
    const window = BrowserWindow.fromWebContents(event.sender)
    if (window) {
      menu.popup({ window })
    }
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

  // SSH Connection Test
  ipcMain.handle('ssh.testConnection', async (_event, config: ResolvedSSHConfig) => {
    return sshConnectionService.testConnection(config)
  })

  ipcMain.handle(
    'ssh.createResolvedConfigFromFormData',
    async (
      _event,
      formData: {
        host: string
        port: number
        user: string
        authType: 'password' | 'key' | 'agent'
        password?: string
        privateKeyPath?: string
        proxy?: SSHProxy
      }
    ) => {
      return sshProfileService.createResolvedConfigFromFormData(formData)
    }
  )
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
      logger.error('Sync test connection error:', error as Error)
      return false
    }
  })

  // ============================================================================
  // SYNC SETUP HANDLERS
  // ============================================================================

  // Setup sync (first time or reconfigure)
  ipcMain.handle('sync.setup', async (_event, config) => {
    try {
      return await syncManager.setupSync(config)
    } catch (error) {
      logger.error('Sync setup error:', error as Error)
      return false
    }
  })

  // Setup sync with master password verification
  ipcMain.handle('sync.setupWithPassword', async (_event, config, masterPassword) => {
    try {
      return await syncManager.setupSyncWithPassword(config, masterPassword)
    } catch (error) {
      logger.error('Sync setup with password error:', error as Error)
      throw error
    }
  })

  // ============================================================================
  // SYNC CONTROL HANDLERS
  // ============================================================================

  // Enable sync
  ipcMain.handle('sync.enable', async (_event, config) => {
    try {
      return await syncManager.enableSync(config)
    } catch (error) {
      logger.error('Sync enable error:', error as Error)
      return false
    }
  })

  // Disable sync
  ipcMain.handle('sync.disable', async () => {
    try {
      await syncManager.disableSync()
      return true
    } catch (error) {
      logger.error('Sync disable error:', error as Error)
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
      logger.error('Get sync config error:', error as Error)
      return null
    }
  })

  // Update sync configuration
  ipcMain.handle('sync.updateConfig', async (_event, config) => {
    try {
      return await syncManager.updateSyncConfig(config)
    } catch (error) {
      logger.error('Update sync config error:', error as Error)
      return false
    }
  })

  // Check if sync is enabled
  ipcMain.handle('sync.isEnabled', async () => {
    try {
      return await syncManager.isSyncEnabled()
    } catch (error) {
      logger.error('Check sync enabled error:', error as Error)
      return false
    }
  })

  // Perform manual sync
  ipcMain.handle('sync.performSync', async () => {
    try {
      await syncManager.performSync()
      return true
    } catch (error) {
      logger.error('Perform sync error:', error as Error)
      return false
    }
  })

  // Force immediate sync
  ipcMain.handle('sync.forceSyncNow', async () => {
    try {
      await syncManager.forceSyncNow()
      return true
    } catch (error) {
      logger.error('Force sync error:', error as Error)
      return false
    }
  })

  // Migrate existing data
  ipcMain.handle('sync.migrateData', async () => {
    try {
      await syncManager.migrateExistingData()
      return true
    } catch (error) {
      logger.error('Migrate data error:', error as Error)
      return false
    }
  })

  // Delete sync configuration
  ipcMain.handle('sync.deleteConfig', async () => {
    try {
      await syncManager.deleteSyncConfig()
      return true
    } catch (error) {
      logger.error('Delete sync config error:', error as Error)
      return false
    }
  })

  // Check if existing data exists
  ipcMain.handle('sync.hasExistingData', async () => {
    try {
      return await syncManager.hasExistingData()
    } catch (error) {
      logger.error('Check existing data error:', error as Error)
      return false
    }
  })
}

/**
 * Setup authentication-related IPC handlers
 */
function setupAuthHandlers(authService: AuthService): void {
  // Check if master password exists
  ipcMain.handle('auth:has-master-password', async () => {
    try {
      return await authService.hasMasterPassword()
    } catch (error) {
      logger.error('Check master password error:', error as Error)
      return false
    }
  })

  // Create master password
  ipcMain.handle('auth:create-master-password', async (_event, password: string, settings) => {
    try {
      await authService.createMasterPassword(password, settings)
      return true
    } catch (error) {
      logger.error('Create master password error:', error as Error)
      return false
    }
  })

  // Unlock with master password
  ipcMain.handle('auth:unlock-with-master-password', async (_event, password: string) => {
    try {
      return await authService.unlockWithMasterPassword(password)
    } catch (error) {
      logger.error('Unlock with password error:', error as Error)
      return false
    }
  })

  // Unlock with keychain
  ipcMain.handle('auth:unlock-with-keychain', async () => {
    try {
      return await authService.unlockWithKeychain()
    } catch (error) {
      logger.error('Unlock with keychain error:', error as Error)
      return false
    }
  })

  // Lock application
  ipcMain.handle('auth:lock', async () => {
    try {
      await authService.lock()
      return true
    } catch (error) {
      logger.error('Lock application error:', error as Error)
      return false
    }
  })

  // Check if application is unlocked
  ipcMain.handle('auth:is-unlocked', () => {
    return authService.isUnlocked()
  })

  // Unlock with password
  ipcMain.handle('auth:unlock-with-password', async (_event, password: string) => {
    try {
      return await authService.unlockWithMasterPassword(password)
    } catch (error) {
      logger.error('Unlock with password error:', error as Error)
      return false
    }
  })

  // Get security settings
  ipcMain.handle('auth:get-security-settings', async () => {
    try {
      return await authService.getSecuritySettings()
    } catch (error) {
      logger.error('Get security settings error:', error as Error)
      return null
    }
  })

  // Update security settings
  ipcMain.handle('auth:update-security-settings', async (_event, settings) => {
    try {
      await authService.updateSecuritySettings(settings)
      return true
    } catch (error) {
      logger.error('Update security settings error:', error as Error)
      return false
    }
  })

  // Change master password
  ipcMain.handle(
    'auth.changeMasterPassword',
    async (_event, currentPassword: string, newPassword: string) => {
      try {
        return await authService.changeMasterPassword(currentPassword, newPassword)
      } catch (error) {
        logger.error('Change master password error:', error as Error)
        return false
      }
    }
  )

  // Reset auto-lock timer (called on user activity)
  ipcMain.handle('auth.resetAutoLockTimer', () => {
    try {
      authService.resetAutoLockTimer()
      return true
    } catch (error) {
      logger.error('Reset auto-lock timer error:', error as Error)
      return false
    }
  })

  // Connect to MongoDB with master password
  ipcMain.handle(
    'auth:connect-mongo-master-password',
    async (_event, config: { mongoUri: string; databaseName: string; masterPassword: string }) => {
      try {
        return await authService.connectToMongoMasterPassword(
          config.mongoUri,
          config.databaseName,
          config.masterPassword
        )
      } catch (error) {
        logger.error('Connect to MongoDB master password error:', error as Error)
        return false
      }
    }
  )

  // Create new master password for empty MongoDB database
  ipcMain.handle(
    'auth:create-mongo-master-password',
    async (_event, config: { mongoUri: string; databaseName: string; masterPassword: string }) => {
      try {
        return await authService.createNewMongoMasterPassword(
          config.mongoUri,
          config.databaseName,
          config.masterPassword
        )
      } catch (error) {
        logger.error('Create MongoDB master password error:', error as Error)
        return false
      }
    }
  )

  // ============================================================================
  // MONGODB AUTHENTICATION HANDLERS
  // ============================================================================

  // Verify MongoDB master password
  ipcMain.handle(
    'auth:verify-mongo-master-password',
    async (_event, mongoUri: string, databaseName: string, masterPassword: string) => {
      try {
        return await authService.verifyMongoPassword(mongoUri, databaseName, masterPassword)
      } catch (error) {
        logger.error('Verify MongoDB master password error:', error as Error)
        return false
      }
    }
  )

  // Check if MongoDB has master password data
  ipcMain.handle(
    'auth:check-mongo-master-password-exists',
    async (_event, mongoUri: string, databaseName: string) => {
      try {
        return await authService.checkMongoMasterPasswordExists(mongoUri, databaseName)
      } catch (error) {
        logger.error('Check MongoDB master password exists error:', error as Error)
        return false
      }
    }
  )
}
