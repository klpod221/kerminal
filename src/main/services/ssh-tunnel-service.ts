import { spawn, ChildProcess } from 'child_process'
import { SSHTunnel, SSHTunnelOptions, SSHTunnelWithProfile } from '../types/ssh'
import { SSHTunnelStorage } from '../storage/ssh-tunnel-storage'
import { SSHProfileService } from './ssh-profile-service'

/**
 * SSH Tunnel Manager Service
 * Manages SSH tunnels for port forwarding
 */
export class SSHTunnelService {
  private readonly tunnelStorage: SSHTunnelStorage
  private readonly profileService: SSHProfileService
  private readonly activeTunnels = new Map<string, ChildProcess>()
  private readonly reconnectTimers = new Map<string, NodeJS.Timeout>()
  private readonly manuallyStoppedTunnels = new Set<string>()

  constructor() {
    this.tunnelStorage = new SSHTunnelStorage()
    this.profileService = new SSHProfileService()
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
    // Validate port availability
    const isPortInUse = await this.tunnelStorage.isPortInUse(tunnelData.localPort)
    if (isPortInUse) {
      throw new Error(`Port ${tunnelData.localPort} is already in use by another tunnel`)
    }

    // Validate profile exists
    const profile = await this.profileService.getProfileById(tunnelData.profileId)
    if (!profile) {
      throw new Error(`SSH Profile with ID ${tunnelData.profileId} not found`)
    }

    // Create tunnel with initial status
    const tunnel = await this.tunnelStorage.create({
      ...tunnelData,
      status: 'stopped'
    })

    console.log(`Created SSH tunnel: ${tunnel.name} (${tunnel.id})`)
    return tunnel
  }

  /**
   * Update an existing tunnel
   */
  async updateTunnel(
    id: string,
    updates: Partial<Omit<SSHTunnel, 'id' | 'created'>>
  ): Promise<SSHTunnel | null> {
    // If updating localPort, check if it's available
    if (updates.localPort) {
      const isPortInUse = await this.tunnelStorage.isPortInUse(updates.localPort, id)
      if (isPortInUse) {
        throw new Error(`Port ${updates.localPort} is already in use by another tunnel`)
      }
    }

    const tunnel = await this.tunnelStorage.update(id, updates)
    if (tunnel) {
      console.log(`Updated SSH tunnel: ${tunnel.name} (${tunnel.id})`)
    }
    return tunnel
  }

  /**
   * Delete a tunnel
   */
  async deleteTunnel(id: string): Promise<boolean> {
    // Stop tunnel if running
    await this.stopTunnel(id)

    const success = await this.tunnelStorage.delete(id)
    if (success) {
      console.log(`Deleted SSH tunnel: ${id}`)
    }
    return success
  }

  /**
   * Start a tunnel
   */
  async startTunnel(id: string, options?: SSHTunnelOptions): Promise<void> {
    const tunnelWithProfile = await this.getTunnelById(id)
    if (!tunnelWithProfile) {
      throw new Error(`Tunnel with ID ${id} not found`)
    }

    const tunnel = tunnelWithProfile

    // Check if already running
    if (this.activeTunnels.has(id)) {
      throw new Error(`Tunnel ${tunnel.name} is already running`)
    }

    // Remove from manually stopped list if starting manually
    this.manuallyStoppedTunnels.delete(id)

    try {
      // Update status to starting
      await this.tunnelStorage.updateStatus(id, 'starting')

      // Build SSH command for tunnel
      const sshCommand = await this.buildTunnelCommand(tunnel)

      // Spawn SSH process
      const sshProcess = spawn(sshCommand.command, sshCommand.args, {
        stdio: ['pipe', 'pipe', 'pipe'],
        env: {
          ...process.env,
          TERM: 'xterm-256color'
        }
      })

      // Store active tunnel
      this.activeTunnels.set(id, sshProcess)

      // Handle process events
      sshProcess.on('spawn', () => {
        console.log(`SSH tunnel started: ${tunnel.name} (${id})`)
        this.tunnelStorage.updateStatus(id, 'running')
        options?.onConnect?.()
      })

      sshProcess.on('error', (error) => {
        console.error(`SSH tunnel error: ${tunnel.name} (${id})`, error)
        this.tunnelStorage.updateStatus(id, 'error', error.message)
        this.activeTunnels.delete(id)
        options?.onError?.(error)
      })

      sshProcess.on('exit', (code, signal) => {
        console.warn(`SSH tunnel exited: ${tunnel.name} (${id}) - Code: ${code}, Signal: ${signal}`)
        this.activeTunnels.delete(id)

        // Only auto-reconnect if:
        // 1. Tunnel has autoStart enabled
        // 2. Exit code indicates an error (not manual termination)
        // 3. Not manually stopped by user
        // 4. Not terminated by SIGTERM (which is used for manual stop)
        if (
          tunnel.autoStart &&
          code !== 0 &&
          signal !== 'SIGTERM' &&
          !this.manuallyStoppedTunnels.has(id)
        ) {
          // Auto-reconnect if enabled and not manually stopped
          this.scheduleReconnect(id, options)
        } else {
          this.tunnelStorage.updateStatus(id, 'stopped')
          options?.onDisconnect?.()
        }
      })

      // Wait a bit to ensure tunnel is established
      await new Promise((resolve) => setTimeout(resolve, 2000))
    } catch (error) {
      this.activeTunnels.delete(id)
      await this.tunnelStorage.updateStatus(id, 'error', (error as Error).message)
      throw error
    }
  }

