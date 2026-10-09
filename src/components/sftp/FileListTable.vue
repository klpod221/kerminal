<!--
  Copyright (C) 2026 Bùi Thanh Xuân (klpod221)
  SPDX-License-Identifier: GPL-3.0-or-later
-->

<template>
  <div
    tabindex="0"
    class="relative h-full w-full select-none overflow-y-auto bg-bg-primary font-mono text-xs text-gray-300 outline-none"
    @dragover.prevent
    @drop.prevent="onDrop"
    @contextmenu.prevent="onEmptyContextMenu"
    @keydown="handleKeyDown"
  >
    <!-- Loading indicator -->
    <div
      v-if="loading"
      class="absolute inset-0 z-20 flex items-center justify-center bg-bg-primary/70 backdrop-blur-xs"
    >
      <div class="flex items-center gap-2.5 rounded-lg border border-gray-800 bg-bg-secondary px-4 py-2 text-xs text-gray-200 shadow-xl">
        <RotateCw class="h-4 w-4 animate-spin text-blue-400" />
        <span>Loading...</span>
      </div>
    </div>

    <!-- Error state -->
    <div
      v-else-if="error"
      class="flex h-48 flex-col items-center justify-center gap-2.5 p-4 text-center text-red-400"
    >
      <AlertTriangle class="h-8 w-8 text-red-400" />
      <span class="max-w-md text-xs">{{ error }}</span>
      <Button
        variant="outline"
        size="sm"
        text="Retry"
        :icon="RotateCw"
        class="mt-2"
        @click="$emit('refresh')"
      />
    </div>

    <!-- Empty folder state using Kerminal's EmptyState component -->
    <div v-else-if="entries.length === 0" class="flex h-full items-center justify-center p-6">
      <EmptyState
        :icon="Folder"
        title="Empty Directory"
        description="No files or folders found in this location."
      />
    </div>

    <!-- File Table -->
    <table v-else class="w-full border-collapse text-left">
      <thead class="sticky top-0 z-10 border-b border-gray-800 bg-bg-secondary text-gray-400 font-medium select-none">
        <tr>
          <th class="w-8 px-2 py-1.5 text-center">
            <input
              type="checkbox"
              :checked="isAllSelected"
              class="rounded border-gray-700 bg-bg-primary text-blue-500 focus:ring-0 cursor-pointer"
              @change="$emit('toggle-select-all')"
            />
          </th>
          <th
            class="cursor-pointer px-3 py-1.5 hover:text-white"
            @click="$emit('sort', 'name')"
          >
            <div class="flex items-center gap-1">
              <span>Name</span>
              <span v-if="sortField === 'name'" class="text-blue-400">{{ sortAsc ? '▲' : '▼' }}</span>
            </div>
          </th>
          <th
            class="w-24 cursor-pointer px-3 py-1.5 hover:text-white"
            @click="$emit('sort', 'size')"
          >
            <div class="flex items-center gap-1">
              <span>Size</span>
              <span v-if="sortField === 'size'" class="text-blue-400">{{ sortAsc ? '▲' : '▼' }}</span>
            </div>
          </th>
          <th
            class="w-20 cursor-pointer px-3 py-1.5 hover:text-white"
            @click="$emit('sort', 'type')"
          >
            <span>Perms</span>
          </th>
          <th
            class="w-36 cursor-pointer px-3 py-1.5 hover:text-white"
            @click="$emit('sort', 'modified')"
          >
            <div class="flex items-center gap-1">
              <span>Modified</span>
              <span v-if="sortField === 'modified'" class="text-blue-400">{{ sortAsc ? '▲' : '▼' }}</span>
            </div>
          </th>
        </tr>
      </thead>
      <tbody class="divide-y divide-gray-800/40">
        <tr
          v-for="item in entries"
          :key="item.path"
          draggable="true"
          class="group transition-colors hover:bg-bg-tertiary cursor-pointer"
          :class="{ 'bg-blue-500/10 text-white font-medium': selectedPaths.has(item.path) }"
          @click="handleRowClick($event, item)"
          @dblclick="$emit('open', item)"
          @contextmenu.prevent.stop="handleContextMenu($event, item)"
          @dragstart="onDragStart($event, item)"
        >
          <td class="px-2 py-1.5 text-center" @click.stop>
            <input
              type="checkbox"
              :checked="selectedPaths.has(item.path)"
              class="rounded border-gray-700 bg-bg-primary text-blue-500 focus:ring-0 cursor-pointer"
              @change="$emit('toggle-select', item.path, true)"
            />
          </td>

          <!-- File Name & Icon -->
          <td class="px-3 py-1.5">
            <div class="flex items-center gap-2">
              <Folder v-if="item.fileType === 'directory'" class="h-4 w-4 shrink-0 text-blue-400" />
              <Link2 v-else-if="item.fileType === 'symlink'" class="h-4 w-4 shrink-0 text-yellow-400" />
              <component :is="getFileIcon(item.name)" v-else class="h-4 w-4 shrink-0 text-gray-400 group-hover:text-gray-200" />

              <span class="truncate" :title="item.name">{{ item.name }}</span>

              <span v-if="item.symlinkTarget" class="text-2xs text-gray-500 truncate">
                → {{ item.symlinkTarget }}
              </span>
            </div>
          </td>

          <!-- Size -->
          <td class="px-3 py-1.5 text-gray-400 font-mono">
            {{ item.fileType === 'directory' ? '--' : formatBytes(item.size || 0) }}
          </td>

          <!-- Permissions -->
          <td class="px-3 py-1.5 text-2xs text-gray-500 font-mono">
            {{ formatPermissions(item.permissions) }}
          </td>

          <!-- Modified Date -->
          <td class="px-3 py-1.5 text-2xs text-gray-400">
            {{ formatDate(item.modified) }}
          </td>
        </tr>
      </tbody>
    </table>

    <!-- Kerminal Context Menu -->
    <ContextMenu
      ref="contextMenuRef"
      :items="contextMenuItems"
      @item-click="handleMenuItemClick"
    />
  </div>
