import { app, BrowserWindow } from 'electron'
import { electronApp as electronAppUtils, optimizer } from '@electron-toolkit/utils'
import { WindowManager } from './services/window-manager'
import { TerminalManager } from './services/terminal-manager'
import { SSHTunnelService } from './services/ssh-tunnel-service'
import { setupIpcHandlers } from './ipc-handlers'

/**
 * Main application class that orchestrates the entire application.
 */
class ElectronApp {
  private readonly windowManager: WindowManager
  private terminalManager: TerminalManager | null = null
  private sshTunnelService: SSHTunnelService | null = null

  constructor() {
    this.windowManager = new WindowManager()
  }

  /**
   * Initializes and starts the application.
   */
  async initialize(): Promise<void> {
    await app.whenReady()
    this.setupApp()
    await this.createMainWindow()
    this.setupAppEvents()
  }

  /**
   * Sets up the basic application configuration.
   */
  private setupApp(): void {
    electronAppUtils.setAppUserModelId('com.electron')

    app.on('browser-window-created', (_, window) => {
      optimizer.watchWindowShortcuts(window)
    })
  }

  /**
   * Creates the main application window and sets up managers.
   */
  private async createMainWindow(): Promise<void> {
    const mainWindow = this.windowManager.createWindow()
    this.terminalManager = new TerminalManager(mainWindow)
    this.sshTunnelService = new SSHTunnelService()

    setupIpcHandlers(this.windowManager, this.terminalManager)

    // Clean up any orphaned SSH tunnel processes first
    try {
      await this.sshTunnelService.killOrphanedTunnelProcesses()
    } catch (error) {
      console.error('Failed to clean up orphaned SSH tunnel processes:', error)
    }

    // Start auto-start tunnels after a short delay
    setTimeout(async () => {
      try {
        await this.sshTunnelService?.startAutoStartTunnels()
        console.log('Auto-start SSH tunnels initialized')
      } catch (error) {
        console.error('Failed to start auto-start SSH tunnels:', error)
      }
    }, 2000) // 2 second delay to ensure app is fully loaded
  }

  /**
   * Sets up application-level event handlers.
   */
  private setupAppEvents(): void {
    app.on('activate', async () => {
      // On macOS, re-create window when dock icon is clicked and no windows are open
      if (BrowserWindow.getAllWindows().length === 0) {
        await this.createMainWindow()
      }
    })

    app.on('window-all-closed', async () => {
      // Quit when all windows are closed, except on macOS
      if (process.platform !== 'darwin') {
        await this.cleanup()
        app.quit()
      }
    })

    app.on('before-quit', async () => {
      await this.cleanup()
    })
  }

  /**
   * Cleans up resources before application exit.
   */
  private async cleanup(): Promise<void> {
    if (this.sshTunnelService) {
      await this.sshTunnelService.stopAllTunnels()
    }
    if (this.terminalManager) {
      this.terminalManager.destroyAll()
    }
  }
}

// Initialize and start the application
const klTermApp = new ElectronApp()
klTermApp.initialize().catch((error) => {
  console.error('Failed to initialize application:', error)
  app.quit()
})