  /**
   * Stop a tunnel
   */
  async stopTunnel(id: string): Promise<void> {
    const tunnel = await this.tunnelStorage.getById(id)
    if (!tunnel) {
      throw new Error(`Tunnel with ID ${id} not found`)
    }

    // Mark as manually stopped to prevent auto-reconnect
    this.manuallyStoppedTunnels.add(id)

    // Clear reconnect timer if exists
    const reconnectTimer = this.reconnectTimers.get(id)
    if (reconnectTimer) {
      clearTimeout(reconnectTimer)
      this.reconnectTimers.delete(id)
    }

    // Stop SSH process
    const sshProcess = this.activeTunnels.get(id)
    if (sshProcess) {
      // Use SIGTERM to properly terminate the SSH process
      sshProcess.kill('SIGTERM')

      // Wait a bit for graceful termination
      await new Promise((resolve) => setTimeout(resolve, 1000))

      // If still running, force kill
      if (!sshProcess.killed) {
        sshProcess.kill('SIGKILL')
      }

      this.activeTunnels.delete(id)
    }

    // Update status
    await this.tunnelStorage.updateStatus(id, 'stopped')
    console.log(`SSH tunnel stopped: ${tunnel.name} (${id})`)
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
        console.error(`Failed to start auto-start tunnel: ${tunnel.name}`, error)
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
        console.error(`Failed to stop tunnel: ${tunnelId}`, error)
      }
    }

    // Clear manually stopped list when stopping all
    this.manuallyStoppedTunnels.clear()
  }

  /**
   * Get tunnel status
   */
  isTunnelRunning(id: string): boolean {
    return this.activeTunnels.has(id)
  }

  /**
   * Check if tunnel is actually running by verifying the process
   */
  async getTunnelRealStatus(id: string): Promise<string> {
    const tunnel = await this.tunnelStorage.getById(id)
    if (!tunnel) return 'stopped'

    // Check if we have the process in our map
    const process = this.activeTunnels.get(id)
    if (!process) {
      // Update status to stopped if we don't track it
      if (tunnel.status !== 'stopped') {
        await this.tunnelStorage.updateStatus(id, 'stopped')
      }
      return 'stopped'
    }

    // Check if process is still alive
    try {
      // Sending signal 0 checks if process exists without killing it
      process.kill(0)
      return tunnel.status
    } catch {
      // Process is dead, clean up
      this.activeTunnels.delete(id)
      await this.tunnelStorage.updateStatus(id, 'stopped')
      return 'stopped'
    }
  }

  /**
   * Get count of running tunnels
   */
  getRunningTunnelCount(): number {
    return this.activeTunnels.size
  }

  /**
   * Kill any orphaned SSH tunnel processes
   * This method should be called on app startup to clean up any leftover processes
   */
  async killOrphanedTunnelProcesses(): Promise<void> {
    try {
      const { spawn } = await import('child_process')

      // Find SSH processes that look like tunnel processes
      const psProcess = spawn('ps', ['aux'], { stdio: ['pipe', 'pipe', 'pipe'] })

      let output = ''
      psProcess.stdout.on('data', (data) => {
        output += data.toString()
      })

      await new Promise((resolve, reject) => {
        psProcess.on('close', (code) => {
          if (code === 0) {
            resolve(void 0)
          } else {
            reject(new Error(`ps command failed with code ${code}`))
          }
        })
      })

      // Look for SSH tunnel processes (containing -L, -R, or -D flags with -N)
      const lines = output.split('\n')
      const tunnelProcesses: string[] = []

      for (const line of lines) {
        if (
          line.includes('ssh') &&
          line.includes('-N') &&
          (line.includes('-L') || line.includes('-R') || line.includes('-D'))
        ) {
          const parts = line.trim().split(/\s+/)
          const pid = parts[1]
          if (pid && !isNaN(Number(pid))) {
            tunnelProcesses.push(pid)
          }
        }
      }

      // Kill orphaned processes
      for (const pid of tunnelProcesses) {
        try {
          process.kill(Number(pid), 'SIGTERM')
          console.log(`Killed orphaned SSH tunnel process: ${pid}`)
        } catch (error) {
          // Process might already be dead or not owned by us
          console.log(`Could not kill process ${pid}:`, error)
        }
      }

      if (tunnelProcesses.length > 0) {
        console.log(`Cleaned up ${tunnelProcesses.length} orphaned SSH tunnel processes`)
      }
    } catch (error) {
      console.error('Failed to clean up orphaned processes:', error)
    }
  }

  /**
   * Build SSH command for tunnel
   */
  private async buildTunnelCommand(
    tunnel: SSHTunnel
  ): Promise<{ command: string; args: string[] }> {
    const args: string[] = []

    // Add tunnel-specific arguments
    switch (tunnel.type) {
      case 'local': {
        if (!tunnel.remoteHost || !tunnel.remotePort) {
          throw new Error('Local tunnel requires remote host and port')
        }
        args.push('-L', `${tunnel.localPort}:${tunnel.remoteHost}:${tunnel.remotePort}`)
        break
      }

      case 'remote': {
        if (!tunnel.remoteHost || !tunnel.remotePort) {
          throw new Error('Remote tunnel requires remote host and port')
        }
        args.push('-R', `${tunnel.remotePort}:${tunnel.remoteHost}:${tunnel.localPort}`)
        break
      }

      case 'dynamic': {
        args.push('-D', tunnel.localPort.toString())
        break
      }

      default:
        throw new Error(`Unsupported tunnel type: ${tunnel.type}`)
    }

    // Add SSH options
    args.push(
      '-N',
      '-T',
      '-o',
      'StrictHostKeyChecking=no',
      '-o',
      'UserKnownHostsFile=/dev/null',
      '-o',
      'ServerAliveInterval=60',
      '-o',
      'ServerAliveCountMax=3',
      '-o',
      'ExitOnForwardFailure=yes'
    )

    // Get profile for connection details
    const profileWithConfig = await this.profileService.getProfileById(tunnel.profileId)
    if (!profileWithConfig) {
      throw new Error(`Profile ${tunnel.profileId} not found`)
    }

    const config = profileWithConfig.resolvedConfig

    // Add port if specified
    if (config.port && config.port !== 22) {
      args.push('-p', config.port.toString())
    }

    // Add SSH key if specified
    if (config.keyPath) {
      args.push('-i', config.keyPath)
    }

    // Add user and host
    args.push(`${config.user}@${config.host}`)

    return {
      command: 'ssh',
      args
    }
  }

  /**
   * Schedule tunnel reconnection
   */
  private scheduleReconnect(id: string, options?: SSHTunnelOptions): void {
    // Clear existing timer
    const existingTimer = this.reconnectTimers.get(id)
    if (existingTimer) {
      clearTimeout(existingTimer)
    }

    // Set reconnecting status
    this.tunnelStorage.updateStatus(id, 'reconnecting')

    // Schedule reconnection (5 seconds delay)
    const timer = setTimeout(async () => {
      this.reconnectTimers.delete(id)

      try {
        await this.startTunnel(id, options)
        options?.onReconnect?.()
      } catch (error) {
        console.error(`Failed to reconnect tunnel: ${id}`, error)
        await this.tunnelStorage.updateStatus(id, 'error', (error as Error).message)
      }
    }, 5000)

    this.reconnectTimers.set(id, timer)
  }
}
