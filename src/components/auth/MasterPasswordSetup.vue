<template>
  <Modal
    id="master-password-setup"
    :show-close-button="false"
    :close-on-backdrop="false"
    :close-on-esc="false"
    title="Setup Master Password"
    size="md"
  >
    <div class="flex flex-col gap-4">
      <div class="bg-yellow-900/20 border border-yellow-700/50 rounded-lg p-4">
        <div class="flex items-start gap-2">
          <AlertCircle :size="20" class="text-yellow-500 mt-0.5 shrink-0" />
          <div class="flex-1">
            <h4 class="text-sm font-medium text-yellow-200 mb-1">
              Important for Multi-Device Sync
            </h4>
            <p class="text-xs text-yellow-100/80">
              If you plan to sync data across multiple devices, you MUST use the
              same master password on all devices. Different passwords will
              prevent data decryption and sync functionality.
            </p>
          </div>
        </div>
      </div>

      <Form ref="masterPasswordSetupForm" @submit="handleSubmit">
        <h4
          class="text-sm font-medium text-gray-100 border-b border-gray-700 pb-2 mb-2"
        >
          Device Information
        </h4>
        <Input
          id="device-name"
          v-model="setupForm.deviceName"
          label="Device Name"
          placeholder="My Arch Linux"
          rules="required"
          :disabled="authStore.isLoading"
          :autofocus="true"
        />

        <h4
          class="text-sm font-medium text-gray-100 border-b border-gray-700 pb-2 mb-2"
        >
          Master Password
        </h4>
        <Input
          id="master-password"
          v-model="setupForm.password"
          label="Password"
          type="password"
          placeholder="Enter a strong password"
          rules="required|password"
          :disabled="authStore.isLoading"
        />

        <Input
          id="confirm-password"
          v-model="setupForm.confirmPassword"
          label="Confirm Password"
          type="password"
          placeholder="Confirm your password"
          rules="required|password|same:master-password"
          :disabled="authStore.isLoading"
        />

        <h4
          class="text-sm font-medium text-gray-100 border-b border-gray-700 pb-2 mb-2"
        >
          Security Options
        </h4>

        <div class="flex flex-col gap-2">
          <Checkbox
            id="use-keychain"
            v-model="setupForm.useKeychain"
            label="Use system keychain"
            helper-text="Store encrypted password in system keychain for convenience"
          />

          <Checkbox
            id="auto-unlock"
            v-model="setupForm.autoUnlock"
            :disabled="!setupForm.useKeychain"
            label="Auto-unlock on startup"
            helper-text="Automatically unlock when application starts (requires keychain)"
          />
        </div>
      </Form>
    </div>

    <template #footer>
      <Button
        type="submit"
        variant="primary"
        :loading="authStore.isLoading"
        :icon="Save"
        @click="handleSubmit"
      >
        Setup Master Password
      </Button>
    </template>
  </Modal>
</template>

<script setup lang="ts">
import { ref } from "vue";
import { Save, AlertCircle } from "lucide-vue-next";
import { message } from "../../utils/message";
import { useOverlay } from "../../composables/useOverlay";
import { useAuthStore } from "../../stores/auth";
import Modal from "../ui/Modal.vue";
import Form from "../ui/Form.vue";
import Input from "../ui/Input.vue";
import Button from "../ui/Button.vue";
import Checkbox from "../ui/Checkbox.vue";

const { closeOverlay } = useOverlay();
const authStore = useAuthStore();
const { setupMasterPassword } = authStore;

const masterPasswordSetupForm = ref<InstanceType<typeof Form> | null>(null);
const setupForm = ref({
  deviceName: "",
  password: "",
  confirmPassword: "",
  useKeychain: true,
  autoUnlock: false,
  autoLockTimeout: 0,
});

const handleSubmit = async () => {
  const isValid = await masterPasswordSetupForm.value?.validate();
  if (!isValid) return;

  const success = await setupMasterPassword(setupForm.value);

  if (success) {
    const wasAutoUnlock =
      setupForm.value.autoUnlock && setupForm.value.useKeychain;

    setupForm.value = {
      deviceName: "",
      password: "",
      confirmPassword: "",
      useKeychain: true,
      autoUnlock: false,
      autoLockTimeout: 0,
    };

    message.success("Master password setup successfully!");

    closeOverlay("master-password-setup");

    if (!wasAutoUnlock) {
      setTimeout(() => {}, 100);
    }
  }
};
</script>
