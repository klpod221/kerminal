import { defineStore } from "pinia";
import { ref, computed } from "vue";
import type {
  SSHProfile,
  SSHGroup,
  CreateSSHProfileRequest,
  UpdateSSHProfileRequest,
  DeleteGroupAction,
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
   * Get profiles grouped by groupId
   */
  const profilesByGroup = computed(() => {
    const grouped = new Map<string | null, SSHProfile[]>();

    profiles.value.forEach(profile => {
      const groupId = profile.groupId || null;
      if (!grouped.has(groupId)) {
        grouped.set(groupId, []);
      }
      grouped.get(groupId)!.push(profile);
    });

    // Sort profiles within each group by sortOrder and name
    grouped.forEach(profileList => {
      profileList.sort((a, b) => {
        if (a.sortOrder !== b.sortOrder) {
          return a.sortOrder - b.sortOrder;
        }
        return a.name.localeCompare(b.name);
      });
    });

    return grouped;
  });

  /**
   * Get ungrouped profiles
   */
  const ungroupedProfiles = computed(() => {
    return profilesByGroup.value.get(null) || [];
  });

  /**
   * Get profiles by group ID
   */
  const getProfilesByGroupId = computed(() => {
    return (groupId: string) => profilesByGroup.value.get(groupId) || [];
  });

  /**
   * Get sorted groups by sortOrder and name
   */
  const sortedGroups = computed(() => {
    return [...groups.value].sort((a, b) => {
      if (a.sortOrder !== b.sortOrder) {
        return a.sortOrder - b.sortOrder;
      }
      return a.name.localeCompare(b.name);
    });
  });

  /**
   * Find profile by ID
   */
  const findProfileById = computed(() => {
    return (id: string) => profiles.value.find(p => p.base.id === id);
  });

  /**
   * Find group by ID
   */
  const findGroupById = computed(() => {
    return (id: string) => groups.value.find(g => g.base.id === id);
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
    profiles.value.push(newProfile);
    return newProfile;
  };

  /**
   * Update existing SSH profile
   */
  const updateProfile = async (id: string, request: UpdateSSHProfileRequest): Promise<SSHProfile> => {
    const updatedProfile = await sshService.updateSSHProfile(id, request);
    const index = profiles.value.findIndex(p => p.base.id === id);

    if (index !== -1) {
      profiles.value[index] = updatedProfile;
    }

    return updatedProfile;
  };

  /**
   * Delete SSH profile
   */
  const deleteProfile = async (id: string): Promise<void> => {
    await sshService.deleteSSHProfile(id);
    profiles.value = profiles.value.filter(p => p.base.id !== id);
  };

  /**
   * Move profile to different group
   */
  const moveProfileToGroup = async (profileId: string, groupId: string | null): Promise<void> => {
    await sshService.moveProfileToGroup(profileId, groupId);

    // Update local state
    const profile = findProfileById.value(profileId);
    if (profile) {
      profile.groupId = groupId || undefined;
    }
  };

  /**
   * Duplicate SSH profile
   */
  const duplicateProfile = async (id: string, newName: string): Promise<SSHProfile> => {
    const duplicatedProfile = await sshService.duplicateSSHProfile(id, newName);
    profiles.value.push(duplicatedProfile);
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
    name: string,
    description?: string,
    color?: string,
    icon?: string
  ): Promise<SSHGroup> => {
    const newGroup = await sshService.createSSHGroup(name, description, color, icon);
    groups.value.push(newGroup);
    return newGroup;
  };

  /**
   * Update existing SSH group
   */
  const updateGroup = async (
    id: string,
    name?: string,
    description?: string,
    color?: string,
    icon?: string
  ): Promise<SSHGroup> => {
    const updatedGroup = await sshService.updateSSHGroup(id, name, description, color, icon);
    const index = groups.value.findIndex(g => g.base.id === id);

    if (index !== -1) {
      groups.value[index] = updatedGroup;
    }

    return updatedGroup;
  };

  /**
   * Delete SSH group with action for existing profiles
   */
  const deleteGroup = async (id: string, action: DeleteGroupAction): Promise<void> => {
    await sshService.deleteSSHGroup(id, action);

    // Update local state
    groups.value = groups.value.filter(g => g.base.id !== id);

    // Update profiles based on action
    if (action.actionType === "moveToGroup" && action.targetGroupId) {
      profiles.value.forEach(profile => {
        if (profile.groupId === id) {
          profile.groupId = action.targetGroupId;
        }
      });
    } else if (action.actionType === "moveToUngrouped") {
      profiles.value.forEach(profile => {
        if (profile.groupId === id) {
          profile.groupId = undefined;
        }
      });
    } else if (action.actionType === "deleteProfiles") {
      profiles.value = profiles.value.filter(p => p.groupId !== id);
    }
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
    profilesByGroup,
    ungroupedProfiles,
    getProfilesByGroupId,
    sortedGroups,
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
