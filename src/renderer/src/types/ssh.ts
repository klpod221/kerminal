/**
 * SSH Profile and Group related types for renderer process
 * @deprecated This file will be removed. Use shared types from '@shared/types/ssh' instead
 */

// Re-export all types from shared types
export * from '@shared/types/ssh'

// Import types for use in extended interfaces
import type { SSHGroup, SSHProfileWithConfig } from '@shared/types/ssh'

/**
 * Interface for SSH Group with profiles
 */
export interface SSHGroupWithProfiles extends SSHGroup {
  profiles: SSHProfileWithConfig[]
}
