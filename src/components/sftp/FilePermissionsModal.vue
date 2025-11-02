<template>
  <Modal
    id="sftp-file-permissions-modal"
    title="File Permissions"
    :icon="Shield"
    icon-background="bg-green-500/20"
    icon-color="text-green-400"
    size="md"
  >
    <Form ref="permissionsForm" @submit="handleSubmit">
      <div class="space-y-4">
        <div>
          <label class="block text-sm font-medium text-gray-300 mb-2">
            File: <span class="font-medium text-white">{{ file?.name }}</span>
          </label>
        </div>

        <!-- Octal Permission Input -->
        <Input
          id="permissions-octal"
          v-model="octalPermissions"
          label="Permissions (Octal)"
          placeholder="0644"
          rules="required|min:3|max:4"
          helper-text="Enter permissions in octal format (e.g., 0644, 0755)"
        />

        <!-- Permission Breakdown -->
        <div class="border border-gray-700 rounded-lg p-4 bg-gray-900/50">
          <div class="text-sm font-medium text-gray-300 mb-3">
            Permission Breakdown
          </div>
          <div class="space-y-3">
            <!-- Owner -->
            <div>
              <div class="text-xs text-gray-400 mb-2">Owner</div>
              <div class="flex gap-4">
                <Checkbox
                  id="owner-read"
                  v-model="permissions.owner.read"
                  label="Read"
                  @update:model-value="updateOctal"
                />
                <Checkbox
                  id="owner-write"
                  v-model="permissions.owner.write"
                  label="Write"
                  @update:model-value="updateOctal"
                />
                <Checkbox
                  id="owner-execute"
                  v-model="permissions.owner.execute"
                  label="Execute"
                  @update:model-value="updateOctal"
                />
              </div>
            </div>

            <!-- Group -->
            <div>
              <div class="text-xs text-gray-400 mb-2">Group</div>
              <div class="flex gap-4">
                <Checkbox
                  id="group-read"
                  v-model="permissions.group.read"
                  label="Read"
                  @update:model-value="updateOctal"
                />
                <Checkbox
                  id="group-write"
                  v-model="permissions.group.write"
                  label="Write"
                  @update:model-value="updateOctal"
                />
                <Checkbox
                  id="group-execute"
                  v-model="permissions.group.execute"
                  label="Execute"
                  @update:model-value="updateOctal"
                />
              </div>
            </div>

            <!-- Others -->
            <div>
              <div class="text-xs text-gray-400 mb-2">Others</div>
              <div class="flex gap-4">
                <Checkbox
                  id="others-read"
                  v-model="permissions.others.read"
                  label="Read"
                  @update:model-value="updateOctal"
                />
                <Checkbox
                  id="others-write"
                  v-model="permissions.others.write"
                  label="Write"
                  @update:model-value="updateOctal"
                />
                <Checkbox
                  id="others-execute"
                  v-model="permissions.others.execute"
                  label="Execute"
                  @update:model-value="updateOctal"
                />
              </div>
            </div>
          </div>
        </div>

        <!-- Quick Presets -->
        <div>
          <div class="text-sm font-medium text-gray-300 mb-2">
            Quick Presets
          </div>
          <div class="flex gap-2 flex-wrap">
            <Button variant="outline" size="sm" @click="setPreset(0o644)">
              0644 (rw-r--r--)
            </Button>
            <Button variant="outline" size="sm" @click="setPreset(0o755)">
              0755 (rwxr-xr-x)
            </Button>
            <Button variant="outline" size="sm" @click="setPreset(0o600)">
              0600 (rw-------)
            </Button>
            <Button variant="outline" size="sm" @click="setPreset(0o777)">
              0777 (rwxrwxrwx)
            </Button>
          </div>
        </div>
      </div>
    </Form>

    <template #footer>
      <Button variant="ghost" @click="closeModal">Cancel</Button>
      <Button variant="primary" :loading="loading" @click="handleSubmit">
        Apply Permissions
      </Button>
    </template>
  </Modal>
