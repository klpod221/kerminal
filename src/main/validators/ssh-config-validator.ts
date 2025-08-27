/**
 * SSH Configuration validator implementation
 */
import { ResolvedSSHConfig, SSHProfile, SSHGroup } from '../types/ssh'
import { ISSHConfigValidator } from '../interfaces/ssh.interface'
import { IValidationResult } from '../interfaces/application.interface'

export class SSHConfigValidator implements ISSHConfigValidator {
  validate(config: ResolvedSSHConfig): IValidationResult {
    const errors: string[] = []
    const warnings: string[] = []

    // Required fields validation
    if (!config.host || config.host.trim() === '') {
      errors.push('Host is required')
    }

    if (!config.user || config.user.trim() === '') {
      errors.push('User is required')
    }

    // Port validation
    if (config.port && (config.port < 1 || config.port > 65535)) {
      errors.push('Port must be between 1 and 65535')
    }

    // Authentication validation
    if (!config.keyPath && !config.password) {
      errors.push('Either SSH key path or password is required')
    }

    // Security warnings
    if (config.password && !config.keyPath) {
      warnings.push('Using password authentication is less secure than key-based authentication')
    }

    if (config.port === 22) {
      warnings.push('Consider using a non-standard port for better security')
    }

    return {
      valid: errors.length === 0,
      errors,
      warnings
    }
  }

  validateProfile(profile: SSHProfile): IValidationResult {
    const errors: string[] = []
    const warnings: string[] = []

    // Required fields
    if (!profile.name || profile.name.trim() === '') {
      errors.push('Profile name is required')
    }

    if (!profile.host || profile.host.trim() === '') {
      errors.push('Host is required')
    }

    if (!profile.user || profile.user.trim() === '') {
      errors.push('User is required')
    }

    // Optional validations
    if (profile.port && (profile.port < 1 || profile.port > 65535)) {
      errors.push('Port must be between 1 and 65535')
    }

    if (profile.name && profile.name.length > 100) {
      warnings.push('Profile name is quite long, consider shortening it')
    }

    return {
      valid: errors.length === 0,
      errors,
      warnings
    }
  }

  validateGroup(group: SSHGroup): IValidationResult {
    const errors: string[] = []
    const warnings: string[] = []

    // Required fields
    if (!group.name || group.name.trim() === '') {
      errors.push('Group name is required')
    }

    // Optional validations
    if (group.defaultPort && (group.defaultPort < 1 || group.defaultPort > 65535)) {
      errors.push('Default port must be between 1 and 65535')
    }

    if (group.name && group.name.length > 100) {
      warnings.push('Group name is quite long, consider shortening it')
    }

    if (!group.defaultUser && !group.defaultHost) {
      warnings.push('Consider setting default user or host to speed up profile creation')
    }

    return {
      valid: errors.length === 0,
      errors,
      warnings
    }
  }
}
