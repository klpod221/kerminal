/*
 * Kerminal - Modern Terminal Emulator & SSH Manager
 * Copyright (C) 2026 Bùi Thanh Xuân (klpod221)
 *
 * This program is free software: you can redistribute it and/or modify
 * it under the terms of the GNU General Public License as published by
 * the Free Software Foundation, either version 3 of the License, or
 * (at your option) any later version.
 *
 * This program is distributed in the hope that it will be useful,
 * but WITHOUT ANY WARRANTY; without even the implied warranty of
 * MERCHANTABILITY or FITNESS FOR A PARTICULAR PURPOSE.  See the
 * GNU General Public License for more details.
 *
 * You should have received a copy of the GNU General Public License
 * along with this program.  If not, see <https://www.gnu.org/licenses/>.
 */

import { api } from "./api";
import type {
  SavedCommand,
  SavedCommandGroup,
  CreateSavedCommandRequest,
  UpdateSavedCommandRequest,
  CreateSavedCommandGroupRequest,
  UpdateSavedCommandGroupRequest,
} from "../types/savedCommand";

/**
 * Saved Command service for frontend
 */
export const savedCommandService = {
  /**
   * Create new saved command
   */
  async createCommand(
    request: CreateSavedCommandRequest,
  ): Promise<SavedCommand> {
    return await api.call("create_saved_command", request);
  },

  /**
   * Get all saved commands
   */
  async getCommands(): Promise<SavedCommand[]> {
    return await api.callRaw("get_saved_commands");
  },

  /**
   * Get saved command by ID
   */
  async getCommand(id: string): Promise<SavedCommand> {
    return await api.callRaw("get_saved_command", id);
  },

  /**
   * Update saved command
   */
  async updateCommand(
    id: string,
    request: UpdateSavedCommandRequest,
  ): Promise<SavedCommand> {
    return await api.callRaw("update_saved_command", id, request);
  },

  /**
   * Delete saved command
   */
  async deleteCommand(id: string): Promise<void> {
    return await api.callRaw("delete_saved_command", id);
  },

  /**
   * Increment command usage count
   */
  async incrementUsage(id: string): Promise<void> {
    return await api.callRaw("increment_command_usage", id);
  },

  /**
   * Toggle command favorite status
   */
  async toggleFavorite(id: string): Promise<SavedCommand> {
    return await api.callRaw("toggle_command_favorite", id);
  },

  /**
   * Create new saved command group
   */
  async createGroup(
    request: CreateSavedCommandGroupRequest,
  ): Promise<SavedCommandGroup> {
    return await api.call("create_saved_command_group", request);
  },

  /**
   * Get all saved command groups
   */
  async getGroups(): Promise<SavedCommandGroup[]> {
    return await api.callRaw("get_saved_command_groups");
  },

  /**
   * Get saved command group by ID
   */
  async getGroup(id: string): Promise<SavedCommandGroup> {
    return await api.callRaw("get_saved_command_group", id);
  },

  /**
   * Update saved command group
   */
  async updateGroup(
    id: string,
    request: UpdateSavedCommandGroupRequest,
  ): Promise<SavedCommandGroup> {
    return await api.callRaw("update_saved_command_group", id, request);
  },

  /**
   * Delete saved command group
   */
  async deleteGroup(id: string): Promise<void> {
    return await api.callRaw("delete_saved_command_group", id);
  },
};
