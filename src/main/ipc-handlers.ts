import { ipcMain, shell, clipboard, dialog } from 'electron'
import { TerminalManager } from './services/terminal-manager'
import { WindowManager } from './services/window-manager'
import { SystemInfoService } from './services/system-info'
import { SSHProfileService } from './services/ssh-profile-service'
import { SSHConnectionService } from './services/ssh-connection-service'

/**
 * Sets up all IPC (Inter-Process Communication) handlers for the application.
 * @param windowManager - The window manager instance.
 * @param terminalManager - The terminal manager instance.
 */
export function setupIpcHandlers(
  windowManager: WindowManager,
  terminalManager: TerminalManager
): void {
  // Create SSH service instances
  const sshProfileService = new SSHProfileService()
  const sshConnectionService = new SSHConnectionService()

  // Terminal-related IPC handlers
  setupTerminalHandlers(terminalManager, sshConnectionService, sshProfileService)

  // Window control IPC handlers
  setupWindowHandlers(windowManager)

  // System information IPC handlers
  setupSystemHandlers()

  // SSH-related IPC handlers
  setupSSHHandlers(sshProfileService, sshConnectionService)

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
