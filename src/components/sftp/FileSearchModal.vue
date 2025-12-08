<template>
  <Modal
    id="sftp-file-search-modal"
    title="Search in files"
    :icon="Search"
    icon-background="bg-purple-500/20"
    icon-color="text-purple-400"
    size="4xl"
  >
    <div class="flex flex-col gap-4">
      <div class="flex gap-2">
        <div class="flex-1">
          <Input
            id="sftp-search-input"
            v-model="query"
            placeholder="Search query (grep)..."
            :disabled="searching"
            @keyup.enter="handleSearch"
            auto-focus
          />
        </div>
        <Button
          variant="primary"
          :loading="searching"
          :disabled="!query.trim()"
          @click="handleSearch"
        >
          Search
        </Button>
      </div>

      <div
        v-if="error"
        class="p-3 bg-red-500/10 border border-red-500/20 rounded text-red-400 text-sm"
      >
        {{ error }}
      </div>

      <div
        v-if="hasSearched && results.length === 0 && !searching"
        class="text-center py-8 text-gray-500"
      >
        No results found for "{{ query }}"
      </div>

      <div
        v-else-if="results.length > 0"
        class="flex flex-col gap-2 max-h-[60vh] overflow-y-auto pr-2"
      >
        <div class="text-sm text-gray-400 mb-2">
          Found {{ results.length }} matches
        </div>

        <div
          v-for="(result, index) in results"
          :key="index"
          class="bg-gray-800/50 border border-gray-700/50 rounded p-3 hover:bg-gray-800 hover:border-blue-500/30 cursor-pointer transition-colors group"
          @click="handleResultClick(result)"
        >
          <div class="flex items-center justify-between mb-1">
            <div class="font-mono text-xs text-blue-400 truncate max-w-[80%]">
              {{ result.filePath }}
            </div>
            <div
              class="font-mono text-xs text-gray-500 group-hover:text-blue-300"
            >
              Line {{ result.lineNumber }}
            </div>
          </div>
          <div
            class="font-mono text-sm text-gray-300 whitespace-pre-wrap break-all bg-gray-900/50 p-2 rounded"
          >
            {{ result.content }}
          </div>
        </div>
      </div>
    </div>

    <template #footer>
      <Button variant="ghost" @click="closeModal">Close</Button>
    </template>
  </Modal>
</template>

<script setup lang="ts">
import { ref, watch } from "vue";
import { Search } from "lucide-vue-next";
import Modal from "../ui/Modal.vue";
import Input from "../ui/Input.vue";
import Button from "../ui/Button.vue";
import { useOverlay } from "../../composables/useOverlay";
import { useSFTPStore } from "../../stores/sftp";
import type { SearchResult, FileEntry } from "../../types/sftp";

const { closeOverlay, openOverlay, getOverlayProp, isOverlayVisible } =
  useOverlay();
const sftpStore = useSFTPStore();

const query = ref("");
const searching = ref(false);
const results = ref<SearchResult[]>([]);
const error = ref<string | null>(null);
const hasSearched = ref(false);

const path = getOverlayProp<string>("sftp-file-search-modal", "path", "");

async function handleSearch() {
  if (!query.value.trim() || !sftpStore.activeSessionId) return;

  searching.value = true;
  error.value = null;
  results.value = [];
  hasSearched.value = false;

  try {
    // Determine path to search: prop path or current remote path
    const searchPath = path.value || sftpStore.browserState.remotePath || ".";

    results.value = await sftpStore.search(
      sftpStore.activeSessionId,
      searchPath,
      query.value,
    );
    hasSearched.value = true;
  } catch (err) {
    error.value = err instanceof Error ? err.message : String(err);
  } finally {
    searching.value = false;
  }
}

function handleResultClick(result: SearchResult) {
  // Construct file entry for editor
  const fileEntry: FileEntry = {
    name: result.filePath.split("/").pop() || result.filePath,
    path: result.filePath,
    fileType: "file",
    size: 0, // Unknown/irrelevant for editor loading
    permissions: 0o644,
    modified: new Date().toISOString(),
    accessed: null,
    symlinkTarget: null,
    uid: null,
    gid: null,
  };

  // Close search modal
  closeModal();

  // Open editor modal with line number
  // Using specific line number prop we added to FileEditorModal
  openOverlay("sftp-file-editor-modal", {
    file: fileEntry,
    isLocal: false,
    line: result.lineNumber,
  });
}

function closeModal() {
  closeOverlay("sftp-file-search-modal");
  // Reset state when closing? Maybe keep results?
  // Let's reset for now to ensure fresh state on reopen unless we want persistence
}

// Watch for modal open to reset/focus?
// The overlay composable handles mounting, but if we want auto-focus logic or clearing previous search:
watch(
  () => isOverlayVisible("sftp-file-search-modal"),
  (visible) => {
    if (visible) {
      // Maybe clear query? Or keep previous?
      // Keep previous for now.
    }
  },
);
</script>
