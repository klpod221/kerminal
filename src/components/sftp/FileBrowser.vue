<template>
  <div class="h-full flex flex-col bg-[#0D0D0D]">
    <!-- Path bar -->
    <div class="flex items-center gap-2 px-4 py-2 border-b border-gray-800">
      <Button
        variant="ghost"
        size="sm"
        @click="goUp"
        :disabled="!canGoUp"
      >
        <ArrowUp :size="16" />
      </Button>
      <div class="flex-1 relative">
        <input
          v-model="pathInput"
          type="text"
          class="w-full px-3 py-1 bg-gray-900 rounded text-sm text-gray-300 font-mono border border-transparent focus:border-blue-500 focus:outline-none"
          @keydown.enter="handlePathSubmit"
          @keydown.escape="handlePathCancel"
          @keydown.arrow-down.prevent="navigateSuggestion(1)"
          @keydown.arrow-up.prevent="navigateSuggestion(-1)"
          @input="handlePathInput"
          @blur="handlePathBlur"
          @focus="handlePathFocus"
          placeholder="Enter path..."
        />
        <!-- Autocomplete suggestions -->
        <div
          v-if="showSuggestions && suggestions.length > 0"
          class="absolute top-full left-0 right-0 mt-1 bg-gray-800 border border-gray-700 rounded shadow-lg z-50 max-h-60 overflow-auto"
        >
          <div
            v-for="(suggestion, index) in suggestions"
            :key="index"
            class="px-3 py-2 hover:bg-gray-700 cursor-pointer text-sm text-gray-200"
            :class="{ 'bg-gray-700': selectedSuggestionIndex === index }"
            @click="selectSuggestion(suggestion)"
            @mouseenter="selectedSuggestionIndex = index"
          >
            {{ suggestion }}
          </div>
        </div>
      </div>
      <Button
        variant="ghost"
        size="sm"
        @click="$emit('createDirectory')"
        title="Create Directory"
      >
        <FolderPlus :size="16" />
      </Button>
      <Button
        variant="ghost"
        size="sm"
        @click="$emit('refresh')"
      >
        <RefreshCw :size="16" />
      </Button>
    </div>

    <!-- File list -->
    <div
      ref="fileListRef"
      class="flex-1 flex flex-col overflow-hidden"
      @dragover.prevent="onDragOver"
      @drop.prevent="onDrop"
      @contextmenu.prevent="showEmptyContextMenu"
      @keydown="handleKeyDown"
      tabindex="0"
    >
      <EmptyState
        v-if="files.length === 0 && !loading"
        :icon="Folder"
        title="Empty directory"
        description="No files or folders in this directory"
      />

      <div v-else class="flex-1 flex flex-col overflow-hidden">
        <!-- Column headers -->
        <div class="flex items-center px-4 py-1.5 border-b border-gray-800 bg-gray-900/50 text-xs font-medium text-gray-400 sticky top-0 z-10">
          <div class="w-8 shrink-0"></div>
          <div
            class="flex items-center gap-2 flex-1 min-w-0 cursor-pointer hover:text-gray-300 select-none"
            @click="toggleSort('name')"
          >
            <span>Name</span>
            <ChevronUp
              v-if="sortColumn === 'name' && sortDirection === 'asc'"
              :size="14"
            />
            <ChevronDown
              v-else-if="sortColumn === 'name' && sortDirection === 'desc'"
              :size="14"
            />
          </div>
          <div
            class="w-24 shrink-0 cursor-pointer hover:text-gray-300 select-none px-2"
            @click="toggleSort('size')"
          >
            <div class="flex items-center justify-end gap-2">
              <span>Size</span>
              <ChevronUp
                v-if="sortColumn === 'size' && sortDirection === 'asc'"
                :size="14"
              />
              <ChevronDown
                v-else-if="sortColumn === 'size' && sortDirection === 'desc'"
                :size="14"
              />
            </div>
          </div>
          <div
            class="w-32 shrink-0 cursor-pointer hover:text-gray-300 select-none px-2"
            @click="toggleSort('modified')"
          >
            <div class="flex items-center justify-end gap-2">
              <span>Modified</span>
              <ChevronUp
                v-if="sortColumn === 'modified' && sortDirection === 'asc'"
                :size="14"
              />
              <ChevronDown
                v-else-if="sortColumn === 'modified' && sortDirection === 'desc'"
                :size="14"
              />
            </div>
          </div>
        </div>

        <!-- File list with scroll -->
        <div class="flex-1 overflow-auto">
          <div class="divide-y divide-gray-800/50">
            <div
              v-for="(file, index) in sortedFiles"
              :key="file.path"
              :ref="(el) => setFileItemRef(index, el as HTMLElement)"
              data-file-item
              class="flex items-center px-4 py-2 hover:bg-gray-800/50 cursor-pointer group transition-colors"
              :class="{
                'bg-blue-500/10 border-l-2 border-blue-500': isSelected(file.path),
                'bg-gray-800/30': focusedIndex === index,
              }"
              @click="handleFileClick($event, file, index)"
              @dblclick="handleFileDoubleClick(file)"
              @contextmenu.prevent="showContextMenu($event, file)"
            >
              <!-- Selection checkbox -->
              <div
                class="w-8 shrink-0 flex items-center justify-center"
                @click.stop="toggleSelection(file.path)"
              >
                <input
                  type="checkbox"
                  :checked="isSelected(file.path)"
                  @click.stop="toggleSelection(file.path)"
                  class="cursor-pointer w-4 h-4"
                />
              </div>

              <!-- Icon + Name -->
              <div class="flex items-center gap-3 flex-1 min-w-0">
                <div class="shrink-0">
                  <Folder
                    v-if="file.fileType === 'directory'"
                    :size="18"
                    class="text-blue-400"
                  />
                  <File
                    v-else-if="file.fileType === 'file'"
                    :size="18"
                    class="text-gray-400"
                  />
                  <Link
                    v-else-if="file.fileType === 'symlink'"
                    :size="18"
                    class="text-yellow-400"
                  />
                  <FileQuestion
                    v-else
                    :size="18"
                    class="text-gray-500"
                  />
                </div>

                <div class="flex-1 min-w-0">
                  <div class="text-sm text-gray-200 truncate flex items-center gap-2">
                    <span>{{ file.name }}</span>
                    <span
                      v-if="file.fileType === 'symlink' && file.symlinkTarget"
                      class="text-gray-500 text-xs"
                      title="Symlink to: {{ file.symlinkTarget }}"
                    >
                      → {{ file.symlinkTarget }}
                    </span>
                  </div>
                </div>
              </div>

              <!-- Size -->
              <div class="w-24 shrink-0 text-sm text-gray-400 text-right px-2">
                <span v-if="file.fileType === 'file' && file.size !== null">
                  {{ formatBytes(file.size) }}
                </span>
                <span v-else-if="file.fileType === 'directory'">—</span>
                <span v-else>—</span>
              </div>

              <!-- Modified -->
              <div class="w-32 shrink-0 text-sm text-gray-400 text-right px-2">
                <span v-if="file.modified">
                  {{ formatDate(file.modified) }}
                </span>
                <span v-else>—</span>
              </div>
            </div>
          </div>
        </div>
      </div>

      <!-- Loading indicator -->
      <div
        v-if="loading"
        class="flex items-center justify-center h-32"
      >
        <div class="text-gray-500 text-sm">Loading...</div>
      </div>
    </div>

    <!-- Context menu -->
    <ContextMenu
      ref="contextMenuRef"
      :items="contextMenuItems"
      @item-click="handleContextMenuClick"
    />
  </div>
