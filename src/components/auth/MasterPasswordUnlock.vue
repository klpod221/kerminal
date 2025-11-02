<template>
  <Modal
    id="master-password-unlock"
    :show-close-button="false"
    :close-on-backdrop="false"
    :close-on-esc="false"
    title="Unlock Master Password"
    size="sm"
  >
    <Form ref="masterPasswordUnlockForm" @submit="handleSubmit">
      <Input
        id="unlock-password"
        v-model="verificationForm.password"
        label="Master Password"
        type="password"
        placeholder="Enter your master password"
        rules="required|password"
        :autofocus="true"
      />
    </Form>

    <template #footer>
      <div class="flex justify-end gap-2">
        <Button type="button" variant="danger" @click="handleForgotPassword">
          Forgot Password
        </Button>
        <Button
          type="submit"
          variant="primary"
          :icon="Unlock"
          :loading="isLoading"
          @click="handleSubmit"
        >
          Unlock
        </Button>
      </div>
    </template>
  </Modal>
</template>

<script setup lang="ts">
import { ref } from "vue";
import { Unlock } from "lucide-vue-next";
import Modal from "../ui/Modal.vue";
import Form from "../ui/Form.vue";
import Input from "../ui/Input.vue";
import Button from "../ui/Button.vue";
import { message } from "../../utils/message";
import { useOverlay } from "../../composables/useOverlay";
import { useAuthStore } from "../../stores/auth";

const { closeOverlay, openOverlay } = useOverlay();
const { unlock } = useAuthStore();

const masterPasswordUnlockForm = ref<InstanceType<typeof Form> | null>(null);
const verificationForm = ref({
  password: "",
});
const isLoading = ref(false);

/**
 * Handle form submission to unlock master password
 */
const handleSubmit = async () => {
  const isValid = await masterPasswordUnlockForm.value?.validate();
  if (!isValid) return;

  isLoading.value = true;
  const success = await unlock(verificationForm.value);

  if (success) {
    verificationForm.value.password = "";
    message.success("Master password unlocked successfully!");
    closeOverlay("master-password-unlock");
  } else {
    message.error("Invalid master password. Please try again.");
  }
  isLoading.value = false;
};

/**
 * Handle forgot password button click
 */
const handleForgotPassword = () => {
  openOverlay("forgot-password-modal");
};
</script>
