<!--
  - Copyright (C) 2026 Bùi Thanh Xuân (klpod221)
  - SPDX-License-Identifier: GPL-3.0-or-later
-->

<template>
  <Modal
    id="sftp-file-delete-modal"
    :title="isMultiple ? `Delete ${files?.length} Items` : 'Delete File'"
    :icon="Trash2"
    icon-background="bg-red-500/20"
    icon-color="text-red-400"
    size="md"
  >
    <div class="space-y-4">
      <p class="text-gray-300">
        <template v-if="isMultiple">
          Are you sure you want to delete
          <span class="font-medium text-white">{{ files?.length }} selected items</span>?
        </template>
        <template v-else>
          Are you sure you want to delete
          <span class="font-medium text-white">{{ file?.name }}</span>?
        </template>
      </p>
      <p class="text-sm text-gray-500">
        {{
          isMultiple
            ? "This will delete all selected files and directories. This action cannot be undone."
            : file?.fileType === "directory"
            ? "This will delete the directory and all its contents."
            : "This action cannot be undone."
        }}
      </p>
    </div>

    <template #footer>
      <Button variant="ghost" @click="closeModal">Cancel</Button>
      <Button variant="danger" :loading="loading" @click="handleSubmit">
        Delete
      </Button>
    </template>
  </Modal>
</template>

<script setup lang="ts">
import { ref, computed } from "vue";
import { Trash2 } from "lucide-vue-next";
import Modal from "../ui/Modal.vue";
import Button from "../ui/Button.vue";
import { useOverlay } from "../../composables/useOverlay";
import type { FileEntry } from "../../types/sftp";

const { closeOverlay, getOverlayProp } = useOverlay();

const loading = ref(false);

const file = getOverlayProp<FileEntry | null>(
  "sftp-file-delete-modal",
  "file",
  null,
  null,
);

const files = getOverlayProp<FileEntry[] | null>(
  "sftp-file-delete-modal",
  "files",
  null,
  null,
);

const isMultiple = computed(() => !!(files.value && files.value.length > 1));

async function handleSubmit() {
  const itemsToDelete = files.value && files.value.length > 0 ? files.value : (file.value ? [file.value] : []);
  if (itemsToDelete.length === 0 || loading.value) return;

  loading.value = true;
  const isLocal = getOverlayProp<boolean>(
    "sftp-file-delete-modal",
    "isLocal",
    false,
    false,
  );

  for (const item of itemsToDelete) {
    const event = new CustomEvent("sftp-delete", {
      detail: {
        path: item.path,
        isDirectory: item.fileType === "directory",
        isLocal: isLocal.value,
      },
    });
    globalThis.dispatchEvent(event);
  }

  closeModal();
  loading.value = false;
}

function closeModal() {
  closeOverlay("sftp-file-delete-modal");
}
</script>