</template>

<script setup lang="ts">
import { ref, computed } from "vue";
import type { FileEntry } from "../../types/sftp";
import type { SortField } from "../../composables/sftp/useFileExplorer";
import Button from "../ui/Button.vue";
import EmptyState from "../ui/EmptyState.vue";
import ContextMenu, { type ContextMenuItem } from "../ui/ContextMenu.vue";
import {
  RotateCw,
  AlertTriangle,
  Folder,
  FolderPlus,
  FolderOpen,
  File,
  FilePlus,
  FileCode,
  FileText,
  FileImage,
  FileArchive,
  Link2,
  Download,
  Upload,
  FileEdit,
  Pencil,
  Trash2,
  Eye,
  Shield,
  Copy,
  ExternalLink,
  CheckSquare,
} from "lucide-vue-next";

interface Props {
  entries: FileEntry[];
  loading?: boolean;
  error?: string | null;
  selectedPaths: Set<string>;
  sortField?: SortField;
  sortAsc?: boolean;
  isRemote?: boolean;
}

const props = withDefaults(defineProps<Props>(), {
  loading: false,
  error: null,
  sortField: "name",
  sortAsc: true,
  isRemote: false,
});

const emit = defineEmits<{
  (e: "open", item: FileEntry): void;
  (e: "refresh"): void;
  (e: "sort", field: SortField): void;
  (e: "toggle-select", path: string, multi: boolean): void;
  (e: "toggle-select-all"): void;
  (e: "clear-selection"): void;
  (e: "action", action: string, item: FileEntry): void;
  (e: "drop-transfer", data: { paths: string[]; isRemoteSource: boolean }): void;
}>();

const contextMenuRef = ref<InstanceType<typeof ContextMenu> | null>(null);
const targetItem = ref<FileEntry | null>(null);

const isAllSelected = computed(() => {
  if (props.entries.length === 0) return false;
  return props.entries.every((e) => props.selectedPaths.has(e.path));
});

