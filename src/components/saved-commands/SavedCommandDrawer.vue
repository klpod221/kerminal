<template>
  <Drawer
    id="saved-command-drawer"
    title="Saved Commands"
    position="right"
    :icon="Terminal"
    icon-background="bg-green-500/20"
    icon-color="text-green-400"
  >
    <template #headerAction>
      <Form>
        <!-- Search -->
        <Input
          id="search-saved-commands"
          v-model="searchQuery"
          type="text"
          placeholder="Search commands..."
          :left-icon="Search"
          :helper="false"
        />
      </Form>
    </template>

    <div
      class="p-4"
      v-if="!savedCommandStore.hasData"
    >
      <div class="text-center">
        <Terminal :size="48" class="mx-auto text-gray-500 mb-4" />
        <p class="text-sm text-gray-500 mb-4">No saved commands available.</p>
        <Button
          variant="outline"
          size="sm"
          :icon="Plus"
          @click="createNewCommand()"
        >
          Create Your First Command
        </Button>
      </div>
    </div>

    <div v-else class="space-y-4 p-4">
      <!-- Quick Actions Bar -->
      <div class="flex items-center justify-between pb-2 border-b border-gray-700">
        <div class="flex items-center space-x-2">
          <Button
            variant="ghost"
            size="sm"
            :icon="Plus"
            @click="createNewCommand()"
          >
            Command
          </Button>
          <Button
            variant="ghost"
            size="sm"
            :icon="FolderPlus"
            @click="createNewGroup()"
          >
            Group
          </Button>
        </div>

        <div class="flex items-center space-x-2">
          <!-- Filter dropdown -->
          <select
            v-model="activeFilter"
            class="px-2 py-1 bg-[#1a1a1a] border border-gray-600 rounded text-sm text-white focus:outline-none focus:ring-1 focus:ring-blue-500"
          >
            <option value="all">All</option>
            <option value="favorites">Favorites</option>
            <option value="recent">Recent</option>
            <option value="unused">Unused</option>
          </select>

          <!-- Sort dropdown -->
          <select
            v-model="sortBy"
            class="px-2 py-1 bg-[#1a1a1a] border border-gray-600 rounded text-sm text-white focus:outline-none focus:ring-1 focus:ring-blue-500"
          >
            <option value="name">Name</option>
            <option value="lastUsed">Last Used</option>
            <option value="usageCount">Usage Count</option>
            <option value="createdAt">Created</option>
          </select>
        </div>
      </div>

      <!-- Stats Bar -->
      <div class="flex items-center space-x-4 text-xs text-gray-400">
        <span>{{ savedCommandStore.commandCount }} commands</span>
        <span>{{ savedCommandStore.groupCount }} groups</span>
        <span v-if="savedCommandStore.favoriteCount > 0">
          {{ savedCommandStore.favoriteCount }} favorites
        </span>
      </div>

      <!-- Grouped Commands -->
      <div
        v-for="groupData in filteredGroupsData"
        :key="groupData.group?.base.id || 'ungrouped'"
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
            <component
              v-if="groupData.group?.icon"
              :is="getIconComponent(groupData.group.icon)"
              :size="14"
              class="text-gray-400"
            />
            <h3 class="text-sm font-medium" :class="groupData.group ? 'text-white' : 'text-gray-400'">
              {{ groupData.group?.name || 'Ungrouped' }}
            </h3>
            <span class="text-xs text-gray-400">
              ({{ groupData.commandCount }})
            </span>
          </div>
          <div
            v-if="groupData.group"
            class="flex items-center space-x-1 transition-opacity"
          >
            <Button
              title="Add command to group"
              variant="ghost"
              size="sm"
              :icon="Plus"
              @click="createNewCommand(groupData.group!.base.id)"
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

        <!-- Group Commands -->
        <div class="space-y-1">
          <!-- Show message for empty groups -->
          <div
            v-if="groupData.commandCount === 0 && !searchQuery"
            class="p-3 text-gray-500 text-sm italic text-center border border-dashed border-gray-600 rounded-lg"
          >
            {{ groupData.group ? 'No commands in this group. Click the + button above to add one.' : 'No ungrouped commands available.' }}
          </div>

          <!-- Commands -->
          <SavedCommandItem
            v-for="command in groupData.commands"
            :key="command.base.id"
            :command="command"
            :fallback-color="groupData.group?.color"
            @execute="executeCommand"
            @copy="copyCommand"
            @toggle-favorite="toggleFavorite"
            @edit="editCommand"
            @delete="deleteCommand"
          />
        </div>
      </div>
    </div>

    <!-- Modals -->
    <SavedCommandModal
      modal-id="saved-command-modal"
      :command="editingCommand"
      :groups="savedCommandStore.groups"
      :default-group-id="defaultGroupId"
      @success="handleCommandSaved"
      @error="handleError"
    />

    <SavedCommandGroupModal
      modal-id="saved-command-group-modal"
      :group="editingGroup"
      @success="handleGroupSaved"
      @error="handleError"
    />
  </Drawer>
</template>

<script setup lang="ts">
import { ref, computed, onMounted, watch } from "vue";
import {
  Terminal,
  Search,
  Plus,
  FolderPlus,
  Edit3,
  Trash2,
  Folder,
  Server,
  Database,
  Settings,
  Code,
  GitBranch,
  Package,
  Shield,
  Zap,
  Wrench,
  Monitor
} from "lucide-vue-next";
import Drawer from "../ui/Drawer.vue";
import Form from "../ui/Form.vue";
import Input from "../ui/Input.vue";
import Button from "../ui/Button.vue";
import SavedCommandItem from "./SavedCommandItem.vue";
import SavedCommandModal from "./SavedCommandModal.vue";
import SavedCommandGroupModal from "./SavedCommandGroupModal.vue";


