<template>
  <Modal
    id="reset-confirm-modal"
    :show-close-button="true"
    title="Reset Master Password"
    size="md"
    :icon="AlertTriangle"
    icon-background="bg-red-500/20"
    icon-color="text-red-400"
  >
    <div class="flex flex-col gap-4">
      <div
        class="flex items-start gap-3 p-4 bg-red-500/10 border border-red-500/20 rounded-lg"
      >
        <AlertTriangle class="w-5 h-5 text-red-400 flex-shrink-0 mt-0.5" />
        <div class="flex-1">
          <h4 class="text-sm font-medium text-red-400 mb-2">
            ⚠️ This action cannot be undone!
          </h4>
          <p class="text-sm text-gray-300">
            Resetting your master password will permanently delete all your data
            including:
          </p>
        </div>
      </div>

      <div class="pl-4 space-y-2">
        <div class="flex items-center gap-2 text-sm text-gray-300">
          <div class="w-1.5 h-1.5 bg-red-400 rounded-full"></div>
          All SSH profiles and connections
        </div>
        <div class="flex items-center gap-2 text-sm text-gray-300">
          <div class="w-1.5 h-1.5 bg-red-400 rounded-full"></div>
          All SSH groups and configurations
        </div>
        <div class="flex items-center gap-2 text-sm text-gray-300">
          <div class="w-1.5 h-1.5 bg-red-400 rounded-full"></div>
          Your master password and encryption keys
        </div>
        <div class="flex items-center gap-2 text-sm text-gray-300">
          <div class="w-1.5 h-1.5 bg-red-400 rounded-full"></div>
          All stored keychain credentials
        </div>
      </div>

      <div class="p-4 bg-gray-800/50 border border-gray-600 rounded-lg">
        <p class="text-sm text-gray-300 mb-3">
          To confirm this action, please type
          <code
            class="px-2 py-1 bg-gray-700 rounded text-red-400 font-mono text-xs"
            >RESET</code
          >
          below:
        </p>

        <Input
          id="reset-confirmation"
          ref="confirmationInput"
          v-model="confirmationText"
          type="text"
          placeholder="Type RESET to confirm"
          :error="errorMessage"
          autocomplete="off"
          @keyup.enter="handleConfirm"
        />
      </div>
    </div>

    <template #footer>
      <Button variant="ghost" @click="handleCancel"> Cancel </Button>
      <Button
        variant="danger"
        :disabled="confirmationText !== 'RESET' || isLoading"
        :loading="isLoading"
        @click="handleConfirm"
      >
        Reset Master Password
      </Button>
    </template>
  </Modal>
</template>

<script setup lang="ts">
import { ref, nextTick, watch } from "vue";
import { AlertTriangle } from "lucide-vue-next";
import { useOverlay } from "../../composables/useOverlay";
import Modal from "../ui/Modal.vue";
import Input from "../ui/Input.vue";
import Button from "../ui/Button.vue";

const emit = defineEmits<{
  confirm: [];
  cancel: [];
}>();

const confirmationText = ref("");
const errorMessage = ref("");
const isLoading = ref(false);
const confirmationInput = ref<InstanceType<typeof Input>>();

const { overlayStore, closeOverlay } = useOverlay();

const handleConfirm = () => {
  if (confirmationText.value !== "RESET") {
    errorMessage.value = "Please type RESET to confirm";
    return;
  }

  emit("confirm");
  closeOverlay("reset-confirm-modal");
  confirmationText.value = "";
  errorMessage.value = "";
};

const handleCancel = () => {
  emit("cancel");
  closeOverlay("reset-confirm-modal");
  confirmationText.value = "";
  errorMessage.value = "";
};

watch(
  () => overlayStore.isVisible("reset-confirm-modal"),
  (isVisible) => {
    if (isVisible) {
      nextTick(() => {
        confirmationInput.value?.focus();
      });
    }
  },
);

watch(confirmationText, () => {
  if (errorMessage.value) {
    errorMessage.value = "";
  }
});
</script>
