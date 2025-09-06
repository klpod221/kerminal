import { spawn, ChildProcess } from 'child_process'
import * as fs from 'fs/promises'
import { Client } from 'ssh2'
import { ResolvedSSHConfig, SSHConnectionOptions, SSHProxy } from '../types/ssh'
import { SSHProfileService } from './ssh-profile-service'

/**
 * SSH Connection Manager
 * Handles creating and managing SSH connections for terminals
 */
export class SSHConnectionService {
  private readonly profileService: SSHProfileService
  private readonly activeConnections = new Map<string, ChildProcess>()

  constructor() {
    this.profileService = new SSHProfileService()
  }

  /**
   * Create an SSH connection for a terminal
   */
  async createSSHConnection(options: SSHConnectionOptions): Promise<ChildProcess> {
    const { profileId, terminalId } = options

    try {
      // Get profile with resolved configuration
      const profile = await this.profileService.getProfileById(profileId)
      if (!profile) {
        throw new Error(`SSH Profile with ID ${profileId} not found`)
      }

      // Validate configuration
      const validation = this.profileService.validateConnectionConfig(profile.resolvedConfig)
      if (!validation.valid) {
        throw new Error(`Invalid SSH configuration: ${validation.errors.join(', ')}`)
      }

      // Build SSH command
      const sshCommand = await this.buildSSHCommand(profile.resolvedConfig)

      // Spawn SSH process
      const sshProcess = spawn(sshCommand.command, sshCommand.args, {
        stdio: ['pipe', 'pipe', 'pipe'],
        env: {
          ...process.env,
          TERM: 'xterm-256color'
        }
      })

      // Store active connection
      this.activeConnections.set(terminalId, sshProcess)

      // Handle process events
      sshProcess.on('spawn', () => {
        console.log(`SSH connection established for terminal ${terminalId}`)
        this.profileService.recordConnection(profileId, 'connected')
        options.onConnect?.()
      })

      sshProcess.on('error', (error) => {
        console.error(`SSH connection error for terminal ${terminalId}:`, error)
        this.profileService.recordConnection(profileId, 'failed')
        this.activeConnections.delete(terminalId)
        options.onError?.(error)
      })

      sshProcess.on('exit', (code, signal) => {
        console.log(
          `SSH connection closed for terminal ${terminalId} (code: ${code}, signal: ${signal})`
        )
        this.activeConnections.delete(terminalId)
        options.onDisconnect?.()
      })

      // Execute initial commands if specified
      if (profile.resolvedConfig.commands && profile.resolvedConfig.commands.length > 0) {
        this.executeInitialCommands(sshProcess, profile.resolvedConfig.commands)
      }

      return sshProcess
    } catch (error) {
      console.error(`Failed to create SSH connection for terminal ${terminalId}:`, error)
      await this.profileService.recordConnection(profileId, 'failed')
      throw error
    }
  }

  /**
   * Get active SSH connection for a terminal
   */
  getConnection(terminalId: string): ChildProcess | undefined {
    return this.activeConnections.get(terminalId)
  }

  /**
   * Close SSH connection for a terminal
   */
  closeConnection(terminalId: string): void {
    const connection = this.activeConnections.get(terminalId)
    if (connection) {
      connection.kill('SIGTERM')
      this.activeConnections.delete(terminalId)
    }
  }

  /**
   * Close all active SSH connections
   */
  closeAllConnections(): void {
    for (const [terminalId, connection] of this.activeConnections) {
      connection.kill('SIGTERM')
      this.activeConnections.delete(terminalId)
    }
  }

  /**
   * Check if terminal has an active SSH connection
   */
  hasConnection(terminalId: string): boolean {
    return this.activeConnections.has(terminalId)
  }

  /**
   * Get count of active connections
   */
  getActiveConnectionCount(): number {
    return this.activeConnections.size
  }

