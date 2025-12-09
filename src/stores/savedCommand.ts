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
import { api } from "../services/api";
import {
  safeJsonParse,
  caseInsensitiveIncludes,
  getCurrentTimestamp,
} from "../utils/helpers";
import { handleError, type ErrorContext } from "../utils/errorHandler";
import { message } from "../utils/message";

export const useSavedCommandStore = defineStore("savedCommand", () => {
  const commands = ref<SavedCommand[]>([]);
  const groups = ref<SavedCommandGroup[]>([]);
  const isLoading = ref(false);

  const favoriteCommands = computed(() =>
    commands.value.filter((c) => c?.id && c.isFavorite),
  );

  const recentCommands = computed(() =>
    commands.value
      .filter((c) => c?.id && c.lastUsedAt)
      .sort(
        (a, b) =>
          new Date(b.lastUsedAt!).getTime() - new Date(a.lastUsedAt!).getTime(),
      )
      .slice(0, 10),
  );

  const popularCommands = computed(() =>
    commands.value
      .filter((c) => c?.id && c.usageCount > 0)
      .sort((a, b) => b.usageCount - a.usageCount)
      .slice(0, 10),
  );

  const commandCount = computed(() => commands.value.length);
  const groupCount = computed(() => groups.value.length);
  const favoriteCount = computed(() => favoriteCommands.value.length);

  const hasData = computed(
    () => commands.value.length > 0 || groups.value.length > 0,
  );

  /**
   * Load all saved commands with error handling
   */
  const loadCommands = async () => {
    isLoading.value = true;

    const context: ErrorContext = {
      operation: "Load Saved Commands",
    };

    try {
      const loadedCommands = await savedCommandService.getCommands();
      commands.value = (loadedCommands || []).filter(
        (command) => command?.id && typeof command.id === "string",
      );
    } catch (err) {
      const errorMessage = handleError(err, context);
      message.error(errorMessage);
      commands.value = [];
    } finally {
      isLoading.value = false;
    }
  };

  /**
   * Load all saved command groups with error handling
   */
  const loadGroups = async () => {
    isLoading.value = true;

    const context: ErrorContext = {
      operation: "Load Saved Command Groups",
    };

    try {
      const loadedGroups = await savedCommandService.getGroups();
      groups.value = (loadedGroups || []).filter(
        (group) => group?.id && typeof group.id === "string",
      );
    } catch (err) {
      const errorMessage = handleError(err, context);
      message.error(errorMessage);
      groups.value = [];
    } finally {
      isLoading.value = false;
    }
  };

  const loadAll = async () => {
    await Promise.all([loadCommands(), loadGroups()]);
  };

  /**
   * Create a new saved command with error handling
   * @param request - Command creation request
   * @returns Created command
   */
  const createCommand = async (
    request: CreateSavedCommandRequest,
  ): Promise<SavedCommand> => {
    isLoading.value = true;

    const context: ErrorContext = {
      operation: "Create Saved Command",
      context: { name: request.name },
    };

    try {
      const command = await savedCommandService.createCommand(request);
      await loadCommands(); // Reload to get updated list
      return command;
    } catch (err) {
      const errorMessage = handleError(err, context);
      message.error(errorMessage);
      throw new Error(errorMessage);
    } finally {
      isLoading.value = false;
    }
  };

  /**
   * Update a saved command with error handling
   * @param id - Command ID to update
   * @param request - Update request
   * @returns Updated command
   */
  const updateCommand = async (
    id: string,
    request: UpdateSavedCommandRequest,
  ): Promise<SavedCommand> => {
    isLoading.value = true;

    const context: ErrorContext = {
      operation: "Update Saved Command",
      context: { commandId: id },
    };

    try {
      const command = await savedCommandService.updateCommand(id, request);
      await loadCommands(); // Reload to get updated list
      return command;
    } catch (err) {
      const errorMessage = handleError(err, context);
      message.error(errorMessage);
      throw new Error(errorMessage);
    } finally {
      isLoading.value = false;
    }
  };

  /**
   * Delete a saved command with error handling
   * @param id - Command ID to delete
   */
  const deleteCommand = async (id: string): Promise<void> => {
    isLoading.value = true;

    const context: ErrorContext = {
      operation: "Delete Saved Command",
      context: { commandId: id },
    };

    try {
      await savedCommandService.deleteCommand(id);
      await loadCommands(); // Reload to get updated list
    } catch (err) {
      const errorMessage = handleError(err, context);
      message.error(errorMessage);
      throw new Error(errorMessage);
    } finally {
      isLoading.value = false;
    }
  };

  /**
   * Execute a saved command in terminal with error handling
   * @param id - Command ID to execute
   * @param terminalId - Optional terminal ID, otherwise uses active terminal
   */
  const executeCommand = async (
    id: string,
    terminalId?: string,
  ): Promise<void> => {
    const context: ErrorContext = {
      operation: "Execute Saved Command",
      context: { commandId: id },
    };

    try {
      const command = commands.value.find((c) => c.id === id);
      if (!command) {
        throw new Error("Command not found");
      }

      let targetTerminalId = terminalId;
      if (!targetTerminalId) {
        const workspaceStore = useWorkspaceStore();

        if (workspaceStore.activePanelId) {
          const activePanel = workspaceStore.findPanelInLayout(
            workspaceStore.activePanelId,
          );
          if (activePanel?.activeTabId) {
            targetTerminalId = activePanel.activeTabId;
          }
        }
      }

      if (!targetTerminalId) {
        throw new Error("No active terminal found");
      }

      const workspaceStore = useWorkspaceStore();
      const terminal = workspaceStore.terminals.find(
        (t) => t.id === targetTerminalId,
      );

      if (!terminal) {
        throw new Error("Terminal not found");
      }

      if (!terminal.ready) {
        throw new Error("Terminal is not ready");
      }

      if (!terminal.backendTerminalId) {
        throw new Error("Terminal backend ID not available");
      }

      await writeToTerminal({
        terminalId: terminal.backendTerminalId,
        data: command.command + "\n", // Add newline to execute
      });

      await savedCommandService.incrementUsage(id);

      const commandIndex = commands.value.findIndex((c) => c.id === id);
      if (commandIndex !== -1) {
        commands.value[commandIndex].usageCount += 1;
        commands.value[commandIndex].lastUsedAt = getCurrentTimestamp();
      }
    } catch (err) {
      const errorMessage = handleError(err, context);
      message.error(errorMessage);
      throw new Error(errorMessage);
    }
  };

  /**
   * Toggle favorite status of a command with error handling
   * @param id - Command ID
   */
  const toggleFavorite = async (id: string): Promise<void> => {
    isLoading.value = true;

    const context: ErrorContext = {
      operation: "Toggle Favorite",
      context: { commandId: id },
    };

    try {
      const updatedCommand = await savedCommandService.toggleFavorite(id);
      const commandIndex = commands.value.findIndex((c) => c.id === id);
      if (commandIndex !== -1) {
        commands.value[commandIndex] = updatedCommand;
      }
    } catch (err) {
      const errorMessage = handleError(err, context);
      message.error(errorMessage);
      throw new Error(errorMessage);
    } finally {
      isLoading.value = false;
    }
  };

  /**
   * Create a saved command group with error handling
   * @param request - Group creation request
   * @returns Created group
   */
  const createGroup = async (
    request: CreateSavedCommandGroupRequest,
  ): Promise<SavedCommandGroup> => {
    isLoading.value = true;

    const context: ErrorContext = {
      operation: "Create Saved Command Group",
      context: { name: request.name },
    };

    try {
      const group = await savedCommandService.createGroup(request);
      await loadGroups(); // Reload to get updated list
      return group;
    } catch (err) {
      const errorMessage = handleError(err, context);
      message.error(errorMessage);
      throw new Error(errorMessage);
    } finally {
      isLoading.value = false;
    }
  };

  /**
   * Update a saved command group with error handling
   * @param id - Group ID to update
   * @param request - Update request
   * @returns Updated group
   */
  const updateGroup = async (
    id: string,
    request: UpdateSavedCommandGroupRequest,
  ): Promise<SavedCommandGroup> => {
    isLoading.value = true;

    const context: ErrorContext = {
      operation: "Update Saved Command Group",
      context: { groupId: id },
    };

    try {
      const group = await savedCommandService.updateGroup(id, request);
      await loadGroups(); // Reload to get updated list
      return group;
    } catch (err) {
      const errorMessage = handleError(err, context);
      message.error(errorMessage);
      throw new Error(errorMessage);
    } finally {
      isLoading.value = false;
    }
  };

  /**
   * Delete a saved command group with error handling
   * @param id - Group ID to delete
   */
  const deleteGroup = async (id: string): Promise<void> => {
    isLoading.value = true;

    const context: ErrorContext = {
      operation: "Delete Saved Command Group",
      context: { groupId: id },
    };

    try {
      await savedCommandService.deleteGroup(id);
      await loadAll(); // Reload both commands and groups
    } catch (err) {
      const errorMessage = handleError(err, context);
      message.error(errorMessage);
      throw new Error(errorMessage);
    } finally {
      isLoading.value = false;
    }
  };

  const getGroupedCommandsData = (
    searchQuery?: string,
  ): GroupedSavedCommandsData[] => {
    const filteredCommands = searchQuery
      ? commands.value.filter(
          (command) =>
            caseInsensitiveIncludes(command.name, searchQuery) ||
            caseInsensitiveIncludes(command.command, searchQuery) ||
            caseInsensitiveIncludes(command.description, searchQuery) ||
            caseInsensitiveIncludes(command.tags, searchQuery),
        )
      : commands.value;

    const groupedMap = new Map<string, SavedCommand[]>();
    const ungroupedCommands: SavedCommand[] = [];

    filteredCommands.forEach((command) => {
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

    groups.value.forEach((group) => {
      const groupCommands = groupedMap.get(group.id) || [];
      if (groupCommands.length > 0 || !searchQuery) {
        // Show empty groups when no search
        result.push({
          group,
          // Sort a copy to avoid mutating the original array
          commands: [...groupCommands].sort((a, b) =>
            a.name.localeCompare(b.name),
          ),
          commandCount: groupCommands.length,
        });
      }
    });

    if (
      ungroupedCommands.length > 0 ||
      (!searchQuery && groups.value.length === 0)
    ) {
      result.push({
        commands: [...ungroupedCommands].sort((a, b) =>
          a.name.localeCompare(b.name),
        ),
        commandCount: ungroupedCommands.length,
      });
    }

    return result;
  };

  const filterCommands = (params: SavedCommandSearchParams): SavedCommand[] => {
    let filtered = [...commands.value];

    if (params.query) {
      filtered = filtered.filter(
        (command) =>
          caseInsensitiveIncludes(command.name, params.query!) ||
          caseInsensitiveIncludes(command.command, params.query!) ||
          caseInsensitiveIncludes(command.description, params.query!) ||
          caseInsensitiveIncludes(command.tags, params.query!),
      );
    }

    if (params.groupId) {
      filtered = filtered.filter(
        (command) => command.groupId === params.groupId,
      );
    }

    if (params.filterBy) {
      switch (params.filterBy) {
        case "favorites":
          filtered = filtered.filter((command) => command.isFavorite);
          break;
        case "recent":
          filtered = filtered.filter((command) => command.lastUsedAt);
          break;
        case "unused":
          filtered = filtered.filter((command) => command.usageCount === 0);
          break;
      }
    }

    if (params.tags && params.tags.length > 0) {
      filtered = filtered.filter((command) => {
        if (!command.tags) return false;
        const commandTags = safeJsonParse<string[]>(command.tags, []);
        return params.tags!.some((tag) => commandTags.includes(tag));
      });
    }

    if (params.sortBy) {
      filtered.sort((a, b) => {
        let comparison = 0;
        switch (params.sortBy) {
          case "name":
            comparison = a.name.localeCompare(b.name);
            break;
          case "lastUsed": {
            const aDate = a.lastUsedAt ? new Date(a.lastUsedAt).getTime() : 0;
            const bDate = b.lastUsedAt ? new Date(b.lastUsedAt).getTime() : 0;
            comparison = bDate - aDate;
            break;
          }
          case "usageCount":
            comparison = b.usageCount - a.usageCount;
            break;
          case "createdAt":
            comparison =
              new Date(b.createdAt).getTime() - new Date(a.createdAt).getTime();
            break;
          case "updatedAt":
            comparison =
              new Date(b.updatedAt).getTime() - new Date(a.updatedAt).getTime();
            break;
        }
        return params.sortOrder === "asc" ? comparison : -comparison;
      });
    }

    return filtered;
  };

  const getCommandById = (id: string): SavedCommand | undefined => {
    return commands.value.find((command) => command.id === id);
  };

  const findCommandById = async (
    id: string,
  ): Promise<SavedCommand | undefined> => {
    let command = getCommandById(id);
    if (command) return command;

    await loadCommands();
    return getCommandById(id);
  };

  const getGroupById = (id: string): SavedCommandGroup | undefined => {
    return groups.value.find((group) => group.id === id);
  };

  const findGroupById = async (
    id: string,
  ): Promise<SavedCommandGroup | undefined> => {
    let group = getGroupById(id);
    if (group) return group;

    await loadGroups();
    return getGroupById(id);
  };

  const upsertCommand = (cmd: SavedCommand) => {
    if (!cmd?.id) return;
    const i = commands.value.findIndex((c) => c?.id === cmd.id);
    if (i === -1) {
      commands.value = [...commands.value, cmd];
    } else {
      commands.value[i] = { ...commands.value[i], ...cmd };
    }
  };

  const removeCommand = (id: string) => {
    commands.value = commands.value.filter((c) => c?.id !== id);
  };

  const upsertGroup = (g: SavedCommandGroup) => {
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

  let unsubscribeCommandRealtime: (() => void) | null = null;
  let unsubscribeGroupRealtime: (() => void) | null = null;

  const startRealtime = async (): Promise<void> => {
    if (unsubscribeCommandRealtime && unsubscribeGroupRealtime) return;
    try {
      if (!unsubscribeCommandRealtime) {
        const u1 = await api.listen<SavedCommand>(
          "saved_command_created",
          upsertCommand,
        );
        const u2 = await api.listen<SavedCommand>(
          "saved_command_updated",
          upsertCommand,
        );
        const u3 = await api.listen<{ id: string }>(
          "saved_command_deleted",
          ({ id }) => removeCommand(id),
        );
        unsubscribeCommandRealtime = () => {
          u1();
          u2();
          u3();
        };
      }

      if (!unsubscribeGroupRealtime) {
        const g1 = await api.listen<SavedCommandGroup>(
          "saved_command_group_created",
          upsertGroup,
        );
        const g2 = await api.listen<SavedCommandGroup>(
          "saved_command_group_updated",
          upsertGroup,
        );
        const g3 = await api.listen<{ id: string }>(
          "saved_command_group_deleted",
          ({ id }) => removeGroup(id),
        );
        unsubscribeGroupRealtime = () => {
          g1();
          g2();
          g3();
        };
      }
    } catch (e) {
      console.error("Failed to subscribe saved command realtime events:", e);
    }
  };

  const stopRealtime = (): void => {
    if (unsubscribeCommandRealtime) {
      unsubscribeCommandRealtime();
      unsubscribeCommandRealtime = null;
    }
    if (unsubscribeGroupRealtime) {
      unsubscribeGroupRealtime();
      unsubscribeGroupRealtime = null;
    }
  };

  return {
    commands,
    groups,
    isLoading,

    favoriteCommands,
    recentCommands,
    popularCommands,
    commandCount,
    groupCount,
    favoriteCount,
    hasData,

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

    getGroupedCommandsData,
    filterCommands,
    getCommandById,
    findCommandById,
    getGroupById,
    findGroupById,

    startRealtime,
    stopRealtime,
  };
});
