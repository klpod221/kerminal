<template>
  <Modal
    id="master-password-change"
    parent-id="master-password-settings"
    :show-close-button="true"
    title="Change Master Password"
    size="md"
  >
    <div class="flex flex-col gap-6">
      <Card>
        <div class="flex items-start gap-4">
          <Key :size="80" class="text-blue-400" />
          <div>
            <h3 class="text-lg font-semibold text-gray-100 mb-1">
              Update Security
            </h3>
            <p class="text-sm text-gray-400">
              Change your master password regularly to keep your data secure.
            </p>
          </div>
        </div>
      </Card>

      <Form ref="changeMasterPasswordForm" @submit="handleSubmit">
        <Input
          id="current-password"
          v-model="changeForm.oldPassword"
          label="Current Master Password"
          type="password"
          placeholder="Enter your current password"
          rules="required|password"
          :disabled="isLoading"
        />

        <Input
          id="new-password"
          v-model="changeForm.newPassword"
          label="New Master Password"
          type="password"
          placeholder="Enter a strong new password"
          rules="required|password|different:current-password"
          :disabled="isLoading"
        />

        <Input
          id="confirm-new-password"
          v-model="changeForm.confirmNewPassword"
          label="Confirm New Password"
          type="password"
          placeholder="Confirm your new password"
          rules="required|password|same:new-password"
          :disabled="isLoading"
        />

        <Card>
          <div class="flex items-start gap-3">
            <AlertTriangle
              :size="20"
              class="text-yellow-500 mt-0.5 flex-shrink-0"
            />
            <div>
              <h5 class="text-sm font-semibold text-yellow-400 mb-1">
                Important Security Notice
              </h5>
              <p class="text-sm text-gray-300 mb-2">
                Changing your master password will:
              </p>
              <ul class="text-sm text-gray-400 list-disc pl-4 space-y-1">
                <li>Re-encrypt all your stored credentials</li>
                <li>Invalidate auto-unlock on other devices</li>
                <li>Require you to re-enter the new password on all devices</li>
              </ul>
            </div>
          </div>
        </Card>
      </Form>
    </div>

    <template #footer>
      <Button
        type="button"
        variant="secondary"
        :disabled="isLoading"
        @click="closeOverlay('master-password-change')"
      >
        Cancel
      </Button>
      <Button
        type="submit"
        variant="primary"
        :loading="isLoading"
        :icon="Key"
        @click="handleSubmit"
      >
        Change Password
      </Button>
    </template>
  </Modal>
</template>

<script setup lang="ts">
import { ref } from "vue";
import { Key, AlertTriangle } from "lucide-vue-next";
import { message } from "../../utils/message";
import { getErrorMessage } from "../../utils/helpers";
import { useOverlay } from "../../composables/useOverlay";
import { useAuthStore } from "../../stores/auth";
import Modal from "../ui/Modal.vue";
import Form from "../ui/Form.vue";
import Input from "../ui/Input.vue";
import Button from "../ui/Button.vue";
import Card from "../ui/Card.vue";

// Import stores and composables
const { closeOverlay } = useOverlay();
const { changeMasterPassword } = useAuthStore();

// State
const changeMasterPasswordForm = ref<InstanceType<typeof Form> | null>(null);
const isLoading = ref(false);
const changeForm = ref({
  oldPassword: "",
  newPassword: "",
  confirmNewPassword: "",
});

// Handle form submission
const handleSubmit = async () => {
  const isValid = await changeMasterPasswordForm.value?.validate();
  if (!isValid) return;

  try {
    isLoading.value = true;

    await changeMasterPassword(changeForm.value);

    // Reset form after successful change
    changeForm.value = {
      oldPassword: "",
      newPassword: "",
      confirmNewPassword: "",
    };

    message.success("Master password changed successfully!");
    closeOverlay("master-password-change");
  } catch (error) {
    console.error("Error changing master password:", error);
    message.error(
      getErrorMessage(
        error,
        "Failed to change master password. Please try again.",
      ),
    );
  } finally {
    isLoading.value = false;
  }
};
</script>