  /**
   * Test SSH connection without establishing a persistent connection using SSH2
   * @param config - Resolved SSH configuration
   * @returns Promise resolving to connection test result
   */
  async testConnection(config: ResolvedSSHConfig): Promise<{
    success: boolean
    message: string
    duration?: number
    error?: string
  }> {
    const startTime = Date.now()

    return new Promise((resolve) => {
      const client = new Client()
      let resolved = false

      const resolveOnce = (result: {
        success: boolean
        message: string
        duration?: number
        error?: string
      }): void => {
        if (!resolved) {
          resolved = true
          resolve(result)
        }
      }

      // Connection timeout
      const timeout = setTimeout(() => {
        if (!resolved) {
          client.end()
          resolveOnce({
            success: false,
            message: 'Connection test timed out',
            duration: Date.now() - startTime,
            error: 'Connection timeout after 15 seconds'
          })
        }
      }, 15000)

      // Connection configuration
      const connectionConfig = {
        host: config.host,
        port: config.port || 22,
        username: config.user,
        readyTimeout: 15000,
        keepaliveInterval: 0
      }

      // Setup authentication and connection
      const connectedRef = { value: false }
      const setupConnectionAsync = async (): Promise<void> => {
        try {
          await this.setupAuthentication(config, connectionConfig)

          // Setup connection handlers
          this.setupConnectionHandlers(
            client,
            timeout,
            connectedRef,
            resolved,
            resolveOnce,
            startTime
          )

          // Connect
          client.connect(connectionConfig)
        } catch (error: unknown) {
          clearTimeout(timeout)
          const errorMessage =
            error instanceof Error ? error.message : 'Unknown authentication error'
          resolveOnce({
            success: false,
            message: `Authentication setup failed: ${errorMessage}`,
            duration: Date.now() - startTime,
            error: errorMessage
          })
        }
      }

      setupConnectionAsync()
    })
  }

  /**
   * Build SSH command with arguments
   */
  private async buildSSHCommand(
    config: ResolvedSSHConfig
  ): Promise<{ command: string; args: string[] }> {
    const args: string[] = []

    // Add port if specified
    if (config.port && config.port !== 22) {
      args.push('-p', config.port.toString())
    }

    // Add SSH key if specified
    if (config.keyPath) {
      // Validate key file exists
      try {
        await fs.access(config.keyPath)
        args.push('-i', config.keyPath)
      } catch {
        throw new Error(`SSH key file not found: ${config.keyPath}`)
      }
    }

    // Add proxy configuration if specified
    if (config.proxy) {
      const proxyArgs = this.buildProxyArgs(config.proxy)
      args.push(...proxyArgs)
    }

    // Add common SSH options
    args.push(
      '-o',
      'StrictHostKeyChecking=no',
      '-o',
      'UserKnownHostsFile=/dev/null',
      '-o',
      'ServerAliveInterval=60',
      '-o',
      'ServerAliveCountMax=3'
    )

    // Add password authentication if no key is provided
    if (!config.keyPath && config.password) {
      // Note: For password authentication, we would typically use sshpass or expect
      // For now, we'll rely on interactive password prompt
      args.push('-o', 'PreferredAuthentications=password')
    }

    // Add user and host
    args.push(`${config.user}@${config.host}`)

    return {
      command: 'ssh',
      args
    }
  }

  /**
   * Execute initial commands after SSH connection is established
   */
  private executeInitialCommands(sshProcess: ChildProcess, commands: string[]): void {
    // Wait a bit for the connection to stabilize
    setTimeout(() => {
      for (const command of commands) {
        if (sshProcess.stdin && !sshProcess.killed) {
          sshProcess.stdin.write(`${command}\n`)
        }
      }
    }, 1000)
  }

  /**
   * Build proxy arguments for SSH command
   */
  private buildProxyArgs(proxy: SSHProxy): string[] {
    const args: string[] = []

    switch (proxy.type) {
      case 'http':
        return this.buildHTTPProxyArgs(proxy)

      case 'socks4':
        args.push('-o', `ProxyCommand=connect -4 -S ${proxy.host}:${proxy.port} %h %p`)
        break

      case 'socks5':
        return this.buildSOCKS5ProxyArgs(proxy)

      case 'jump':
        return this.buildJumpHostArgs(proxy)

      default:
        throw new Error(`Unsupported proxy type: ${proxy.type}`)
    }

    return args
  }

  /**
   * Build HTTP proxy arguments
   */
  private buildHTTPProxyArgs(proxy: SSHProxy): string[] {
    const args: string[] = []
    const proxyAuth = proxy.username && proxy.password ? `${proxy.username}:${proxy.password}@` : ''

    args.push('-o', `ProxyCommand=connect -H ${proxyAuth}${proxy.host}:${proxy.port} %h %p`)
    return args
  }

  /**
   * Build SOCKS5 proxy arguments
   */
  private buildSOCKS5ProxyArgs(proxy: SSHProxy): string[] {
    const args: string[] = []
    const proxyAuth = proxy.username && proxy.password ? `${proxy.username}:${proxy.password}@` : ''

    args.push('-o', `ProxyCommand=connect -5 -S ${proxyAuth}${proxy.host}:${proxy.port} %h %p`)
    return args
  }

  /**
   * Build jump host arguments
   */
  private buildJumpHostArgs(proxy: SSHProxy): string[] {
    const args: string[] = []

    if (!proxy.jumpHost || !proxy.jumpUser) {
      throw new Error('Jump host and user are required for jump proxy')
    }

    let jumpCommand = `${proxy.jumpUser}@${proxy.jumpHost}`
    if (proxy.jumpPort && proxy.jumpPort !== 22) {
      jumpCommand += `:${proxy.jumpPort}`
    }

    args.push('-J', jumpCommand)

    // Add jump host key if specified
    if (proxy.jumpKeyPath) {
      args.push('-o', `ProxyCommand=ssh -i ${proxy.jumpKeyPath} -W %h:%p ${jumpCommand}`)
    }

    return args
  }

