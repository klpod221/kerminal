/// <reference types="vite/client" />

import type { SystemInfo, NetworkStatus, NetworkInterface } from './src/types/system'

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
