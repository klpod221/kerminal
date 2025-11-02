<template>
  <Drawer
    id="ssh-profile-drawer"
    title="SSH Profiles"
    position="left"
    :icon="Server"
    icon-background="bg-orange-500/20"
    icon-color="text-orange-400"
  >
    <template #headerAction>
      <Form>
        <!-- Search -->
        <Input
          id="search-ssh-profiles"
          v-model="searchQuery"
          type="text"
          placeholder="Search SSH profiles..."
          :left-icon="Search"
          :helper="false"
        />
      </Form>
    </template>

    <EmptyState
      v-if="!sshStore.hasData"
      :icon="Server"
      title="No SSH Profiles"
      description="Create your first SSH profile to get started."
      action-text="Create Your First Profile"
      :action-icon="Plus"
      action-variant="outline"
      @action="createNewProfile"
    />

    <div v-else class="space-y-4 p-4">
      <!-- Divider -->
      <div
        v-if="filteredConfigHosts.length > 0 && filteredGroupsData.length > 0"
        class="border-t border-gray-700"
      />

      <!-- Grouped Profiles -->
      <div
        v-for="groupData in filteredGroupsData"
        :key="groupData.group?.id || 'ungrouped'"
        class="space-y-2"
      >
        <!-- Group Header -->
        <div class="flex items-center justify-between">
          <div class="flex items-center space-x-2">
            <div
              v-if="groupData.group"
              class="w-3 h-3 rounded-full"
              :style="{ backgroundColor: groupData.group.color || '#6b7280' }"
            ></div>
            <h3
              class="text-sm font-medium"
              :class="groupData.group ? 'text-white' : 'text-gray-400'"
            >
              {{ groupData.group?.name || "Ungrouped" }}
            </h3>
            <span class="text-xs text-gray-400">
              ({{ groupData.profileCount }})
            </span>
          </div>
          <div
            v-if="groupData.group"
            class="flex items-center space-x-1 transition-opacity"
          >
            <Button
              title="Add profile to group"
              variant="ghost"
              size="sm"
              :icon="Plus"
              @click="addProfileToGroup(groupData.group!.id)"
            />
            <Button
              title="Edit group"
              variant="ghost"
              size="sm"
              :icon="Edit3"
              @click="editGroup(groupData.group!)"
            />
            <Button
              title="Delete group"
              variant="ghost"
              size="sm"
              :icon="Trash2"
              @click="confirmDeleteGroup(groupData.group!)"
            />
          </div>
        </div>

        <!-- Group Profiles -->
        <div class="space-y-1">
          <!-- Show message for empty groups -->
          <div
            v-if="groupData.profileCount === 0 && !searchQuery"
            class="p-3 text-gray-500 text-sm italic text-center border border-dashed border-gray-600 rounded-lg"
          >
            {{
              groupData.group
                ? "No profiles in this group. Click the + button above to add one."
                : "No ungrouped profiles available."
            }}
          </div>

          <!-- Profiles -->
          <SSHProfileItem
            v-for="profile in groupData.profiles"
            :key="profile.id"
            :profile="profile"
            :fallback-color="groupData.group?.color"
            @connect="connectToProfile"
            @edit="editProfile"
            @delete="deleteProfile"
          />
        </div>
      </div>

      <!-- No search results -->
      <div
        v-if="
          searchQuery &&
          filteredGroupsData.length === 0 &&
          filteredConfigHosts.length === 0
        "
        class="p-3 text-gray-500 text-sm italic text-center border border-dashed border-gray-600 rounded-lg wrap-anywhere"
      >
        <p class="text-sm">No results found matching "{{ searchQuery }}"</p>
      </div>
    </div>

    <!-- Divider -->
    <div
      v-if="filteredConfigHosts.length > 0"
      class="border-t border-gray-700 mt-4"
    />

    <!-- SSH Config Hosts Section -->
    <div class="space-y-4 p-4">
      <div v-if="filteredConfigHosts.length > 0" class="space-y-2">
        <div class="flex items-center justify-between">
          <div class="flex items-center space-x-2">
            <component :is="FileCode" class="w-4 h-4 text-blue-400" />
            <h3 class="text-sm font-medium text-blue-300">From .ssh/config</h3>
            <span class="text-xs text-gray-400">
              ({{ filteredConfigHosts.length }})
            </span>
          </div>
        </div>
      </div>

      <div class="space-y-1">
        <SSHConfigHostItem
          v-for="host in filteredConfigHosts"
          :key="host.name"
          :host="host"
          @connect="connectToConfigHost"
        />
      </div>
    </div>

    <!-- Footer -->
    <template #footer>
      <div class="flex justify-between items-center gap-2">
        <div class="flex gap-2">
          <Button
            variant="ghost"
            size="sm"
            :icon="FolderPlus"
            text="New Group"
            @click="openOverlay('ssh-group-modal')"
          />
        </div>

        <Button
          variant="warning"
          size="sm"
          :icon="Plus"
          text="New Profile"
          @click="createNewProfile"
        />
      </div>
    </template>
  </Drawer>
</template>

