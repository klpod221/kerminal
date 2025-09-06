/**
 * SSH Profile and Group related types for renderer process
 * @deprecated This file will be removed. Use shared types from '@shared/types/ssh' instead
 */

// Re-export all types from shared types
export * from '@shared/types/ssh'

// Import types for use in extended interfaces
import type {
  SSHGroup,
  SSHProfile,
  SSHTunnel,
  SSHProfileWithConfig,
  SSHTunnelWithProfile
} from '@shared/types/ssh'

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
