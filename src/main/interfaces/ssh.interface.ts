/**
 * SSH-related interfaces for better separation of concerns
 */
import {
  SSHProfile,
  SSHGroup,
  SSHConnection,
  SSHProfileWithConfig,
  ResolvedSSHConfig
} from '../types/ssh'
import { IValidator } from './application.interface'

type StorageCreateData<T> = Omit<T, 'id' | 'created' | 'updated'>

export interface IStorageService<T> {
  getAll(): Promise<T[]>
  getById(id: string): Promise<T | null>
  create(data: StorageCreateData<T>): Promise<T>
  update(id: string, updates: Partial<T>): Promise<T | null>
  delete(id: string): Promise<boolean>
}

export interface ISSHProfileService {
  // Group management
  createGroup(groupData: StorageCreateData<SSHGroup>): Promise<SSHGroup>
  getGroupById(id: string): Promise<SSHGroup | null>
  createGroup(groupData: Omit<SSHGroup, 'id' | 'created' | 'updated'>): Promise<SSHGroup>
  updateGroup(id: string, updates: Partial<SSHGroup>): Promise<SSHGroup | null>
  deleteGroup(id: string): Promise<boolean>

  // Profile management
  getAllProfiles(): Promise<SSHProfileWithConfig[]>
  getProfileById(id: string): Promise<SSHProfileWithConfig | null>
  getProfilesByGroupId(groupId: string): Promise<SSHProfileWithConfig[]>
  createProfile(profileData: StorageCreateData<SSHProfile>): Promise<SSHProfile>
  getRecentlyConnectedProfiles(limit?: number): Promise<SSHProfileWithConfig[]>
  createProfile(profileData: Omit<SSHProfile, 'id' | 'created' | 'updated'>): Promise<SSHProfile>
  updateProfile(id: string, updates: Partial<SSHProfile>): Promise<SSHProfile | null>
  deleteProfile(id: string): Promise<boolean>
  toggleProfileFavorite(id: string): Promise<SSHProfile | null>
  searchProfiles(query: string): Promise<SSHProfileWithConfig[]>

  // Connection management
  recordConnection(profileId: string, status: 'connected' | 'failed'): Promise<void>
  getRecentConnections(limit?: number): Promise<SSHConnection[]>
  getConnectionStats(): Promise<unknown>
  cleanupOldConnections(daysOld?: number): Promise<number>

  // Utility methods
  validateConnectionConfig(config: ResolvedSSHConfig): { valid: boolean; errors: string[] }
  getGroupsWithProfiles(): Promise<Array<SSHGroup & { profiles: SSHProfileWithConfig[] }>>
  getUngroupedProfiles(): Promise<SSHProfileWithConfig[]>
}

export interface ISSHConnectionService {
  getConnection(terminalId: string): unknown
  closeConnection(terminalId: string): void
  getActiveConnectionCount(): number
}

export interface ISSHProfileStorage extends IStorageService<SSHProfile> {
  getByGroupId(groupId: string): Promise<SSHProfile[]>
  getFavorites(): Promise<SSHProfile[]>
  getRecentlyConnected(limit?: number): Promise<SSHProfile[]>
  toggleFavorite(id: string): Promise<SSHProfile | null>
  updateLastConnected(id: string): Promise<SSHProfile | null>
  search(query: string): Promise<SSHProfile[]>
  deleteByGroupId(groupId: string): Promise<number>
}

export interface ISSHGroupStorage extends IStorageService<SSHGroup> {
  // Additional group-specific methods can be added here
}

export interface ISSHConnectionStorage extends IStorageService<SSHConnection> {
  getByProfileId(profileId: string): Promise<SSHConnection[]>
  getRecent(limit?: number): Promise<SSHConnection[]>
  deleteByProfileId(profileId: string): Promise<number>
  clearOldConnections(daysOld?: number): Promise<number>
  getStats(): Promise<unknown>
}

export interface ISSHConfigValidator extends IValidator<ResolvedSSHConfig> {
  validateProfile(profile: SSHProfile): { valid: boolean; errors: string[] }
  validateGroup(group: SSHGroup): { valid: boolean; errors: string[] }
}