</template>

<script setup lang="ts">
import { ref, watch } from "vue";
import { Shield } from "lucide-vue-next";
import Modal from "../ui/Modal.vue";
import Form from "../ui/Form.vue";
import Input from "../ui/Input.vue";
import Checkbox from "../ui/Checkbox.vue";
import Button from "../ui/Button.vue";
import { useOverlay } from "../../composables/useOverlay";
import { message } from "../../utils/message";
import type { FileEntry } from "../../types/sftp";

const { closeOverlay, getOverlayProp } = useOverlay();

const permissionsForm = ref<InstanceType<typeof Form> | null>(null);
const loading = ref(false);
const octalPermissions = ref("0644");

const file = getOverlayProp<FileEntry | null>(
  "sftp-file-permissions-modal",
  "file",
  null,
  null,
);

interface PermissionFlags {
  read: boolean;
  write: boolean;
  execute: boolean;
}

const permissions = ref<{
  owner: PermissionFlags;
  group: PermissionFlags;
  others: PermissionFlags;
}>({
  owner: { read: false, write: false, execute: false },
  group: { read: false, write: false, execute: false },
  others: { read: false, write: false, execute: false },
});

// Initialize permissions from file
watch(
  () => file.value,
  (fileValue) => {
    if (fileValue) {
      const mode = fileValue.permissions;
      octalPermissions.value = `0${mode.toString(8)}`;
      updatePermissionsFromOctal(mode);
    }
  },
  { immediate: true },
);

watch(octalPermissions, (octal) => {
  const match = octal.match(/^0?([0-7]{3,4})$/);
  if (match) {
    const octalValue = parseInt(match[1], 8);
    updatePermissionsFromOctal(octalValue);
  }
});

function updatePermissionsFromOctal(mode: number) {
  permissions.value = {
    owner: {
      read: !!(mode & 0o400),
      write: !!(mode & 0o200),
      execute: !!(mode & 0o100),
    },
    group: {
      read: !!(mode & 0o040),
      write: !!(mode & 0o020),
      execute: !!(mode & 0o010),
    },
    others: {
      read: !!(mode & 0o004),
      write: !!(mode & 0o002),
      execute: !!(mode & 0o001),
    },
  };
}

function updateOctal() {
  let mode = 0;
  if (permissions.value.owner.read) mode |= 0o400;
  if (permissions.value.owner.write) mode |= 0o200;
  if (permissions.value.owner.execute) mode |= 0o100;
  if (permissions.value.group.read) mode |= 0o040;
  if (permissions.value.group.write) mode |= 0o020;
  if (permissions.value.group.execute) mode |= 0o010;
  if (permissions.value.others.read) mode |= 0o004;
  if (permissions.value.others.write) mode |= 0o002;
  if (permissions.value.others.execute) mode |= 0o001;

  octalPermissions.value = `0${mode.toString(8)}`;
}

function setPreset(mode: number) {
  updatePermissionsFromOctal(mode);
  octalPermissions.value = `0${mode.toString(8)}`;
}

async function handleSubmit() {
  if (!file.value || loading.value) return;

  const match = octalPermissions.value.match(/^0?([0-7]{3,4})$/);
  if (!match) {
    message.error("Invalid permissions format. Use octal format (e.g., 0644)");
    return;
  }

  const mode = parseInt(match[1], 8);

  loading.value = true;

  // Emit event to parent to handle permissions change
  const event = new CustomEvent("sftp-permissions", {
    detail: { path: file.value.path, mode },
  });
  window.dispatchEvent(event);
  closeModal();
  loading.value = false;
}

function closeModal() {
  octalPermissions.value = "0644";
  permissions.value = {
    owner: { read: false, write: false, execute: false },
    group: { read: false, write: false, execute: false },
    others: { read: false, write: false, execute: false },
  };
  closeOverlay("sftp-file-permissions-modal");
}
</script>