<script setup lang="ts">
import { ref, computed, onMounted } from "vue";
import type {
  SSHProfile,
  SSHGroup,
  SSHConfigHost,
  DeleteGroupAction,
} from "../../types/ssh";
import { requiresPassword } from "../../types/ssh";
import Drawer from "../ui/Drawer.vue";
import Form from "../ui/Form.vue";
import Input from "../ui/Input.vue";
import Button from "../ui/Button.vue";
import EmptyState from "../ui/EmptyState.vue";
import SSHProfileItem from "./SSHProfileItem.vue";
import SSHConfigHostItem from "./SSHConfigHostItem.vue";
import {
  Search,
  Server,
  FolderPlus,
  Plus,
  Edit3,
  Trash2,
  FileCode,
} from "lucide-vue-next";
import { useOverlay } from "../../composables/useOverlay";
import { useSSHStore } from "../../stores/ssh";
import { useWorkspaceStore } from "../../stores/workspace";
import { caseInsensitiveIncludes } from "../../utils/helpers";
import { showConfirm } from "../../utils/message";

const searchQuery = ref("");

const { openOverlay, closeOverlay } = useOverlay();
const sshStore = useSSHStore();
const workspaceStore = useWorkspaceStore();

onMounted(async () => {
  await sshStore.loadConfigHosts();
});

/**
 * Filter groups and profiles based on search query
 * Returns optimized data structure with filtered profiles
 */
const filteredGroupsData = computed(() => {
  const groupsWithProfilesData = sshStore.groupsWithProfiles.groupedData;
  const ungroupedData = sshStore.groupsWithProfiles.getUngroupedData();

  if (!searchQuery.value.trim()) {
    const allData = [];

    groupsWithProfilesData.forEach((groupData, groupId) => {
      if (groupId !== null) {
        allData.push(groupData);
      }
    });

    allData.push(ungroupedData);

    return allData;
  }

  const query = searchQuery.value.trim();
  const filteredData = [];

  groupsWithProfilesData.forEach((groupData, groupId) => {
    if (groupId === null) return;

    const filteredProfiles = groupData.profiles.filter(
      (profile: SSHProfile) =>
        caseInsensitiveIncludes(profile.name, query) ||
        caseInsensitiveIncludes(profile.host, query) ||
        caseInsensitiveIncludes(profile.username, query) ||
        caseInsensitiveIncludes(`${profile.username}@${profile.host}`, query),
    );

    if (filteredProfiles.length > 0) {
      filteredData.push({
        ...groupData,
        profiles: filteredProfiles,
        profileCount: filteredProfiles.length,
      });
    }
  });

  const filteredUngroupedProfiles = ungroupedData.profiles.filter(
    (profile: SSHProfile) =>
      caseInsensitiveIncludes(profile.name, query) ||
      caseInsensitiveIncludes(profile.host, query) ||
      caseInsensitiveIncludes(profile.username, query) ||
      caseInsensitiveIncludes(`${profile.username}@${profile.host}`, query),
  );

  if (filteredUngroupedProfiles.length > 0) {
    filteredData.push({
      ...ungroupedData,
      profiles: filteredUngroupedProfiles,
      profileCount: filteredUngroupedProfiles.length,
    });
  }

  return filteredData;
});

/**
 * Filter SSH config hosts based on search query
 */
const filteredConfigHosts = computed(() => {
  if (!searchQuery.value.trim()) {
    return sshStore.configHosts;
  }

  const query = searchQuery.value.trim();
  return sshStore.configHosts.filter(
    (host: SSHConfigHost) =>
      caseInsensitiveIncludes(host.name, query) ||
      caseInsensitiveIncludes(host.hostname, query) ||
      (host.user && caseInsensitiveIncludes(host.user, query)) ||
      (host.user &&
        caseInsensitiveIncludes(`${host.user}@${host.hostname}`, query)),
  );
});

const createNewProfile = () => {
  openOverlay("ssh-profile-modal");
};

const connectToProfile = (profile: SSHProfile) => {
  const activePanelId = workspaceStore.activePanelId || "panel-1";

  workspaceStore.addSSHTab(activePanelId, profile.id, profile.name);

  closeOverlay("ssh-profile-drawer");
};

const editProfile = (profile: SSHProfile) => {
  openOverlay("ssh-profile-modal", { sshProfileId: profile.id });
};

const deleteProfile = async (profile: SSHProfile) => {
  try {
    await sshStore.deleteProfile(profile.id);
  } catch (error) {
    console.error("Failed to delete profile:", error);
  }
};

const addProfileToGroup = (groupId: string) => {
  openOverlay("ssh-profile-modal", { groupId });
};

const editGroup = (group: SSHGroup) => {
  openOverlay("ssh-group-modal", { sshGroupId: group.id });
};

const confirmDeleteGroup = async (group: SSHGroup) => {
  const confirmed = await showConfirm(
    "Delete Group",
    `Delete group '${group.name}'? Profiles in this group will be moved to ungrouped.`,
  );
  if (confirmed) {
    deleteGroup(group);
  }
};

const deleteGroup = async (group: SSHGroup) => {
  try {
    const action: DeleteGroupAction = { actionType: "moveToUngrouped" };
    await sshStore.deleteGroup(group.id, action);
  } catch (error) {
    console.error("Failed to delete group:", error);
  }
};

const connectToConfigHost = async (host: SSHConfigHost) => {
  try {
    const activePanelId = workspaceStore.activePanelId || "panel-1";
    const displayName = host.user
      ? `${host.user}@${host.hostname}`
      : host.hostname;

    if (requiresPassword(host)) {
      await openOverlay("ssh-config-password-modal", {
        host,
        onConnect: async (password: string) => {
          workspaceStore.addSSHConfigTab(
            activePanelId,
            host.name,
            displayName,
            password,
          );
        },
      });
    } else {
      workspaceStore.addSSHConfigTab(activePanelId, host.name, displayName);
      closeOverlay("ssh-profile-drawer");
    }
  } catch (error) {
    console.error("Failed to connect to SSH config host:", error);
  }
};
</script>
