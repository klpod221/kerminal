import { defineStore } from "pinia";
import { ref, computed } from "vue";
import { Store } from "@tauri-apps/plugin-store";
import type { ConnectionHistoryEntry } from "../types/ssh";
import { message } from "../utils/message";

// Tauri store instance
let store: Store | null = null;

// Initialize Tauri store
const initStore = async () => {
  store ??= await Store.load("recent_connections.json");
  return store;
};

export const useConnectionHistoryStore = defineStore(
  "connectionHistory",
  () => {
    const recentConnections = ref<ConnectionHistoryEntry[]>([]);
    const isLoading = ref(false);
    const MAX_ENTRIES = 10;

    /**
     * Load history from Tauri store
     */
    const loadHistory = async () => {
      isLoading.value = true;
      try {
        const storeInstance = await initStore();
        const savedHistory =
          await storeInstance.get<ConnectionHistoryEntry[]>("history");

        if (savedHistory) {
          recentConnections.value = savedHistory;
        }
      } catch (error) {
        console.error("Failed to load connection history:", error);
      } finally {
        isLoading.value = false;
      }
    };

    /**
     * Add entry to history
     */
    const addEntry = async (
      entry: Omit<ConnectionHistoryEntry, "lastConnected">,
    ) => {
      try {
        const storeInstance = await initStore();

        // Create new entry with current timestamp
        const newEntry: ConnectionHistoryEntry = {
          ...entry,
          lastConnected: Date.now(),
        };

        // Remove existing entry with same ID if exists
        const filtered = recentConnections.value.filter(
          (e) => !(e.id === entry.id && e.type === entry.type),
        );

        // Add new entry to top
        const updatedHistory = [newEntry, ...filtered].slice(0, MAX_ENTRIES);

        recentConnections.value = updatedHistory;

        // Save to store
        await storeInstance.set("history", updatedHistory);
        await storeInstance.save();
      } catch (error) {
        console.error("Failed to save connection history:", error);
        message.error("Failed to save to recent connections");
      }
    };

    /**
     * Clear history
     */
    const clearHistory = async () => {
      try {
        const storeInstance = await initStore();
        recentConnections.value = [];
        await storeInstance.set("history", []);
        await storeInstance.save();
        message.success("Recent connections cleared");
      } catch (error) {
        console.error("Failed to clear connection history:", error);
        message.error("Failed to clear recent connections");
      }
    };

    return {
      recentConnections: computed(() => recentConnections.value),
      isLoading: computed(() => isLoading.value),

      loadHistory,
      addEntry,
      clearHistory,
    };
  },
);
