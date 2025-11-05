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
