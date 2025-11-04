<template>
  <div class="h-full flex flex-col bg-[#0D0D0D]">
    <!-- Combined Path & Search bar -->
    <div class="flex items-center gap-2 px-4 py-2 border-b border-gray-800">
      <Button variant="ghost" size="sm" @click="goUp" :disabled="!canGoUp">
        <ArrowUp :size="16" />
      </Button>
      <div class="flex-1 relative flex items-center gap-1">
        <div
          class="flex items-center gap-1 px-2 py-1 bg-gray-900 rounded text-xs text-gray-400 border border-transparent"
        >
          <button
            type="button"
            class="px-1.5 py-0.5 rounded transition-colors"
            :class="
              !isSearchMode
                ? 'bg-blue-500/20 text-blue-400'
                : 'text-gray-500 hover:text-gray-300'
            "
            @click="isSearchMode = false"
            title="Path mode"
          >
            /
          </button>
          <button
            type="button"
            class="px-1.5 py-0.5 rounded transition-colors"
            :class="
              isSearchMode
                ? 'bg-blue-500/20 text-blue-400'
                : 'text-gray-500 hover:text-gray-300'
            "
            @click="isSearchMode = true"
            title="Search mode"
          >
            <Search :size="12" />
          </button>
        </div>
        <div class="flex-1 relative">
          <!-- Path input -->
          <input
            v-if="!isSearchMode"
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
          <!-- Search input -->
          <div v-else class="flex items-center gap-1">
            <input
              ref="searchInputRef"
              v-model="searchQuery"
              type="text"
              class="flex-1 px-2 py-1 bg-gray-900 rounded text-sm text-gray-300 border border-transparent focus:border-blue-500 focus:outline-none"
              placeholder="Search files..."
              @keydown.enter="handleSearchEnter"
            />
            <Button
              variant="ghost"
              size="sm"
              class="h-7 px-2"
              :class="{ 'bg-blue-500/20': searchOptions.useRegex }"
              @click="searchOptions.useRegex = !searchOptions.useRegex"
              title="Use regex"
            >
              <span class="text-xs">.*</span>
            </Button>
          </div>
          <!-- Path autocomplete suggestions -->
          <div
            v-if="!isSearchMode && showSuggestions && suggestions.length > 0"
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
      </div>
      <Button
        variant="ghost"
        size="sm"
        @click="$emit('createDirectory')"
        title="Create Directory"
      >
        <FolderPlus :size="16" />
      </Button>
      <Button variant="ghost" size="sm" @click="$emit('refresh')">
        <RefreshCw :size="16" />
      </Button>
    </div>

    <!-- File list -->
    <div
      ref="fileListRef"
      data-filebrowser="true"
      class="flex-1 flex flex-col overflow-hidden relative"
      :class="{
        'bg-blue-500/5': isDragOver && dragOverIndex === -1,
      }"
      @dragover.prevent="onDragOver"
      @drop.stop.prevent="onDrop"
      @dragenter.prevent="onDragEnter"
      @dragleave="onDragLeave"
      @contextmenu.prevent="showEmptyContextMenu"
      @keydown="handleKeyDown"
      tabindex="0"
    >
      <EmptyState
        v-if="filteredFiles.length === 0 && searchQuery && !loading"
        :icon="Folder"
        title="No results"
        :description="`No files match '${searchQuery}'`"
      />
      <EmptyState
        v-else-if="files.length === 0 && !loading"
        :icon="Folder"
        title="Empty directory"
        description="No files or folders in this directory"
      />

      <div v-else class="flex-1 flex flex-col overflow-hidden">
        <!-- Column headers -->
        <div
          class="flex items-center px-4 py-1.5 border-b border-gray-800 bg-gray-900/50 text-xs font-medium text-gray-400 sticky top-0 z-10"
        >
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
                v-else-if="
                  sortColumn === 'modified' && sortDirection === 'desc'
                "
                :size="14"
              />
            </div>
          </div>
        </div>

        <!-- File list with scroll -->
        <div class="flex-1 overflow-auto">
          <div class="divide-y divide-gray-800/50">
            <div
              v-for="(file, index) in filteredFiles"
              :key="file.path"
              :ref="(el) => setFileItemRef(index, el as HTMLElement)"
              data-file-item
              draggable="true"
              class="flex items-center px-4 py-2 hover:bg-gray-800/50 cursor-pointer group transition-colors"
              :class="{
                'bg-blue-500/10 border-l-2 border-blue-500': isSelected(
                  file.path,
                ),
                'bg-gray-800/30': focusedIndex === index,
                'opacity-50':
                  isDragging && draggedFiles.some((f) => f.path === file.path),
                'bg-blue-500/20 border-l-2 border-blue-400':
                  dragOverIndex === index && isDragOver,
              }"
              @dragstart="handleDragStart($event, file)"
              @dragend="handleDragEnd"
              @dragover.prevent="handleDragOver($event, file, index)"
              @dragleave="handleDragLeave(index)"
              @drop.stop.prevent="handleFileDrop($event, file)"
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
                  <FileQuestion v-else :size="18" class="text-gray-500" />
                </div>

                <div class="flex-1 min-w-0">
                  <div
                    class="text-sm text-gray-200 truncate flex items-center gap-2"
                  >
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
      <div v-if="loading" class="flex items-center justify-center h-32">
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
  Search,
} from "lucide-vue-next";
import type { FileEntry, FileBrowserDragData } from "../../types/sftp";
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
  (e: "edit", file: FileEntry): void;
  (e: "download", files: FileEntry[]): void;
  (e: "rename", file: FileEntry): void;
  (e: "delete", files: FileEntry[]): void;
  (e: "permissions", files: FileEntry[]): void;
  (e: "upload", files: FileList): void;
  (e: "createDirectory"): void;
  (e: "createFile"): void;
  (
    e: "drag-files",
    files: FileEntry[],
    targetPath: string,
    isSourceRemote: boolean,
  ): void;
}>();

