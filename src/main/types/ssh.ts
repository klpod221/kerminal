/**
 * SSH Profile and Group related types and interfaces
 */

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
 * Interface for resolved SSH connection configuration
 * This combines profile data with group defaults
 */
export interface ResolvedSSHConfig {
  host: string
  port: number
  user: string
  keyPath?: string
  password?: string
  commands?: string[]
}

/**
 * Interface for SSH connection options
 */
export interface SSHConnectionOptions {
  profileId: string
  terminalId: string
  onConnect?: () => void
  onDisconnect?: () => void
  onError?: (error: Error) => void
}

/**
 * Interface for SSH Profile with resolved configuration
 */
export interface SSHProfileWithConfig extends SSHProfile {
  resolvedConfig: ResolvedSSHConfig
  group?: SSHGroup
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
 * Interface for command execution options
 */
export interface CommandExecutionOptions {
  terminalId: string
  command: string
  addToHistory?: boolean
}
