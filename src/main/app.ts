import { app, BrowserWindow } from 'electron'
import { electronApp as electronAppUtils, optimizer } from '@electron-toolkit/utils'
import { WindowManager } from './services/window-manager'
import { TerminalManager } from './services/terminal-manager'
import { SSHTunnelService } from './services/ssh-tunnel-service'
import { setupIpcHandlers } from './ipc-handlers'
import { ConsoleLogger } from './utils/logger'
import { autoUpdater } from 'electron-updater'

/**
 * Main application class that orchestrates the entire application.
 */
class ElectronApp {
  private readonly windowManager: WindowManager
  private terminalManager: TerminalManager | null = null
  private sshTunnelService: SSHTunnelService | null = null
  private readonly logger = new ConsoleLogger('ElectronApp')

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

    this.setupUpdater()
  }

  /**
   * Sets up the basic application configuration.
   */
  private setupApp(): void {
    electronAppUtils.setAppUserModelId('com.klpod221.kerminal')

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

    // Start auto-start tunnels after a short delay
    setTimeout(async () => {
      try {
        await this.sshTunnelService?.startAutoStartTunnels()
      } catch (error) {
        this.logger.error('Failed to start auto-start SSH tunnels:', error as Error)
      }
    }, 2000)
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
   * Sets up the auto-updater logic.
   */
  private setupUpdater(): void {
    this.logger.info('Updater setup complete. Checking for updates...')

    // Listen to autoUpdater events
    autoUpdater.on('checking-for-update', () => {
      this.logger.info('Checking for update...')
    })

    autoUpdater.on('update-available', (info) => {
      this.logger.info('Update available.', info)
    })

    autoUpdater.on('update-not-available', (info) => {
      this.logger.info('Update not available.', info)
    })

    autoUpdater.on('error', (err) => {
      this.logger.error('Error in auto-updater. ' + err)
    })

    autoUpdater.on('download-progress', (progressObj) => {
      let log_message = 'Download speed: ' + progressObj.bytesPerSecond
      log_message = log_message + ' - Downloaded ' + progressObj.percent + '%'
      log_message = log_message + ' (' + progressObj.transferred + '/' + progressObj.total + ')'
      this.logger.info(log_message)
    })

    autoUpdater.on('update-downloaded', (info) => {
      this.logger.info('Update downloaded. Application will be quit for update.', info)
      autoUpdater.quitAndInstall()
    })

    // Check for updates and notify
    autoUpdater.checkForUpdatesAndNotify()
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
const kerminalApp = new ElectronApp()
const logger = new ConsoleLogger('Main')
;(async () => {
  try {
    await kerminalApp.initialize()
  } catch (error) {
    logger.error('Failed to initialize application:', error as Error)
    app.quit()
  }
})()
