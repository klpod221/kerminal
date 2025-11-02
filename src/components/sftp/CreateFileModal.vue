<template>
  <Modal
    id="sftp-create-file-modal"
    title="Create File"
    :icon="FilePlus"
    icon-background="bg-blue-500/20"
    icon-color="text-blue-400"
    size="md"
  >
    <Form ref="createFileForm" @submit="handleSubmit">
      <Input
        id="file-name"
        v-model="fileName"
        label="File Name"
        placeholder="Enter file name"
        rules="required|min:1|max:255"
        autofocus
      />
      <div class="text-xs text-gray-500 mt-2">
        File will be created in:
        <span class="font-mono text-gray-400">{{ currentPath }}</span>
      </div>
    </Form>

    <template #footer>
      <Button variant="ghost" @click="closeModal">Cancel</Button>
      <Button variant="primary" :loading="loading" @click="handleSubmit">
        Create
      </Button>
    </template>
  </Modal>
</template>

<script setup lang="ts">
import { ref, watch } from "vue";
import { FilePlus } from "lucide-vue-next";
import Modal from "../ui/Modal.vue";
import Form from "../ui/Form.vue";
import Input from "../ui/Input.vue";
import Button from "../ui/Button.vue";
import { useOverlay } from "../../composables/useOverlay";
import { message } from "../../utils/message";

const { closeOverlay, getOverlayProp } = useOverlay();

const createFileForm = ref<InstanceType<typeof Form> | null>(null);
const loading = ref(false);
const fileName = ref("");

const currentPath = getOverlayProp<string>(
  "sftp-create-file-modal",
  "currentPath",
  "",
  "/",
);

const isLocal = getOverlayProp<boolean>(
  "sftp-create-file-modal",
  "isLocal",
  false,
  false,
);

watch(
  () => currentPath.value,
  () => {
    if (currentPath.value) {
      fileName.value = "";
    }
  },
);

async function handleSubmit() {
  const isValid = await createFileForm.value?.validate();
  if (!isValid || !fileName.value) return;

  if (fileName.value.includes("/") || fileName.value.includes("\\")) {
    message.error("File name cannot contain path separators");
    return;
  }

  loading.value = true;

  // Emit event to parent to handle file creation
  const event = new CustomEvent("sftp-create-file", {
    detail: {
      path: currentPath.value,
      name: fileName.value,
      isLocal: isLocal.value,
    },
  });
  window.dispatchEvent(event);
  closeModal();
  loading.value = false;
}

function closeModal() {
  fileName.value = "";
  closeOverlay("sftp-create-file-modal");
}
</script>
