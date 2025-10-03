<template>
  <Modal id="ssh-key-modal" :title="modalTitle" size="lg">
    <Form ref="keyForm">
      <!-- Mode Tabs -->
      <div v-if="!keyId" class="flex gap-2 mb-6 border-b border-gray-700">
        <button
          v-for="tab in modes"
          :key="tab.value"
          type="button"
          class="px-4 py-2 text-sm font-medium transition-colors"
          :class="
            currentMode === tab.value
              ? 'text-blue-400 border-b-2 border-blue-400'
              : 'text-gray-400 hover:text-white'
          "
          @click="currentMode = tab.value as 'manual' | 'import'"
        >
          {{ tab.label }}
        </button>
      </div>

      <!-- Manual/Edit Mode -->
      <div v-if="currentMode === 'manual'" class="space-y-1">
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
          rules="required"
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
      </div>

      <!-- Import Mode -->
      <div v-else-if="currentMode === 'import'" class="space-y-1">
        <Input
          id="import-key-name"
          v-model="formData.name"
          label="Key Name"
          placeholder="My SSH Key"
          rules="required"
        />

        <div class="space-y-2">
          <label class="block text-sm font-medium text-gray-300">
            Private Key File
          </label>
          <div class="flex gap-2">
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
              Browse
            </Button>
          </div>
          <input
            ref="fileInput"
            type="file"
            class="hidden"
            @change="handleFileSelect"
          />
        </div>

        <Input
          id="import-passphrase"
          v-model="formData.passphrase"
          label="Passphrase (Optional)"
          type="password"
          placeholder="Enter passphrase if key is protected"
        />

        <Textarea
          id="import-description"
          v-model="formData.description"
          label="Description (Optional)"
          placeholder="Additional notes about this key"
          :rows="2"
        />
      </div>
    </Form>

    <template #footer>
      <div class="flex justify-end gap-2">
        <Button variant="secondary" @click="closeOverlay('ssh-key-modal')"
          >Cancel</Button
        >
        <Button
          variant="primary"
          :loading="isLoading"
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
import { useSshKeyStore } from "../../stores/sshKey";
import { useOverlay } from "../../composables/useOverlay";
import { message } from "../../utils/message";

// Props
const props = defineProps<{
  keyId?: string | null;
}>();

// Store and composables
const sshKeyStore = useSshKeyStore();
const { closeOverlay, getOverlayProp } = useOverlay();

// Use overlay prop with fallback to direct prop
const keyId = getOverlayProp("ssh-key-modal", "keyId", props.keyId, null);

// State
const keyForm = ref<InstanceType<typeof Form> | null>(null);
const isLoading = ref(false);
const currentMode = ref<"manual" | "import">("manual");
const fileInput = ref<HTMLInputElement | null>(null);
const selectedFileName = ref("");

const formData = ref({
  name: "",
  privateKey: "",
  publicKey: "",
  passphrase: "",
  description: "",
});

// Computed
const modalTitle = computed(() =>
  keyId.value ? "Edit SSH Key" : "Add SSH Key",
);

const submitButtonText = computed(() => {
  if (keyId.value) return "Update Key";
  return currentMode.value === "import" ? "Import Key" : "Add Key";
});

const modes = [
  { value: "manual", label: "Manual Entry" },
  { value: "import", label: "Import from File" },
];

// Functions

const selectKeyFile = () => {
  fileInput.value?.click();
};

const handleFileSelect = (event: Event) => {
  const target = event.target as HTMLInputElement;
  const file = target.files?.[0];
  if (!file) return;

  selectedFileName.value = file.name;

  const reader = new FileReader();
  reader.onload = (e) => {
    const content = e.target?.result as string;
    formData.value.privateKey = content;
  };
  reader.onerror = () => {
    message.error("Failed to read file");
    selectedFileName.value = "";
  };
  reader.readAsText(file);
};

const handleSubmit = async () => {
  const isValid = await keyForm.value?.validate();
  if (!isValid) return;

  isLoading.value = true;

  try {
    if (keyId.value) {
      // Update existing key
      await sshKeyStore.updateKey(keyId.value, {
        name: formData.value.name,
        description: formData.value.description || undefined,
      });
      message.success("SSH key updated successfully");
      closeOverlay("ssh-key-modal");
    } else if (currentMode.value === "import") {
      // Import key from file (keyType auto-detected)
      await sshKeyStore.importKeyFromFile(
        formData.value.name,
        formData.value.privateKey,
        formData.value.passphrase || undefined,
      );
      message.success("SSH key imported successfully");
      closeOverlay("ssh-key-modal");
    } else {
      // Create new key (keyType auto-detected)
      await sshKeyStore.createKey({
        name: formData.value.name,
        privateKey: formData.value.privateKey,
        publicKey: formData.value.publicKey || undefined,
        passphrase: formData.value.passphrase || undefined,
        description: formData.value.description || undefined,
      });
      message.success("SSH key created successfully");
      closeOverlay("ssh-key-modal");
    }
  } catch (error) {
    console.error("Error saving SSH key:", error);
  } finally {
    isLoading.value = false;
  }
};

const loadKey = async () => {
  if (!keyId.value) return;

  isLoading.value = true;
  try {
    await sshKeyStore.loadKeys();
    const key = sshKeyStore.keys.find((k) => k.id === keyId.value);

    if (key) {
      formData.value = {
        name: key.name,
        privateKey: key.privateKey,
        publicKey: key.publicKey || "",
        passphrase: key.passphrase || "",
        description: key.description || "",
      };
    }
  } catch (error) {
    console.error("Error loading SSH key:", error);
    message.error("Failed to load SSH key");
  } finally {
    isLoading.value = false;
  }
};

// Watch for keyId changes
watch(
  keyId,
  (newKeyId) => {
    if (newKeyId) {
      loadKey();
    } else {
      // Reset form for new key
      formData.value = {
        name: "",
        privateKey: "",
        publicKey: "",
        passphrase: "",
        description: "",
      };
      selectedFileName.value = "";
      currentMode.value = "manual";
    }
  },
  { immediate: true },
);
</script>