const fileListRef = ref<HTMLElement>();
const contextMenuRef = ref<InstanceType<typeof ContextMenu> | null>(null);
const selectedFile = ref<FileEntry | null>(null);
const searchInputRef = ref<HTMLInputElement>();

const isEmptyContextMenu = ref(false);

// Search state
const isSearchMode = ref(false);
const searchQuery = ref("");
const searchOptions = ref({
  useRegex: false,
  searchContent: false,
  filterByName: true,
  filterByExtension: true,
  filterBySize: false,
  filterByDate: false,
});

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

// Drag & drop state
const isDragging = ref(false);
const draggedFiles = ref<FileEntry[]>([]);
const dragOverIndex = ref<number>(-1);
const isDragOver = ref(false);

function setFileItemRef(index: number, el: HTMLElement | null) {
  fileItemRefs.value[index] = el;
}

watch(
  () => props.currentPath,
  (newPath) => {
    if (pathInput.value !== newPath && !isSearchMode.value) {
      pathInput.value = newPath;
    }
  },
);

watch(isSearchMode, (newValue) => {
  if (newValue) {
    nextTick(() => {
      searchInputRef.value?.focus();
    });
  } else {
    searchQuery.value = "";
  }
});

const suggestions = computed(() => {
  if (!showSuggestions.value || !pathInput.value) {
    return [];
  }

  const input = pathInput.value.trim();
  if (!input) return [];

  const parts = input.split("/").filter((p) => p);
  const parentPath =
    parts.length === 0
      ? "/"
      : input.startsWith("/")
        ? `/${parts.slice(0, -1).join("/")}`
        : parts.slice(0, -1).join("/");

  const directoryNames = props.files
    .filter((f) => f.fileType === "directory")
    .map((f) => f.name);

  const lastSegment = input.split("/").pop() || "";

  const matching = directoryNames.filter((name) =>
    name.toLowerCase().startsWith(lastSegment.toLowerCase()),
  );

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
    .slice(0, 10);
});

