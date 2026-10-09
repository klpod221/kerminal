<!--
  Copyright (C) 2026 Bùi Thanh Xuân (klpod221)
  SPDX-License-Identifier: GPL-3.0-or-later
-->

<template>
  <Modal
    id="sftp-file-preview-modal"
    :title="`Preview: ${file?.name || 'File'}`"
    :icon="Eye"
    icon-background="bg-blue-500/20"
    icon-color="text-blue-400"
    size="5xl"
  >
    <div v-if="loading" class="flex h-64 items-center justify-center text-gray-400 font-mono text-xs">
      <div class="flex items-center gap-2">
        <RotateCw class="h-4 w-4 animate-spin text-blue-400" />
        <span>Loading preview content...</span>
      </div>
    </div>

    <div v-else-if="error" class="flex h-64 flex-col items-center justify-center gap-2 text-red-400 font-mono text-xs">
      <AlertTriangle class="h-6 w-6 text-red-400" />
      <span>{{ error }}</span>
      <Button variant="secondary" size="sm" @click="loadContent">Retry</Button>
    </div>

    <div v-else class="flex flex-col gap-3 h-[60vh] font-mono text-xs">
      <!-- File metadata bar -->
      <div class="flex items-center justify-between border-b border-gray-800 pb-2 text-2xs text-gray-400">
        <span class="truncate">{{ file?.path }}</span>
        <span>{{ formatBytes(file?.size || 0) }}</span>
      </div>

      <!-- Preview body -->
      <div class="flex-1 overflow-auto rounded border border-gray-800 bg-bg-primary p-3">
        <!-- Image preview -->
        <div v-if="previewType === 'image'" class="flex h-full items-center justify-center">
          <img
            :src="mediaSrc"
            alt="Preview"
            class="max-h-full max-w-full rounded object-contain shadow-md"
          />
        </div>

        <!-- Video preview -->
        <div v-else-if="previewType === 'video'" class="flex h-full items-center justify-center">
          <video :src="mediaSrc" controls class="max-h-full max-w-full rounded"></video>
        </div>

        <!-- Audio preview -->
        <div v-else-if="previewType === 'audio'" class="flex h-full items-center justify-center">
          <audio :src="mediaSrc" controls class="w-full max-w-md"></audio>
        </div>

        <!-- Text / Code preview -->
        <div v-else class="h-full">
          <pre class="h-full overflow-auto font-mono text-xs text-gray-200 leading-relaxed">{{ textContent }}</pre>
        </div>
      </div>
    </div>

    <template #footer>
      <div class="flex items-center justify-between w-full font-mono">
        <span class="text-2xs text-gray-500">
          {{ previewType.toUpperCase() }} preview
        </span>
        <Button variant="secondary" size="sm" @click="close">Close</Button>
      </div>
    </template>
  </Modal>
</template>

<script setup lang="ts">
import { ref, computed, watch } from "vue";
import { Eye, RotateCw, AlertTriangle } from "lucide-vue-next";
import Modal from "../ui/Modal.vue";
import Button from "../ui/Button.vue";
import { useOverlay } from "../../composables/useOverlay";
import type { FileEntry } from "../../types/sftp";
import { readSFTPFile } from "../../services/sftp";

const { closeOverlay, getOverlayProp } = useOverlay();

const file = getOverlayProp<FileEntry | null>(
  "sftp-file-preview-modal",
  "file",
  null,
  null,
);

const sessionId = getOverlayProp<string>(
  "sftp-file-preview-modal",
  "sessionId",
  "",
  "",
);

const loading = ref(false);
const error = ref<string | null>(null);
const textContent = ref("");
const mediaSrc = ref("");

const fileExt = computed(() => {
  if (!file.value) return "";
  return file.value.name.split(".").pop()?.toLowerCase() || "";
});

const previewType = computed(() => {
  const ext = fileExt.value;
  if (["png", "jpg", "jpeg", "gif", "webp", "svg"].includes(ext)) return "image";
  if (["mp4", "webm", "mkv"].includes(ext)) return "video";
  if (["mp3", "wav", "ogg", "flac"].includes(ext)) return "audio";
  return "text";
});

async function loadContent() {
  if (!file.value || !sessionId.value) return;
  loading.value = true;
  error.value = null;

  try {
    if (previewType.value === "text") {
      const data = await readSFTPFile(sessionId.value, file.value.path);
      textContent.value = data;
    } else {
      textContent.value = "(Tệp đa phương tiện không hỗ trợ hiển thị văn bản)";
    }
  } catch (e: any) {
    error.value = e?.message || "Không thể đọc nội dung file";
  } finally {
    loading.value = false;
  }
}

watch(
  () => file.value?.path,
  () => {
    if (file.value) {
      loadContent();
    }
  },
  { immediate: true },
);

function close() {
  closeOverlay("sftp-file-preview-modal");
}

function formatBytes(bytes: number): string {
  if (bytes === 0) return "0 B";
  const k = 1024;
  const sizes = ["B", "KB", "MB", "GB"];
  const i = Math.floor(Math.log(bytes) / Math.log(k));
  return `${parseFloat((bytes / Math.pow(k, i)).toFixed(1))} ${sizes[i]}`;
}
</script>