</template>

<script setup lang="ts">
import { ref, computed, nextTick, watch, onMounted } from "vue";
import {
  ArrowUp,
  RefreshCw,
  Folder,
  File,
  Link,
  FileQuestion,
  FolderPlus,
  FolderOpen,
  Download,
  Pencil,
  Trash2,
  Shield,
  FilePlus,
  ChevronUp,
  ChevronDown,
} from "lucide-vue-next";
import type { FileEntry } from "../../types/sftp";
import Button from "../ui/Button.vue";
import EmptyState from "../ui/EmptyState.vue";
import ContextMenu, { type ContextMenuItem } from "../ui/ContextMenu.vue";

interface Props {
  files: FileEntry[];
  currentPath: string;
  loading?: boolean;
  isRemote?: boolean;
  selectedFiles?: Set<string>;
}

const props = withDefaults(defineProps<Props>(), {
  loading: false,
  isRemote: false,
  selectedFiles: () => new Set(),
});

const emit = defineEmits<{
  (e: "navigate", path: string): void;
  (e: "refresh"): void;
  (e: "select", path: string): void;
  (e: "open", file: FileEntry): void;
  (e: "download", file: FileEntry): void;
  (e: "rename", file: FileEntry): void;
  (e: "delete", file: FileEntry): void;
  (e: "permissions", file: FileEntry): void;
  (e: "upload", files: FileList): void;
  (e: "createDirectory"): void;
  (e: "createFile"): void;
}>();

