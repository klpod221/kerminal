/**
 * Shared SSH types between main and renderer processes
 * This file contains common types used by both processes to avoid duplication
 */

/**
 * SSH Proxy configuration
 */
export interface SSHProxy {
  type: 'http' | 'socks4' | 'socks5' | 'jump'
  host: string
  port: number
  username?: string
  password?: string
  // For jump host proxy
  jumpHost?: string
  jumpPort?: number
  jumpUser?: string
  jumpKeyPath?: string
  jumpPassword?: string
}

/**
 * Interface for SSH Tunnel configuration
 */
export interface SSHTunnel {
  id: string
  name: string
  description?: string
  profileId: string
  type: 'local' | 'remote' | 'dynamic'
  localPort: number
  remoteHost?: string
  remotePort?: number
  autoStart: boolean
  status: 'stopped' | 'starting' | 'running' | 'error' | 'reconnecting'
  created: Date
  updated: Date
  lastStarted?: Date
  lastError?: string
}

/**
 * Interface for SSH Group configuration
 */
export interface SSHGroup {
  id: string
  name: string
  description?: string
  defaultUser?: string
  defaultHost?: string
  defaultPort?: number
  defaultKeyPath?: string
  defaultPassword?: string
  defaultProxy?: SSHProxy
  color?: string
  created: Date
  updated: Date
}

/**
 * Interface for SSH Profile configuration
 */
export interface SSHProfile {
  id: string
  name: string
  description?: string
  groupId?: string
  host: string
  port?: number
  user: string
  keyPath?: string
  password?: string
  proxy?: SSHProxy
  commands?: string[]
  color?: string
  favorite: boolean
  lastConnected?: Date
  created: Date
  updated: Date
}

/**
 * Interface for SSH Connection record
 */
export interface SSHConnection {
  id: string
  profileId: string
  profileName: string
  host: string
  user: string
  connectedAt: Date
  duration?: number
  status: 'connected' | 'disconnected' | 'failed'
}

/**
 * Interface for saved command
 */
export interface SavedCommand {
  id: string
  name: string
  command: string
  description?: string
  created: Date
  updated: Date
}

/**
 * Interface for SSH Profile with resolved configuration
 * This combines profile data with group defaults
 */
export interface ResolvedSSHConfig {
  host: string
  port: number
  user: string
  keyPath?: string
  password?: string
  proxy?: SSHProxy
  commands?: string[]
}

/**
 * Interface for SSH Profile with resolved configuration and group info
 */
export interface SSHProfileWithConfig extends SSHProfile {
  resolvedConfig: ResolvedSSHConfig
  group?: SSHGroup
}

/**
 * Interface for SSH Tunnel with profile information
 */
export interface SSHTunnelWithProfile extends SSHTunnel {
  profile: SSHProfileWithConfig
}

/**
 * Interface for command execution options
 */
export interface CommandExecutionOptions {
  terminalId: string
  command: string
  addToHistory?: boolean
}

/**
 * Interface for SSH Tunnel connection options
 */
export interface SSHTunnelOptions {
  tunnelId: string
  onConnect?: () => void
  onDisconnect?: () => void
  onError?: (error: Error) => void
  onReconnect?: () => void
}
