import { Client } from 'ssh2'
import * as net from 'net'
import * as s5 from 'socksv5'
import { SSHTunnel, SSHTunnelOptions, SSHTunnelWithProfile } from '../types/ssh'
import { SSHTunnelStorage } from '../storage/ssh-tunnel-storage'
import { SSHProfileService } from './ssh-profile-service'
import { ConsoleLogger } from '../utils/logger'

type ActiveServer = net.Server | s5.Server

/**
 * SSH Tunnel Manager Service
 * Manages SSH tunnels for port forwarding
 */
export class SSHTunnelService {
  private readonly tunnelStorage: SSHTunnelStorage
  private readonly profileService: SSHProfileService
  private readonly activeTunnels = new Map<string, Client>()
  private readonly activeServers = new Map<string, ActiveServer>()
  private readonly reconnectTimers = new Map<string, NodeJS.Timeout>()
  private readonly manuallyStoppedTunnels = new Set<string>()
  private readonly logger = new ConsoleLogger('SSHTunnelService')

  constructor(sshProfileService: SSHProfileService) {
    this.tunnelStorage = new SSHTunnelStorage()
    this.profileService = sshProfileService
  }

  /**
   * Get all tunnels with profile information
   */
  async getAllTunnels(): Promise<SSHTunnelWithProfile[]> {
    const tunnels = await this.tunnelStorage.getAll()
    const tunnelsWithProfile: SSHTunnelWithProfile[] = []

    for (const tunnel of tunnels) {
      const profileWithConfig = await this.profileService.getProfileById(tunnel.profileId)
      if (profileWithConfig) {
        tunnelsWithProfile.push({
          ...tunnel,
          profile: profileWithConfig
        })
      }
    }

    return tunnelsWithProfile
  }

  /**
   * Get tunnel by ID
   */
  async getTunnelById(id: string): Promise<SSHTunnelWithProfile | null> {
    const tunnel = await this.tunnelStorage.getById(id)
    if (!tunnel) return null

    const profileWithConfig = await this.profileService.getProfileById(tunnel.profileId)
    if (!profileWithConfig) return null

    return {
      ...tunnel,
      profile: profileWithConfig
    }
  }

  /**
   * Create a new tunnel
   */
  async createTunnel(
    tunnelData: Omit<
      SSHTunnel,
      'id' | 'created' | 'updated' | 'status' | 'lastStarted' | 'lastError'
    >
  ): Promise<SSHTunnel> {
    const isPortInUse = await this.tunnelStorage.isPortInUse(tunnelData.localPort)
    if (isPortInUse) {
      throw new Error(`Port ${tunnelData.localPort} is already in use by another tunnel`)
    }

    const profile = await this.profileService.getProfileById(tunnelData.profileId)
    if (!profile) {
      throw new Error(`SSH Profile with ID ${tunnelData.profileId} not found`)
    }

    return this.tunnelStorage.create({
      ...tunnelData,
      status: 'stopped'
    })
  }

  /**
   * Update an existing tunnel
   */
  async updateTunnel(
    id: string,
    updates: Partial<Omit<SSHTunnel, 'id' | 'created'>>
  ): Promise<SSHTunnel | null> {
    if (updates.localPort) {
      const isPortInUse = await this.tunnelStorage.isPortInUse(updates.localPort, id)
      if (isPortInUse) {
        throw new Error(`Port ${updates.localPort} is already in use by another tunnel`)
      }
    }

    return this.tunnelStorage.update(id, updates)
  }

  /**
   * Delete a tunnel
   */
  async deleteTunnel(id: string): Promise<boolean> {
    await this.stopTunnel(id)
    return this.tunnelStorage.delete(id)
  }