const fileListRef = ref<HTMLElement>();
const contextMenuRef = ref<InstanceType<typeof ContextMenu> | null>(null);
const selectedFile = ref<FileEntry | null>(null);

const isEmptyContextMenu = ref(false);

// Path input state
const pathInput = ref(props.currentPath);
const showSuggestions = ref(false);
const selectedSuggestionIndex = ref(-1);

// Sorting state
const sortColumn = ref<"name" | "size" | "modified">("name");
const sortDirection = ref<"asc" | "desc">("asc");

// Keyboard navigation
const focusedIndex = ref<number>(-1);
const lastFocusedIndex = ref<number>(-1);
const fileItemRefs = ref<(HTMLElement | null)[]>([]);

function setFileItemRef(index: number, el: HTMLElement | null) {
  fileItemRefs.value[index] = el;
}

// Watch currentPath changes
watch(
  () => props.currentPath,
  (newPath) => {
    if (pathInput.value !== newPath) {
      pathInput.value = newPath;
    }
  },
);

// Generate suggestions based on current path and files
const suggestions = computed(() => {
  if (!showSuggestions.value || !pathInput.value) {
    return [];
  }

  const input = pathInput.value.trim();
  if (!input) return [];

  // Get parent directory path
  const parts = input.split("/").filter((p) => p);
  const parentPath = parts.length === 0
    ? "/"
    : input.startsWith("/")
      ? `/${parts.slice(0, -1).join("/")}`
      : parts.slice(0, -1).join("/");

  // Get all directory names from current files
  const directoryNames = props.files
    .filter((f) => f.fileType === "directory")
    .map((f) => f.name);

  // Get the partial path being typed (last segment)
  const lastSegment = input.split("/").pop() || "";

  // Filter directories that match the partial path
  const matching = directoryNames.filter((name) =>
    name.toLowerCase().startsWith(lastSegment.toLowerCase()),
  );

  // Generate full paths for suggestions
  return matching
    .map((name) => {
      if (input.endsWith("/")) {
        return input + name;
      }
      if (parentPath === "/") {
        return `/${name}`;
      }
      return `${parentPath}/${name}`;
    })
    .slice(0, 10); // Limit to 10 suggestions
});

const contextMenuItems = computed<ContextMenuItem[]>(() => {
  // Empty area context menu - only show New File and New Folder
  if (isEmptyContextMenu.value) {
    return [
      {
        id: "new-file",
        label: "New File",
        action: "createFile",
        icon: FilePlus,
      },
      {
        id: "new-folder",
        label: "New Folder",
        action: "createDirectory",
        icon: FolderPlus,
      },
    ];
  }

  // File/folder context menu
  if (!selectedFile.value) return [];

  const items: ContextMenuItem[] = [
    {
      id: "open",
      label: "Open",
      action: "open",
      icon: selectedFile.value.fileType === "directory" ? FolderOpen : File,
    },
  ];

  if (props.isRemote && selectedFile.value.fileType === "file") {
    items.push({
      id: "download",
      label: "Download",
      action: "download",
      icon: Download,
    });
  }

  // Add separator and New File/Folder options
  items.push(
    {
      type: "divider",
      id: "divider-1",
    },
    {
      id: "new-file",
      label: "New File",
      action: "createFile",
      icon: FilePlus,
    },
    {
      id: "new-folder",
      label: "New Folder",
      action: "createDirectory",
      icon: FolderPlus,
    },
    {
      type: "divider",
      id: "divider-2",
    },
  );

  items.push(
    {
      id: "rename",
      label: "Rename",
      action: "rename",
      icon: Pencil,
    },
    {
      id: "delete",
      label: "Delete",
      action: "delete",
      danger: true,
      icon: Trash2,
    },
  );

  if (props.isRemote) {
    items.push({
      id: "permissions",
      label: "Permissions",
      action: "permissions",
      icon: Shield,
    });
  }

  return items;
});

