<template>
  <Modal
    id="history-search-modal"
    title="Command History"
    :icon="History"
    icon-background="bg-blue-500/20"
    icon-color="text-blue-400"
    :show-close-button="true"
    size="2xl"
  >
    <div class="space-y-4">
      <!-- Search and Export Bar -->
      <div class="flex items-center gap-3">
        <div class="flex-1">
          <Input
            id="history-search"
            v-model="searchQuery"
            placeholder="Search commands..."
            :icon="Search"
            :space="false"
          />
        </div>
        <Button
          variant="secondary"
          size="md"
          :icon="Download"
          :loading="exporting"
          @click="handleExport"
        >
          Export
        </Button>
      </div>

      <!-- Results Count -->
      <div
        v-if="!loading && filteredEntries.length > 0"
        class="flex items-center justify-between text-xs text-gray-400 px-1"
      >
        <span>
          Showing
          <span class="font-medium text-gray-300">{{
            filteredEntries.length
          }}</span>
          of
          <span class="font-medium text-gray-300">{{ totalCount }}</span>
          results
        </span>
        <span v-if="searchQuery" class="text-gray-500">
          Filtered by: "{{ searchQuery }}"
        </span>
      </div>

      <!-- Loading State -->
      <Transition name="fade" mode="out-in">
        <div
          v-if="loading && filteredEntries.length === 0"
          key="loading"
          class="flex flex-col items-center justify-center py-12"
        >
          <div
            class="animate-spin rounded-full h-10 w-10 border-2 border-gray-700 border-t-blue-400 mb-3"
          ></div>
          <p class="text-sm text-gray-400">Loading history...</p>
        </div>
      </Transition>

      <!-- Empty State -->
      <Transition name="fade" mode="out-in">
        <EmptyState
          v-if="!loading && !isSearching && filteredEntries.length === 0"
          key="empty"
          :icon="Search"
          title="No history found"
          :description="
            searchQuery
              ? `No commands match '${searchQuery}'`
              : 'No commands in history or history is empty.'
          "
        />
      </Transition>

      <!-- Results List -->
      <Transition name="fade" mode="out-in">
        <div
          v-if="!isSearching && filteredEntries.length > 0"
          key="results"
          class="history-list-container space-y-2.5"
        >
          <HistoryItem
            v-for="entry in visibleItems"
            :key="entry.index"
            :entry="entry"
            :highlight="searchQuery"
            @execute="handleExecute"
          />

          <!-- Loading more indicator -->
          <div
            v-if="isLoadingMore"
            class="flex items-center justify-center py-4"
          >
            <div
              class="animate-spin rounded-full h-5 w-5 border-2 border-gray-600 border-t-blue-400"
            ></div>
          </div>

          <!-- Sentinel element for intersection observer -->
          <div
            v-if="hasMore && !isLoadingMore"
            ref="sentinelRef"
            class="h-1"
          ></div>
        </div>
      </Transition>
    </div>
  </Modal>
</template>

<script setup lang="ts">
import { ref, computed, watch, onMounted } from "vue";
import { Search, Download, History } from "lucide-vue-next";
import Modal from "../ui/Modal.vue";
import Input from "../ui/Input.vue";
import Button from "../ui/Button.vue";
import EmptyState from "../ui/EmptyState.vue";
import HistoryItem from "./HistoryItem.vue";
import { writeToTerminal } from "../../services/terminal";
import type { CommandHistoryEntry } from "../../types/history";
import { useOverlay } from "../../composables/useOverlay";
import { useHistoryStore } from "../../stores/history";
import { useLazyLoad } from "../../composables/useLazyLoad";
import { useDebounce } from "../../composables/useDebounce";
import { message } from "../../utils/message";
import { save } from "@tauri-apps/plugin-dialog";

const { closeOverlay, getOverlayProp } = useOverlay();
const historyStore = useHistoryStore();

const terminalId = getOverlayProp<string | null>(
  "history-search-modal",
  "terminalId",
  null,
  null,
);

// Watch for terminalId changes to reload history
watch(terminalId, async (newTerminalId, oldTerminalId) => {
  if (newTerminalId && newTerminalId !== oldTerminalId) {
    // Clear search results and reset query when switching terminals
    historyStore.clearSearchResults();
    searchQuery.value = ""; // Reset search query
    // Reload history for new terminal
    await handleSearch();
  }
});

const searchQuery = ref("");
// Debounce search query with 500ms delay
const debouncedSearchQuery = useDebounce(searchQuery, { delay: 500 });

const filteredEntries = computed(
  () => historyStore.searchResults?.entries || [],
);
const totalCount = computed(() => historyStore.searchResults?.totalCount || 0);
const loading = computed(() => historyStore.isLoading);
const exporting = ref(false);
const isSearching = ref(false); // Track if we're currently searching (for debounce)

// Lazy loading
const {
  visibleItems,
  // @ts-ignore
  sentinelRef,
  hasMore,
  isLoadingMore,
  reset: resetLazyLoad,
} = useLazyLoad(filteredEntries, {
  initialLoad: 20,
  loadMoreCount: 20,
  threshold: 0.1,
  rootMargin: "100px",
});

// Reset lazy load when search query changes (immediate for better UX)
watch(searchQuery, () => {
  resetLazyLoad();
});

// Reset lazy load when entries change
watch(filteredEntries, () => {
  resetLazyLoad();
});

const handleSearch = async () => {
  if (!terminalId.value) return;

  isSearching.value = true;
  try {
    // With lazy loading, we can load all history without UI performance issues
    // Only visible items are rendered, so loading all entries is safe
    // Backend caches history, so subsequent searches are fast
    await historyStore.searchHistory(
      terminalId.value,
      searchQuery.value,
      undefined, // undefined = load all (no limit)
    );
  } catch (error) {
    // Error is already handled in store
  } finally {
    // Small delay to prevent flickering
    setTimeout(() => {
      isSearching.value = false;
    }, 100);
  }
};

const handleExecute = async (entry: CommandHistoryEntry) => {
  // Close modal first
  closeOverlay("history-search-modal");

  // Execute command in terminal
  if (terminalId.value) {
    try {
      await writeToTerminal({
        terminalId: terminalId.value,
        data: entry.command + "\n",
      });
    } catch (error) {
      console.error("Failed to execute command:", error);
      message.error(`Failed to execute command: ${error}`);
    }
  }
};

const handleExport = async () => {
  if (!terminalId.value) return;

  try {
    const filePath = await save({
      filters: [
        { name: "Text", extensions: ["txt"] },
        { name: "JSON", extensions: ["json"] },
      ],
      defaultPath: "terminal-history",
    });

    if (!filePath) return;

    exporting.value = true;

    const format = filePath.endsWith(".json") ? "json" : "txt";
    await historyStore.exportHistory(
      terminalId.value,
      format,
      filePath,
      searchQuery.value || undefined,
    );
  } catch (error) {
    // Error is already handled in store
  } finally {
    exporting.value = false;
  }
};

// Watch debounced search query and perform search
watch(debouncedSearchQuery, () => {
  handleSearch();
});

// Load initial history on mount
onMounted(async () => {
  if (!terminalId.value) return;
  await handleSearch();
});
</script>

<style scoped>
.history-list-container {
  max-height: 500px;
  overflow-y: auto;
  padding-right: 4px;
}

/* Smooth transitions */
.fade-enter-active,
.fade-leave-active {
  transition: opacity 0.2s ease;
}

.fade-enter-from,
.fade-leave-to {
  opacity: 0;
}
</style>