  /**
   * Start a tunnel
   */
  async startTunnel(id: string, options?: SSHTunnelOptions): Promise<void> {
    const tunnelWithProfile = await this.getTunnelById(id)
    if (!tunnelWithProfile) {
      throw new Error(`Tunnel with ID ${id} not found`)
    }
    if (this.activeTunnels.has(id)) {
      throw new Error(`Tunnel ${tunnelWithProfile.name} is already running`)
    }
    this.manuallyStoppedTunnels.delete(id)
    await this.tunnelStorage.updateStatus(id, 'starting')

    const client = new Client()
    const {
      profile: { resolvedConfig: config }
    } = tunnelWithProfile
    const connectConfig: Record<string, unknown> = {
      host: config.host,
      port: config.port || 22,
      username: config.user,
      readyTimeout: 10000,
      keepaliveInterval: 60000,
      keepaliveCountMax: 3
    }
    if (config.keyPath) {
      const fs = await import('fs')
      try {
        connectConfig.privateKey = fs.readFileSync(config.keyPath)
      } catch (error) {
        await this.tunnelStorage.updateStatus(id, 'error', `Failed to read SSH key: ${error}`)
        throw new Error(`Failed to read SSH key: ${error}`)
      }
      if (config.password) connectConfig.passphrase = config.password
    } else if (config.password) {
      connectConfig.password = config.password
    } else {
      await this.tunnelStorage.updateStatus(id, 'error', 'No authentication method provided')
      throw new Error('No authentication method provided')
    }

    client.on('ready', () => {
      this.logger.info(`SSH connection established for tunnel: ${tunnelWithProfile.name}`)
      try {
        switch (tunnelWithProfile.type) {
          case 'local':
            this.setupLocalTunnel(client, tunnelWithProfile, id, options)
            break
          case 'remote':
            this.setupRemoteTunnel(client, tunnelWithProfile, id, options)
            break
          case 'dynamic':
            this.setupDynamicTunnel(client, tunnelWithProfile, id, options)
            break
          default:
            throw new Error('Unsupported tunnel type')
        }
      } catch (err: unknown) {
        const errorObj = err instanceof Error ? err : new Error(String(err))
        this.logger.error(`Tunnel setup error:`, errorObj)
        this.tunnelStorage.updateStatus(id, 'error', errorObj.message)
        options?.onError?.(errorObj)
        client.end()
      }
    })

    client.on('error', (error) => {
      this.logger.error(`SSH tunnel error: ${tunnelWithProfile.name} (${id})`, error)
      this.activeTunnels.delete(id)
      this.closeLocalServer(id)
      this.tunnelStorage.updateStatus(id, 'error', error.message)
      options?.onError?.(error)
      if (tunnelWithProfile.autoStart && !this.manuallyStoppedTunnels.has(id)) {
        this.scheduleReconnect(id, options)
      }
    })

    client.on('close', () => {
      this.logger.warn(`SSH tunnel closed: ${tunnelWithProfile.name} (${id})`)
      this.activeTunnels.delete(id)
      this.closeLocalServer(id)
      if (tunnelWithProfile.autoStart && !this.manuallyStoppedTunnels.has(id)) {
        this.scheduleReconnect(id, options)
      } else {
        this.tunnelStorage.updateStatus(id, 'stopped')
        options?.onDisconnect?.()
      }
    })

    client.connect(connectConfig)
    this.activeTunnels.set(id, client)
  }

  /**
   * Stop a tunnel
   */
  async stopTunnel(id: string): Promise<void> {
    const tunnel = await this.tunnelStorage.getById(id)
    if (!tunnel) {
      throw new Error(`Tunnel with ID ${id} not found`)
    }
    this.manuallyStoppedTunnels.add(id)
    const reconnectTimer = this.reconnectTimers.get(id)
    if (reconnectTimer) {
      clearTimeout(reconnectTimer)
      this.reconnectTimers.delete(id)
    }

    this.closeLocalServer(id)

    const client = this.activeTunnels.get(id)
    if (client) {
      client.end()
      this.activeTunnels.delete(id)
    }
    await this.tunnelStorage.updateStatus(id, 'stopped')
  }

  /**
   * Start all auto-start tunnels
   */
  async startAutoStartTunnels(): Promise<void> {
    const autoStartTunnels = await this.tunnelStorage.getAutoStart()
    for (const tunnel of autoStartTunnels) {
      try {
        await this.startTunnel(tunnel.id)
      } catch (error) {
        this.logger.error(`Failed to start auto-start tunnel: ${tunnel.name}`, error as Error)
      }
    }
  }

  /**
   * Stop all running tunnels
   */
  async stopAllTunnels(): Promise<void> {
    const runningTunnels = Array.from(this.activeTunnels.keys())
    for (const tunnelId of runningTunnels) {
      try {
        await this.stopTunnel(tunnelId)
      } catch (error) {
        this.logger.error(`Failed to stop tunnel: ${tunnelId}`, error as Error)
      }
    }
    this.manuallyStoppedTunnels.clear()
  }

  /**
   * Get tunnel status
   */
  isTunnelRunning(id: string): boolean {
    return this.activeTunnels.has(id)
  }

  // --- PRIVATE HELPER METHODS ---