  /**
   * Setup authentication for SSH2 connection
   */
  private async setupAuthentication(
    config: ResolvedSSHConfig,
    connectionConfig: Record<string, unknown>
  ): Promise<void> {
    if (config.keyPath) {
      // SSH Key authentication
      try {
        const privateKey = await fs.readFile(config.keyPath)
        connectionConfig.privateKey = privateKey
        if (config.password) {
          connectionConfig.passphrase = config.password
        }
      } catch (error) {
        throw new Error(
          `Failed to read SSH key: ${error instanceof Error ? error.message : 'Unknown error'}`
        )
      }
    } else if (config.password) {
      // Password authentication
      connectionConfig.password = config.password
    } else {
      throw new Error('No authentication method provided (password or SSH key)')
    }
  }

  /**
   * Setup connection handlers for SSH2 client
   */
  private setupConnectionHandlers(
    client: Client,
    timeout: NodeJS.Timeout,
    connectedRef: { value: boolean },
    resolved: boolean,
    resolveOnce: (result: {
      success: boolean
      message: string
      duration?: number
      error?: string
    }) => void,
    startTime: number
  ): void {
    client.on('ready', () => {
      connectedRef.value = true
      clearTimeout(timeout)

      // Test with a simple command
      this.executeTestCommand(client, resolveOnce, startTime)
    })

    client.on('error', (err) => {
      clearTimeout(timeout)
      if (!connectedRef.value) {
        resolveOnce({
          success: false,
          message: this.parseSSH2Error(err),
          duration: Date.now() - startTime,
          error: err.message
        })
      }
    })

    client.on('end', () => {
      clearTimeout(timeout)
      if (!connectedRef.value && !resolved) {
        resolveOnce({
          success: false,
          message: 'Connection ended unexpectedly',
          duration: Date.now() - startTime,
          error: 'Connection ended'
        })
      }
    })
  }

  /**
   * Execute test command on SSH connection
   */
  private executeTestCommand(
    client: Client,
    resolveOnce: (result: {
      success: boolean
      message: string
      duration?: number
      error?: string
    }) => void,
    startTime: number
  ): void {
    client.exec('echo "test"', (err, stream) => {
      if (err) {
        client.end()
        resolveOnce({
          success: false,
          message: `Command execution failed: ${err.message}`,
          duration: Date.now() - startTime,
          error: err.message
        })
        return
      }

      this.setupStreamHandlers(stream, client, resolveOnce, startTime)
    })
  }

  /**
   * Setup stream handlers for command execution
   */
  private setupStreamHandlers(
    stream: NodeJS.ReadableStream & NodeJS.WritableStream & { stderr: NodeJS.ReadableStream },
    client: Client,
    resolveOnce: (result: {
      success: boolean
      message: string
      duration?: number
      error?: string
    }) => void,
    startTime: number
  ): void {
    stream.on('close', () => {
      client.end()
      resolveOnce({
        success: true,
        message: 'Connection and command execution successful',
        duration: Date.now() - startTime
      })
    })

    stream.on('data', () => {
      // Command output - we don't need to process it
    })

    stream.stderr.on('data', (data: Buffer) => {
      client.end()
      resolveOnce({
        success: false,
        message: `Command failed: ${data.toString()}`,
        duration: Date.now() - startTime,
        error: data.toString()
      })
    })
  }

  /**
   * Parse SSH2 error messages to provide user-friendly feedback
   */
  private parseSSH2Error(error: Error): string {
    const message = error.message.toLowerCase()

    // Connection refused
    if (message.includes('econnrefused')) {
      return 'Connection refused - The server is not accepting connections on this port'
    }

    // Host not found
    if (message.includes('enotfound') || message.includes('getaddrinfo')) {
      return 'Host not found - Please check the hostname or IP address'
    }

    // Timeout
    if (message.includes('timeout')) {
      return 'Connection timed out - The server is not responding'
    }

    // Authentication failed
    if (message.includes('authentication') || message.includes('auth')) {
      return 'Authentication failed - Please check your credentials'
    }

    // Key-related errors
    if (message.includes('key') && message.includes('format')) {
      return 'SSH key has invalid format - Please check the key file'
    }

    // Network unreachable
    if (message.includes('enetunreach')) {
      return 'Network unreachable - Please check your network connection'
    }

    // Generic fallback
    return `Connection failed: ${error.message}`
  }
}
