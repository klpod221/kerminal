import type { SSHGroup, SSHProfile, SSHProfileWithConfig, SSHTunnelWithProfile } from './ssh'

/**
 * Props interface for SSHProfileModal component
 */
export interface SSHProfileModalProps {
  visible?: boolean
  profile?: SSHProfileWithConfig | null
  groups?: SSHGroup[]
  preselectedGroup?: SSHGroup | null
}

/**
 * SSHProfileModal component emits
 */
export interface SSHProfileModalEmits {
  close: []
  save: [profile: SSHProfile]
}

/**
 * Props interface for SSHGroupModal component
 */
export interface SSHGroupModalProps {
  show: boolean
  group?: SSHGroup | null
}

/**
 * SSHGroupModal component emits
 */
export interface SSHGroupModalEmits {
  close: []
  save: [group: SSHGroup]
}

/**
 * Props interface for SSHTunnelModal component
 */
export interface SSHTunnelModalProps {
  visible: boolean
  tunnel?: SSHTunnelWithProfile | null
  sshProfiles: SSHProfile[]
}

/**
 * SSHTunnelModal component emits
 */
export interface SSHTunnelModalEmits {
  close: []
  save: [tunnel: SSHTunnelWithProfile]
}

/**
 * Props interface for SyncSettingsModal component
 */
export interface SyncSettingsModalProps {
  visible: boolean
}

/**
 * SyncSettingsModal component emits
 */
export interface SyncSettingsModalEmits {
  close: []
}

/**
 * Props interface for SavedCommandModal component
 */
export interface SavedCommandModalProps {
  visible: boolean
  command?: import('./ssh').SavedCommand | null
}

/**
 * SavedCommandModal component emits
 */
export interface SavedCommandModalEmits {
  close: []
  save: [command: import('./ssh').SavedCommand]
}