const canGoUp = computed(() => {
  return props.currentPath !== "/" && props.currentPath !== "";
});

const sortedFiles = computed(() => {
  const files = [...props.files];

  return files.sort((a, b) => {
    // Directories first (unless sorting by size)
    if (sortColumn.value !== "size") {
      if (a.fileType === "directory" && b.fileType !== "directory") {
        return -1;
      }
      if (a.fileType !== "directory" && b.fileType === "directory") {
        return 1;
      }
    }

    let comparison = 0;
    switch (sortColumn.value) {
      case "name":
        comparison = a.name.localeCompare(b.name);
        break;
      case "size":
        const sizeA = a.size ?? 0;
        const sizeB = b.size ?? 0;
        comparison = sizeA - sizeB;
        break;
      case "modified":
        const dateA = a.modified ? new Date(a.modified).getTime() : 0;
        const dateB = b.modified ? new Date(b.modified).getTime() : 0;
        comparison = dateA - dateB;
        break;
    }

    return sortDirection.value === "asc" ? comparison : -comparison;
  });
});

function formatDate(dateString: string): string {
  const date = new Date(dateString);
  const now = new Date();
  const diffTime = Math.abs(now.getTime() - date.getTime());
  const diffDays = Math.ceil(diffTime / (1000 * 60 * 60 * 24));

  if (diffDays === 0) {
    return date.toLocaleTimeString([], { hour: "2-digit", minute: "2-digit" });
  } else if (diffDays === 1) {
    return "Yesterday";
  } else if (diffDays < 7) {
    return date.toLocaleDateString([], { weekday: "short" });
  } else if (date.getFullYear() === now.getFullYear()) {
    return date.toLocaleDateString([], { month: "short", day: "numeric" });
  } else {
    return date.toLocaleDateString([], {
      year: "numeric",
      month: "short",
      day: "numeric",
    });
  }
}

function toggleSort(column: "name" | "size" | "modified") {
  if (sortColumn.value === column) {
    sortDirection.value = sortDirection.value === "asc" ? "desc" : "asc";
  } else {
    sortColumn.value = column;
    sortDirection.value = "asc";
  }
}

function formatBytes(bytes: number): string {
  const units = ["B", "KB", "MB", "GB", "TB"];
  let size = bytes;
  let unitIndex = 0;
  while (size >= 1024 && unitIndex < units.length - 1) {
    size /= 1024;
    unitIndex++;
  }
  return `${size.toFixed(1)} ${units[unitIndex]}`;
}

function goUp() {
  if (!canGoUp.value) return;

  const isAbsolute = props.currentPath.startsWith("/");
  const parts = props.currentPath.split("/").filter((p) => p);

  if (parts.length === 0) {
    emit("navigate", "/");
    return;
  }

  const parentParts = parts.slice(0, -1);
  const parentPath = parentParts.length === 0
    ? "/"
    : isAbsolute
      ? `/${parentParts.join("/")}`
      : parentParts.join("/");

  emit("navigate", parentPath);
}

function handleFileClick(
  event: MouseEvent,
  file: FileEntry,
  index: number,
) {
  // Update focused index
  const wasSelected = isSelected(file.path);
  focusedIndex.value = index;

  // Handle Ctrl/Cmd + Click for multi-select
  if (event.ctrlKey || event.metaKey) {
    lastFocusedIndex.value = index;
    toggleSelection(file.path);
    return;
  }

  // Handle Shift + Click for range select
  if (event.shiftKey && lastFocusedIndex.value >= 0) {
    const start = Math.min(lastFocusedIndex.value, index);
    const end = Math.max(lastFocusedIndex.value, index);

    for (let i = start; i <= end; i++) {
      const filePath = sortedFiles.value[i].path;
      if (!props.selectedFiles.has(filePath)) {
        emit("select", filePath);
      }
    }
    return;
  }

  // Single click: select this item (clear others first)
  if (!event.ctrlKey && !event.metaKey && !event.shiftKey) {
    lastFocusedIndex.value = index;

    // If clicking on an already-selected item and it's the only one, just focus it
    if (wasSelected && props.selectedFiles.size === 1) {
      return; // Already selected and it's the only one, just update focus
    }

    // Save current selections before clearing
    const selectedCount = props.selectedFiles.size;
    const currentSelections = Array.from(props.selectedFiles);

    // Clear all selections first
    currentSelections.forEach((path) => {
      emit("select", path); // Toggle off
    });

    // Then select this one (only if it wasn't selected, or if multiple were selected before)
    if (!wasSelected || selectedCount > 1) {
      // Use nextTick to ensure previous clears are processed
      nextTick(() => {
        if (!props.selectedFiles.has(file.path)) {
          emit("select", file.path);
        }
      });
    }
  }
}

