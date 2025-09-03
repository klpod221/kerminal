import { spawn, ChildProcess } from 'child_process'
import * as fs from 'fs/promises'
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
}
