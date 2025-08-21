import { ipcMain, shell, clipboard } from 'electron'
import { TerminalManager } from './services/terminal-manager'
import { WindowManager } from './services/window-manager'
import { SystemInfoService } from './services/system-info'

/**
 * Sets up all IPC (Inter-Process Communication) handlers for the application.
 * @param windowManager - The window manager instance.
 * @param terminalManager - The terminal manager instance.
 */
export function setupIpcHandlers(
  windowManager: WindowManager,
  terminalManager: TerminalManager
): void {
  // Terminal-related IPC handlers
  setupTerminalHandlers(terminalManager)

  // Window control IPC handlers
  setupWindowHandlers(windowManager)

  // System information IPC handlers
  setupSystemHandlers()

  // Utility IPC handlers
  setupUtilityHandlers()

  // Test IPC handler
  ipcMain.on('ping', () => console.log('pong'))
}

/**
 * Sets up terminal-related IPC handlers.
 * @param terminalManager - The terminal manager instance.
 */
function setupTerminalHandlers(terminalManager: TerminalManager): void {
  // Handle data from the renderer
  ipcMain.on('terminal.keystroke', (_event, { terminalId, data }) => {
    terminalManager.writeToTerminal(terminalId, data)
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

  // Handle terminal destruction
  ipcMain.on('terminal.destroy', (_event, { terminalId }) => {
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
}
