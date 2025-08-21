import { BrowserWindow } from 'electron'
import * as os from 'os'
import * as pty from 'node-pty'

/**
 * Manages terminal instances and PTY processes.
 */
export class TerminalManager {
  private terminals: Record<string, pty.IPty> = {}
  private initialBuffers: Record<string, string[]> = {}
  private isRendererReady = false
  private readonly shellPath: string

  constructor(private readonly mainWindow: BrowserWindow) {
    this.shellPath =
      os.platform() === 'win32'
        ? process.env.COMSPEC || 'cmd.exe'
        : process.env.SHELL || '/bin/bash'
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
      name: 'xterm-color',
      cols: 80,
      rows: 30,
      cwd: process.env.HOME,
      env: {
        ...process.env,
        LANG: 'en_US.UTF-8',
        TERM: 'xterm-256color',
        COLORTERM: 'truecolor'
      }
    })

    this.terminals[terminalId] = ptyProcess
    this.initialBuffers[terminalId] = []

    this.setupTerminalHandlers(ptyProcess, terminalId)
    this.setInitialTitle(terminalId)

    return ptyProcess
  }

  /**
   * Sets up event handlers for a terminal process.
   * @param ptyProcess - The PTY process to set up handlers for.
   * @param terminalId - Unique identifier for the terminal.
   */
  private setupTerminalHandlers(ptyProcess: pty.IPty, terminalId: string): void {
    ptyProcess.onData((data) => {
      if (this.isRendererReady) {
        this.mainWindow.webContents.send('terminal.incomingData', data, terminalId)
      } else {
        if (!this.initialBuffers[terminalId]) {
          this.initialBuffers[terminalId] = []
        }
        this.initialBuffers[terminalId].push(data)
      }

      this.handleTitleChange(data, terminalId)
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
        this.mainWindow.webContents.send('terminal.titleChanged', {
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
        this.mainWindow.webContents.send('terminal.titleChanged', { terminalId, title })
      }
    }
  }

  /**
   * Writes data to a terminal.
   * @param terminalId - Unique identifier for the terminal.
   * @param data - Data to write to the terminal.
   */
  writeToTerminal(terminalId: string, data: string): void {
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
      this.mainWindow.webContents.send(
        'terminal.incomingData',
        this.initialBuffers[terminalId].join(''),
        terminalId
      )
      this.initialBuffers[terminalId] = []
    }
  }

  /**
   * Destroys a terminal instance.
   * @param terminalId - Unique identifier for the terminal.
   */
  destroyTerminal(terminalId: string): void {
    const terminal = this.terminals[terminalId]
    if (terminal) {
      terminal.kill()
      delete this.terminals[terminalId]
      delete this.initialBuffers[terminalId]
    }
  }

  /**
   * Destroys all terminal instances.
   */
  destroyAll(): void {
    Object.keys(this.terminals).forEach((terminalId) => {
      this.destroyTerminal(terminalId)
    })
  }
}
