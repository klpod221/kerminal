/**
 * SSH Profile and Group related types for renderer process
 * These types mirror the main process types but are tailored for frontend use
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
 * Interface for SSH Tunnel with profile information
 */
export interface SSHTunnelWithProfile extends SSHTunnel {
  profile: SSHProfile
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
 * Interface for resolved SSH connection configuration
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
 * Interface for SSH Profile with resolved configuration
 */
export interface SSHProfileWithConfig extends SSHProfile {
  resolvedConfig: ResolvedSSHConfig
  group?: SSHGroup
}

/**
 * Interface for SSH Group with profiles
 */
export interface SSHGroupWithProfiles extends SSHGroup {
  profiles: SSHProfileWithConfig[]
}

/**
 * Interface for SSH connection statistics
 */
export interface SSHConnectionStats {
  totalConnections: number
  uniqueProfiles: number
  successfulConnections: number
  failedConnections: number
  averageDuration: number
}

/**
 * Interface for SSH connection test result
 */
export interface SSHConnectionTestResult {
  success: boolean
  error?: string
}

/**
 * Interface for creating new SSH Group
 */
export type CreateSSHGroupData = Omit<SSHGroup, 'id' | 'created' | 'updated'>

/**
 * Interface for updating SSH Group
 */
export type UpdateSSHGroupData = Partial<Omit<SSHGroup, 'id' | 'created'>>

/**
 * Interface for creating new SSH Profile
 */
export type CreateSSHProfileData = Omit<SSHProfile, 'id' | 'created' | 'updated'>

/**
 * Interface for updating SSH Profile
 */
export type UpdateSSHProfileData = Partial<Omit<SSHProfile, 'id' | 'created'>>

/**
 * Interface for creating new SSH Tunnel
 */
export type CreateSSHTunnelData = Omit<
  SSHTunnel,
  'id' | 'created' | 'updated' | 'status' | 'lastStarted' | 'lastError'
>

/**
 * Interface for updating SSH Tunnel
 */
export type UpdateSSHTunnelData = Partial<Omit<SSHTunnel, 'id' | 'created'>>

/**
 * SSH Tunnel form state
 */
export interface SSHTunnelFormState {
  isOpen: boolean
  mode: 'create' | 'edit'
  tunnelId?: string
  tunnel?: SSHTunnel
  preselectedProfileId?: string
}

/**
 * SSH Tunnel manager state
 */
export interface SSHTunnelManagerState {
  tunnels: SSHTunnelWithProfile[]
  loading: boolean
  error?: string
}

/**
 * SSH Drawer state
 */
export interface SSHDrawerState {
  isOpen: boolean
  searchQuery: string
  selectedGroupId?: string
  showFavoritesOnly: boolean
}

/**
 * SSH Profile form state
 */
export interface SSHProfileFormState {
  isOpen: boolean
  mode: 'create' | 'edit'
  groupId?: string
  profileId?: string
  profile?: SSHProfile
}

/**
 * SSH Group form state
 */
export interface SSHGroupFormState {
  isOpen: boolean
  mode: 'create' | 'edit'
  groupId?: string
  group?: SSHGroup
}