function handleKeyDown(event: KeyboardEvent) {
  if (!fileListRef.value || sortedFiles.value.length === 0) return;

  // Arrow keys for navigation
  if (event.key === "ArrowDown" || event.key === "ArrowUp") {
    event.preventDefault();
    const direction = event.key === "ArrowDown" ? 1 : -1;
    const currentIndex = focusedIndex.value < 0 ? 0 : focusedIndex.value;
    const newIndex = Math.max(
      0,
      Math.min(sortedFiles.value.length - 1, currentIndex + direction),
    );
    focusedIndex.value = newIndex;
    lastFocusedIndex.value = newIndex;

    // Scroll into view
    nextTick(() => {
      const element = fileItemRefs.value[newIndex];
      if (element) {
        element.scrollIntoView({ block: "nearest", behavior: "smooth" });
      }
    });
    return;
  }

  // Enter to open
  if (event.key === "Enter" && focusedIndex.value >= 0) {
    const file = sortedFiles.value[focusedIndex.value];
    handleFileDoubleClick(file);
    return;
  }

  // Space to toggle selection
  if (event.key === " " && focusedIndex.value >= 0) {
    event.preventDefault();
    const file = sortedFiles.value[focusedIndex.value];
    toggleSelection(file.path);
    return;
  }

  // Ctrl/Cmd + A to select all
  if ((event.ctrlKey || event.metaKey) && event.key === "a") {
    event.preventDefault();
    sortedFiles.value.forEach((file) => {
      if (!props.selectedFiles.has(file.path)) {
        emit("select", file.path);
      }
    });
    return;
  }
}

function handleFileDoubleClick(file: FileEntry) {
  // Double click: open/navigate
  if (file.fileType === "directory") {
    emit("navigate", file.path);
  } else {
    emit("open", file);
  }
}

function isSelected(path: string): boolean {
  return props.selectedFiles.has(path);
}

function toggleSelection(path: string) {
  emit("select", path);
}

function showContextMenu(event: MouseEvent, file: FileEntry) {
  event.preventDefault();
  event.stopPropagation();
  event.stopImmediatePropagation();

  isEmptyContextMenu.value = false;
  selectedFile.value = file;

  // Use nextTick to ensure selectedFile is set and context menu items are computed
  nextTick(() => {
    if (contextMenuRef.value && selectedFile.value) {
      contextMenuRef.value.show(event.clientX, event.clientY);
    }
  });
}

function showEmptyContextMenu(event: MouseEvent) {
  // Only show if clicking on empty area (not on a file item)
  const target = event.target as HTMLElement;
  const fileItem = target.closest('[data-file-item]');

  if (!fileItem) {
    event.preventDefault();
    event.stopPropagation();
    event.stopImmediatePropagation();

    isEmptyContextMenu.value = true;
    selectedFile.value = null;

    nextTick(() => {
      if (contextMenuRef.value) {
        contextMenuRef.value.show(event.clientX, event.clientY);
      }
    });
  }
}

function handleContextMenuClick(item: ContextMenuItem) {
  const action = item.action || item.id;

  // Handle empty area actions
  if (isEmptyContextMenu.value) {
    switch (action) {
      case "createFile":
        emit("createFile");
        break;
      case "createDirectory":
        emit("createDirectory");
        break;
    }
    isEmptyContextMenu.value = false;
    return;
  }

  // Handle file/folder actions
  if (!selectedFile.value) return;

  switch (action) {
    case "open":
      emit("open", selectedFile.value);
      break;
    case "download":
      emit("download", selectedFile.value);
      break;
    case "rename":
      emit("rename", selectedFile.value);
      break;
    case "delete":
      emit("delete", selectedFile.value);
      break;
    case "permissions":
      emit("permissions", selectedFile.value);
      break;
    case "createFile":
      emit("createFile");
      break;
    case "createDirectory":
      emit("createDirectory");
      break;
  }

  selectedFile.value = null;
  isEmptyContextMenu.value = false;
}