const contextMenuItems = computed<ContextMenuItem[]>(() => {
  const count = props.selectedPaths.size;
  const item = targetItem.value;

  // Case 1: Empty area right-clicked
  if (!item) {
    return [
      { id: "new-folder", label: "New Folder", icon: FolderPlus },
      { id: "new-file", label: "New File", icon: FilePlus },
      { id: "divider-1", type: "divider" },
      { id: "select-all", label: "Select All", icon: CheckSquare, shortcut: "Ctrl+A" },
      { id: "refresh", label: "Refresh", icon: RotateCw, shortcut: "F5" },
    ];
  }

  // Case 2: Multi-selection right-clicked
  if (count > 1 && props.selectedPaths.has(item.path)) {
    return [
      {
        id: props.isRemote ? "download" : "upload",
        label: props.isRemote ? `Download ${count} Items` : `Upload ${count} Items`,
        icon: props.isRemote ? Download : Upload,
      },
      { id: "copy-path", label: `Copy ${count} Paths`, icon: Copy },
      { id: "divider-1", type: "divider" },
      { id: "delete", label: `Delete ${count} Items`, icon: Trash2, danger: true, shortcut: "Del" },
    ];
  }

  // Case 3: Single Folder right-clicked
  const isDir = item.fileType === "directory";
  if (isDir) {
    return [
      { id: "open", label: "Open Folder", icon: FolderOpen, shortcut: "Enter" },
      {
        id: props.isRemote ? "download" : "upload",
        label: props.isRemote ? "Download to Local" : "Upload to Server",
        icon: props.isRemote ? Download : Upload,
      },
      { id: "divider-1", type: "divider" },
      { id: "new-folder-inside", label: "New Subfolder", icon: FolderPlus },
      { id: "new-file-inside", label: "New File Inside", icon: FilePlus },
      { id: "divider-2", type: "divider" },
      { id: "rename", label: "Rename", icon: Pencil, shortcut: "F2" },
      { id: "copy-path", label: "Copy Path", icon: Copy },
      ...(props.isRemote ? [{ id: "permissions", label: "Permissions (chmod)", icon: Shield }] : []),
      { id: "divider-3", type: "divider" },
      { id: "delete", label: "Delete Folder", icon: Trash2, danger: true, shortcut: "Del" },
    ];
  }

  // Case 4: Single File right-clicked
  const ext = item.name.split(".").pop()?.toLowerCase() || "";
  const isText = ["txt", "md", "js", "ts", "vue", "rs", "go", "py", "json", "html", "css", "yaml", "yml", "sh", "sql", "toml"].includes(ext);
  const isMedia = ["png", "jpg", "jpeg", "gif", "svg", "webp", "mp4", "mp3"].includes(ext);

  return [
    ...(isText ? [{ id: "edit", label: "Edit File", icon: FileEdit, shortcut: "Enter" }] : []),
    ...(isMedia || isText ? [{ id: "preview", label: "Quick Preview", icon: Eye }] : []),
    ...(!props.isRemote && !isText && !isMedia ? [{ id: "open-system", label: "Open with Default App", icon: ExternalLink }] : []),
    {
      id: props.isRemote ? "download" : "upload",
      label: props.isRemote ? "Download to Local" : "Upload to Server",
      icon: props.isRemote ? Download : Upload,
    },
    { id: "divider-1", type: "divider" },
    { id: "rename", label: "Rename", icon: Pencil, shortcut: "F2" },
    { id: "copy-path", label: "Copy Path", icon: Copy },
    ...(props.isRemote ? [{ id: "permissions", label: "Permissions (chmod)", icon: Shield }] : []),
    { id: "divider-2", type: "divider" },
    { id: "delete", label: "Delete", icon: Trash2, danger: true, shortcut: "Del" },
  ];
});

function handleRowClick(event: MouseEvent, item: FileEntry) {
  const multi = event.ctrlKey || event.metaKey || event.shiftKey;
  emit("toggle-select", item.path, multi);
}

