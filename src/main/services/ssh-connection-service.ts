import { ChildProcess } from 'child_process'
import * as fs from 'fs/promises'
import { Client } from 'ssh2'
import { ResolvedSSHConfig } from '../types/ssh'

/**
 * SSH Connection Manager
 * Handles creating and managing SSH connections for terminals
 */
export class SSHConnectionService {
  private readonly activeConnections = new Map<string, ChildProcess>()

  constructor() {
    // Constructor is empty for now
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
      // Command output - don't need to process it
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