function onDragOver(event: DragEvent) {
  event.preventDefault();
  if (event.dataTransfer) {
    event.dataTransfer.dropEffect = "copy";
  }
}

async function onDrop(event: DragEvent) {
  event.preventDefault();
  if (!event.dataTransfer) return;

  const files: File[] = [];

  // Try to get file paths from dataTransfer items (for desktop drag & drop)
  if (event.dataTransfer.items && event.dataTransfer.items.length > 0) {
    for (let i = 0; i < event.dataTransfer.items.length; i++) {
      const item = event.dataTransfer.items[i];

      // Check for file path in items
      if (item.kind === "file") {
        const entry = item.webkitGetAsEntry?.();
        if (entry && entry.isFile) {
          // For FileSystemFileEntry, we need to get the file
          const fileEntry = entry as FileSystemFileEntry;
          const file = await new Promise<File | null>((resolve) => {
            fileEntry.file(resolve, () => resolve(null));
          });
          if (file) {
            files.push(file);
          }
        } else {
          // Try to get as File directly
          const file = item.getAsFile();
          if (file) {
            files.push(file);
          }
        }
      }
    }
  }

  // Fallback to files API if items didn't work
  if (files.length === 0 && event.dataTransfer.files && event.dataTransfer.files.length > 0) {
    files.push(...Array.from(event.dataTransfer.files));
  }

  if (files.length > 0) {
    emit("upload", files as unknown as FileList);
  }
}

function handlePathInput() {
  // Show suggestions as user types
  if (pathInput.value && pathInput.value !== props.currentPath) {
    showSuggestions.value = true;
    selectedSuggestionIndex.value = -1;
  }
}

function handlePathSubmit() {
  // If a suggestion is selected, use it
  if (selectedSuggestionIndex.value >= 0 && suggestions.value[selectedSuggestionIndex.value]) {
    const suggestion = suggestions.value[selectedSuggestionIndex.value];
    pathInput.value = suggestion;
    showSuggestions.value = false;
    emit("navigate", suggestion);
    return;
  }

  // Otherwise, use the input value
  const path = pathInput.value.trim();
  if (path && path !== props.currentPath) {
    showSuggestions.value = false;
    emit("navigate", path);
  } else {
    showSuggestions.value = false;
  }
}

function handlePathCancel() {
  pathInput.value = props.currentPath;
  showSuggestions.value = false;
  selectedSuggestionIndex.value = -1;
}

function handlePathBlur() {
  // Delay hiding suggestions to allow click on suggestion
  setTimeout(() => {
    showSuggestions.value = false;
    selectedSuggestionIndex.value = -1;
    // Reset to current path if not navigated
    if (pathInput.value !== props.currentPath) {
      pathInput.value = props.currentPath;
    }
  }, 200);
}

function handlePathFocus() {
  showSuggestions.value = true;
  selectedSuggestionIndex.value = -1;
}

function navigateSuggestion(direction: number) {
  if (!showSuggestions.value || suggestions.value.length === 0) {
    return;
  }

  selectedSuggestionIndex.value += direction;

  if (selectedSuggestionIndex.value < 0) {
    selectedSuggestionIndex.value = suggestions.value.length - 1;
  } else if (selectedSuggestionIndex.value >= suggestions.value.length) {
    selectedSuggestionIndex.value = 0;
  }
}

function selectSuggestion(suggestion: string) {
  pathInput.value = suggestion;
  showSuggestions.value = false;
  selectedSuggestionIndex.value = -1;
  emit("navigate", suggestion);
}

// Reset focus when files change
watch(
  () => props.files.length,
  () => {
    focusedIndex.value = -1;
    lastFocusedIndex.value = -1;
    fileItemRefs.value = [];
  },
);

onMounted(() => {
  // Focus the file list for keyboard navigation
  if (fileListRef.value) {
    fileListRef.value.focus();
  }
});
</script>

