<!--
  Copyright (C) 2026 Bùi Thanh Xuân (klpod221)
  SPDX-License-Identifier: GPL-3.0-or-later
-->

<template>
  <div class="flex flex-col border-b border-gray-800 bg-bg-secondary px-3 py-2 text-xs">
    <!-- Top row: Pane title, quick actions -->
    <div class="flex items-center justify-between pb-1.5">
      <div class="flex items-center gap-2">
        <span
          class="inline-flex items-center gap-1.5 rounded px-2 py-0.5 text-2xs font-semibold uppercase tracking-wider"
          :class="isRemote ? 'bg-purple-500/10 text-purple-400 border border-purple-500/20' : 'bg-cyan-500/10 text-cyan-400 border border-cyan-500/20'"
        >
          <component :is="isRemote ? Server : HardDrive" class="h-3 w-3" />
          {{ isRemote ? 'Remote Server' : 'Local Machine' }}
        </span>
        <span v-if="selectedCount && selectedCount > 0" class="text-2xs text-gray-400 font-mono">
          ({{ selectedCount }} selected)
        </span>
      </div>

      <!-- Quick action icons -->
      <div class="flex items-center gap-1">
        <Button
          variant="ghost"
          size="sm"
          :icon="FolderPlus"
          title="New Folder"
          class="h-7 w-7 p-0 text-gray-400 hover:text-white"
          @click="$emit('new-folder')"
        />

        <Button
          variant="ghost"
          size="sm"
          :icon="FilePlus"
          title="New File"
          class="h-7 w-7 p-0 text-gray-400 hover:text-white"
          @click="$emit('new-file')"
        />

        <!-- Edit button (visible when 1 file selected) -->
        <Button
          v-if="selectedCount === 1"
          variant="ghost"
          size="sm"
          :icon="FileEdit"
          title="Edit file"
          class="h-7 w-7 p-0 text-blue-400 hover:text-blue-300 hover:bg-blue-500/10"
          @click="$emit('edit')"
        />

        <!-- Rename button (visible when 1 item selected) -->
        <Button
          v-if="selectedCount === 1"
          variant="ghost"
          size="sm"
          :icon="Pencil"
          title="Rename (F2)"
          class="h-7 w-7 p-0 text-yellow-400 hover:text-yellow-300 hover:bg-yellow-500/10"
          @click="$emit('rename')"
        />

        <!-- Delete button (visible when item(s) selected) -->
        <Button
          v-if="selectedCount && selectedCount > 0"
          variant="ghost"
          size="sm"
          :icon="Trash2"
          title="Delete selected (Del)"
          class="h-7 w-7 p-0 text-red-400 hover:text-red-300 hover:bg-red-500/10"
          @click="$emit('delete')"
        />

        <Button
          v-if="isRemote"
          variant="ghost"
          size="sm"
          :icon="Download"
          title="Download to Local"
          :disabled="!selectedCount"
          class="h-7 w-7 p-0 text-gray-400 hover:text-cyan-400 disabled:opacity-30"
          @click="$emit('transfer-action')"
        />
        <Button
          v-else
          variant="ghost"
          size="sm"
          :icon="Upload"
          title="Upload to Server"
          :disabled="!selectedCount"
          class="h-7 w-7 p-0 text-gray-400 hover:text-purple-400 disabled:opacity-30"
          @click="$emit('transfer-action')"
        />

        <Button
          variant="ghost"
          size="sm"
          :icon="showHidden ? Eye : EyeOff"
          :title="showHidden ? 'Hide hidden files' : 'Show hidden files'"
          class="h-7 w-7 p-0"
          :class="showHidden ? 'text-blue-400' : 'text-gray-400 hover:text-white'"
          @click="$emit('toggle-hidden')"
        />
      </div>
    </div>

    <!-- Bottom row: Nav controls, Path Input, and Search Filter -->
    <div class="flex items-center gap-2">
      <div class="flex items-center gap-0.5 shrink-0">
        <Button
          variant="ghost"
          size="sm"
          :icon="ArrowLeft"
          :disabled="!canGoBack"
          title="Back"
          class="h-7 w-7 p-0 text-gray-400 hover:text-white disabled:opacity-30"
          @click="$emit('back')"
        />

        <Button
          variant="ghost"
          size="sm"
          :icon="ArrowRight"
          :disabled="!canGoForward"
          title="Forward"
          class="h-7 w-7 p-0 text-gray-400 hover:text-white disabled:opacity-30"
          @click="$emit('forward')"
        />

        <Button
          variant="ghost"
          size="sm"
          :icon="ArrowUp"
          title="Up one directory"
          class="h-7 w-7 p-0 text-gray-400 hover:text-white"
          @click="$emit('up')"
        />

        <Button
          variant="ghost"
          size="sm"
          :icon="RotateCw"
          title="Refresh"
          class="h-7 w-7 p-0 text-gray-400 hover:text-white"
          :class="{ 'animate-spin': loading }"
          @click="$emit('refresh')"
        />
      </div>

      <!-- Path Input -->
      <div class="relative flex-1 min-w-0">
        <input
          v-model="pathInput"
          type="text"
          class="w-full rounded border border-gray-800 bg-bg-primary px-2.5 py-1 font-mono text-xs text-gray-200 outline-none transition-colors focus:border-blue-500/80"
          placeholder="/path/to/directory"
          @keydown.enter="handlePathSubmit"
        />
      </div>

      <!-- Quick Search Filter -->
      <div class="relative w-28 sm:w-36 shrink-0">
        <Search class="absolute left-2 top-1.5 h-3 w-3 text-gray-500" />
        <input
          v-model="searchModel"
          type="text"
          placeholder="Filter..."
          class="w-full rounded border border-gray-800 bg-bg-primary py-1 pl-6 pr-2 text-xs text-gray-200 outline-none transition-colors focus:border-blue-500/80"
        />
      </div>
    </div>
  </div>