import { useOverlay } from "../../composables/useOverlay";
import { useSavedCommandStore } from "../../stores/savedCommand";
import { message } from "../../utils/message";
import type { SavedCommand, SavedCommandGroup, SavedCommandSearchParams } from "../../types/savedCommand";

const { openOverlay } = useOverlay();
const savedCommandStore = useSavedCommandStore();
const showSuccess = (msg: string) => message.success(msg);
const showError = (msg: string) => message.error(msg);

// State
const searchQuery = ref("");
const activeFilter = ref<"all" | "favorites" | "recent" | "unused">("all");
const sortBy = ref<"name" | "lastUsed" | "usageCount" | "createdAt">("name");
const editingCommand = ref<SavedCommand | undefined>();
const editingGroup = ref<SavedCommandGroup | undefined>();
const defaultGroupId = ref<string>("");

// Icon mapping
const iconComponents: Record<string, any> = {
  folder: Folder,
  terminal: Terminal,
  server: Server,
  database: Database,
  settings: Settings,
  code: Code,
  "git-branch": GitBranch,
  package: Package,
  shield: Shield,
  zap: Zap,
  wrench: Wrench,
  monitor: Monitor,
};

// Computed
const filteredGroupsData = computed(() => {
  const searchParams: SavedCommandSearchParams = {
    query: searchQuery.value,
    filterBy: activeFilter.value,
    sortBy: sortBy.value,
    sortOrder: "asc",
  };

  // Get filtered commands
  const filteredCommands = savedCommandStore.filterCommands(searchParams);

  // Group the filtered commands
  return savedCommandStore.getGroupedCommandsData(searchQuery.value).map(groupData => ({
    ...groupData,
    commands: groupData.commands.filter(command =>
      filteredCommands.some(fc => fc.base.id === command.base.id)
    ),
    commandCount: groupData.commands.filter(command =>
      filteredCommands.some(fc => fc.base.id === command.base.id)
    ).length,
  })).filter(groupData =>
    groupData.commandCount > 0 || (!searchQuery.value && groupData.group)
  );
});

// Methods
const getIconComponent = (iconName: string) => {
  return iconComponents[iconName] || Terminal;
};

const createNewCommand = (groupId?: string) => {
  editingCommand.value = undefined;
  defaultGroupId.value = groupId || "";
  openOverlay("saved-command-modal");
};

const createNewGroup = () => {
  editingGroup.value = undefined;
  openOverlay("saved-command-group-modal");
};

const editCommand = (command: SavedCommand) => {
  editingCommand.value = command;
  defaultGroupId.value = "";
  openOverlay("saved-command-modal");
};

const editGroup = (group: SavedCommandGroup) => {
  editingGroup.value = group;
  openOverlay("saved-command-group-modal");
};

const executeCommand = async (command: SavedCommand) => {
  try {
    await savedCommandStore.executeCommand(command.base.id);
    showSuccess(`Executed: ${command.name}`);
  } catch (error) {
    console.error("Failed to execute command:", error);
    showError("Failed to execute command");
  }
};

const copyCommand = async (command: SavedCommand) => {
  try {
    await navigator.clipboard.writeText(command.command);
    showSuccess("Command copied to clipboard");
  } catch (error) {
    console.error("Failed to copy command:", error);
    showError("Failed to copy command");
  }
};

const toggleFavorite = async (command: SavedCommand) => {
  try {
    await savedCommandStore.toggleFavorite(command.base.id);
    showSuccess(
      command.isFavorite
        ? "Removed from favorites"
        : "Added to favorites"
    );
  } catch (error) {
    console.error("Failed to toggle favorite:", error);
    showError("Failed to update favorite status");
  }
};

const deleteCommand = async (command: SavedCommand) => {
  try {
    await savedCommandStore.deleteCommand(command.base.id);
    showSuccess("Command deleted successfully");
  } catch (error) {
    console.error("Failed to delete command:", error);
    showError("Failed to delete command");
  }
};

const confirmDeleteGroup = (group: SavedCommandGroup) => {
  // Use PopConfirm component inline for group deletion
  if (confirm(`Delete group '${group.name}'? Commands in this group will be moved to ungrouped.`)) {
    deleteGroup(group);
  }
};

const deleteGroup = async (group: SavedCommandGroup) => {
  try {
    await savedCommandStore.deleteGroup(group.base.id);
    showSuccess("Group deleted successfully");
  } catch (error) {
    console.error("Failed to delete group:", error);
    showError("Failed to delete group");
  }
};

const handleCommandSaved = () => {
  showSuccess(
    editingCommand.value
      ? "Command updated successfully"
      : "Command created successfully"
  );
  editingCommand.value = undefined;
  defaultGroupId.value = "";
};

const handleGroupSaved = () => {
  showSuccess(
    editingGroup.value
      ? "Group updated successfully"
      : "Group created successfully"
  );
  editingGroup.value = undefined;
};

const handleError = (error: string) => {
  showError(error);
};

// Load data on mount
onMounted(async () => {
  try {
    await savedCommandStore.loadAll();
  } catch (error) {
    console.error("Failed to load saved commands:", error);
    showError("Failed to load saved commands");
  }
});

// Watch for overlay visibility to refresh data
watch(() => savedCommandStore.loading, (loading) => {
  if (!loading && savedCommandStore.error) {
    showError(savedCommandStore.error);
  }
});
</script>
