/// <reference types="vite/client" />

interface SystemInfo {
  platform: string
  release: string
  arch: string
  hostname: string
  uptime: number
  totalMemory: number
  freeMemory: number
  loadAverage: number[]
  cpus: Array<{
    model: string
    speed: number
    times: {
      user: number
      nice: number
      sys: number
      idle: number
      irq: number
    }
  }>
  osRelease?: string
  cpuInfo?: string
  memInfo?: string
  gpuInfo?: string
  resolution?: string
}

interface NetworkInterface {
  name: string
  address: string
  netmask: string
  mac: string
  isConnected?: boolean
}

interface NetworkStatus {
  isConnected: boolean
  primaryInterface: NetworkInterface | null
  interfaces: NetworkInterface[]
}

interface Window {
  api: {
    send: (channel: string, data: unknown) => void
    on: (channel: string, func: (...args: unknown[]) => void) => () => void
    invoke: (channel: string, ...args: unknown[]) => Promise<unknown>
    getSystemInfo: () => Promise<SystemInfo | null>
    getNetworkInfo: () => Promise<NetworkInterface[]>
    getNetworkStatus: () => Promise<NetworkStatus>
  }
}
