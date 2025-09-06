import * as os from 'os'
import { spawn } from 'child_process'
import type { NetworkInterface, NetworkStatus } from '../types/main'
import { ConsoleLogger } from '../utils/logger'

/**
 * Provides system information and utilities.
 */
export class SystemInfoService {
  private static readonly logger = new ConsoleLogger('SystemInfoService')
  /**
   * Executes a command and returns its output.
   * @param command - Command to execute.
   * @param args - Command arguments.
   * @returns Promise resolving to command output.
   */
  private static executeCommand(command: string, args: string[] = []): Promise<string> {
    return new Promise((resolve) => {
      const process = spawn(command, args)
      let output = ''

      process.stdout.on('data', (data) => {
        output += data.toString()
      })

      process.on('close', () => {
        resolve(output)
      })

      process.on('error', () => {
        resolve('')
      })
    })
  }

  /**
   * Checks if there's an active internet connection.
   * @returns Promise resolving to boolean indicating internet connectivity.
   */
  private static async checkInternetConnectivity(): Promise<boolean> {
    try {
      // Try to ping a reliable DNS server (Google's 8.8.8.8)
      const result = await this.executeCommand('ping', ['-c', '1', '-W', '3', '8.8.8.8'])
      return result.includes('1 received') || result.includes('1 packets received')
    } catch {
      return false
    }
  }

  /**
   * Filters out virtual and Docker network interfaces.
   * @param interfaceName - Name of the network interface.
   * @returns Boolean indicating if interface should be included.
   */
  private static isPhysicalInterface(interfaceName: string): boolean {
    const virtualPatterns = [
      /^docker/i,
      /^br-/i,
      /^veth/i,
      /^lo$/i,
      /^virbr/i,
      /^vmnet/i,
      /^vboxnet/i,
      /^tun/i,
      /^tap/i
    ]

    return !virtualPatterns.some((pattern) => pattern.test(interfaceName))
  }

  /**
   * Gets the default route interface on Linux.
   * @returns Promise resolving to the default interface name or null.
   */
  private static async getDefaultInterface(): Promise<string | null> {
    try {
      const result = await this.executeCommand('ip', ['route', 'show', 'default'])
      const regex = /dev\s+(\w+)/
      const match = regex.exec(result)
      return match ? match[1] : null
    } catch {
      return null
    }
  }

  /**
   * Gets Linux-specific system information.
   * @returns Promise resolving to Linux system information.
   */
  private static async getLinuxSystemInfo(): Promise<Record<string, string>> {
    if (os.platform() !== 'linux') {
      return {}
    }

    const [osRelease, cpuInfo, memInfo, gpuInfo, resolution] = await Promise.all([
      this.executeCommand('cat', ['/etc/os-release']),
      this.executeCommand('cat', ['/proc/cpuinfo']),
      this.executeCommand('cat', ['/proc/meminfo']),
      this.executeCommand('lspci', ['-v']),
      this.executeCommand('xrandr', ['--current'])
    ])

    return {
      osRelease,
      cpuInfo,
      memInfo,
      gpuInfo,
      resolution
    }
  }

  /**
   * Gets comprehensive system information.
   * @returns Promise resolving to system information or null on error.
   */
  static async getSystemInfo(): Promise<Record<string, unknown> | null> {
    try {
      // Get basic OS information
      const osInfo = {
        platform: os.platform(),
        release: os.release(),
        arch: os.arch(),
        hostname: os.hostname(),
        uptime: os.uptime(),
        totalMemory: os.totalmem(),
        freeMemory: os.freemem(),
        loadAverage: os.loadavg(),
        cpus: os.cpus()
      }

      // Get additional Linux-specific information
      const linuxInfo = await this.getLinuxSystemInfo()

      return {
        ...osInfo,
        ...linuxInfo
      }
    } catch (error) {
      this.logger.error('Error getting system info:', error as Error)
      return null
    }
  }

  /**
   * Gets network interface information.
   * @returns Promise resolving to array of network interfaces.
   */
  static async getNetworkInfo(): Promise<NetworkInterface[]> {
    try {
      const networkInterfaces = os.networkInterfaces()
      const interfaces: NetworkInterface[] = []

      for (const [name, nets] of Object.entries(networkInterfaces)) {
        if (nets && this.isPhysicalInterface(name)) {
          for (const net of nets) {
            if (net.family === 'IPv4' && !net.internal) {
              interfaces.push({
                name,
                address: net.address,
                netmask: net.netmask,
                mac: net.mac
              })
            }
          }
        }
      }

      return interfaces
    } catch (error) {
      this.logger.error('Error getting network info:', error as Error)
      return []
    }
  }

  /**
   * Gets comprehensive network status including connectivity check.
   * @returns Promise resolving to network status information.
   */
  static async getNetworkStatus(): Promise<NetworkStatus> {
    try {
      const [interfaces, isConnected, defaultInterface] = await Promise.all([
        this.getNetworkInfo(),
        this.checkInternetConnectivity(),
        this.getDefaultInterface()
      ])

      // Find the primary interface (default route interface)
      let primaryInterface = interfaces.find((iface) => iface.name === defaultInterface) || null

      // If no default interface found, use the first available interface
      if (!primaryInterface && interfaces.length > 0) {
        primaryInterface = interfaces[0]
      }

      // Mark the primary interface as connected if internet is available
      if (primaryInterface && isConnected) {
        primaryInterface.isConnected = true
      }

      return {
        isConnected,
        primaryInterface,
        interfaces
      }
    } catch (error) {
      this.logger.error('Error getting network status:', error as Error)
      return {
        isConnected: false,
        primaryInterface: null,
        interfaces: []
      }
    }
  }
}
