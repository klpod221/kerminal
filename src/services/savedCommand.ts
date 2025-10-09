import { api } from './api';
import type {
  SavedCommand,
  SavedCommandGroup,
  CreateSavedCommandRequest,
  UpdateSavedCommandRequest,
  CreateSavedCommandGroupRequest,
  UpdateSavedCommandGroupRequest,
} from '../types/savedCommand';

/**
 * Saved Command service for frontend
 */
export const savedCommandService = {
  // === Saved Command Operations ===

  /**
   * Create new saved command
   */
  async createCommand(request: CreateSavedCommandRequest): Promise<SavedCommand> {
    return await api.call('create_saved_command', request);
  },

  /**
   * Get all saved commands
   */
  async getCommands(): Promise<SavedCommand[]> {
    return await api.callRaw('get_saved_commands');
  },

  /**
   * Get saved command by ID
   */
  async getCommand(id: string): Promise<SavedCommand> {
    return await api.callRaw('get_saved_command', id);
  },

  /**
   * Update saved command
   */
  async updateCommand(id: string, request: UpdateSavedCommandRequest): Promise<SavedCommand> {
    return await api.callRaw('update_saved_command', id, request);
  },

  /**
   * Delete saved command
   */
  async deleteCommand(id: string): Promise<void> {
    return await api.callRaw('delete_saved_command', id);
  },

  /**
   * Increment command usage count
   */
  async incrementUsage(id: string): Promise<void> {
    return await api.callRaw('increment_command_usage', id);
  },

  /**
   * Toggle command favorite status
   */
  async toggleFavorite(id: string): Promise<SavedCommand> {
    return await api.callRaw('toggle_command_favorite', id);
  },

  // === Saved Command Group Operations ===

  /**
   * Create new saved command group
   */
  async createGroup(request: CreateSavedCommandGroupRequest): Promise<SavedCommandGroup> {
    return await api.call('create_saved_command_group', request);
  },

  /**
   * Get all saved command groups
   */
  async getGroups(): Promise<SavedCommandGroup[]> {
    return await api.callRaw('get_saved_command_groups');
  },

  /**
   * Get saved command group by ID
   */
  async getGroup(id: string): Promise<SavedCommandGroup> {
    return await api.callRaw('get_saved_command_group', id);
  },

  /**
   * Update saved command group
   */
  async updateGroup(id: string, request: UpdateSavedCommandGroupRequest): Promise<SavedCommandGroup> {
    return await api.callRaw('update_saved_command_group', id, request);
  },

  /**
   * Delete saved command group
   */
  async deleteGroup(id: string): Promise<void> {
    return await api.callRaw('delete_saved_command_group', id);
  },
};
