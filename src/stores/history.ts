import { defineStore } from "pinia";
import { ref, computed } from "vue";
import type {
  CommandHistoryEntry,
  SearchHistoryResponse,
} from "../types/history";
import { historyService } from "../services/history";
import { handleError, type ErrorContext } from "../utils/errorHandler";
import { message } from "../utils/message";

export const useHistoryStore = defineStore("history", () => {
  const historyCache = ref<Map<string, CommandHistoryEntry[]>>(new Map());
  const isLoading = ref(false);
  const searchResults = ref<SearchHistoryResponse | null>(null);

  /**
   * Get history for a terminal (with caching)
   */
  const getHistory = async (
    terminalId: string,
    limit?: number,
  ): Promise<CommandHistoryEntry[]> => {
    const cacheKey = `${terminalId}-${limit || "all"}`;

    // Check cache first
    if (historyCache.value.has(cacheKey)) {
      return historyCache.value.get(cacheKey)!;
    }

    isLoading.value = true;
    const context: ErrorContext = {
      operation: "Get Terminal History",
      context: { terminalId },
    };

    try {
      const history = await historyService.getHistory({
        terminalId,
        limit,
      });

      // Cache the result
      historyCache.value.set(cacheKey, history);

      return history;
    } catch (err) {
      const errorMessage = handleError(err, context);
      message.error(errorMessage);
      return [];
    } finally {
      isLoading.value = false;
    }
  };

  /**
   * Search history for a terminal
   */
  const searchHistory = async (
    terminalId: string,
    query: string,
    limit?: number,
  ): Promise<SearchHistoryResponse> => {
    isLoading.value = true;
    const context: ErrorContext = {
      operation: "Search History",
      context: { terminalId, query },
    };

    try {
      const result = await historyService.searchHistory({
        terminalId,
        query,
        limit,
      });

      searchResults.value = result;
      return result;
    } catch (err) {
      const errorMessage = handleError(err, context);
      message.error(errorMessage);
      return { entries: [], totalCount: 0 };
    } finally {
      isLoading.value = false;
    }
  };

  /**
   * Export history to file
   */
  const exportHistory = async (
    terminalId: string,
    format: "json" | "txt",
    filePath: string,
    query?: string,
  ): Promise<string> => {
    isLoading.value = true;
    const context: ErrorContext = {
      operation: "Export History",
      context: { terminalId, format },
    };

    try {
      const result = await historyService.exportHistory({
        terminalId,
        format,
        filePath,
        query,
      });

      message.success(`History exported successfully to ${result}`);
      return result;
    } catch (err) {
      const errorMessage = handleError(err, context);
      message.error(errorMessage);
      throw err;
    } finally {
      isLoading.value = false;
    }
  };

  /**
   * Clear cache for a terminal
   */
  const clearCache = async (terminalId?: string) => {
    // Note: This is a frontend-only cache clear
    // The backend cache will be cleared automatically when terminal is closed
    if (terminalId) {
      // Clear search results for this terminal
      if (searchResults.value) {
        searchResults.value = null;
      }
      // Frontend cache is managed by backend, so we just clear search results
    } else {
      // Clear all search results
      searchResults.value = null;
    }
  };

  /**
   * Clear search results
   */
  const clearSearchResults = () => {
    searchResults.value = null;
  };

  return {
    // State
    historyCache: computed(() => historyCache.value),
    isLoading: computed(() => isLoading.value),
    searchResults: computed(() => searchResults.value),

    // Actions
    getHistory,
    searchHistory,
    exportHistory,
    clearCache,
    clearSearchResults,
  };
});
