import { defineStore } from "pinia";
import { ref, computed } from "vue";
import type {
  SSHProfile,
  SSHGroup,
  CreateSSHProfileRequest,
  UpdateSSHProfileRequest,
  DeleteGroupAction,
  CreateSSHGroupRequest,
  UpdateSSHGroupRequest,
} from "../types/ssh";
import * as sshService from "../services/sshProfile";

/**
 * SSH Profiles and Groups Store
 * Manages SSH profiles and groups with reactive state management and caching
 */
export const useSSHStore = defineStore("ssh", () => {
  // === State ===
  const profiles = ref<SSHProfile[]>([]);
  const groups = ref<SSHGroup[]>([]);

  // === Computed ===

  /**
   * Optimized data structure containing group info with profiles and fast lookup indices
   * Returns an object with complete group info, profiles, and lookup maps for fast access
   */
  const groupsWithProfiles = computed(() => {
    const groupMap = new Map<string, SSHGroup>();
    const profileMap = new Map<string, SSHProfile>();

    // Create lookup maps
    groups.value.forEach(group => groupMap.set(group.id, group));
    profiles.value.forEach(profile => profileMap.set(profile.id, profile));

    // Build grouped structure with full group information
    const groupedData = new Map<string | null, {
      group: SSHGroup | null;
      profiles: SSHProfile[];
      profileCount: number;
    }>();

    // Initialize all groups
    groups.value.forEach(group => {
      groupedData.set(group.id, {
        group,
        profiles: [],
        profileCount: 0
      });
    });

    // Initialize ungrouped section
    groupedData.set(null, {
      group: null,
      profiles: [],
      profileCount: 0
    });

    // Distribute profiles
    profiles.value.forEach(profile => {
      const groupId = profile.groupId || null;
      if (!groupedData.has(groupId)) {
        const group = groupId ? groupMap.get(groupId) || null : null;
        groupedData.set(groupId, {
          group,
          profiles: [],
          profileCount: 0
        });
      }

      const groupData = groupedData.get(groupId)!;
      groupData.profiles.push(profile);
      groupData.profileCount++;
    });

    return {
      groupedData,
      groupMap,
      profileMap,
      // Helper methods for quick access
      getGroup: (id: string) => groupMap.get(id),
      getProfile: (id: string) => profileMap.get(id),
      getGroupWithProfiles: (id: string | null) => groupedData.get(id),
      getUngroupedData: () => groupedData.get(null)!
    };
  });

  /**
   * Get ungrouped profiles
   */
  const ungroupedProfiles = computed(() => {
    return groupsWithProfiles.value.getUngroupedData().profiles;
  });

  /**
   * Get profiles by group ID
   */
  const getProfilesByGroupId = computed(() => {
    return (groupId: string) => groupsWithProfiles.value.getGroupWithProfiles(groupId)?.profiles || [];
  });

  /**
   * Find profile by ID
   */
  const findProfileById = computed(() => {
    return (id: string) => groupsWithProfiles.value.getProfile(id);
  });

  /**
   * Find group by ID
   */
  const findGroupById = computed(() => {
    return (id: string) => groupsWithProfiles.value.getGroup(id);
  });

  /**
   * Check if store has data
   */
  const hasData = computed(() => {
    return profiles.value.length > 0 || groups.value.length > 0;
  });

  // === Profile Actions ===

  /**
   * Load all SSH profiles from backend
   */
  const loadProfiles = async (): Promise<void> => {
    const loadedProfiles = await sshService.getSSHProfiles();
    profiles.value = loadedProfiles;
  };

  /**
   * Create new SSH profile
   */
  const createProfile = async (request: CreateSSHProfileRequest): Promise<SSHProfile> => {
    const newProfile = await sshService.createSSHProfile(request);
    await loadProfiles(); // Reload to ensure data consistency
    return newProfile;
  };

  /**
   * Update existing SSH profile
   */
  const updateProfile = async (id: string, request: UpdateSSHProfileRequest): Promise<SSHProfile> => {
    const updatedProfile = await sshService.updateSSHProfile(id, request);
    await loadProfiles(); // Reload to ensure data consistency
    return updatedProfile;
  };

  /**
   * Delete SSH profile
   */
  const deleteProfile = async (id: string): Promise<void> => {
    await sshService.deleteSSHProfile(id);
    await loadProfiles(); // Reload to ensure data consistency
  };

  /**
   * Move profile to different group
   */
  const moveProfileToGroup = async (profileId: string, groupId: string | null): Promise<void> => {
    await sshService.moveProfileToGroup(profileId, groupId);
    await loadProfiles(); // Reload to ensure data consistency
  };

  /**
   * Duplicate SSH profile
   */
  const duplicateProfile = async (id: string, newName: string): Promise<SSHProfile> => {
    const duplicatedProfile = await sshService.duplicateSSHProfile(id, newName);
    await loadProfiles(); // Reload to ensure data consistency
    return duplicatedProfile;
  };

  // === Group Actions ===

  /**
   * Load all SSH groups from backend
   */
  const loadGroups = async (): Promise<void> => {
    const loadedGroups = await sshService.getSSHGroups();
    groups.value = loadedGroups;
  };

  /**
   * Create new SSH group
   */
  const createGroup = async (
    request: CreateSSHGroupRequest
  ): Promise<SSHGroup> => {
    const newGroup = await sshService.createSSHGroup(request);
    await loadGroups(); // Reload to ensure data consistency
    return newGroup;
  };

  /**
   * Update existing SSH group
   */
  const updateGroup = async (
    id: string,
    request: UpdateSSHGroupRequest
  ): Promise<SSHGroup> => {
    const updatedGroup = await sshService.updateSSHGroup(id, request);
    await loadGroups(); // Reload to ensure data consistency
    return updatedGroup;
  };

  /**
   * Delete SSH group with action for existing profiles
   */
  const deleteGroup = async (id: string, action: DeleteGroupAction): Promise<void> => {
    await sshService.deleteSSHGroup(id, action);
    // Backend handles all profile movements/deletions, just reload all data
    await loadAll();
  };

  // === Utility Actions ===

  /**
   * Load all data (profiles and groups)
   */
  const loadAll = async (): Promise<void> => {
    await Promise.all([loadProfiles(), loadGroups()]);
  };

  /**
   * Refresh all data
   */
  const refresh = async (): Promise<void> => {
    await loadAll();
  };

  /**
   * Clear all data (for logout/reset)
   */
  const clearAll = (): void => {
    profiles.value = [];
    groups.value = [];
  };

  return {
    // State
    profiles,
    groups,

    // Computed
    groupsWithProfiles,
    ungroupedProfiles,
    getProfilesByGroupId,
    findProfileById,
    findGroupById,
    hasData,

    // Profile Actions
    loadProfiles,
    createProfile,
    updateProfile,
    deleteProfile,
    moveProfileToGroup,
    duplicateProfile,

    // Group Actions
    loadGroups,
    createGroup,
    updateGroup,
    deleteGroup,

    // Utility Actions
    loadAll,
    refresh,
    clearAll,
  };
});
