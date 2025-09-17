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
        <Button type="button" variant="danger"> Forgot Password </Button>
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
import Modal from "./ui/Modal.vue";
import Form from "./ui/Form.vue";
import Input from "./ui/Input.vue";
import Button from "./ui/Button.vue";
import { message } from "../utils/message";
import { useOverlay } from "../composables/useOverlay";
import { useAuthStore } from "../stores/auth";

// Stores and composables
const { closeOverlay } = useOverlay();
const { unlock } = useAuthStore();

// Form state
const masterPasswordUnlockForm = ref();
const verificationForm = ref({
  password: "",
});
const isLoading = ref(false);

// Handle form submission
const handleSubmit = async () => {
  const isValid = await masterPasswordUnlockForm.value.validate();
  if (!isValid) return;

  try {
    isLoading.value = true;
    await unlock(verificationForm.value);
    verificationForm.value.password = "";
    message.success("Master password unlocked successfully!");
    closeOverlay("master-password-unlock");
  } catch (error) {
    console.error("Error during master password unlock:", error);
    message.error("Failed to unlock master password. Please try again.");
  } finally {
    isLoading.value = false;
  }
};
</script>
