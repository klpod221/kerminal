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
  CommandHistoryEntry,
  GetTerminalHistoryRequest,
  SearchHistoryRequest,
  SearchHistoryResponse,
  ExportHistoryRequest,
} from "../types/history";

/**
 * History service for frontend
 */
export const historyService = {
  /**
   * Get history for a terminal
   */
  async getHistory(
    request: GetTerminalHistoryRequest,
  ): Promise<CommandHistoryEntry[]> {
    return await api.call<CommandHistoryEntry[]>(
      "get_terminal_history",
      request,
    );
  },

  /**
   * Search history for a terminal
   */
  async searchHistory(
    request: SearchHistoryRequest,
  ): Promise<SearchHistoryResponse> {
    return await api.call<SearchHistoryResponse>("search_history", request);
  },

  /**
   * Export history to file
   */
  async exportHistory(request: ExportHistoryRequest): Promise<string> {
    return await api.call<string>("export_history", request);
  },
};
