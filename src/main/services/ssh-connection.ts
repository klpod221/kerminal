import { Client, ConnectConfig, ClientChannel } from 'ssh2'
import { BrowserWindow } from 'electron'
import * as fs from 'fs'
import { ResolvedSSHConfig } from '../types/ssh'

/**
 * SSH Connection Manager using ssh2 library
 */
export class SSHConnection {
  private readonly client: Client
  private isConnected = false
  private readonly terminalId: string
  private readonly mainWindow: BrowserWindow
  private readonly config: ResolvedSSHConfig
  private readonly profileName: string
  private stream: ClientChannel | null = null

  constructor(
    terminalId: string,
    config: ResolvedSSHConfig,
    mainWindow: BrowserWindow,
    profileName: string
  ) {
    this.terminalId = terminalId
    this.config = config
    this.mainWindow = mainWindow
    this.profileName = profileName
    this.client = new Client()
    this.setupEventHandlers()
  }

  private safeSend(channel: string, ...args: unknown[]): void {
    try {
      if (
        this.mainWindow &&
        !this.mainWindow.isDestroyed() &&
        this.mainWindow.webContents &&
        // @ts-ignore - some electron versions/types may not include isDestroyed on webContents
        (typeof this.mainWindow.webContents.isDestroyed === 'function'
          ? !this.mainWindow.webContents.isDestroyed()
          : !!this.mainWindow.webContents)
      ) {
        this.mainWindow.webContents.send(channel, ...args)
      }
    } catch (err) {
      console.error(`SSHConnection.safeSend error for ${channel}:`, err)
    }
  }

  /**
   * Connect to SSH server
   */
  async connect(): Promise<void> {
    return new Promise((resolve, reject) => {
      const connectConfig: ConnectConfig = {
        host: this.config.host,
        port: this.config.port || 22,
        username: this.config.user,
        readyTimeout: 10000,
        keepaliveInterval: 60000,
        keepaliveCountMax: 3
      }

      // Setup authentication
      if (this.config.keyPath) {
        try {
          connectConfig.privateKey = fs.readFileSync(this.config.keyPath)
        } catch (error) {
          reject(new Error(`Failed to read SSH key: ${error}`))
          return
        }
      } else if (this.config.password) {
        connectConfig.password = this.config.password
      } else {
        reject(new Error('No authentication method provided'))
        return
      }

      // Setup connection handlers
      this.client.once('ready', () => {
        console.log(`SSH connection ready for terminal ${this.terminalId}`)
        this.isConnected = true
        this.startShell()
        resolve()
      })

      this.client.once('error', (error) => {
        console.error(`SSH connection error for terminal ${this.terminalId}:`, error)
        reject(new Error(error.message || 'SSH connection failed'))
      })

      // Connect to SSH server
      this.client.connect(connectConfig)
    })
  }

