<template>
  <Modal
    id="sftp-file-delete-modal"
    title="Delete File"
    :icon="Trash2"
    icon-background="bg-red-500/20"
    icon-color="text-red-400"
    size="md"
  >
    <div class="space-y-4">
      <p class="text-gray-300">
        Are you sure you want to delete
        <span class="font-medium text-white">{{ file?.name }}</span>?
      </p>
      <p class="text-sm text-gray-500">
        {{
          file?.fileType === "directory"
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
import { ref } from "vue";
import { Trash2 } from "lucide-vue-next";
import Modal from "../ui/Modal.vue";
import Button from "../ui/Button.vue";
import { useOverlay } from "../../composables/useOverlay";
import { message } from "../../utils/message";
import { getErrorMessage } from "../../utils/helpers";
import type { FileEntry } from "../../types/sftp";

const { closeOverlay, getOverlayProp } = useOverlay();

const loading = ref(false);

const file = getOverlayProp<FileEntry | null>(
  "sftp-file-delete-modal",
  "file",
  null,
  null,
);

async function handleSubmit() {
  if (!file.value || loading.value) return;

  loading.value = true;
  try {
    const isLocal = getOverlayProp<boolean>(
      "sftp-file-delete-modal",
      "isLocal",
      false,
      false,
    );
    
    // Emit event to parent to handle delete
    const event = new CustomEvent("sftp-delete", {
      detail: {
        path: file.value.path,
        isDirectory: file.value.fileType === "directory",
        isLocal: isLocal.value,
      },
    });
    window.dispatchEvent(event);
    closeModal();
  } catch (error) {
    console.error("Failed to delete:", error);
    message.error(getErrorMessage(error, "Failed to delete file"));
    loading.value = false;
  }
}

function closeModal() {
  closeOverlay("sftp-file-delete-modal");
}
</script>

