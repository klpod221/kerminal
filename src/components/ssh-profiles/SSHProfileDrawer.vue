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
              @click="addProfileToGroup(groupData.group.id)"
            />
            <Button
              title="Edit group"
              variant="ghost"
              size="sm"
              :icon="Edit3"
              @click="editGroup(groupData.group)"
            />
            <Button
              title="Delete group"
              variant="ghost"
              size="sm"
              :icon="Trash2"
              @click="confirmDeleteGroup(groupData.group)"
            />
          </div>
        </div>

        <!-- Group Profiles -->
        <div class="space-y-1" :class="groupData.group ? 'ml-5' : ''">
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
        class="p-4 text-center text-gray-500"
      >
        <p class="text-sm">No profiles found matching "{{ searchQuery }}"</p>
      </div>
    </div>

    <!-- Footer -->
    <template #footer>
      <div class="flex justify-between items-center">
        <Button
          variant="ghost"
          size="sm"
          :icon="FolderPlus"
          text="Create Group"
          @click="openOverlay('ssh-group-modal')"
        />

        <Button
          variant="warning"
          size="sm"
          :icon="Plus"
          text="New Profile"
          @click="createNewProfile"
        />
      </div>
    </template>

    <!-- Delete Group Confirmation Modal -->
    <Modal
      v-if="deleteGroupState.isVisible"
      id="delete-group-modal"
      :title="`Delete Group '${deleteGroupState.group?.name}'`"
      size="md"
      @close="cancelDeleteGroup"
    >
      <div class="space-y-4">
        <p class="text-gray-300">
          This group contains <strong>{{ deleteGroupState.profileCount }}</strong> profile(s).
          What would you like to do with them?
        </p>

        <div class="space-y-3">
          <label class="flex items-center space-x-3 cursor-pointer">
            <input
              type="radio"
              v-model="deleteGroupState.action"
              value="moveToUngrouped"
              class="w-4 h-4 text-blue-500"
            />
            <span class="text-gray-200">Move profiles to "Ungrouped"</span>
          </label>

          <label class="flex items-center space-x-3 cursor-pointer">
            <input
              type="radio"
              v-model="deleteGroupState.action"
              value="moveToGroup"
              class="w-4 h-4 text-blue-500"
            />
            <span class="text-gray-200">Move profiles to another group:</span>
          </label>

          <div v-if="deleteGroupState.action === 'moveToGroup'" class="ml-7">
            <select
              v-model="deleteGroupState.targetGroupId"
              class="w-full px-3 py-2 bg-gray-700 border border-gray-600 rounded-lg text-gray-200"
            >
              <option value="">Select a group...</option>
              <option
                v-for="group in availableGroupsForMove"
                :key="group.id"
                :value="group.id"
              >
                {{ group.name }}
              </option>
            </select>
          </div>

          <label class="flex items-center space-x-3 cursor-pointer">
            <input
              type="radio"
              v-model="deleteGroupState.action"
              value="deleteProfiles"
              class="w-4 h-4 text-red-500"
            />
            <span class="text-red-400">Delete all profiles in this group</span>
          </label>
        </div>

        <div class="bg-yellow-500/10 border border-yellow-500/20 rounded-lg p-3">
          <p class="text-yellow-400 text-sm">
            <strong>Warning:</strong> This action cannot be undone.
          </p>
        </div>
      </div>

      <template #footer>
        <div class="flex justify-end space-x-2">
          <Button
            variant="secondary"
            @click="cancelDeleteGroup"
          >
            Cancel
          </Button>
          <Button
            variant="danger"
            :disabled="deleteGroupState.action === 'moveToGroup' && !deleteGroupState.targetGroupId"
            :loading="deleteGroupState.isDeleting"
            @click="confirmDeleteGroupAction"
          >
            Delete Group
          </Button>
        </div>
      </template>
    </Modal>
  </Drawer>
</template>

<script setup lang="ts">
import { ref, computed } from "vue";
import type { SSHProfile, SSHGroup, DeleteGroupAction } from "../../types/ssh";
import Drawer from "../ui/Drawer.vue";
import Form from "../ui/Form.vue";
import Input from "../ui/Input.vue";
import Button from "../ui/Button.vue";
import Modal from "../ui/Modal.vue";
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
const { openOverlay } = useOverlay();
const sshStore = useSSHStore();
const workspaceStore = useWorkspaceStore();

// Delete group confirmation state
const deleteGroupState = ref({
  isVisible: false,
  group: null as SSHGroup | null,
  profileCount: 0,
  action: "moveToUngrouped" as "moveToUngrouped" | "moveToGroup" | "deleteProfiles",
  targetGroupId: "",
  isDeleting: false,
});

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

// Available groups for moving profiles (exclude the group being deleted)
const availableGroupsForMove = computed(() => {
  return sshStore.groups.filter(group => group.id !== deleteGroupState.value.group?.id);
});

// Delete group confirmation functions
const confirmDeleteGroup = (group: SSHGroup) => {
  const groupData = sshStore.groupsWithProfiles.getGroupWithProfiles(group.id);
  const profileCount = groupData?.profileCount || 0;
  
  deleteGroupState.value = {
    isVisible: true,
    group,
    profileCount,
    action: "moveToUngrouped",
    targetGroupId: "",
    isDeleting: false,
  };
};

const cancelDeleteGroup = () => {
  deleteGroupState.value.isVisible = false;
  deleteGroupState.value.group = null;
  deleteGroupState.value.action = "moveToUngrouped";
  deleteGroupState.value.targetGroupId = "";
  deleteGroupState.value.isDeleting = false;
};

const confirmDeleteGroupAction = async () => {
  if (!deleteGroupState.value.group) return;

  deleteGroupState.value.isDeleting = true;
  try {
    let action: DeleteGroupAction;
    
    switch (deleteGroupState.value.action) {
      case "moveToUngrouped":
        action = { actionType: "moveToUngrouped" };
        break;
      case "moveToGroup":
        if (!deleteGroupState.value.targetGroupId) {
          console.error('Target group ID is required for moveToGroup action');
          return;
        }
        action = { actionType: "moveToGroup", targetGroupId: deleteGroupState.value.targetGroupId };
        break;
      case "deleteProfiles":
        action = { actionType: "deleteProfiles" };
        break;
      default:
        action = { actionType: "moveToUngrouped" };
    }

    await sshStore.deleteGroup(deleteGroupState.value.group.id, action);
    cancelDeleteGroup();
  } catch (error) {
    console.error('Failed to delete group:', error);
  } finally {
    deleteGroupState.value.isDeleting = false;
  }
};
</script>
