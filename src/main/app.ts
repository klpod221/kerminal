import { app, BrowserWindow } from 'electron'
import { electronApp as electronAppUtils, optimizer } from '@electron-toolkit/utils'
import { WindowManager } from './services/window-manager'
import { TerminalManager } from './services/terminal-manager'
import { setupIpcHandlers } from './ipc-handlers'

/**
 * Main application class that orchestrates the entire application.
 */
class ElectronApp {
  private readonly windowManager: WindowManager
  private terminalManager: TerminalManager | null = null

  constructor() {
    this.windowManager = new WindowManager()
  }

  /**
   * Initializes and starts the application.
   */
  async initialize(): Promise<void> {
    await app.whenReady()
    this.setupApp()
    this.createMainWindow()
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
  private createMainWindow(): void {
    const mainWindow = this.windowManager.createWindow()
    this.terminalManager = new TerminalManager(mainWindow)

    setupIpcHandlers(this.windowManager, this.terminalManager)
  }

  /**
   * Sets up application-level event handlers.
   */
  private setupAppEvents(): void {
    app.on('activate', () => {
      // On macOS, re-create window when dock icon is clicked and no windows are open
      if (BrowserWindow.getAllWindows().length === 0) {
        this.createMainWindow()
      }
    })

    app.on('window-all-closed', () => {
      // Quit when all windows are closed, except on macOS
      if (process.platform !== 'darwin') {
        this.cleanup()
        app.quit()
      }
    })

    app.on('before-quit', () => {
      this.cleanup()
    })
  }

  /**
   * Cleans up resources before application exit.
   */
  private cleanup(): void {
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
