import { defineStore } from "pinia";
import { ref, computed } from "vue";
import type {
  SavedCommand,
  SavedCommandGroup,
  CreateSavedCommandRequest,
  UpdateSavedCommandRequest,
  CreateSavedCommandGroupRequest,
  UpdateSavedCommandGroupRequest,
  GroupedSavedCommandsData,
  SavedCommandSearchParams,
} from "../types/savedCommand";
import { savedCommandService } from "../services/savedCommand";
import { writeToTerminal } from "../services/terminal";
import { useWorkspaceStore } from "./workspace";
import {
  safeJsonParse,
  caseInsensitiveIncludes,
  getCurrentTimestamp
} from "../utils/helpers";

export const useSavedCommandStore = defineStore("savedCommand", () => {
  // State
  const commands = ref<SavedCommand[]>([]);
  const groups = ref<SavedCommandGroup[]>([]);
  const loading = ref(false);
  const error = ref<string | null>(null);

  // Computed
  const favoriteCommands = computed(() =>
    commands.value.filter(c => c && c.id && c.isFavorite)
  );

  const recentCommands = computed(() =>
    commands.value
      .filter(c => c && c.id && c.lastUsedAt)
      .sort((a, b) => new Date(b.lastUsedAt!).getTime() - new Date(a.lastUsedAt!).getTime())
      .slice(0, 10)
  );

  const popularCommands = computed(() =>
    commands.value
      .filter(c => c && c.id && c.usageCount > 0)
      .sort((a, b) => b.usageCount - a.usageCount)
      .slice(0, 10)
  );

  const commandCount = computed(() => commands.value.length);
  const groupCount = computed(() => groups.value.length);
  const favoriteCount = computed(() => favoriteCommands.value.length);

  const hasData = computed(() => commands.value.length > 0 || groups.value.length > 0);

  // Actions
  const loadCommands = async () => {
    loading.value = true;
    error.value = null;

    try {
      const loadedCommands = await savedCommandService.getCommands();
      commands.value = (loadedCommands || []).filter(command =>
        command &&
        command.id &&
        typeof command.id === 'string'
      );
    } catch (err) {
      error.value = err instanceof Error ? err.message : 'Failed to load saved commands';
      console.error('Failed to load saved commands:', err);
      commands.value = [];
    } finally {
      loading.value = false;
    }
  };

  const loadGroups = async () => {
    loading.value = true;
    error.value = null;

    try {
      const loadedGroups = await savedCommandService.getGroups();
      groups.value = (loadedGroups || []).filter(group =>
        group &&
        group.id &&
        typeof group.id === 'string'
      );
    } catch (err) {
      error.value = err instanceof Error ? err.message : 'Failed to load saved command groups';
      console.error('Failed to load saved command groups:', err);
      groups.value = [];
    } finally {
      loading.value = false;
    }
  };

  const loadAll = async () => {
    await Promise.all([loadCommands(), loadGroups()]);
  };

  const createCommand = async (request: CreateSavedCommandRequest): Promise<SavedCommand> => {
    loading.value = true;
    error.value = null;

    try {
      const command = await savedCommandService.createCommand(request);
      await loadCommands(); // Reload to get updated list
      return command;
    } catch (err) {
      error.value = err instanceof Error ? err.message : 'Failed to create saved command';
      throw err;
    } finally {
      loading.value = false;
    }
  };

  const updateCommand = async (id: string, request: UpdateSavedCommandRequest): Promise<SavedCommand> => {
    loading.value = true;
    error.value = null;

    try {
      const command = await savedCommandService.updateCommand(id, request);
      await loadCommands(); // Reload to get updated list
      return command;
    } catch (err) {
      error.value = err instanceof Error ? err.message : 'Failed to update saved command';
      throw err;
    } finally {
      loading.value = false;
    }
  };

  const deleteCommand = async (id: string): Promise<void> => {
    loading.value = true;
    error.value = null;

    try {
      await savedCommandService.deleteCommand(id);
      await loadCommands(); // Reload to get updated list
    } catch (err) {
      error.value = err instanceof Error ? err.message : 'Failed to delete saved command';
      throw err;
    } finally {
      loading.value = false;
    }
  };

  const executeCommand = async (id: string, terminalId?: string): Promise<void> => {
    try {
      // Find the command
      const command = commands.value.find(c => c.id === id);
      if (!command) {
        throw new Error('Command not found');
      }

      // Get workspace store to find active terminal if terminalId not provided
      let targetTerminalId = terminalId;
      if (!targetTerminalId) {
        const workspaceStore = useWorkspaceStore();

        if (workspaceStore.activePanelId) {
          const activePanel = workspaceStore.findPanelInLayout(workspaceStore.activePanelId);
          if (activePanel?.activeTabId) {
            // Tab ID is the same as Terminal ID
            targetTerminalId = activePanel.activeTabId;
          }
        }
      }

      if (!targetTerminalId) {
        throw new Error('No active terminal found');
      }

      // Check if terminal exists and is ready
      const workspaceStore = useWorkspaceStore();
      const terminal = workspaceStore.terminals.find(t => t.id === targetTerminalId);

      if (!terminal) {
        throw new Error('Terminal not found');
      }

      if (!terminal.ready) {
        throw new Error('Terminal is not ready');
      }

      if (!terminal.backendTerminalId) {
        throw new Error('Terminal backend ID not available');
      }

      // Execute the command in terminal using backend terminal ID
      await writeToTerminal({
        terminalId: terminal.backendTerminalId,
        data: command.command + '\n', // Add newline to execute
      });

      // Increment usage count
      await savedCommandService.incrementUsage(id);

      // Update local state without full reload for better UX
      const commandIndex = commands.value.findIndex(c => c.id === id);
      if (commandIndex !== -1) {
        commands.value[commandIndex].usageCount += 1;
        commands.value[commandIndex].lastUsedAt = getCurrentTimestamp();
      }
    } catch (err) {
      console.error('Failed to execute command:', err);
      error.value = err instanceof Error ? err.message : 'Failed to execute command';
      throw err;
    }
  };

  const toggleFavorite = async (id: string): Promise<void> => {
    loading.value = true;
    error.value = null;

    try {
      const updatedCommand = await savedCommandService.toggleFavorite(id);
      // Update local state
      const commandIndex = commands.value.findIndex(c => c.id === id);
      if (commandIndex !== -1) {
        commands.value[commandIndex] = updatedCommand;
      }
    } catch (err) {
      error.value = err instanceof Error ? err.message : 'Failed to toggle favorite';
      throw err;
    } finally {
      loading.value = false;
    }
  };

  const createGroup = async (request: CreateSavedCommandGroupRequest): Promise<SavedCommandGroup> => {
    loading.value = true;
    error.value = null;

    try {
      const group = await savedCommandService.createGroup(request);
      await loadGroups(); // Reload to get updated list
      return group;
    } catch (err) {
      error.value = err instanceof Error ? err.message : 'Failed to create saved command group';
      throw err;
    } finally {
      loading.value = false;
    }
  };

  const updateGroup = async (id: string, request: UpdateSavedCommandGroupRequest): Promise<SavedCommandGroup> => {
    loading.value = true;
    error.value = null;

    try {
      const group = await savedCommandService.updateGroup(id, request);
      await loadGroups(); // Reload to get updated list
      return group;
    } catch (err) {
      error.value = err instanceof Error ? err.message : 'Failed to update saved command group';
      throw err;
    } finally {
      loading.value = false;
    }
  };

  const deleteGroup = async (id: string): Promise<void> => {
    loading.value = true;
    error.value = null;

    try {
      await savedCommandService.deleteGroup(id);
      await loadAll(); // Reload both commands and groups
    } catch (err) {
      error.value = err instanceof Error ? err.message : 'Failed to delete saved command group';
      throw err;
    } finally {
      loading.value = false;
    }
  };

  // Helper methods for UI
  const getGroupedCommandsData = (searchQuery?: string): GroupedSavedCommandsData[] => {
    const filteredCommands = searchQuery
      ? commands.value.filter(command =>
          caseInsensitiveIncludes(command.name, searchQuery) ||
          caseInsensitiveIncludes(command.command, searchQuery) ||
          caseInsensitiveIncludes(command.description, searchQuery) ||
          caseInsensitiveIncludes(command.tags, searchQuery)
        )
      : commands.value;

    // Group commands by group
    const groupedMap = new Map<string, SavedCommand[]>();
    const ungroupedCommands: SavedCommand[] = [];

    filteredCommands.forEach(command => {
      if (command.groupId) {
        if (!groupedMap.has(command.groupId)) {
          groupedMap.set(command.groupId, []);
        }
        groupedMap.get(command.groupId)!.push(command);
      } else {
        ungroupedCommands.push(command);
      }
    });

    const result: GroupedSavedCommandsData[] = [];

    // Add groups with commands
    groups.value.forEach(group => {
      const groupCommands = groupedMap.get(group.id) || [];
      if (groupCommands.length > 0 || !searchQuery) { // Show empty groups when no search
        result.push({
          group,
          commands: groupCommands.sort((a, b) => a.name.localeCompare(b.name)),
          commandCount: groupCommands.length,
        });
      }
    });

    // Add ungrouped commands
    if (ungroupedCommands.length > 0 || (!searchQuery && groups.value.length === 0)) {
      result.push({
        commands: ungroupedCommands.sort((a, b) => a.name.localeCompare(b.name)),
        commandCount: ungroupedCommands.length,
      });
    }

    return result;
  };

  const filterCommands = (params: SavedCommandSearchParams): SavedCommand[] => {
    let filtered = [...commands.value];

    // Filter by query
    if (params.query) {
      filtered = filtered.filter(command =>
        caseInsensitiveIncludes(command.name, params.query!) ||
        caseInsensitiveIncludes(command.command, params.query!) ||
        caseInsensitiveIncludes(command.description, params.query!) ||
        caseInsensitiveIncludes(command.tags, params.query!)
      );
    }

    // Filter by group
    if (params.groupId) {
      filtered = filtered.filter(command => command.groupId === params.groupId);
    }

    // Filter by type
    if (params.filterBy) {
      switch (params.filterBy) {
        case 'favorites':
          filtered = filtered.filter(command => command.isFavorite);
          break;
        case 'recent':
          filtered = filtered.filter(command => command.lastUsedAt);
          break;
        case 'unused':
          filtered = filtered.filter(command => command.usageCount === 0);
          break;
      }
    }

    // Filter by tags
    if (params.tags && params.tags.length > 0) {
      filtered = filtered.filter(command => {
        if (!command.tags) return false;
        const commandTags = safeJsonParse<string[]>(command.tags, []);
        return params.tags!.some(tag => commandTags.includes(tag));
      });
    }

    // Sort
    if (params.sortBy) {
      filtered.sort((a, b) => {
        let comparison = 0;
        switch (params.sortBy) {
          case 'name':
            comparison = a.name.localeCompare(b.name);
            break;
          case 'lastUsed':
            const aDate = a.lastUsedAt ? new Date(a.lastUsedAt).getTime() : 0;
            const bDate = b.lastUsedAt ? new Date(b.lastUsedAt).getTime() : 0;
            comparison = bDate - aDate;
            break;
          case 'usageCount':
            comparison = b.usageCount - a.usageCount;
            break;
          case 'createdAt':
            comparison = new Date(b.createdAt).getTime() - new Date(a.createdAt).getTime();
            break;
          case 'updatedAt':
            comparison = new Date(b.updatedAt).getTime() - new Date(a.updatedAt).getTime();
            break;
        }
        return params.sortOrder === 'asc' ? comparison : -comparison;
      });
    }

    return filtered;
  };

  const getCommandById = (id: string): SavedCommand | undefined => {
    return commands.value.find(command => command.id === id);
  };

  const findCommandById = async (id: string): Promise<SavedCommand | undefined> => {
    // First try to find in loaded commands
    let command = getCommandById(id);
    if (command) return command;

    // If not found, reload and try again
    await loadCommands();
    return getCommandById(id);
  };

  const getGroupById = (id: string): SavedCommandGroup | undefined => {
    return groups.value.find(group => group.id === id);
  };

  const findGroupById = async (id: string): Promise<SavedCommandGroup | undefined> => {
    // First try to find in loaded groups
    let group = getGroupById(id);
    if (group) return group;

    // If not found, reload and try again
    await loadGroups();
    return getGroupById(id);
  };

  return {
    // State
    commands,
    groups,
    loading,
    error,

    // Computed
    favoriteCommands,
    recentCommands,
    popularCommands,
    commandCount,
    groupCount,
    favoriteCount,
    hasData,

    // Actions
    loadCommands,
    loadGroups,
    loadAll,
    createCommand,
    updateCommand,
    deleteCommand,
    executeCommand,
    toggleFavorite,
    createGroup,
    updateGroup,
    deleteGroup,

    // Helper methods
    getGroupedCommandsData,
    filterCommands,
    getCommandById,
    findCommandById,
    getGroupById,
    findGroupById,
  };
});
