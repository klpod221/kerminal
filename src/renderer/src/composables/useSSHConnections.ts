/**
 * Composable for SSH connection management
 */
import { ref, computed } from 'vue'
import type { Ref, ComputedRef } from 'vue'
import type { SSHProfileWithConfig, SSHGroup, SSHConnection } from '../types/ssh'

export interface UseSSHConnectionsState {
  profiles: Ref<SSHProfileWithConfig[]>
  groups: Ref<SSHGroup[]>
  recentConnections: Ref<SSHConnection[]>
  isLoading: Ref<boolean>
  error: Ref<string | null>
}

export interface UseSSHConnectionsActions {
  loadProfiles: () => Promise<void>
  loadGroups: () => Promise<void>
  loadRecentConnections: () => Promise<void>
  refreshAll: () => Promise<void>
  connectToProfile: (profile: SSHProfileWithConfig) => void
  createProfile: (profileData: Partial<SSHProfileWithConfig>) => Promise<void>
  updateProfile: (id: string, updates: Partial<SSHProfileWithConfig>) => Promise<void>
  deleteProfile: (id: string) => Promise<void>
  toggleFavorite: (id: string) => Promise<void>
  searchProfiles: (query: string) => Promise<SSHProfileWithConfig[]>
}

export interface UseSSHConnectionsReturn extends UseSSHConnectionsState, UseSSHConnectionsActions {
  profilesByGroup: ComputedRef<{ group: SSHGroup; profiles: SSHProfileWithConfig[] }[]>
  favoriteProfiles: ComputedRef<SSHProfileWithConfig[]>
  ungroupedProfiles: ComputedRef<SSHProfileWithConfig[]>
  groupsWithProfiles: ComputedRef<(SSHGroup & { profiles: SSHProfileWithConfig[] })[]>
  hasProfiles: ComputedRef<boolean>
  hasGroups: ComputedRef<boolean>
}

export function useSSHConnections(): UseSSHConnectionsReturn {
  // State
  const profiles = ref<SSHProfileWithConfig[]>([])
  const groups = ref<SSHGroup[]>([])
  const recentConnections = ref<SSHConnection[]>([])
  const isLoading = ref(false)
  const error = ref<string | null>(null)

  // Computed
  const favoriteProfiles = computed(() => profiles.value.filter((profile) => profile.favorite))

  const ungroupedProfiles = computed(() => profiles.value.filter((profile) => !profile.groupId))

  const groupsWithProfiles = computed(() =>
    groups.value.map((group) => ({
      ...group,
      profiles: profiles.value.filter((profile) => profile.groupId === group.id)
    }))
  )

  const profilesByGroup = computed(() =>
    groups.value.map((group) => ({
      group,
      profiles: profiles.value.filter((profile) => profile.groupId === group.id)
    }))
  )

  const hasProfiles = computed(() => profiles.value.length > 0)
  const hasGroups = computed(() => groups.value.length > 0)

  // Actions
  const loadProfiles = async (): Promise<void> => {
    try {
      isLoading.value = true
      error.value = null
      const result = (await window.api.invoke('ssh-profiles.getAll')) as SSHProfileWithConfig[]
      profiles.value = result.map((profile) => ({
        ...profile,
        created: new Date(profile.created),
        updated: new Date(profile.updated),
        lastConnected: profile.lastConnected ? new Date(profile.lastConnected) : undefined
      }))
    } catch (err) {
      error.value = 'Failed to load SSH profiles'
      console.error('Failed to load SSH profiles:', err)
    } finally {
      isLoading.value = false
    }
  }

  const loadGroups = async (): Promise<void> => {
    try {
      const result = (await window.api.invoke('ssh-groups.getAll')) as SSHGroup[]
      groups.value = result.map((group) => ({
        ...group,
        created: new Date(group.created),
        updated: new Date(group.updated)
      }))
    } catch (err) {
      error.value = 'Failed to load SSH groups'
      console.error('Failed to load SSH groups:', err)
    }
  }

  const loadRecentConnections = async (): Promise<void> => {
    try {
      const result = (await window.api.invoke('ssh-connections.getRecent', 10)) as SSHConnection[]
      recentConnections.value = result.map((connection) => ({
        ...connection,
        connectedAt: new Date(connection.connectedAt)
      }))
    } catch (err) {
      error.value = 'Failed to load recent connections'
      console.error('Failed to load recent connections:', err)
    }
  }

  const refreshAll = async (): Promise<void> => {
    await Promise.all([loadProfiles(), loadGroups(), loadRecentConnections()])
  }

  const connectToProfile = (profile: SSHProfileWithConfig): void => {
    // This will be handled by parent component
    console.log('Connecting to profile:', profile.name)
  }

  const createProfile = async (profileData: Partial<SSHProfileWithConfig>): Promise<void> => {
    try {
      await window.api.invoke('ssh-profiles.create', profileData)
      await loadProfiles()
    } catch (err) {
      error.value = 'Failed to create SSH profile'
      console.error('Failed to create SSH profile:', err)
      throw err
    }
  }

  const updateProfile = async (
    id: string,
    updates: Partial<SSHProfileWithConfig>
  ): Promise<void> => {
    try {
      await window.api.invoke('ssh-profiles.update', id, updates)
      await loadProfiles()
    } catch (err) {
      error.value = 'Failed to update SSH profile'
      console.error('Failed to update SSH profile:', err)
      throw err
    }
  }

  const deleteProfile = async (id: string): Promise<void> => {
    try {
      await window.api.invoke('ssh-profiles.delete', id)
      await loadProfiles()
    } catch (err) {
      error.value = 'Failed to delete SSH profile'
      console.error('Failed to delete SSH profile:', err)
      throw err
    }
  }

  const toggleFavorite = async (id: string): Promise<void> => {
    try {
      await window.api.invoke('ssh-profiles.toggleFavorite', id)
      // Update local state optimistically
      const profile = profiles.value.find((p) => p.id === id)
      if (profile) {
        profile.favorite = !profile.favorite
      }
    } catch (err) {
      error.value = 'Failed to toggle favorite'
      console.error('Failed to toggle favorite:', err)
      // Reload profiles to revert optimistic update
      await loadProfiles()
    }
  }

  const searchProfiles = async (query: string): Promise<SSHProfileWithConfig[]> => {
    try {
      const result = (await window.api.invoke(
        'ssh-profiles.search',
        query
      )) as SSHProfileWithConfig[]
      return result.map((profile) => ({
        ...profile,
        created: new Date(profile.created),
        updated: new Date(profile.updated),
        lastConnected: profile.lastConnected ? new Date(profile.lastConnected) : undefined
      }))
    } catch (err) {
      error.value = 'Failed to search profiles'
      console.error('Failed to search profiles:', err)
      return []
    }
  }

  return {
    // State
    profiles,
    groups,
    recentConnections,
    isLoading,
    error,

    // Computed
    profilesByGroup,
    favoriteProfiles,
    ungroupedProfiles,
    groupsWithProfiles,
    hasProfiles,
    hasGroups,

    // Actions
    loadProfiles,
    loadGroups,
    loadRecentConnections,
    refreshAll,
    connectToProfile,
    createProfile,
    updateProfile,
    deleteProfile,
    toggleFavorite,
    searchProfiles
  }
}
