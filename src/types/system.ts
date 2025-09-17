/**
 * System information types
 */

/**
 * Interface for CPU information
 */
export interface CPUInfo {
  model: string;
  speed: number;
  times: {
    user: number;
    nice: number;
    sys: number;
    idle: number;
    irq: number;
  };
}

/**
 * Interface for system information
 */
export interface SystemInfo {
  platform: string;
  release: string;
  arch: string;
  hostname: string;
  uptime: number;
  totalMemory: number;
  freeMemory: number;
  loadAverage: number[];
  cpus: CPUInfo[];
  osRelease?: string;
  cpuInfo?: string;
  memInfo?: string;
  gpuInfo?: string;
  resolution?: string;
}

/**
 * Interface for network interface information
 */
export interface NetworkInterface {
  name: string;
  address: string;
  netmask: string;
  mac: string;
  isConnected?: boolean;
}

/**
 * Interface for network status
 */
export interface NetworkStatus {
  isConnected: boolean;
  primaryInterface: NetworkInterface | null;
  interfaces: NetworkInterface[];
}
