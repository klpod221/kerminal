import { defineStore } from "pinia";
import { ref, computed } from "vue";
import type {
  SSHProfile,
  SSHGroup,
  SSHConfigHost,
  CreateSSHProfileRequest,
  UpdateSSHProfileRequest,
  DeleteGroupAction,
  CreateSSHGroupRequest,
  UpdateSSHGroupRequest,
} from "../types/ssh";
import * as sshService from "../services/sshProfile";
import { api } from "../services/api";

/**
 * SSH Profiles and Groups Store
 * Manages SSH profiles and groups with reactive state management and caching
 */
export const useSSHStore = defineStore("ssh", () => {
  const profiles = ref<SSHProfile[]>([]);
  const groups = ref<SSHGroup[]>([]);
  const configHosts = ref<SSHConfigHost[]>([]);
  const configHostsLoaded = ref(false);

  /**
   * Optimized data structure containing group info with profiles and fast lookup indices
   * Returns an object with complete group info, profiles, and lookup maps for fast access
   */
  const groupsWithProfiles = computed(() => {
    const groupMap = new Map<string, SSHGroup>();
    const profileMap = new Map<string, SSHProfile>();

    groups.value.forEach((group) => groupMap.set(group.id, group));
    profiles.value.forEach((profile) => profileMap.set(profile.id, profile));

    const groupedData = new Map<
      string | null,
      {
        group: SSHGroup | null;
        profiles: SSHProfile[];
        profileCount: number;
      }
    >();

    groups.value.forEach((group) => {
      groupedData.set(group.id, {
        group,
        profiles: [],
        profileCount: 0,
      });
    });

    groupedData.set(null, {
      group: null,
      profiles: [],
      profileCount: 0,
    });

    profiles.value.forEach((profile) => {
      const groupId = profile.groupId || null;
      if (!groupedData.has(groupId)) {
        const group = groupId ? groupMap.get(groupId) || null : null;
        groupedData.set(groupId, {
          group,
          profiles: [],
          profileCount: 0,
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
      getGroup: (id: string) => groupMap.get(id),
      getProfile: (id: string) => profileMap.get(id),
      getGroupWithProfiles: (id: string | null) => groupedData.get(id),
      getUngroupedData: () => groupedData.get(null)!,
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
    return (groupId: string) =>
      groupsWithProfiles.value.getGroupWithProfiles(groupId)?.profiles || [];
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
  const createProfile = async (
    request: CreateSSHProfileRequest,
  ): Promise<SSHProfile> => {
    const newProfile = await sshService.createSSHProfile(request);
    await loadProfiles(); // Reload to ensure data consistency
    return newProfile;
  };

  /**
   * Update existing SSH profile
   */
  const updateProfile = async (
    id: string,
    request: UpdateSSHProfileRequest,
  ): Promise<SSHProfile> => {
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
  const moveProfileToGroup = async (
    profileId: string,
    groupId: string | null,
  ): Promise<void> => {
    await sshService.moveProfileToGroup(profileId, groupId);
    await loadProfiles(); // Reload to ensure data consistency
  };

  /**
   * Duplicate SSH profile
   */
  const duplicateProfile = async (
    id: string,
    newName: string,
  ): Promise<SSHProfile> => {
    const duplicatedProfile = await sshService.duplicateSSHProfile(id, newName);
    await loadProfiles(); // Reload to ensure data consistency
    return duplicatedProfile;
  };

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
    request: CreateSSHGroupRequest,
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
    request: UpdateSSHGroupRequest,
  ): Promise<SSHGroup> => {
    const updatedGroup = await sshService.updateSSHGroup(id, request);
    await loadGroups(); // Reload to ensure data consistency
    return updatedGroup;
  };

  /**
   * Delete SSH group with action for existing profiles
   */
  const deleteGroup = async (
    id: string,
    action: DeleteGroupAction,
  ): Promise<void> => {
    await sshService.deleteSSHGroup(id, action);
    await loadAll();
  };

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
    configHosts.value = [];
    configHostsLoaded.value = false;
  };

  /**
   * Load SSH config hosts from ~/.ssh/config
   * Uses cache to avoid re-parsing on every call
   */
  const loadConfigHosts = async (force = false): Promise<void> => {
    if (!force && configHostsLoaded.value) {
      return;
    }

    try {
      const hosts = await sshService.getSSHConfigHosts();
      configHosts.value = hosts;
      configHostsLoaded.value = true;
    } catch (error) {
      console.error("Failed to load SSH config hosts:", error);
      configHosts.value = [];
      configHostsLoaded.value = false;
    }
  };

  const upsertProfile = (p: SSHProfile) => {
    if (!p?.id) return;
    const i = profiles.value.findIndex((x) => x?.id === p.id);
    if (i === -1) {
      profiles.value = [...profiles.value, p];
    } else {
      profiles.value[i] = { ...profiles.value[i]!, ...p };
    }
  };

  const removeProfile = (id: string) => {
    profiles.value = profiles.value.filter((p) => p?.id !== id);
  };

  const upsertGroup = (g: SSHGroup) => {
    if (!g?.id) return;
    const i = groups.value.findIndex((x) => x?.id === g.id);
    if (i === -1) {
      groups.value = [...groups.value, g];
    } else {
      groups.value[i] = { ...groups.value[i]!, ...g };
    }
  };

  const removeGroup = (id: string) => {
    groups.value = groups.value.filter((g) => g?.id !== id);
  };

  let unsubscribeProfileRealtime: (() => void) | null = null;
  let unsubscribeGroupRealtime: (() => void) | null = null;

  const startRealtime = async (): Promise<void> => {
    if (unsubscribeProfileRealtime && unsubscribeGroupRealtime) return;
    try {
      if (!unsubscribeProfileRealtime) {
        const u1 = await api.listen<SSHProfile>("ssh_profile_created", upsertProfile);
        const u2 = await api.listen<SSHProfile>("ssh_profile_updated", upsertProfile);
        const u3 = await api.listen<{ id: string }>(
          "ssh_profile_deleted",
          ({ id }) => removeProfile(id),
        );
        unsubscribeProfileRealtime = () => {
          u1();
          u2();
          u3();
        };
      }

      if (!unsubscribeGroupRealtime) {
        const g1 = await api.listen<SSHGroup>("ssh_group_created", upsertGroup);
        const g2 = await api.listen<SSHGroup>("ssh_group_updated", upsertGroup);
        const g3 = await api.listen<{ id: string }>(
          "ssh_group_deleted",
          ({ id }) => removeGroup(id),
        );
        unsubscribeGroupRealtime = () => {
          g1();
          g2();
          g3();
        };
      }
    } catch (e) {
      console.error("Failed to subscribe SSH realtime events:", e);
    }
  };

  const stopRealtime = (): void => {
    if (unsubscribeProfileRealtime) {
      unsubscribeProfileRealtime();
      unsubscribeProfileRealtime = null;
    }
    if (unsubscribeGroupRealtime) {
      unsubscribeGroupRealtime();
      unsubscribeGroupRealtime = null;
    }
  };

  return {
    profiles,
    groups,
    configHosts,
    configHostsLoaded,

    groupsWithProfiles,
    ungroupedProfiles,
    getProfilesByGroupId,
    findProfileById,
    findGroupById,
    hasData,

    loadProfiles,
    createProfile,
    updateProfile,
    deleteProfile,
    moveProfileToGroup,
    duplicateProfile,

    loadGroups,
    createGroup,
    updateGroup,
    deleteGroup,

    loadConfigHosts,

    loadAll,
    refresh,
    clearAll,

    startRealtime,
    stopRealtime,
  };
});
