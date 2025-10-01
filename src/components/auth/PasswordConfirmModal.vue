<template>
  <Modal
    id="password-confirm-modal"
    :show-close-button="true"
    title="Confirm Master Password"
    size="sm"
    :icon="Key"
    icon-background="bg-blue-500/20"
    icon-color="text-blue-400"
  >
    <Form @submit.prevent="handleConfirm" class="flex flex-col gap-4">
      <p class="text-sm text-gray-300">
        {{ message || "Please enter your master password to continue:" }}
      </p>

      <Input
        id="confirm-password"
        ref="passwordInput"
        v-model="password"
        type="password"
        label="Master Password"
        placeholder="Enter your master password"
        :error="errorMessage"
        required
        autocomplete="current-password"
        @keyup.enter="handleConfirm"
      />
    </Form>

    <template #footer>
      <Button variant="ghost" @click="handleCancel">
        Cancel
      </Button>
      <Button
        variant="primary"
        :disabled="!password || isVerifying"
        :loading="isVerifying"
        @click="handleConfirm"
      >
        Confirm
      </Button>
    </template>
  </Modal>
</template>

<script setup lang="ts">
import { ref, nextTick, watch } from "vue";
import { Key } from "lucide-vue-next";
import { useOverlay } from "../../composables/useOverlay";
import { useAuthStore } from "../../stores/auth";
import { getErrorMessage } from "../../utils/helpers";
import Modal from "../ui/Modal.vue";
import Form from "../ui/Form.vue";
import Input from "../ui/Input.vue";
import Button from "../ui/Button.vue";

interface PasswordConfirmModalProps {
  message?: string;
}

defineProps<PasswordConfirmModalProps>();

const emit = defineEmits<{
  confirm: [password: string];
  cancel: [];
}>();

// State
const password = ref("");
const errorMessage = ref("");
const isVerifying = ref(false);
const passwordInput = ref<InstanceType<typeof Input>>();

// Stores and composables
const { overlayStore, closeOverlay } = useOverlay();
const authStore = useAuthStore();

// Methods
const handleConfirm = async () => {
  if (!password.value) {
    errorMessage.value = "Password is required";
    return;
  }

  isVerifying.value = true;
  errorMessage.value = "";

  try {
    // Verify password by attempting unlock (this won't change auth state if already unlocked)
    const isValid = await authStore.unlock({ password: password.value });

    if (isValid) {
      emit("confirm", password.value);
      closeOverlay("password-confirm-modal");
      // Reset state
      password.value = "";
      errorMessage.value = "";
    } else {
      errorMessage.value = "Invalid master password";
    }
  } catch (error) {
    errorMessage.value = getErrorMessage(error, "Failed to verify password");
  } finally {
    isVerifying.value = false;
  }
};

const handleCancel = () => {
  emit("cancel");
  closeOverlay("password-confirm-modal");
  // Reset state
  password.value = "";
  errorMessage.value = "";
};

// Focus password input when modal opens
watch(
  () => overlayStore.isVisible("password-confirm-modal"),
  (isVisible) => {
    if (isVisible) {
      nextTick(() => {
        passwordInput.value?.focus();
      });
    }
  },
);
</script>
