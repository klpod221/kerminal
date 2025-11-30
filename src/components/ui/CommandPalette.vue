<template>
  <Modal
    id="command-palette"
    :visible="false"
    :show-close-button="false"
    size="lg"
    class="command-palette-modal"
  >
    <div class="flex flex-col h-[50vh] max-h-[500px]">
      <!-- Search Input -->
      <div class="flex items-center px-4 py-3 border-b border-gray-700">
        <Search class="w-5 h-5 text-gray-400 mr-3" />
        <input
          ref="searchInput"
          v-model="searchQuery"
          type="text"
          class="flex-1 bg-transparent border-none outline-none text-white placeholder-gray-500 text-lg"
          placeholder="Type a command..."
          @keydown.down.prevent="navigate('down')"
          @keydown.up.prevent="navigate('up')"
          @keydown.enter.prevent="executeSelected"
        />
        <div class="flex items-center gap-2">
          <span class="text-xs text-gray-500 px-1.5 py-0.5 rounded border border-gray-700">Esc</span>
        </div>
      </div>

      <!-- Command List -->
      <div class="flex-1 overflow-y-auto p-2" ref="listContainer">
        <div v-if="filteredCommands.length === 0" class="p-4 text-center text-gray-500">
          No commands found matching "{{ searchQuery }}"
        </div>

        <div v-else class="space-y-1">
          <button
            v-for="(command, index) in filteredCommands"
            :key="command.id"
            class="w-full flex items-center justify-between px-3 py-2 rounded-md text-left transition-colors group"
            :class="[
              selectedIndex === index
                ? 'bg-blue-600 text-white'
                : 'text-gray-300 hover:bg-gray-800'
            ]"
            @click="executeCommand(command)"
            @mousemove="selectedIndex = index"
          >
            <div class="flex items-center gap-3">
              <component
                :is="getCategoryIcon(command.category)"
                class="w-4 h-4"
                :class="selectedIndex === index ? 'text-blue-200' : 'text-gray-500'"
              />
              <span :class="selectedIndex === index ? 'text-white' : 'text-gray-200'">
                {{ command.label }}
              </span>
            </div>

            <div class="flex items-center gap-2">
              <span
                v-if="command.shortcut"
                class="text-xs px-1.5 py-0.5 rounded"
                :class="selectedIndex === index ? 'bg-blue-500 text-blue-100' : 'bg-gray-800 text-gray-400'"
              >
                {{ command.shortcut }}
              </span>
            </div>
          </button>
        </div>
      </div>

      <!-- Footer -->
      <div class="px-4 py-2 border-t border-gray-700 text-xs text-gray-500 flex justify-between">
        <span>{{ filteredCommands.length }} commands</span>
        <div class="flex gap-3">
          <span><kbd class="font-sans">↑↓</kbd> to navigate</span>
          <span><kbd class="font-sans">↵</kbd> to select</span>
        </div>
      </div>
    </div>
  </Modal>
</template>

<script setup lang="ts">
import { ref, computed, watch, nextTick } from "vue";
import { Search, Terminal, Navigation, Settings, Command, FileText, Globe } from "lucide-vue-next";
import Modal from "./Modal.vue";
import { useKeyboardShortcutsStore } from "../../stores/keyboardShortcuts";
import { useGlobalShortcuts } from "../../composables/useGlobalShortcuts";
import { useOverlay } from "../../composables/useOverlay";
import { formatShortcut } from "../../types/shortcuts";

const searchInput = ref<HTMLInputElement | null>(null);
const listContainer = ref<HTMLElement | null>(null);
const searchQuery = ref("");
const selectedIndex = ref(0);

const shortcutsStore = useKeyboardShortcutsStore();
const { executeShortcutAction } = useGlobalShortcuts();
const { closeOverlay } = useOverlay();

// Get all available commands from the store
const allCommands = computed(() => {
  return shortcutsStore.activeShortcuts.map(shortcut => ({
    id: shortcut.id,
    label: shortcut.label,
    category: shortcut.category,
    shortcut: formatShortcut(shortcut),
    original: shortcut
  }));
});

// Fuzzy search implementation
const filteredCommands = computed(() => {
  if (!searchQuery.value) return allCommands.value;

  const query = searchQuery.value.toLowerCase();

  return allCommands.value
    .filter(cmd => {
      // Simple fuzzy matching: check if characters appear in order
      const text = cmd.label.toLowerCase();
      let i = 0;
      let j = 0;
      while (i < text.length && j < query.length) {
        if (text[i] === query[j]) {
          j++;
        }
        i++;
      }
      return j === query.length;
    })
    .sort((a, b) => {
      // Sort by relevance (exact match first, then starts with, then others)
      const aLabel = a.label.toLowerCase();
      const bLabel = b.label.toLowerCase();

      if (aLabel === query) return -1;
      if (bLabel === query) return 1;

      if (aLabel.startsWith(query) && !bLabel.startsWith(query)) return -1;
      if (!aLabel.startsWith(query) && bLabel.startsWith(query)) return 1;

      return 0;
    });
});

// Reset selection when search changes
watch(searchQuery, () => {
  selectedIndex.value = 0;
});

// Focus input when mounted/visible
watch(() => searchInput.value, (el) => {
  if (el) {
    nextTick(() => el.focus());
  }
});

const navigate = (direction: 'up' | 'down') => {
  if (direction === 'up') {
    selectedIndex.value = Math.max(0, selectedIndex.value - 1);
  } else {
    selectedIndex.value = Math.min(filteredCommands.value.length - 1, selectedIndex.value + 1);
  }

  scrollToSelected();
};

const scrollToSelected = () => {
  nextTick(() => {
    if (!listContainer.value) return;

    const selectedEl = listContainer.value.children[0]?.children[selectedIndex.value] as HTMLElement;
    if (selectedEl) {
      selectedEl.scrollIntoView({ block: 'nearest' });
    }
  });
};

const executeSelected = () => {
  if (filteredCommands.value.length > 0) {
    executeCommand(filteredCommands.value[selectedIndex.value]);
  }
};

const executeCommand = (command: any) => {
  closeOverlay("command-palette");
  // Small delay to allow overlay to close smoothly
  setTimeout(() => {
    executeShortcutAction(command.id);
  }, 50);
};

const getCategoryIcon = (category: string) => {
  switch (category.toLowerCase()) {
    case 'terminal': return Terminal;
    case 'navigation': return Navigation;
    case 'settings': return Settings;
    case 'file': return FileText;
    case 'web': return Globe;
    default: return Command;
  }
};
</script>

<style scoped>
/* Custom scrollbar for the list */
::-webkit-scrollbar {
  width: 6px;
}
::-webkit-scrollbar-track {
  background: transparent;
}
::-webkit-scrollbar-thumb {
  background: #4b5563;
  border-radius: 3px;
}
::-webkit-scrollbar-thumb:hover {
  background: #6b7280;
}
</style>