  /**
   * Sets up a Local Port Forwarding tunnel (ssh -L)
   */
  private setupLocalTunnel(
    client: Client,
    tunnel: SSHTunnelWithProfile,
    id: string,
    options?: SSHTunnelOptions
  ): void {
    if (!tunnel.remoteHost || !tunnel.remotePort) {
      throw new Error('Local tunnel requires remote host and port')
    }
    const server = net.createServer((socket) => {
      this.logger.info(`Connection received on local port ${tunnel.localPort}`)
      client.forwardOut(
        socket.remoteAddress || '127.0.0.1',
        socket.remotePort || 0,
        tunnel.remoteHost!,
        tunnel.remotePort!,
        (err, stream) => {
          if (err) {
            this.logger.error('forwardOut error:', err)
            socket.end()
            return
          }
          socket.pipe(stream)
          stream.pipe(socket)
          socket.on('close', () => stream.close())
          stream.on('close', () => socket.end())
        }
      )
    })

    server.on('error', (err: NodeJS.ErrnoException) => {
      this.logger.error(`Local server error for tunnel ${id}:`, err)
      this.tunnelStorage.updateStatus(id, 'error', err.message)
      options?.onError?.(err)
      client.end()
    })

    server.listen(tunnel.localPort, '127.0.0.1', () => {
      this.logger.info(`Local tunnel server listening on 127.0.0.1:${tunnel.localPort}`)
      this.activeServers.set(id, server)
      this.tunnelStorage.updateStatus(id, 'running')
      options?.onConnect?.()
    })
  }

  /**
   * Sets up a Remote Port Forwarding tunnel (ssh -R)
   */
  private setupRemoteTunnel(
    client: Client,
    tunnel: SSHTunnelWithProfile,
    id: string,
    options?: SSHTunnelOptions
  ): void {
    if (!tunnel.remoteHost || !tunnel.remotePort) {
      throw new Error('Remote tunnel requires a local destination host and port')
    }
    client.forwardIn('0.0.0.0', tunnel.localPort, (err) => {
      if (err) {
        this.logger.error(`forwardIn error:`, err)
        this.tunnelStorage.updateStatus(id, 'error', err.message)
        options?.onError?.(err)
        client.end()
        return
      }
      this.logger.info(`Remote forwarding enabled on remote port ${tunnel.localPort}`)
      this.tunnelStorage.updateStatus(id, 'running')
      options?.onConnect?.()
    })

    client.on('tcp connection', (info, accept) => {
      this.logger.info('Incoming remote connection:', info)
      const stream = accept()
      const localSocket = net.connect(tunnel.remotePort!, tunnel.remoteHost!, () => {
        stream.pipe(localSocket)
        localSocket.pipe(stream)
        stream.on('close', () => localSocket.end())
        localSocket.on('close', () => stream.end())
      })
      localSocket.on('error', (err) => {
        this.logger.error('Local service connection error:', err)
        stream.close()
      })
    })
  }

  /**
   * Sets up a Dynamic Port Forwarding tunnel / SOCKS5 proxy (ssh -D)
   */
  private setupDynamicTunnel(
    client: Client,
    tunnel: SSHTunnelWithProfile,
    id: string,
    options?: SSHTunnelOptions
  ): void {
    const socksServer = s5.createServer((info, accept) => {
      this.logger.info(`SOCKS request to: ${info.dstAddr}:${info.dstPort}`)
      client.forwardOut(info.srcAddr, info.srcPort, info.dstAddr, info.dstPort, (err, stream) => {
        if (err) {
          this.logger.error('SOCKS forwarding error:', err)
          return
        }
        const socket = accept(true)
        if (socket) {
          stream.pipe(socket)
          socket.pipe(stream)
          stream.on('close', () => socket.end())
          socket.on('close', () => stream.end())
        }
      })
    })

    socksServer.on('error', (err: Error) => {
      this.logger.error(`SOCKS server error for tunnel ${id}:`, err)
      this.tunnelStorage.updateStatus(id, 'error', err.message)
      options?.onError?.(err)
      client.end()
    })

    socksServer.listen(tunnel.localPort, '127.0.0.1', () => {
      this.logger.info(`SOCKS5 proxy server listening on 127.0.0.1:${tunnel.localPort}`)
      this.activeServers.set(id, socksServer)
      this.tunnelStorage.updateStatus(id, 'running')
      options?.onConnect?.()
    })
  }

  /**
   * Closes the local server associated with a tunnel
   */
  private closeLocalServer(id: string): void {
    const server = this.activeServers.get(id)
    if (server) {
      server.close()
      this.activeServers.delete(id)
      this.logger.info(`Closed local server for tunnel ${id}`)
    }
  }

  /**
   * Schedule tunnel reconnection
   */
  private scheduleReconnect(id: string, options?: SSHTunnelOptions): void {
    const existingTimer = this.reconnectTimers.get(id)
    if (existingTimer) {
      clearTimeout(existingTimer)
    }

    this.tunnelStorage.updateStatus(id, 'reconnecting')

    const timer = setTimeout(async () => {
      this.reconnectTimers.delete(id)
      try {
        await this.startTunnel(id, options)
        options?.onReconnect?.()
      } catch (error) {
        this.logger.error(`Failed to reconnect tunnel: ${id}`, error as Error)
        await this.tunnelStorage.updateStatus(id, 'error', (error as Error).message)
      }
    }, 5000)

    this.reconnectTimers.set(id, timer)
  }
}