function handleContextMenu(event: MouseEvent, item: FileEntry) {
  event.preventDefault();
  event.stopPropagation();
  if (!props.selectedPaths.has(item.path)) {
    emit("toggle-select", item.path, false);
  }
  targetItem.value = item;
  contextMenuRef.value?.show(event.clientX, event.clientY);
}

function onEmptyContextMenu(event: MouseEvent) {
  const target = event.target as HTMLElement | null;
  if (target?.closest("tr")) return;
  targetItem.value = null;
  contextMenuRef.value?.show(event.clientX, event.clientY);
}

function handleMenuItemClick(item: ContextMenuItem) {
  if (item.id === "refresh") {
    emit("refresh");
  } else if (item.id === "select-all") {
    emit("toggle-select-all");
  } else if (item.id === "new-folder") {
    emit("action", "new-folder-current", {} as any);
  } else if (item.id === "new-file") {
    emit("action", "new-file-current", {} as any);
  } else if (targetItem.value) {
    if (item.id === "open") {
      emit("open", targetItem.value);
    } else {
      emit("action", item.id, targetItem.value);
    }
  }
}

function handleKeyDown(event: KeyboardEvent) {
  if (event.ctrlKey || event.metaKey) {
    if (event.key.toLowerCase() === "a") {
      event.preventDefault();
      emit("toggle-select-all");
      return;
    }
  }

  if (event.key === "Escape") {
    event.preventDefault();
    emit("clear-selection");
    return;
  }

  if (props.selectedPaths.size === 0) return;
  const selectedList = props.entries.filter((e) => props.selectedPaths.has(e.path));
  if (selectedList.length === 0) return;

  if (event.key === "Delete" || event.key === "Backspace") {
    event.preventDefault();
    emit("action", "delete", selectedList[0]);
  } else if (event.key === "F2" && selectedList.length === 1) {
    event.preventDefault();
    emit("action", "rename", selectedList[0]);
  } else if (event.key === "Enter" && selectedList.length === 1) {
    event.preventDefault();
    emit("open", selectedList[0]);
  }
}

function onDragStart(event: DragEvent, item: FileEntry) {
  if (!event.dataTransfer) return;
  const paths = props.selectedPaths.has(item.path)
    ? Array.from(props.selectedPaths)
    : [item.path];
  event.dataTransfer.setData(
    "application/kerminal-files",
    JSON.stringify({ paths, isRemoteSource: props.isRemote }),
  );
}

function onDrop(event: DragEvent) {
  if (!event.dataTransfer) return;
  const raw = event.dataTransfer.getData("application/kerminal-files");
  if (!raw) return;
  try {
    const data = JSON.parse(raw);
    emit("drop-transfer", data);
  } catch {
    // ignore
  }
}

function getFileIcon(filename: string) {
  const ext = filename.split(".").pop()?.toLowerCase();
  switch (ext) {
    case "js":
    case "ts":
    case "vue":
    case "rs":
    case "go":
    case "py":
    case "json":
    case "html":
    case "css":
    case "yaml":
    case "yml":
      return FileCode;
    case "png":
    case "jpg":
    case "jpeg":
    case "gif":
    case "svg":
    case "webp":
      return FileImage;
    case "zip":
    case "tar":
    case "gz":
    case "xz":
    case "7z":
      return FileArchive;
    case "txt":
    case "md":
    case "log":
      return FileText;
    default:
      return File;
  }
}

function formatBytes(bytes: number): string {
  if (bytes === 0) return "0 B";
  const k = 1024;
  const sizes = ["B", "KB", "MB", "GB", "TB"];
  const i = Math.floor(Math.log(bytes) / Math.log(k));
  return `${parseFloat((bytes / Math.pow(k, i)).toFixed(1))} ${sizes[i]}`;
}

function formatPermissions(mode: number): string {
  if (!mode) return "---";
  const octal = (mode & 0o777).toString(8).padStart(3, "0");
  return octal;
}

function formatDate(iso: string): string {
  try {
    const d = new Date(iso);
    return `${d.toLocaleDateString()} ${d.toLocaleTimeString([], { hour: "2-digit", minute: "2-digit" })}`;
  } catch {
    return iso;
  }
}
</script>

