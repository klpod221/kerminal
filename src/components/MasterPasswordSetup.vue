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
      <Card>
        <div class="flex items-start gap-4">
          <Lock :size="80" class="text-amber-400" />
          <div>
            <h3 class="text-lg font-semibold text-gray-100 mb-1">
              Secure Your Data
            </h3>
            <p class="text-red-400">This password cannot be recovered.</p>
          </div>
        </div>
      </Card>

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
          :disabled="isLoading"
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
          :disabled="isLoading"
        />

        <Input
          id="confirm-password"
          v-model="setupForm.confirmPassword"
          label="Confirm Password"
          type="password"
          placeholder="Confirm your password"
          rules="required|password|same:master-password"
          :disabled="isLoading"
        />

        <h4
          class="text-sm font-medium text-gray-100 border-b border-gray-700 pb-2 mb-2"
        >
          Security Options
        </h4>
        <Checkbox
          id="use-keychain"
          v-model="setupForm.useKeychain"
          class="mb-2"
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
      </Form>
    </div>

    <template #footer>
      <Button
        type="submit"
        variant="primary"
        :loading="isLoading"
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
import { Lock, Save } from "lucide-vue-next";
import { message } from "../utils/message";
import { useOverlay } from "../composables/useOverlay";
import { useAuthStore } from "../stores/auth";
import Modal from "./ui/Modal.vue";
import Form from "./ui/Form.vue";
import Input from "./ui/Input.vue";
import Button from "./ui/Button.vue";
import Checkbox from "./ui/Checkbox.vue";
import Card from "./ui/Card.vue";

// Import stores and composables
const { closeOverlay } = useOverlay();
const { setupMasterPassword } = useAuthStore();

// Form state
const masterPasswordSetupForm = ref<InstanceType<typeof Form> | null>(null);
const setupForm = ref({
  deviceName: "",
  password: "",
  confirmPassword: "",
  useKeychain: true,
  autoUnlock: false,
});
const isLoading = ref(false);

// Handle form submission
const handleSubmit = async () => {
  const isValid = await masterPasswordSetupForm.value?.validate();
  if (!isValid) return;

  try {
    isLoading.value = true;

    await setupMasterPassword(setupForm.value);

    setupForm.value = {
      deviceName: "",
      password: "",
      confirmPassword: "",
      useKeychain: true,
      autoUnlock: false,
    };

    message.success("Master password setup successfully!");
    closeOverlay("master-password-setup");
  } catch (error) {
    console.error("Error during master password setup:", error);
    message.error("Failed to set up master password. Please try again.");
  } finally {
    isLoading.value = false;
  }
};
</script>
