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
      <div class="flex-1 px-3 py-1 bg-gray-900 rounded text-sm text-gray-300 font-mono">
        {{ currentPath }}
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
      class="flex-1 overflow-auto"
      @dragover.prevent="onDragOver"
      @drop.prevent="onDrop"
      @contextmenu.prevent="showEmptyContextMenu"
    >
      <EmptyState
        v-if="files.length === 0 && !loading"
        :icon="Folder"
        title="Empty directory"
        description="No files or folders in this directory"
      />

      <div v-else class="p-2 space-y-1">
        <div
          v-for="file in sortedFiles"
          :key="file.path"
          data-file-item
          class="flex items-center gap-3 px-2 rounded hover:bg-gray-800 cursor-pointer group"
          :class="{
            'bg-gray-800': isSelected(file.path),
          }"
          @click="handleFileClick(file)"
          @contextmenu.prevent="showContextMenu($event, file)"
        >
          <!-- Icon -->
          <div class="shrink-0">
            <Folder
              v-if="file.fileType === 'directory'"
              :size="20"
              class="text-blue-400"
            />
            <File
              v-else-if="file.fileType === 'file'"
              :size="20"
              class="text-gray-400"
            />
            <Link
              v-else-if="file.fileType === 'symlink'"
              :size="20"
              class="text-yellow-400"
            />
            <FileQuestion
              v-else
              :size="20"
              class="text-gray-500"
            />
          </div>

          <!-- File name -->
          <div class="flex-1 min-w-0">
            <div class="text-sm text-gray-200 truncate">
              {{ file.name }}
              <span
                v-if="file.fileType === 'symlink' && file.symlinkTarget"
                class="text-gray-500 text-xs ml-1"
              >
                → {{ file.symlinkTarget }}
              </span>
            </div>
            <div class="text-xs text-gray-500 mt-0.5">
              {{ formatFileInfo(file) }}
            </div>
          </div>

          <!-- Selection checkbox (hidden by default, shown on hover or selection) -->
          <div
            class="shrink-0 opacity-0 group-hover:opacity-100 transition-opacity"
            :class="{ 'opacity-100': isSelected(file.path) }"
            @click.stop="toggleSelection(file.path)"
          >
            <input
              type="checkbox"
              :checked="isSelected(file.path)"
              @click.stop="toggleSelection(file.path)"
              class="cursor-pointer"
            />
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
import { ref, computed, nextTick } from "vue";
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
  return [...props.files].sort((a, b) => {
    // Directories first
    if (a.fileType === "directory" && b.fileType !== "directory") {
      return -1;
    }
    if (a.fileType !== "directory" && b.fileType === "directory") {
      return 1;
    }
    // Then alphabetical
    return a.name.localeCompare(b.name);
  });
});

function formatFileInfo(file: FileEntry): string {
  const parts: string[] = [];
  if (file.fileType === "file" && file.size !== null) {
    parts.push(formatBytes(file.size));
  }
  if (file.modified) {
    const date = new Date(file.modified);
    parts.push(date.toLocaleDateString());
  }
  return parts.join(" • ");
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

function handleFileClick(file: FileEntry) {
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
</script>

