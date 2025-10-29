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

    <EmptyState
      v-if="!savedCommandStore.hasData"
      :icon="Terminal"
      title="No Saved Commands"
      description="Create your first saved command to get started."
      action-text="Create Your First Command"
      :action-icon="Plus"
      action-variant="outline"
      @action="createNewCommand()"
    />

    <div v-else class="space-y-4 p-4">
      <!-- Filter & Sort Bar -->
      <div class="space-y-2 pb-3 border-b border-gray-700">
        <div class="flex items-center justify-between">
          <!-- Filter & Sort Controls -->
          <Form class="grid grid-cols-2 gap-2 w-full">
            <Select
              id="filter-select"
              v-model="activeFilter"
              label="Filter"
              :options="[
                { value: 'all', label: 'All' },
                { value: 'favorites', label: 'Favorites' },
                { value: 'recent', label: 'Recent' },
                { value: 'unused', label: 'Unused' },
              ]"
              class="w-full"
              size="sm"
            />

            <Select
              id="sort-select"
              v-model="sortBy"
              label="Sort By"
              :options="[
                { value: 'name', label: 'Name' },
                { value: 'lastUsed', label: 'Last Used' },
                { value: 'usageCount', label: 'Usage Count' },
                { value: 'createdAt', label: 'Created Date' },
              ]"
              class="w-full"
              size="sm"
            />
          </Form>
        </div>

        <!-- Stats -->
        <div class="flex items-center gap-4 text-xs text-gray-400">
          <span
            >Showing {{ filteredCommandCount }} of
            {{ savedCommandStore.commandCount }} commands</span
          >
          <span v-if="activeFilter !== 'all'" class="text-blue-400">
            • Filtered by {{ activeFilterLabel }}
          </span>
        </div>
      </div>

      <!-- Grouped Commands -->
      <div
        v-for="groupData in filteredGroupsData"
        :key="groupData.group?.id || 'ungrouped'"
        class="space-y-2"
      >
        <!-- Group Header -->
        <div class="flex items-center justify-between">
          <div class="flex items-center space-x-2">
            <component
              v-if="groupData.group?.icon"
              :is="getIconComponent(groupData.group.icon)"
              :size="14"
              class="text-gray-400"
              :style="{ color: groupData.group.color || '#6b7280' }"
            />
            <div
              v-else
              class="w-3 h-3 rounded-full"
              :style="{ backgroundColor: groupData.group?.color || '#6b7280' }"
            ></div>
            <h3
              class="text-sm font-medium"
              :class="groupData.group ? 'text-white' : 'text-gray-400'"
            >
              {{ groupData.group?.name || "Ungrouped" }}
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
              @click="createNewCommand(groupData.group!.id)"
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
            {{
              groupData.group
                ? "No commands in this group. Click the + button above to add one."
                : "No ungrouped commands available."
            }}
          </div>

          <!-- Commands -->
          <SavedCommandItem
            v-for="command in groupData.commands"
            :key="command.id"
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

    <!-- Footer -->
    <template #footer>
      <div class="flex justify-between items-center gap-2">
        <div class="flex gap-2">
          <Button
            variant="ghost"
            size="sm"
            :icon="FolderPlus"
            text="New Group"
            @click="createNewGroup()"
          />
        </div>

        <Button
          variant="success"
          size="sm"
          :icon="Plus"
          text="New Command"
          @click="createNewCommand()"
        />
      </div>
    </template>
  </Drawer>

  <!-- Modals -->
  <SavedCommandModal
    modal-id="saved-command-modal"
    :groups="savedCommandStore.groups"
    @success="handleCommandSaved"
    @error="handleError"
  />

  <SavedCommandGroupModal
    modal-id="saved-command-group-modal"
    @success="handleGroupSaved"
    @error="handleError"
  />
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
  Monitor,
} from "lucide-vue-next";
import Drawer from "../ui/Drawer.vue";
import Form from "../ui/Form.vue";
import Input from "../ui/Input.vue";
import Select from "../ui/Select.vue";
import Button from "../ui/Button.vue";
import EmptyState from "../ui/EmptyState.vue";
import SavedCommandItem from "./SavedCommandItem.vue";
import SavedCommandModal from "./SavedCommandModal.vue";
import SavedCommandGroupModal from "./SavedCommandGroupModal.vue";

import { useOverlay } from "../../composables/useOverlay";
import { useSavedCommandStore } from "../../stores/savedCommand";
import { message } from "../../utils/message";
import type {
  SavedCommand,
  SavedCommandGroup,
  SavedCommandSearchParams,
} from "../../types/savedCommand";

const { openOverlay } = useOverlay();
const savedCommandStore = useSavedCommandStore();
const showSuccess = (msg: string) => message.success(msg);
const showError = (msg: string) => message.error(msg);

