<template>
  <Modal
    id="sftp-file-rename-modal"
    title="Rename File"
    :icon="FileText"
    icon-background="bg-blue-500/20"
    icon-color="text-blue-400"
    size="md"
  >
    <Form ref="renameForm" @submit="handleSubmit">
      <Input
        id="new-name"
        v-model="newName"
        label="New Name"
        placeholder="Enter new name"
        rules="required|min:1|max:255"
        autofocus
      />
    </Form>

    <template #footer>
      <Button variant="ghost" @click="closeModal">Cancel</Button>
      <Button variant="primary" :loading="loading" @click="handleSubmit">
        Rename
      </Button>
    </template>
  </Modal>
</template>

<script setup lang="ts">
import { ref, watch } from "vue";
import { FileText } from "lucide-vue-next";
import Modal from "../ui/Modal.vue";
import Form from "../ui/Form.vue";
import Input from "../ui/Input.vue";
import Button from "../ui/Button.vue";
import { useOverlay } from "../../composables/useOverlay";
import { message } from "../../utils/message";
import { getErrorMessage } from "../../utils/helpers";
import type { FileEntry } from "../../types/sftp";

const { closeOverlay, getOverlayProp } = useOverlay();

const renameForm = ref<InstanceType<typeof Form> | null>(null);
const loading = ref(false);
const newName = ref("");

const file = getOverlayProp<FileEntry | null>(
  "sftp-file-rename-modal",
  "file",
  null,
  null,
);

watch(
  () => file.value,
  (fileValue) => {
    if (fileValue) {
      newName.value = fileValue.name;
    }
  },
  { immediate: true },
);

async function handleSubmit() {
  const isValid = await renameForm.value?.validate();
  if (!isValid || !file.value || !newName.value) return;

  // Check if name contains path separators
  if (newName.value.includes("/") || newName.value.includes("\\")) {
    message.error("File name cannot contain path separators");
    return;
  }

  const newPath = file.value.path.replace(/[^/]+$/, newName.value);

  if (newPath === file.value.path) {
    closeModal();
    return;
  }

  loading.value = true;
  try {
    const isLocal = getOverlayProp<boolean>(
      "sftp-file-rename-modal",
      "isLocal",
      false,
      false,
    );

    // Emit event to parent to handle rename
    const event = new CustomEvent("sftp-rename", {
      detail: { oldPath: file.value.path, newPath, isLocal: isLocal.value },
    });
    window.dispatchEvent(event);
    closeModal();
  } catch (error) {
    console.error("Failed to rename:", error);
    message.error(
      getErrorMessage(error, "Failed to rename file"),
    );
    loading.value = false;
  }
}

function closeModal() {
  newName.value = "";
  closeOverlay("sftp-file-rename-modal");
}
</script>

