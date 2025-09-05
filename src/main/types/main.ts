/**
 * Window configuration interface
 */
export interface WindowConfig {
  width?: number
  height?: number
  show?: boolean
  autoHideMenuBar?: boolean
  frame?: boolean
}

/**
 * Context menu item interface
 */
export interface MenuItem {
  label: string
  enabled: boolean
  action?: () => void
}

/**
 * Network interface information
 */
export interface NetworkInterface {
  name: string
  address: string
  netmask: string
  mac: string
  isConnected?: boolean
}

/**
 * Network status information
 */
export interface NetworkStatus {
  isConnected: boolean
  primaryInterface: NetworkInterface | null
  interfaces: NetworkInterface[]
}
