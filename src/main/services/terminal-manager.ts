import { BrowserWindow } from 'electron'
import * as os from 'os'
import * as pty from 'node-pty'
import { ResolvedSSHConfig } from '../types/ssh'
import { SSHConnection } from './ssh-connection'
import { TerminalBufferManager } from './terminal-buffer-manager'

/**
 * Manages terminal instances and PTY processes.
 */
export class TerminalManager {
  private terminals: Record<string, pty.IPty> = {}
  private initialBuffers: Record<string, string[]> = {}
  private readonly sshTerminals: Record<string, { profileId: string; config: ResolvedSSHConfig }> =
    {}
  private readonly sshConnections: Record<string, SSHConnection> = {}
  private isRendererReady = false
  private readonly shellPath: string
  private readonly bufferManager: TerminalBufferManager

  constructor(private readonly mainWindow: BrowserWindow) {
    this.shellPath =
      os.platform() === 'win32'
        ? process.env.COMSPEC || 'cmd.exe'
        : process.env.SHELL || '/bin/bash'

    this.bufferManager = TerminalBufferManager.getInstance()
  }

  /**
   * Safely send IPC to renderer only if the window and webContents are alive.
   * Protects against "Object has been destroyed" errors when app/windows are closing.
   */
  private safeSend(channel: string, ...args: unknown[]): void {
    try {
      if (
        this.mainWindow &&
        !this.mainWindow.isDestroyed() &&
        this.mainWindow.webContents &&
        // webContents has isDestroyed in recent electron versions
        // fall back to truthy check if not available
        // @ts-ignore - some electron versions do not expose webContents.isDestroyed on types
        (typeof this.mainWindow.webContents.isDestroyed === 'function'
          ? !this.mainWindow.webContents.isDestroyed()
          : !!this.mainWindow.webContents)
      ) {
        this.mainWindow.webContents.send(channel, ...args)
      } else {
        // Window/webContents already destroyed — ignore send
        console.warn(
          `safeSend: skipped sending ${channel} because window/webContents are destroyed`
        )
      }
    } catch (err) {
      console.error(`safeSend error for channel ${channel}:`, err)
    }
  }

  /**
   * Sets the renderer ready state.
   * @param ready - Whether the renderer is ready to receive data.
   */
  setRendererReady(ready: boolean): void {
    this.isRendererReady = ready
  }

  /**
   * Creates a new terminal instance.
   * @param terminalId - Unique identifier for the terminal.
   * @returns The created PTY process.
   */
  createTerminal(terminalId: string): pty.IPty {
    const ptyProcess = pty.spawn(this.shellPath, [], {
      name: 'xterm-256color',
      cols: 80,
      rows: 30,
      cwd: process.env.HOME,
      env: {
        ...process.env,
        LANG: 'en_US.UTF-8',
        TERM: 'xterm-256color',
        COLORTERM: 'truecolor',
        TERMINFO: '/usr/share/terminfo',
        INPUTRC: '/dev/null',
        BASH_COMPLETION_COMPAT_DIR: ''
      }
    })

    this.terminals[terminalId] = ptyProcess
    this.initialBuffers[terminalId] = []

    this.setupTerminalHandlers(ptyProcess, terminalId)
    this.setInitialTitle(terminalId)

    return ptyProcess
  }

  /**
   * Creates a new SSH terminal instance using ssh2 library.
   * @param terminalId - Unique identifier for the terminal.
   * @param config - SSH configuration.
   * @param profileId - SSH profile ID.
   * @returns Promise that resolves when SSH connection is established.
   */
  async createSSHTerminal(
    terminalId: string,
    config: ResolvedSSHConfig,
    profileId: string,
    profileName: string
  ): Promise<void> {
    try {
      console.log(`Creating SSH terminal ${terminalId} for ${config.user}@${config.host}`)

      // Create SSH connection instance
      const sshConnection = new SSHConnection(terminalId, config, this.mainWindow, profileName)

      // Store SSH connection and terminal info
      this.sshConnections[terminalId] = sshConnection
      this.sshTerminals[terminalId] = { profileId, config }
      this.initialBuffers[terminalId] = []

      // Connect to SSH server
      await sshConnection.connect()

      console.log(`SSH terminal ${terminalId} connected successfully`)
    } catch (error) {
      console.error(`Failed to create SSH terminal ${terminalId}:`, error)

      // Clean up on failure
      delete this.sshConnections[terminalId]
      delete this.sshTerminals[terminalId]
      delete this.initialBuffers[terminalId]

      throw error
    }
  }

