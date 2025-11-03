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
    <Form
      ref="passwordConfirmForm"
      @submit="handleConfirm"
      class="flex flex-col gap-4"
    >
      <p v-if="message" class="text-sm text-gray-300 mb-2">
        {{ message }}
      </p>
      <Input
        id="confirm-password"
        ref="passwordInput"
        v-model="password"
        type="password"
        label="Master Password"
        placeholder="Enter your master password"
        rules="required|password"
        autocomplete="current-password"
      />
    </Form>

    <template #footer>
      <Button variant="ghost" @click="handleCancel"> Cancel </Button>
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
import { message } from "../../utils/message";
import Modal from "../ui/Modal.vue";
import Form from "../ui/Form.vue";
import Input from "../ui/Input.vue";
import Button from "../ui/Button.vue";

const emit = defineEmits<{
  confirm: [password: string];
  cancel: [];
}>();

const passwordConfirmForm = ref<InstanceType<typeof Form>>();
const password = ref("");
const isVerifying = ref(false);
const passwordInput = ref<InstanceType<typeof Input>>();

const { overlayStore, closeOverlay } = useOverlay();
const authStore = useAuthStore();

const handleConfirm = async () => {
  const isValid = await passwordConfirmForm.value?.validate();
  if (!isValid) return;

  isVerifying.value = true;

  const isPasswordValid = await authStore.unlock({ password: password.value });

  if (isPasswordValid) {
    emit("confirm", password.value);
    closeOverlay("password-confirm-modal");
    password.value = "";
  } else {
    message.error("Invalid master password");
  }
  isVerifying.value = false;
};

const handleCancel = () => {
  emit("cancel");
  closeOverlay("password-confirm-modal");
  password.value = "";
};

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
