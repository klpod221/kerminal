<template>
  <Modal
    id="sftp-create-directory-modal"
    title="Create Directory"
    :icon="FolderPlus"
    icon-background="bg-green-500/20"
    icon-color="text-green-400"
    size="md"
  >
    <Form ref="createDirectoryForm" @submit="handleSubmit">
      <Input
        id="directory-name"
        v-model="directoryName"
        label="Directory Name"
        placeholder="Enter directory name"
        rules="required|min:1|max:255"
        autofocus
      />
      <div class="text-xs text-gray-500 mt-2">
        Directory will be created in:
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
import { FolderPlus } from "lucide-vue-next";
import Modal from "../ui/Modal.vue";
import Form from "../ui/Form.vue";
import Input from "../ui/Input.vue";
import Button from "../ui/Button.vue";
import { useOverlay } from "../../composables/useOverlay";
import { message } from "../../utils/message";

const { closeOverlay, getOverlayProp } = useOverlay();

const createDirectoryForm = ref<InstanceType<typeof Form> | null>(null);
const loading = ref(false);
const directoryName = ref("");

const currentPath = getOverlayProp<string>(
  "sftp-create-directory-modal",
  "currentPath",
  "",
  "/",
);

watch(
  () => currentPath.value,
  () => {
    if (currentPath.value) {
      directoryName.value = "";
    }
  },
);

async function handleSubmit() {
  const isValid = await createDirectoryForm.value?.validate();
  if (!isValid || !directoryName.value) return;

  if (directoryName.value.includes("/") || directoryName.value.includes("\\")) {
    message.error("Directory name cannot contain path separators");
    return;
  }

  loading.value = true;
  const isLocal = getOverlayProp<boolean>(
    "sftp-create-directory-modal",
    "isLocal",
    false,
    false,
  );

  // Emit event to parent to handle directory creation
  const event = new CustomEvent("sftp-create-directory", {
    detail: {
      path: currentPath.value,
      name: directoryName.value,
      isLocal: isLocal.value,
    },
  });
  globalThis.dispatchEvent(event);
  closeModal();
  loading.value = false;
}

function closeModal() {
  directoryName.value = "";
  closeOverlay("sftp-create-directory-modal");
}
</script>
