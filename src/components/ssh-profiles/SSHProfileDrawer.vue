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

    <div
      class="p-4"
      v-if="!sshStore.hasData"
    >
      <p class="text-sm text-gray-500">No SSH profiles or groups available.</p>
    </div>

    <div v-else class="space-y-4 p-4">
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
            <h3 class="text-sm font-medium" :class="groupData.group ? 'text-white' : 'text-gray-400'">
              {{ groupData.group?.name || 'Ungrouped' }}
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
            {{ groupData.group ? 'No profiles in this group. Click the + button above to add one.' : 'No ungrouped profiles available.' }}
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
        v-if="searchQuery && filteredGroupsData.length === 0"
        class="p-3 text-gray-500 text-sm italic text-center border border-dashed border-gray-600 rounded-lg wrap-anywhere"
      >
        <p class="text-sm">No profiles found matching "{{ searchQuery }}"</p>
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
import { ref, computed } from "vue";
import type { SSHProfile, SSHGroup, DeleteGroupAction } from "../../types/ssh";
import Drawer from "../ui/Drawer.vue";
import Form from "../ui/Form.vue";
import Input from "../ui/Input.vue";
import Button from "../ui/Button.vue";
import SSHProfileItem from "./SSHProfileItem.vue";
import {
  Search,
  Server,
  FolderPlus,
  Plus,
  Edit3,
  Trash2,
} from "lucide-vue-next";
import { useOverlay } from "../../composables/useOverlay";
import { useSSHStore } from "../../stores/ssh";
import { useWorkspaceStore } from "../../stores/workspace";

// State
const searchQuery = ref("");

// Composables and store
const { openOverlay, closeOverlay } = useOverlay();
const sshStore = useSSHStore();
const workspaceStore = useWorkspaceStore();

/**
 * Filter groups and profiles based on search query
 * Returns optimized data structure with filtered profiles
 */
const filteredGroupsData = computed(() => {
  const groupsWithProfilesData = sshStore.groupsWithProfiles.groupedData;
  const ungroupedData = sshStore.groupsWithProfiles.getUngroupedData();

  // If no search query, return all data
  if (!searchQuery.value.trim()) {
    const allData = [];

    // Add all groups with their profiles
    groupsWithProfilesData.forEach((groupData, groupId) => {
      if (groupId !== null) { // Skip ungrouped here, will add separately
        allData.push(groupData);
      }
    });

    // Add ungrouped at the end
    allData.push(ungroupedData);

    return allData;
  }

  const query = searchQuery.value.toLowerCase().trim();
  const filteredData = [];

  // Filter grouped profiles
  groupsWithProfilesData.forEach((groupData, groupId) => {
    if (groupId === null) return; // Skip ungrouped here

    const filteredProfiles = groupData.profiles.filter((profile: SSHProfile) =>
      profile.name.toLowerCase().includes(query) ||
      profile.host.toLowerCase().includes(query) ||
      profile.username.toLowerCase().includes(query) ||
      `${profile.username}@${profile.host}`.toLowerCase().includes(query)
    );

    if (filteredProfiles.length > 0) {
      filteredData.push({
        ...groupData,
        profiles: filteredProfiles,
        profileCount: filteredProfiles.length
      });
    }
  });

  // Filter ungrouped profiles
  const filteredUngroupedProfiles = ungroupedData.profiles.filter((profile: SSHProfile) =>
    profile.name.toLowerCase().includes(query) ||
    profile.host.toLowerCase().includes(query) ||
    profile.username.toLowerCase().includes(query) ||
    `${profile.username}@${profile.host}`.toLowerCase().includes(query)
  );

  if (filteredUngroupedProfiles.length > 0) {
    filteredData.push({
      ...ungroupedData,
      profiles: filteredUngroupedProfiles,
      profileCount: filteredUngroupedProfiles.length
    });
  }

  return filteredData;
});

// Profile actions
const createNewProfile = () => {
  console.log('Creating new profile...');
  openOverlay('ssh-profile-modal');
};

const connectToProfile = (profile: SSHProfile) => {
  console.log('Connecting to:', profile.name);

  // Get the active panel ID from workspace store
  const activePanelId = workspaceStore.activePanelId || "panel-1";

  // Add SSH terminal tab
  workspaceStore.addSSHTab(activePanelId, profile.id, profile.name);

  // Close the drawer after connecting
  closeOverlay('ssh-profile-drawer');
};

const editProfile = (profile: SSHProfile) => {
  console.log('Editing profile:', profile.name);
  openOverlay('ssh-profile-modal', { sshProfileId: profile.id });
};

const deleteProfile = async (profile: SSHProfile) => {
  console.log('Deleting profile:', profile.name);
  try {
    await sshStore.deleteProfile(profile.id);
    // Note: No need to manually update state as store handles it
  } catch (error) {
    console.error('Failed to delete profile:', error);
  }
};

// Group actions
const addProfileToGroup = (groupId: string) => {
  console.log('Adding profile to group:', groupId);
  openOverlay('ssh-profile-modal', { groupId });
};

const editGroup = (group: SSHGroup) => {
  console.log('Editing group:', group.name);
  openOverlay('ssh-group-modal', { sshGroupId: group.id });
};

const confirmDeleteGroup = (group: SSHGroup) => {
  if (
    confirm(
      `Delete group '${group.name}'? Profiles in this group will be moved to ungrouped.`,
    )
  ) {
    deleteGroup(group);
  }
};

const deleteGroup = async (group: SSHGroup) => {
  try {
    const action: DeleteGroupAction = { actionType: "moveToUngrouped" };
    await sshStore.deleteGroup(group.id, action);
  } catch (error) {
    console.error('Failed to delete group:', error);
  }
};
</script>