  /**
   * Check if terminal is SSH terminal
   */
  isSSHTerminal(terminalId: string): boolean {
    return terminalId in this.sshTerminals
  }

  /**
   * Get SSH terminal info
   */
  getSSHTerminalInfo(terminalId: string): { profileId: string; config: ResolvedSSHConfig } | null {
    return this.sshTerminals[terminalId] || null
  }

  /**
   * Get SSH connection instance
   */
  getSSHConnection(terminalId: string): SSHConnection | null {
    return this.sshConnections[terminalId] || null
  }

  /**
   * Sets up event handlers for a terminal process.
   * @param ptyProcess - The PTY process to set up handlers for.
   * @param terminalId - Unique identifier for the terminal.
   */
  private setupTerminalHandlers(ptyProcess: pty.IPty, terminalId: string): void {
    ptyProcess.onData((data) => {
      // Filter out terminfo warnings while preserving other output
      let filteredData = data
      if (typeof data === 'string') {
        // Remove terminfo kcbt warnings and autocomplete error messages
        filteredData = data
          .replace(/.*terminfo\[kcbt\]: parameter not set.*\n?/g, '')
          .replace(
            /.*\.autocomplete__key-bindings:\d+: terminfo\[kcbt\]: parameter not set.*\n?/g,
            ''
          )
          .replace(/.*bash: completion: function `.*' not found.*\n?/g, '')
      }

      // Always save to buffer manager for persistence
      if (filteredData) {
        this.bufferManager.saveToBuffer(terminalId, filteredData)
      }

      if (this.isRendererReady && filteredData) {
        this.safeSend('terminal.incomingData', filteredData, terminalId)
      } else if (filteredData) {
        if (!this.initialBuffers[terminalId]) {
          this.initialBuffers[terminalId] = []
        }
        this.initialBuffers[terminalId].push(filteredData)
      }

      this.handleTitleChange(data, terminalId)
    })

    // Handle terminal process exit
    ptyProcess.onExit((exitCode) => {
      console.log(`Terminal ${terminalId} exited with code ${exitCode.exitCode}`)

      // Auto close the tab when terminal exits
      this.safeSend('terminal.autoClose', {
        terminalId,
        reason: `Terminal exited with code ${exitCode.exitCode}`,
        exitCode: exitCode.exitCode
      })

      // Clean up terminal
      delete this.terminals[terminalId]
      delete this.initialBuffers[terminalId]

      // Clear buffer from buffer manager
      this.bufferManager.clearBuffer(terminalId)
    })
  }

  /**
   * Sets the initial title for a terminal.
   * @param terminalId - Unique identifier for the terminal.
   */
  private setInitialTitle(terminalId: string): void {
    setTimeout(() => {
      if (this.isRendererReady) {
        const user = process.env.USER || process.env.USERNAME || 'user'
        const hostname = os.hostname()
        const initialTitle = `${user}@${hostname}`
        this.safeSend('terminal.titleChanged', {
          terminalId,
          title: initialTitle
        })
      }
    }, 500)
  }

  /**
   * Handles title change escape sequences from terminal output.
   * @param data - Terminal output data.
   * @param terminalId - Unique identifier for the terminal.
   */
  private handleTitleChange(data: string, terminalId: string): void {
    const esc = String.fromCharCode(27) // ESC
    const bel = String.fromCharCode(7) // BEL
    const titleRegex = new RegExp(esc + ']0;([^' + bel + esc + ']*)' + bel)
    const titleMatch = titleRegex.exec(data)

    if (titleMatch) {
      const title = titleMatch[1].trim()
      if (title && this.isRendererReady) {
        this.safeSend('terminal.titleChanged', { terminalId, title })
      }
    }
  }

  /**
   * Writes data to a terminal.
   * @param terminalId - Unique identifier for the terminal.
   * @param data - Data to write to the terminal.
   */
  writeToTerminal(terminalId: string, data: string): void {
    // Check if it's an SSH terminal
    const sshConnection = this.sshConnections[terminalId]
    if (sshConnection) {
      sshConnection.writeToShell(data)
      return
    }

    // Handle regular PTY terminal
    const terminal = this.terminals[terminalId]
    if (terminal) {
      terminal.write(data)
    }
  }

  /**
   * Resizes a terminal.
   * @param terminalId - Unique identifier for the terminal.
   * @param cols - Number of columns.
   * @param rows - Number of rows.
   */
  resizeTerminal(terminalId: string, cols: number, rows: number): void {
    // Check if it's an SSH terminal
    const sshConnection = this.sshConnections[terminalId]
    if (sshConnection) {
      sshConnection.resize(cols, rows)
      return
    }

    // Handle regular PTY terminal
    const terminal = this.terminals[terminalId]
    if (terminal) {
      terminal.resize(cols, rows)
    }
  }

  /**
   * Handles terminal ready event and sends buffered data.
   * @param terminalId - Unique identifier for the terminal.
   */
  handleTerminalReady(terminalId: string): void {
    if (!this.isRendererReady) {
      this.isRendererReady = true
    }

    if (this.initialBuffers[terminalId] && this.initialBuffers[terminalId].length > 0) {
      this.safeSend('terminal.incomingData', this.initialBuffers[terminalId].join(''), terminalId)
      this.initialBuffers[terminalId] = []
    }
  }

  /**
   * Destroys a terminal instance.
   * @param terminalId - Unique identifier for the terminal.
   */
  destroyTerminal(terminalId: string): void {
    // Handle SSH connection cleanup
    const sshConnection = this.sshConnections[terminalId]
    if (sshConnection) {
      sshConnection.disconnect()
      delete this.sshConnections[terminalId]
      delete this.sshTerminals[terminalId]
      delete this.initialBuffers[terminalId]

      // Clear buffer from buffer manager
      this.bufferManager.clearBuffer(terminalId)
      return
    }

    // Handle regular PTY terminal cleanup
    const terminal = this.terminals[terminalId]
    if (terminal) {
      terminal.kill()
      delete this.terminals[terminalId]
      delete this.initialBuffers[terminalId]
      delete this.sshTerminals[terminalId]

      // Clear buffer from buffer manager
      this.bufferManager.clearBuffer(terminalId)
    }
  }

  /**
   * Destroys all terminal instances.
   */
  destroyAll(): void {
    // Destroy all SSH connections
    Object.keys(this.sshConnections).forEach((terminalId) => {
      this.destroyTerminal(terminalId)
    })

    // Destroy all PTY terminals
    Object.keys(this.terminals).forEach((terminalId) => {
      this.destroyTerminal(terminalId)
    })

    // Cleanup buffer manager
    this.bufferManager.cleanup()
  }

  /**
   * Get buffer for specific terminal
   * @param terminalId - Terminal identifier
   * @returns Buffer lines array
   */
  getTerminalBuffer(terminalId: string): string[] {
    return this.bufferManager.getBuffer(terminalId)
  }

  /**
   * Get buffer as string for specific terminal
   * @param terminalId - Terminal identifier
   * @returns Buffer as joined string
   */
  getTerminalBufferAsString(terminalId: string): string {
    return this.bufferManager.getBufferAsString(terminalId)
  }

  /**
   * Check if terminal has buffer
   * @param terminalId - Terminal identifier
   * @returns Whether buffer exists
   */
  hasTerminalBuffer(terminalId: string): boolean {
    return this.bufferManager.hasBuffer(terminalId)
  }

  /**
   * Get buffer manager statistics
   * @returns Buffer statistics
   */
  getBufferStats(): { totalTerminals: number; totalLines: number; memoryUsage: number } {
    return this.bufferManager.getStats()
  }

  /**
   * Cleanup orphaned buffers
   */
  cleanupOrphanedBuffers(): void {
    const activeTerminalIds = [...Object.keys(this.terminals), ...Object.keys(this.sshConnections)]
    this.bufferManager.cleanupOrphanedBuffers(activeTerminalIds)
  }
}