  /**
   * Start interactive shell
   */
  private startShell(): void {
    const shellOptions = {
      term: 'xterm-256color',
      cols: 80,
      rows: 30,
      env: {
        LANG: 'en_US.UTF-8',
        TERM: 'xterm-256color',
        COLORTERM: 'truecolor',
        TERMINFO: '/usr/share/terminfo',
        INPUTRC: '/dev/null',
        BASH_COMPLETION_COMPAT_DIR: ''
      }
    }

    this.client.shell(shellOptions, (err, stream) => {
      if (err) {
        console.error(`Failed to start shell for terminal ${this.terminalId}:`, err)
        this.safeSend('terminal.sshError', { terminalId: this.terminalId, error: err.message })
        return
      }

      console.log(`Shell started for SSH terminal ${this.terminalId}`)

      // Handle shell data
      stream.on('data', (data: Buffer) => {
        let output = data.toString()

        // Filter out terminfo warnings while preserving other output
        output = output
          .replace(/.*terminfo\[kcbt\]: parameter not set.*\n?/g, '')
          .replace(
            /.*\.autocomplete__key-bindings:\d+: terminfo\[kcbt\]: parameter not set.*\n?/g,
            ''
          )
          .replace(/.*bash: completion: function `.*' not found.*\n?/g, '')

        if (output) {
          this.safeSend('terminal.incomingData', output, this.terminalId)
          this.handleTitleChange(output)
        }
      })

      // Handle shell close
      stream.on('close', () => {
        console.log(`SSH shell closed for terminal ${this.terminalId}`)
        this.isConnected = false
        this.safeSend('terminal.sshDisconnected', { terminalId: this.terminalId })
        // Auto close the tab when SSH shell closes
        this.safeSend('terminal.autoClose', {
          terminalId: this.terminalId,
          reason: 'SSH shell closed'
        })
      })

      // Handle shell errors
      stream.on('error', (error: Error) => {
        console.error(`SSH shell error for terminal ${this.terminalId}:`, error)
        this.safeSend('terminal.sshError', { terminalId: this.terminalId, error: error.message })
      })

      // Store stream for writing data
      this.stream = stream

      // Set initial title
      this.setInitialTitle()

      // Execute initial commands if specified
      if (this.config.commands && this.config.commands.length > 0) {
        setTimeout(() => {
          for (const command of this.config.commands!) {
            console.log(`Executing command: ${command}`)
            this.writeToShell(`${command}\n`)
          }
        }, 1000)
      }
    })
  }

  /**
   * Setup event handlers for SSH client
   */
  private setupEventHandlers(): void {
    this.client.on('error', (error) => {
      console.error(`SSH client error for terminal ${this.terminalId}:`, error)
      this.safeSend('terminal.sshError', { terminalId: this.terminalId, error: error.message })
    })

    this.client.on('end', () => {
      console.log(`SSH connection ended for terminal ${this.terminalId}`)
      this.isConnected = false
    })

    this.client.on('close', () => {
      console.log(`SSH connection closed for terminal ${this.terminalId}`)
      this.isConnected = false
      this.safeSend('terminal.sshDisconnected', { terminalId: this.terminalId })
      // Auto close the tab when SSH connection closes
      this.safeSend('terminal.autoClose', {
        terminalId: this.terminalId,
        reason: 'SSH connection closed'
      })
    })
  }

  /**
   * Write data to shell
   */
  writeToShell(data: string): void {
    if (this.stream && this.isConnected) {
      this.stream.write(data)
    }
  }

  /**
   * Resize terminal
   */
  resize(cols: number, rows: number): void {
    if (this.stream && this.isConnected) {
      this.stream.setWindow(rows, cols)
    }
  }

  /**
   * Set initial SSH title
   */
  private setInitialTitle(): void {
    setTimeout(() => {
      this.safeSend('terminal.titleChanged', {
        terminalId: this.terminalId,
        title: this.profileName
      })
    }, 500)
  }

  /**
   * Handle title change from shell output
   */
  private handleTitleChange(data: string): void {
    const esc = String.fromCharCode(27) // ESC
    const bel = String.fromCharCode(7) // BEL
    const titleRegex = new RegExp(esc + ']0;([^' + bel + esc + ']*)' + bel)
    const titleMatch = titleRegex.exec(data)

    if (titleMatch) {
      const title = titleMatch[1].trim()
      if (title) {
        this.safeSend('terminal.titleChanged', { terminalId: this.terminalId, title: title })
      }
    }
  }

  /**
   * Execute a command on the remote server
   */
  executeCommand(command: string): Promise<string> {
    return new Promise((resolve, reject) => {
      if (!this.isConnected) {
        reject(new Error('SSH not connected'))
        return
      }

      this.client.exec(command, (err, stream) => {
        if (err) {
          reject(new Error(err.message || 'Command execution failed'))
          return
        }

        let output = ''
        let errorOutput = ''

        stream.on('data', (data: Buffer) => {
          output += data.toString()
        })

        stream.stderr.on('data', (data: Buffer) => {
          errorOutput += data.toString()
        })

        stream.on('close', (code: number) => {
          if (code === 0) {
            resolve(output)
          } else {
            reject(new Error(`Command failed with code ${code}: ${errorOutput}`))
          }
        })
      })
    })
  }

  /**
   * Disconnect SSH connection
   */
  disconnect(): void {
    if (this.client) {
      this.client.end()
      this.isConnected = false
    }
  }

  /**
   * Check if connected
   */
  getConnectionStatus(): boolean {
    return this.isConnected
  }

  /**
   * Get connection info
   */
  getConnectionInfo(): { host: string; user: string; port: number } {
    return {
      host: this.config.host,
      user: this.config.user,
      port: this.config.port || 22
    }
  }
}