</template>

<script setup lang="ts">
import { ref, watch, computed } from "vue";
import Button from "../ui/Button.vue";
import {
  ArrowLeft,
  ArrowRight,
  ArrowUp,
  RotateCw,
  FolderPlus,
  FilePlus,
  FileEdit,
  Pencil,
  Trash2,
  Upload,
  Download,
  Eye,
  EyeOff,
  Search,
  Server,
  HardDrive,
} from "lucide-vue-next";

interface Props {
  currentPath: string;
  loading?: boolean;
  canGoBack?: boolean;
  canGoForward?: boolean;
  showHidden?: boolean;
  searchQuery?: string;
  isRemote?: boolean;
  selectedCount?: number;
}

const props = withDefaults(defineProps<Props>(), {
  loading: false,
  canGoBack: false,
  canGoForward: false,
  showHidden: false,
  searchQuery: "",
  isRemote: false,
  selectedCount: 0,
});

const emit = defineEmits<{
  (e: "back"): void;
  (e: "forward"): void;
  (e: "up"): void;
  (e: "refresh"): void;
  (e: "navigate", path: string): void;
  (e: "new-folder"): void;
  (e: "new-file"): void;
  (e: "edit"): void;
  (e: "rename"): void;
  (e: "delete"): void;
  (e: "transfer-action"): void;
  (e: "toggle-hidden"): void;
  (e: "update:searchQuery", query: string): void;
}>();

const pathInput = ref(props.currentPath);

watch(
  () => props.currentPath,
  (newVal) => {
    pathInput.value = newVal;
  },
);

const searchModel = computed({
  get: () => props.searchQuery,
  set: (val: string) => emit("update:searchQuery", val),
});

function handlePathSubmit() {
  const p = pathInput.value.trim();
  if (p) {
    emit("navigate", p);
  }
}
</script>

