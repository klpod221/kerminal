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
import {
  withRetry,
  handleError,
  type ErrorContext,
} from "../utils/errorHandler";
import { message } from "../utils/message";

/**
 * Test SSH connection with error handling
 * @param request - SSH connection test request
 * @throws Enhanced error if test fails
 */
async function testConnection(request: {
  host: string;
  port: number;
  username: string;
  authMethod: string;
  authData: any;
  timeout?: number;
  keepAlive?: boolean;
  compression?: boolean;
  proxy?: any;
}): Promise<void> {
  const context: ErrorContext = {
    operation: "Test SSH Connection",
    context: { host: request.host, username: request.username },
  };

  try {
    await withRetry(
      () => sshService.testSSHConnection(request),
      { maxRetries: 1 },
      context,
    );
  } catch (error) {
    const errorMessage = handleError(error, context);
    message.error(errorMessage);
    throw new Error(errorMessage);
  }
}

/**
 * SSH Profiles and Groups Store
 * Manages SSH profiles and groups with reactive state management and caching
 */
export const useSSHStore = defineStore("ssh", () => {
  const profiles = ref<SSHProfile[]>([]);
  const groups = ref<SSHGroup[]>([]);
  const configHosts = ref<SSHConfigHost[]>([]);
  const configHostsLoaded = ref(false);
  const isLoading = ref(false);

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

      const groupData = groupedData.get(groupId);
      if (groupData) {
        groupData.profiles.push(profile);
        groupData.profileCount++;
      }
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
   * Load all SSH profiles from backend with error handling
   */
  const loadProfiles = async (): Promise<void> => {
    isLoading.value = true;
    const context: ErrorContext = {
      operation: "Load SSH Profiles",
    };

    try {
      const loadedProfiles = await withRetry(
        () => sshService.getSSHProfiles(),
        { maxRetries: 2 },
        context,
      );
      profiles.value = loadedProfiles;
    } catch (error) {
      const errorMessage = handleError(error, context);
      message.error(errorMessage);
      profiles.value = [];
      throw new Error(errorMessage);
    } finally {
      isLoading.value = false;
    }
  };

  /**
   * Create new SSH profile with error handling
   * @param request - Profile creation request
   * @returns Created profile
   */
  const createProfile = async (
    request: CreateSSHProfileRequest,
  ): Promise<SSHProfile> => {
    isLoading.value = true;
    const context: ErrorContext = {
      operation: "Create SSH Profile",
      context: { name: request.name, host: request.host },
    };

    try {
      const newProfile = await withRetry(
        () => sshService.createSSHProfile(request),
        { maxRetries: 1 },
        context,
      );
      await loadProfiles(); // Reload to ensure data consistency
      return newProfile;
    } catch (error) {
      const errorMessage = handleError(error, context);
      message.error(errorMessage);
      throw new Error(errorMessage);
    } finally {
      isLoading.value = false;
    }
  };

  /**
   * Update existing SSH profile with error handling
   * @param id - Profile ID to update
   * @param request - Update request
   * @returns Updated profile
   */
  const updateProfile = async (
    id: string,
    request: UpdateSSHProfileRequest,
  ): Promise<SSHProfile> => {
    isLoading.value = true;
    const context: ErrorContext = {
      operation: "Update SSH Profile",
      context: { profileId: id },
    };

    try {
      const updatedProfile = await withRetry(
        () => sshService.updateSSHProfile(id, request),
        { maxRetries: 1 },
        context,
      );
      await loadProfiles(); // Reload to ensure data consistency
      return updatedProfile;
    } catch (error) {
      const errorMessage = handleError(error, context);
      message.error(errorMessage);
      throw new Error(errorMessage);
    } finally {
      isLoading.value = false;
    }
  };

  /**
   * Delete SSH profile with error handling
   * @param id - Profile ID to delete
   */
  const deleteProfile = async (id: string): Promise<void> => {
    isLoading.value = true;
    const context: ErrorContext = {
      operation: "Delete SSH Profile",
      context: { profileId: id },
    };

    try {
      await sshService.deleteSSHProfile(id);
      await loadProfiles(); // Reload to ensure data consistency
    } catch (error) {
      const errorMessage = handleError(error, context);
      message.error(errorMessage);
      throw new Error(errorMessage);
    } finally {
      isLoading.value = false;
    }
  };

  /**
   * Move profile to different group
   */
  const moveProfileToGroup = async (
    profileId: string,
    groupId: string | null,
  ): Promise<void> => {
    isLoading.value = true;
    try {
      await sshService.moveProfileToGroup(profileId, groupId);
      await loadProfiles(); // Reload to ensure data consistency
    } finally {
      isLoading.value = false;
    }
  };

  /**
   * Duplicate SSH profile with error handling
   * @param id - Profile ID to duplicate
   * @param newName - Name for the duplicated profile
   * @returns Duplicated profile
   */
  const duplicateProfile = async (
    id: string,
    newName: string,
  ): Promise<SSHProfile> => {
    isLoading.value = true;
    const context: ErrorContext = {
      operation: "Duplicate SSH Profile",
      context: { profileId: id, newName },
    };

    try {
      const duplicatedProfile = await withRetry(
        () => sshService.duplicateSSHProfile(id, newName),
        { maxRetries: 1 },
        context,
      );
      await loadProfiles(); // Reload to ensure data consistency
      return duplicatedProfile;
    } catch (error) {
      const errorMessage = handleError(error, context);
      message.error(errorMessage);
      throw new Error(errorMessage);
    } finally {
      isLoading.value = false;
    }
  };

  /**
   * Load all SSH groups from backend with error handling
   */
  const loadGroups = async (): Promise<void> => {
    isLoading.value = true;
    const context: ErrorContext = {
      operation: "Load SSH Groups",
    };

    try {
      const loadedGroups = await withRetry(
        () => sshService.getSSHGroups(),
        { maxRetries: 2 },
        context,
      );
      groups.value = loadedGroups;
    } catch (error) {
      const errorMessage = handleError(error, context);
      message.error(errorMessage);
      groups.value = [];
      throw new Error(errorMessage);
    } finally {
      isLoading.value = false;
    }
  };

  /**
   * Create new SSH group with error handling
   * @param request - Group creation request
   * @returns Created group
   */
  const createGroup = async (
    request: CreateSSHGroupRequest,
  ): Promise<SSHGroup> => {
    isLoading.value = true;
    const context: ErrorContext = {
      operation: "Create SSH Group",
      context: { name: request.name },
    };

    try {
      const newGroup = await withRetry(
        () => sshService.createSSHGroup(request),
        { maxRetries: 1 },
        context,
      );
      await loadGroups(); // Reload to ensure data consistency
      return newGroup;
    } catch (error) {
      const errorMessage = handleError(error, context);
      message.error(errorMessage);
      throw new Error(errorMessage);
    } finally {
      isLoading.value = false;
    }
  };

  /**
   * Update existing SSH group with error handling
   * @param id - Group ID to update
   * @param request - Update request
   * @returns Updated group
   */
  const updateGroup = async (
    id: string,
    request: UpdateSSHGroupRequest,
  ): Promise<SSHGroup> => {
    isLoading.value = true;
    const context: ErrorContext = {
      operation: "Update SSH Group",
      context: { groupId: id },
    };

    try {
      const updatedGroup = await withRetry(
        () => sshService.updateSSHGroup(id, request),
        { maxRetries: 1 },
        context,
      );
      await loadGroups(); // Reload to ensure data consistency
      return updatedGroup;
    } catch (error) {
      const errorMessage = handleError(error, context);
      message.error(errorMessage);
      throw new Error(errorMessage);
    } finally {
      isLoading.value = false;
    }
  };

  /**
   * Delete SSH group with action for existing profiles and error handling
   * @param id - Group ID to delete
   * @param action - Action to take for profiles in the group
   */
  const deleteGroup = async (
    id: string,
    action: DeleteGroupAction,
  ): Promise<void> => {
    isLoading.value = true;
    const context: ErrorContext = {
      operation: "Delete SSH Group",
      context: { groupId: id },
    };

    try {
      await sshService.deleteSSHGroup(id, action);
      await loadAll();
    } catch (error) {
      const errorMessage = handleError(error, context);
      message.error(errorMessage);
      throw new Error(errorMessage);
    } finally {
      isLoading.value = false;
    }
  };

  /**
   * Load all data (profiles and groups)
   */
  const loadAll = async (): Promise<void> => {
    isLoading.value = true;
    try {
      await Promise.all([loadProfiles(), loadGroups()]);
    } finally {
      isLoading.value = false;
    }
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
   * Load SSH config hosts from ~/.ssh/config with error handling
   * Uses cache to avoid re-parsing on every call
   * @param force - Force reload even if already loaded
   */
  const loadConfigHosts = async (force = false): Promise<void> => {
    if (!force && configHostsLoaded.value) {
      return;
    }

    isLoading.value = true;
    const context: ErrorContext = {
      operation: "Load SSH Config Hosts",
    };

    try {
      const hosts = await withRetry(
        () => sshService.getSSHConfigHosts(),
        { maxRetries: 1 },
        context,
      );
      configHosts.value = hosts;
      configHostsLoaded.value = true;
    } catch (error) {
      const errorMessage = handleError(error, context);
      message.error(errorMessage);
      configHosts.value = [];
      configHostsLoaded.value = false;
    } finally {
      isLoading.value = false;
    }
  };

  const upsertProfile = (p: SSHProfile) => {
    if (!p?.id) return;
    const i = profiles.value.findIndex((x) => x?.id === p.id);
    if (i === -1) {
      profiles.value = [...profiles.value, p];
    } else {
      profiles.value[i] = { ...profiles.value[i], ...p };
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
      groups.value[i] = { ...groups.value[i], ...g };
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
        const u1 = await api.listen<SSHProfile>(
          "ssh_profile_created",
          upsertProfile,
        );
        const u2 = await api.listen<SSHProfile>(
          "ssh_profile_updated",
          upsertProfile,
        );
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
    isLoading,

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

    testConnection,

    startRealtime,
    stopRealtime,
  };
});