const contextMenuItems = computed<ContextMenuItem[]>(() => {
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

  if (!selectedFile.value) return [];

  const items: ContextMenuItem[] = [
    {
      id: "open",
      label: "Open",
      action: "open",
      icon: selectedFile.value.fileType === "directory" ? FolderOpen : File,
    },
  ];

  if (selectedFile.value.fileType === "file") {
    items.push({
      id: "edit",
      label: "Edit",
      action: "edit",
      icon: Pencil,
    });
  }

  if (props.isRemote && selectedFile.value.fileType === "file") {
    items.push({
      id: "download",
      label: "Download",
      action: "download",
      icon: Download,
    });
  }

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

const filteredFiles = computed(() => {
  if (!searchQuery.value.trim()) {
    return sortedFiles.value;
  }

  const query = searchQuery.value.trim();
  let regex: RegExp | null = null;

  try {
    if (searchOptions.value.useRegex) {
      regex = new RegExp(query, "i");
    }
  } catch (error) {
    // Invalid regex, fall back to plain text search
    regex = null;
  }

  const matchText = (text: string): boolean => {
    if (regex) {
      return regex.test(text);
    }
    return text.toLowerCase().includes(query.toLowerCase());
  };

  const matchExtension = (fileName: string): boolean => {
    const ext = fileName.substring(fileName.lastIndexOf(".") + 1).toLowerCase();
    return matchText(ext);
  };

  const matchSize = (size: number | null): boolean => {
    if (!size) return false;
    const queryLower = query.toLowerCase();
    if (
      queryLower.includes("kb") ||
      queryLower.includes("mb") ||
      queryLower.includes("gb")
    ) {
      const sizeKB = size / 1024;
      const sizeMB = size / (1024 * 1024);
      const sizeGB = size / (1024 * 1024 * 1024);
      return (
        matchText(`${Math.round(sizeKB)}kb`) ||
        matchText(`${Math.round(sizeMB)}mb`) ||
        matchText(`${sizeGB.toFixed(2)}gb`)
      );
    }
    return matchText(size.toString());
  };

  const matchDate = (dateString: string | null): boolean => {
    if (!dateString) return false;
    const date = new Date(dateString);
    const dateStr = date.toLocaleDateString();
    const timeStr = date.toLocaleTimeString();
    return matchText(dateStr) || matchText(timeStr);
  };

  return sortedFiles.value.filter((file) => {
    let matches = false;

    if (searchOptions.value.filterByName) {
      matches = matches || matchText(file.name);
    }

    if (searchOptions.value.filterByExtension && file.fileType === "file") {
      matches = matches || matchExtension(file.name);
    }

    if (searchOptions.value.filterBySize && file.fileType === "file") {
      matches = matches || matchSize(file.size);
    }

    if (searchOptions.value.filterByDate) {
      matches = matches || matchDate(file.modified);
    }

    if (!matches && searchOptions.value.filterByName) {
      matches = matchText(file.name);
    }

    return matches;
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
  const parentPath =
    parentParts.length === 0
      ? "/"
      : isAbsolute
        ? `/${parentParts.join("/")}`
        : parentParts.join("/");

  emit("navigate", parentPath);
}

function handleFileClick(event: MouseEvent, file: FileEntry, index: number) {
  const wasSelected = isSelected(file.path);
  focusedIndex.value = index;

  if (event.ctrlKey || event.metaKey) {
    lastFocusedIndex.value = index;
    toggleSelection(file.path);
    return;
  }

  if (event.shiftKey && lastFocusedIndex.value >= 0) {
    const start = Math.min(lastFocusedIndex.value, index);
    const end = Math.max(lastFocusedIndex.value, index);

    for (let i = start; i <= end; i++) {
      const filePath = filteredFiles.value[i].path;
      if (!props.selectedFiles.has(filePath)) {
        emit("select", filePath);
      }
    }
    return;
  }

  if (!event.ctrlKey && !event.metaKey && !event.shiftKey) {
    lastFocusedIndex.value = index;

    if (wasSelected && props.selectedFiles.size === 1) {
      return;
    }

    const selectedCount = props.selectedFiles.size;
    const currentSelections = Array.from(props.selectedFiles);

    currentSelections.forEach((path) => {
      emit("select", path);
    });

    if (!wasSelected || selectedCount > 1) {
      nextTick(() => {
        if (!props.selectedFiles.has(file.path)) {
          emit("select", file.path);
        }
      });
    }
  }
}

function handleKeyDown(event: KeyboardEvent) {
  if (!fileListRef.value || filteredFiles.value.length === 0) return;

  if (event.key === "ArrowDown" || event.key === "ArrowUp") {
    event.preventDefault();
    const direction = event.key === "ArrowDown" ? 1 : -1;
    const currentIndex = focusedIndex.value < 0 ? 0 : focusedIndex.value;
    const newIndex = Math.max(
      0,
      Math.min(filteredFiles.value.length - 1, currentIndex + direction),
    );
    focusedIndex.value = newIndex;
    lastFocusedIndex.value = newIndex;

    nextTick(() => {
      const element = fileItemRefs.value[newIndex];
      if (element) {
        element.scrollIntoView({ block: "nearest", behavior: "smooth" });
      }
    });
    return;
  }

  if (event.key === "Enter" && focusedIndex.value >= 0) {
    const file = filteredFiles.value[focusedIndex.value];
    handleFileDoubleClick(file);
    return;
  }

  if (event.key === " " && focusedIndex.value >= 0) {
    event.preventDefault();
    const file = filteredFiles.value[focusedIndex.value];
    toggleSelection(file.path);
    return;
  }

  if ((event.ctrlKey || event.metaKey) && event.key === "a") {
    event.preventDefault();
    filteredFiles.value.forEach((file) => {
      if (!props.selectedFiles.has(file.path)) {
        emit("select", file.path);
      }
    });
    return;
  }
}

function handleFileDoubleClick(file: FileEntry) {
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

function handleSearchEnter() {
  if (filteredFiles.value.length > 0) {
    focusedIndex.value = 0;
    nextTick(() => {
      const element = fileItemRefs.value[0];
      if (element) {
        element.scrollIntoView({ block: "nearest", behavior: "smooth" });
      }
    });
  }
}

function showContextMenu(event: MouseEvent, file: FileEntry) {
  event.preventDefault();
  event.stopPropagation();
  event.stopImmediatePropagation();

  isEmptyContextMenu.value = false;

  // If there are multiple files selected and the clicked file is one of them,
  // we'll process all selected files for certain actions (delete, download, permissions)
  // For other actions (rename, open, edit), we'll only process the clicked file
  selectedFile.value = file;

  nextTick(() => {
    if (contextMenuRef.value && selectedFile.value) {
      contextMenuRef.value.show(event.clientX, event.clientY);
    }
  });
}

function showEmptyContextMenu(event: MouseEvent) {
  const target = event.target as HTMLElement;
  const fileItem = target.closest("[data-file-item]");

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

  if (!selectedFile.value) return;

  const multiFileActions = new Set(["delete", "download", "permissions"]);

  const shouldProcessAll =
    multiFileActions.has(action) &&
    props.selectedFiles.has(selectedFile.value.path) &&
    props.selectedFiles.size > 1;

  if (shouldProcessAll) {
    // Process all selected files for multi-file actions
    const filesToProcess: FileEntry[] = [];
    for (const file of sortedFiles.value) {
      if (props.selectedFiles.has(file.path)) {
        filesToProcess.push(file);
      }
    }

    // Emit all files at once for multi-file actions
    switch (action) {
      case "download":
        emit("download", filesToProcess);
        break;
      case "delete":
        emit("delete", filesToProcess);
        break;
      case "permissions":
        emit("permissions", filesToProcess);
        break;
    }
  } else {
    // Process only the clicked file
    switch (action) {
      case "open":
        emit("open", selectedFile.value);
        break;
      case "edit":
        emit("edit", selectedFile.value);
        break;
      case "download":
        emit("download", [selectedFile.value]);
        break;
      case "rename":
        emit("rename", selectedFile.value);
        break;
      case "delete":
        emit("delete", [selectedFile.value]);
        break;
      case "permissions":
        emit("permissions", [selectedFile.value]);
        break;
      case "createFile":
        emit("createFile");
        break;
      case "createDirectory":
        emit("createDirectory");
        break;
    }
  }

  selectedFile.value = null;
  isEmptyContextMenu.value = false;
}

function handleDragStart(event: DragEvent, file: FileEntry) {
  if (!event.dataTransfer) return;

  const filesToDrag =
    props.selectedFiles.size > 0 && isSelected(file.path)
      ? sortedFiles.value.filter((f) => props.selectedFiles.has(f.path))
      : [file];

  draggedFiles.value = filesToDrag;
  isDragging.value = true;

  const dragData = {
    files: filesToDrag.map((f) => ({
      path: f.path,
      name: f.name,
      fileType: f.fileType,
      size: f.size,
    })),
    isRemote: props.isRemote,
    sourcePath: props.currentPath,
  };

  const dragDataString = JSON.stringify(dragData);

  event.dataTransfer.effectAllowed = "copyMove";
  event.dataTransfer.dropEffect = "copy";

  // Set data in multiple formats to ensure it's available
  event.dataTransfer.setData("application/x-filebrowser-files", dragDataString);
  event.dataTransfer.setData(
    "text/plain",
    filesToDrag.map((f) => f.name).join(", "),
  );
  event.dataTransfer.setData("text/json", dragDataString);

  if (event.dataTransfer.setDragImage && event.target) {
    const dragImage = document.createElement("div");
    dragImage.textContent = `${filesToDrag.length} item${filesToDrag.length > 1 ? "s" : ""}`;
    dragImage.style.position = "absolute";
    dragImage.style.top = "-1000px";
    dragImage.style.color = "#e5e7eb"; // text-gray-200
    dragImage.style.backgroundColor = "#1f2937"; // bg-gray-800
    dragImage.style.padding = "8px 12px";
    dragImage.style.borderRadius = "6px";
    dragImage.style.fontSize = "14px";
    dragImage.style.fontWeight = "500";
    dragImage.style.border = "1px solid #374151"; // border-gray-700
    document.body.appendChild(dragImage);
    event.dataTransfer.setDragImage(dragImage, 0, 0);
    setTimeout(() => document.body.removeChild(dragImage), 0);
  }
}

function handleDragEnd() {
  isDragging.value = false;
  draggedFiles.value = [];
  dragOverIndex.value = -1;
  isDragOver.value = false;
}

function handleDragOver(event: DragEvent, file: FileEntry, index: number) {
  event.preventDefault();
  if (!event.dataTransfer) return;

  const isInternalDrag = event.dataTransfer.types.includes(
    "application/x-filebrowser-files",
  );

  if (isInternalDrag) {
    if (file.fileType === "directory") {
      dragOverIndex.value = index;
      event.dataTransfer.dropEffect = "copy";
    } else {
      event.dataTransfer.dropEffect = "copy";
    }
  } else {
    if (file.fileType === "directory") {
      dragOverIndex.value = index;
      event.dataTransfer.dropEffect = "copy";
    } else {
      dragOverIndex.value = -1;
      event.dataTransfer.dropEffect = "copy";
    }
  }
  isDragOver.value = true;
}

function handleDragLeave(index: number) {
  if (dragOverIndex.value === index) {
    dragOverIndex.value = -1;
  }
}

function handleFileDrop(event: DragEvent, file: FileEntry) {
  event.preventDefault();
  event.stopPropagation();
  if (!event.dataTransfer) return;

  isDragOver.value = false;
  dragOverIndex.value = -1;

  const dragData = event.dataTransfer.getData(
    "application/x-filebrowser-files",
  );
  if (dragData) {
    const data = JSON.parse(dragData) as FileBrowserDragData;
    const targetPath =
      file.fileType === "directory" ? file.path : props.currentPath;

    const draggedFileEntries: FileEntry[] = data.files.map((f) => ({
      name: f.name || "",
      path: f.path || "",
      fileType: f.fileType || "file",
      size: f.size ?? null,
      modified: f.modified || new Date().toISOString(),
      accessed: null,
      permissions: 0o644,
      symlinkTarget: null,
      uid: null,
      gid: null,
    }));

    emit("drag-files", draggedFileEntries, targetPath, data.isRemote);
    return;
  }

  handleExternalDrop(event);
}

function onDragEnter(event: DragEvent) {
  event.preventDefault();
  if (!event.dataTransfer) return;

  if (
    event.dataTransfer.types.includes("application/x-filebrowser-files") ||
    event.dataTransfer.types.includes("Files")
  ) {
    isDragOver.value = true;
  }
}

function onDragLeave(event: DragEvent) {
  const rect = fileListRef.value?.getBoundingClientRect();
  if (rect) {
    const x = event.clientX;
    const y = event.clientY;
    if (x < rect.left || x > rect.right || y < rect.top || y > rect.bottom) {
      isDragOver.value = false;
      dragOverIndex.value = -1;
    }
  }
}

function onDragOver(event: DragEvent) {
  event.preventDefault();
  event.stopPropagation();
  if (!event.dataTransfer) return;

  event.dataTransfer.dropEffect = "copy";
}

async function onDrop(event: DragEvent) {
  event.preventDefault();
  event.stopPropagation();

  if (!event.dataTransfer) return;

  isDragOver.value = false;
  dragOverIndex.value = -1;

  // Try to get data - it might be empty in drop event, so we need to check items
  let dragData = "";

  // First try getData
  try {
    dragData = event.dataTransfer.getData("application/x-filebrowser-files");
  } catch (e) {
    // Ignore
  }

  // If getData doesn't work, the data might not be available in drop event
  // This is expected behavior - dataTransfer.getData only works in drop event

  if (dragData) {
    try {
      const data = JSON.parse(dragData) as FileBrowserDragData;
      const targetPath = props.currentPath;

      const draggedFileEntries: FileEntry[] = data.files.map((f) => ({
        name: f.name || "",
        path: f.path || "",
        fileType: f.fileType || "file",
        size: f.size ?? null,
        modified: f.modified || new Date().toISOString(),
        accessed: null,
        permissions: 0o644,
        symlinkTarget: null,
        uid: null,
        gid: null,
      }));

      emit("drag-files", draggedFileEntries, targetPath, data.isRemote);
      return;
    } catch (error) {
      // Ignore parse errors
    }
  }

  handleExternalDrop(event);
}

// Recursively collect all files from a directory entry
async function collectFilesFromDirectory(
  entry: FileSystemDirectoryEntry,
  relativePath: string = "",
): Promise<File[]> {
  const files: File[] = [];
  const reader = entry.createReader();

  const readEntries = (): Promise<FileSystemEntry[]> => {
    return new Promise((resolve, reject) => {
      reader.readEntries(
        (entries) => resolve(entries),
        (error) => reject(error),
      );
    });
  };

  let entries: FileSystemEntry[] = [];
  try {
    entries = await readEntries();

    // Read all batches (readEntries may return partial results)
    while (entries.length > 0) {
      for (const entry of entries) {
        if (entry.isFile) {
          const fileEntry = entry as FileSystemFileEntry;
          const file = await new Promise<File | null>((resolve, reject) => {
            fileEntry.file(resolve, reject);
          });
          if (file) {
            // Preserve directory structure by adding path property
            const fileWithPath = file as File & { path?: string };
            fileWithPath.path =
              relativePath === ""
                ? entry.name
                : `${relativePath}/${entry.name}`;
            files.push(file);
          }
        } else if (entry.isDirectory) {
          const dirEntry = entry as FileSystemDirectoryEntry;
          const newRelativePath =
            relativePath === "" ? entry.name : `${relativePath}/${entry.name}`;
          const subFiles = await collectFilesFromDirectory(
            dirEntry,
            newRelativePath,
          );
          files.push(...subFiles);
        }
      }
      // Try to read more entries
      const moreEntries = await readEntries();
      entries = moreEntries;
    }
  } catch (error) {
    console.error("Error reading directory:", error);
  }

  return files;
}

async function handleExternalDrop(event: DragEvent) {
  if (!event.dataTransfer) return;

  const files: File[] = [];

  if (event.dataTransfer.items && event.dataTransfer.items.length > 0) {
    for (let i = 0; i < event.dataTransfer.items.length; i++) {
      const item = event.dataTransfer.items[i];

      if (item.kind === "file") {
        const entry = item.webkitGetAsEntry?.();
        if (entry) {
          if (entry.isFile) {
            const fileEntry = entry as FileSystemFileEntry;
            const file = await new Promise<File | null>((resolve, reject) => {
              fileEntry.file(resolve, reject);
            });
            if (file) {
              files.push(file);
            }
          } else if (entry.isDirectory) {
            // Handle directory drag & drop
            const dirEntry = entry as FileSystemDirectoryEntry;
            const dirFiles = await collectFilesFromDirectory(dirEntry);
            files.push(...dirFiles);
          }
        } else {
          const file = item.getAsFile();
          if (file) {
            files.push(file);
          }
        }
      }
    }
  }

  if (
    files.length === 0 &&
    event.dataTransfer.files &&
    event.dataTransfer.files.length > 0
  ) {
    files.push(...Array.from(event.dataTransfer.files));
  }

  if (files.length > 0) {
    emit("upload", files as unknown as FileList);
  }
}

function handlePathInput() {
  if (pathInput.value && pathInput.value !== props.currentPath) {
    showSuggestions.value = true;
    selectedSuggestionIndex.value = -1;
  }
}

function handlePathSubmit() {
  if (
    selectedSuggestionIndex.value >= 0 &&
    suggestions.value[selectedSuggestionIndex.value]
  ) {
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
  setTimeout(() => {
    showSuggestions.value = false;
    selectedSuggestionIndex.value = -1;
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

watch(
  () => props.files.length,
  () => {
    focusedIndex.value = -1;
    lastFocusedIndex.value = -1;
    fileItemRefs.value = [];
  },
);

onMounted(() => {
  if (fileListRef.value) {
    fileListRef.value.focus();
  }
});
</script>
