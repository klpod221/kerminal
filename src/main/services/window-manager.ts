import { BrowserWindow, shell } from 'electron'
import { join } from 'path'
import { is } from '@electron-toolkit/utils'
import { IWindowManager } from '../interfaces/terminal.interface'
import type { WindowConfig } from '../types/main'
import icon from '../../../resources/icon.png?asset'
import { ConsoleLogger } from '../utils/logger'

/**
 * Manages the main application window and window controls.
 * Implements IWindowManager for better separation of concerns.
 */
export class WindowManager implements IWindowManager {
  private mainWindow: BrowserWindow | null = null
  private readonly logger = new ConsoleLogger('WindowManager')
  private readonly defaultConfig: WindowConfig = {
    width: 900,
    height: 670,
    show: false,
    autoHideMenuBar: true,
    frame: false
  }

  /**
   * Creates and configures the main application window.
   * @param config - Window configuration options.
   * @returns The created BrowserWindow instance.
   */
  createWindow(config: WindowConfig = {}): BrowserWindow {
    const windowConfig = { ...this.defaultConfig, ...config }

    this.mainWindow = new BrowserWindow({
      ...windowConfig,
      ...(process.platform === 'linux' ? { icon } : {}),
      webPreferences: {
        preload: join(__dirname, '../preload/index.js'),
        sandbox: false
      }
    })

    this.setupWindowHandlers()
    this.loadContent()

    return this.mainWindow
  }

  /**
   * Sets up event handlers for the main window.
   */
  private setupWindowHandlers(): void {
    if (!this.mainWindow) return

    this.mainWindow.on('ready-to-show', () => {
      this.mainWindow?.show()
    })

    this.mainWindow.webContents.setWindowOpenHandler((details) => {
      shell.openExternal(details.url)
      return { action: 'deny' }
    })

    // Send maximize state changes to renderer (use safe checks)
    const safeSend = (channel: string, ...args: unknown[]): void => {
      try {
        if (
          this.mainWindow &&
          !this.mainWindow.isDestroyed() &&
          this.mainWindow.webContents &&
          // @ts-ignore - webContents.isDestroyed may not be present on all types
          (typeof this.mainWindow.webContents.isDestroyed === 'function'
            ? !this.mainWindow.webContents.isDestroyed()
            : !!this.mainWindow.webContents)
        ) {
          this.mainWindow.webContents.send(channel, ...args)
        }
      } catch (err) {
        this.logger.error(`WindowManager.safeSend error for ${channel}:`, err as Error)
      }
    }

    this.mainWindow.on('maximize', () => {
      safeSend('window-maximized', true)
    })

    this.mainWindow.on('unmaximize', () => {
      safeSend('window-maximized', false)
    })
  }

  /**
   * Loads the appropriate content based on development/production environment.
   */
  private loadContent(): void {
    if (!this.mainWindow) return

    if (is.dev && process.env['ELECTRON_RENDERER_URL']) {
      this.mainWindow.loadURL(process.env['ELECTRON_RENDERER_URL'])
    } else {
      this.mainWindow.loadFile(join(__dirname, '../renderer/index.html'))
    }
  }

  /**
   * Gets the main window instance.
   * @returns The main window instance or null if not created.
   */
  getMainWindow(): BrowserWindow | null {
    return this.mainWindow
  }

  /**
   * Minimizes the main window.
   */
  minimize(): void {
    this.mainWindow?.minimize()
  }

  /**
   * Toggles maximize/unmaximize state of the main window.
   */
  toggleMaximize(): void {
    if (!this.mainWindow) return

    if (this.mainWindow.isMaximized()) {
      this.mainWindow.unmaximize()
    } else {
      this.mainWindow.maximize()
    }
  }

  /**
   * Closes the main window.
   */
  close(): void {
    this.mainWindow?.close()
  }

  /**
   * Checks if the main window is maximized.
   * @returns True if maximized, false otherwise.
   */
  isMaximized(): boolean {
    return this.mainWindow?.isMaximized() ?? false
  }
}