const searchQuery = ref("");
const activeFilter = ref<"all" | "favorites" | "recent" | "unused">("all");
const sortBy = ref<"name" | "lastUsed" | "usageCount" | "createdAt">("name");

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

const sortOrder = computed(() => {
  return sortBy.value === "name" ? "asc" : "desc";
});

const filteredCommands = computed(() => {
  const searchParams: SavedCommandSearchParams = {
    query: searchQuery.value,
    filterBy: activeFilter.value,
    sortBy: sortBy.value,
    sortOrder: sortOrder.value,
  };

  return savedCommandStore.filterCommands(searchParams);
});

const filteredCommandCount = computed(() => filteredCommands.value.length);

const activeFilterLabel = computed(() => {
  const labels: Record<string, string> = {
    favorites: "Favorites",
    recent: "Recent",
    unused: "Unused",
  };
  return labels[activeFilter.value] || "All";
});

const filteredGroupsData = computed(() => {
  const getSortValue = (command: SavedCommand): number | string => {
    switch (sortBy.value) {
      case "name":
        return command.name.toLowerCase();
      case "lastUsed":
        return command.lastUsedAt ? new Date(command.lastUsedAt).getTime() : 0;
      case "usageCount":
        return command.usageCount;
      case "createdAt":
        return new Date(command.createdAt).getTime();
      default:
        return command.name.toLowerCase();
    }
  };

  return savedCommandStore
    .getGroupedCommandsData(searchQuery.value)
    .map((groupData) => {
      const filteredGroupCommands = groupData.commands.filter((command) =>
        filteredCommands.value.some((fc) => fc.id === command.id),
      );

      const sortedCommands = [...filteredGroupCommands].sort((a, b) => {
        const aVal = getSortValue(a);
        const bVal = getSortValue(b);

        if (typeof aVal === "string" && typeof bVal === "string") {
          const comparison = aVal.localeCompare(bVal);
          return sortOrder.value === "asc" ? comparison : -comparison;
        }

        const comparison = (aVal as number) - (bVal as number);
        return sortOrder.value === "asc" ? comparison : -comparison;
      });

      return {
        ...groupData,
        commands: sortedCommands,
        commandCount: filteredGroupCommands.length,
      };
    })
    .filter(
      (groupData) =>
        groupData.commandCount > 0 || (!searchQuery.value && groupData.group),
    );
});

const getIconComponent = (iconName: string) => {
  return iconComponents[iconName] || Terminal;
};

const createNewCommand = (groupId?: string) => {
  openOverlay("saved-command-modal", {
    commandId: undefined,
    defaultGroupId: groupId,
  });
};

const createNewGroup = () => {
  openOverlay("saved-command-group-modal", { groupId: undefined });
};

const editCommand = (command: SavedCommand) => {
  openOverlay("saved-command-modal", {
    commandId: command.id,
  });
};

const editGroup = (group: SavedCommandGroup) => {
  openOverlay("saved-command-group-modal", {
    groupId: group.id,
  });
};

const executeCommand = async (command: SavedCommand) => {
  try {
    await savedCommandStore.executeCommand(command.id);
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
    await savedCommandStore.toggleFavorite(command.id);
    showSuccess(
      command.isFavorite ? "Removed from favorites" : "Added to favorites",
    );
  } catch (error) {
    console.error("Failed to toggle favorite:", error);
    showError("Failed to update favorite status");
  }
};

const deleteCommand = async (command: SavedCommand) => {
  try {
    await savedCommandStore.deleteCommand(command.id);
    showSuccess("Command deleted successfully");
  } catch (error) {
    console.error("Failed to delete command:", error);
    showError("Failed to delete command");
  }
};

const confirmDeleteGroup = (group: SavedCommandGroup) => {
  if (
    confirm(
      `Delete group '${group.name}'? Commands in this group will be moved to ungrouped.`,
    )
  ) {
    deleteGroup(group);
  }
};

const deleteGroup = async (group: SavedCommandGroup) => {
  try {
    await savedCommandStore.deleteGroup(group.id);
    showSuccess("Group deleted successfully");
  } catch (error) {
    console.error("Failed to delete group:", error);
    showError("Failed to delete group");
  }
};

const handleCommandSaved = () => {
  showSuccess("Command saved successfully");
};

const handleGroupSaved = () => {
  showSuccess("Group saved successfully");
};

const handleError = (error: string) => {
  showError(error);
};

onMounted(async () => {
  try {
    await savedCommandStore.loadAll();
  } catch (error) {
    console.error("Failed to load saved commands:", error);
    showError("Failed to load saved commands");
  }
});

watch(
  () => savedCommandStore.loading,
  (loading) => {
    if (!loading && savedCommandStore.error) {
      showError(savedCommandStore.error);
    }
  },
);
</script>
