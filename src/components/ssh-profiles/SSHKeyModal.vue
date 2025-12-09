<template>
  <Modal id="ssh-key-modal" :title="modalTitle" size="lg">
    <Form ref="keyForm" @submit="handleSubmit">
      <!-- Edit mode info -->
      <div
        v-if="keyId"
        class="mb-4 p-3 bg-blue-900/20 border border-blue-700 rounded-lg"
      >
        <p class="text-sm text-blue-300">
          <strong>Edit Mode:</strong> Key fields are empty for security. Only
          fill them if you want to update the key data.
        </p>
      </div>

      <Input
        id="key-name"
        v-model="formData.name"
        label="Key Name"
        placeholder="My SSH Key"
        rules="required"
      />

      <Textarea
        id="private-key"
        v-model="formData.privateKey"
        label="Private Key"
        placeholder="-----BEGIN OPENSSH PRIVATE KEY-----&#10;...&#10;-----END OPENSSH PRIVATE KEY-----"
        :rows="8"
        :rules="keyId ? '' : 'required'"
      />

      <!-- Import from file button below textarea -->
      <div class="flex gap-2 mb-2">
        <Input
          id="file-path"
          v-model="selectedFileName"
          placeholder="No file selected"
          :disabled="true"
          class="flex-1"
        />
        <Button
          type="button"
          class="h-fit"
          variant="secondary"
          :icon="Upload"
          @click="selectKeyFile"
        >
          Import from File
        </Button>
      </div>
      <input
        ref="fileInput"
        type="file"
        class="hidden"
        @change="handleFileSelect"
      />

      <Textarea
        id="public-key"
        v-model="formData.publicKey"
        label="Public Key (Optional)"
        placeholder="ssh-rsa AAAAB3NzaC1yc2EA..."
        :rows="3"
      />

      <Input
        id="passphrase"
        v-model="formData.passphrase"
        label="Passphrase (Optional)"
        type="password"
        placeholder="Enter passphrase if key is protected"
      />

      <Textarea
        id="description"
        v-model="formData.description"
        label="Description (Optional)"
        placeholder="Additional notes about this key"
        :rows="2"
      />
    </Form>

    <template #footer>
      <div class="flex justify-end gap-2">
        <Button variant="secondary" @click="closeOverlay('ssh-key-modal')"
          >Cancel</Button
        >
        <Button
          variant="primary"
          :loading="sshKeyStore.isLoading"
          :icon="Save"
          @click="handleSubmit"
        >
          {{ submitButtonText }}
        </Button>
      </div>
    </template>
  </Modal>
</template>

<script setup lang="ts">
import { ref, computed, watch } from "vue";
import Modal from "../ui/Modal.vue";
import Form from "../ui/Form.vue";
import Input from "../ui/Input.vue";
import Textarea from "../ui/Textarea.vue";
import Button from "../ui/Button.vue";
import { Save, Upload } from "lucide-vue-next";
import { useSSHKeyStore } from "../../stores/sshKey";
import { useOverlay } from "../../composables/useOverlay";
import { message } from "../../utils/message";
import type { UpdateSSHKeyRequest } from "../../types/ssh";

const props = defineProps<{
  keyId?: string | null;
}>();

const sshKeyStore = useSSHKeyStore();
const { closeOverlay, getOverlayProp } = useOverlay();

const keyId = getOverlayProp("ssh-key-modal", "keyId", props.keyId, null);

const keyForm = ref<InstanceType<typeof Form> | null>(null);
const fileInput = ref<HTMLInputElement | null>(null);
const selectedFileName = ref("");

const formData = ref({
  name: "",
  privateKey: "",
  publicKey: "",
  passphrase: "",
  description: "",
});

const modalTitle = computed(() =>
  keyId.value ? "Edit SSH Key" : "Add SSH Key",
);

const submitButtonText = computed(() => {
  if (keyId.value) return "Update Key";
  return "Add Key";
});

const selectKeyFile = () => {
  fileInput.value?.click();
};

const handleFileSelect = async (event: Event) => {
  const target = event.target as HTMLInputElement;
  const file = target.files?.[0];
  if (!file) return;

  selectedFileName.value = file.name;

  try {
    const content = await file.text();
    formData.value.privateKey = content;
  } catch (error) {
    console.error("Failed to read file:", error);
    message.error("Failed to read file");
    selectedFileName.value = "";
  }
};

const handleSubmit = async () => {
  const isValid = await keyForm.value?.validate();
  if (!isValid) return;

  if (keyId.value) {
    const updateRequest: UpdateSSHKeyRequest = {
      name: formData.value.name,
      description: formData.value.description || null,
    };

    if (formData.value.privateKey.trim()) {
      updateRequest.privateKey = formData.value.privateKey;
    }
    if (formData.value.publicKey.trim()) {
      updateRequest.publicKey = formData.value.publicKey || null;
    }
    if (formData.value.passphrase.trim()) {
      updateRequest.passphrase = formData.value.passphrase || null;
    }

    await sshKeyStore.updateKey(keyId.value, updateRequest);
  } else {
    await sshKeyStore.createKey({
      name: formData.value.name,
      privateKey: formData.value.privateKey,
      publicKey: formData.value.publicKey || undefined,
      passphrase: formData.value.passphrase || undefined,
      description: formData.value.description || undefined,
    });
  }

  closeModal();
};

const loadKey = async () => {
  if (!keyId.value) return;

  await sshKeyStore.loadKeys();
  const key = sshKeyStore.keys.find((k) => k.id === keyId.value);

  if (key) {
    formData.value = {
      name: key.name,
      privateKey: "", // Keep empty to avoid showing encrypted data
      publicKey: "", // Keep empty unless user wants to update
      passphrase: "", // Keep empty unless user wants to update
      description: key.description || "",
    };
  }
};

const closeModal = () => {
  formData.value = {
    name: "",
    privateKey: "",
    publicKey: "",
    passphrase: "",
    description: "",
  };
  closeOverlay("ssh-key-modal");
};

watch(
  keyId,
  (newKeyId) => {
    if (newKeyId) {
      loadKey();
    } else {
      formData.value = {
        name: "",
        privateKey: "",
        publicKey: "",
        passphrase: "",
        description: "",
      };
      selectedFileName.value = "";
    }
  },
  { immediate: true },
);
</script>
