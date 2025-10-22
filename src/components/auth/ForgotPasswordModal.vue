<template>
  <Modal
    id="forgot-password-modal"
    :show-close-button="true"
    title="Forgot Master Password"
    size="md"
    :icon="HelpCircle"
    icon-background="bg-yellow-500/20"
    icon-color="text-yellow-400"
  >
    <div class="flex flex-col gap-4">
      <!-- Warning Notice -->
      <div
        class="flex items-start gap-3 p-4 bg-yellow-500/10 border border-yellow-500/20 rounded-lg"
      >
        <AlertTriangle class="w-5 h-5 text-yellow-400 flex-shrink-0 mt-0.5" />
        <div class="flex-1">
          <h4 class="text-sm font-medium text-yellow-400 mb-2">
            Master Password Cannot Be Recovered
          </h4>
          <p class="text-sm text-gray-300">
            Due to the encryption design, there is no way to recover your master
            password. The only option is to reset it, which will delete all your
            encrypted data.
          </p>
        </div>
      </div>

      <!-- Information Section -->
      <div class="space-y-3">
        <h4 class="text-sm font-medium text-gray-200">
          What happens when you reset?
        </h4>

        <div class="pl-4 space-y-2">
          <div class="flex items-center gap-2 text-sm text-gray-300">
            <div class="w-1.5 h-1.5 bg-red-400 rounded-full"></div>
            All SSH profiles will be permanently deleted
          </div>
          <div class="flex items-center gap-2 text-sm text-gray-300">
            <div class="w-1.5 h-1.5 bg-red-400 rounded-full"></div>
            All SSH groups and configurations will be removed
          </div>
          <div class="flex items-center gap-2 text-sm text-gray-300">
            <div class="w-1.5 h-1.5 bg-red-400 rounded-full"></div>
            Your master password and encryption keys will be deleted
          </div>
          <div class="flex items-center gap-2 text-sm text-gray-300">
            <div class="w-1.5 h-1.5 bg-red-400 rounded-full"></div>
            All stored keychain credentials will be removed
          </div>
        </div>
      </div>

      <!-- Tips Section -->
      <div class="p-4 bg-blue-500/10 border border-blue-500/20 rounded-lg">
        <div class="flex items-start gap-3">
          <Info class="w-5 h-5 text-blue-400 flex-shrink-0 mt-0.5" />
          <div class="flex-1">
            <h4 class="text-sm font-medium text-blue-400 mb-2">
              Tips to remember:
            </h4>
            <ul class="text-sm text-gray-300 space-y-1 list-disc list-inside">
              <li>Try common passwords you might have used</li>
              <li>Check if you have written it down somewhere secure</li>
              <li>Try variations with uppercase/lowercase</li>
              <li>Consider using a password manager in the future</li>
            </ul>
          </div>
        </div>
      </div>

      <!-- Confirmation Section -->
      <div class="p-4 bg-gray-800/50 border border-gray-600 rounded-lg">
        <p class="text-sm text-gray-300 mb-3">
          If you're sure you want to reset, type
          <code
            class="px-2 py-1 bg-gray-700 rounded text-red-400 font-mono text-xs"
            >RESET</code
          >
          below:
        </p>

        <Input
          id="forgot-password-confirmation"
          ref="confirmationInput"
          v-model="confirmationText"
          type="text"
          placeholder="Type RESET to confirm"
          :error="errorMessage"
          autocomplete="off"
          @keyup.enter="handleReset"
        />
      </div>
    </div>

    <template #footer>
      <Button variant="ghost" @click="handleCancel"> Cancel </Button>
      <Button
        variant="danger"
        :disabled="confirmationText !== 'RESET' || isLoading"
        :loading="isLoading"
        @click="handleReset"
      >
        Reset Master Password
      </Button>
    </template>
  </Modal>
</template>

<script setup lang="ts">
import { ref, nextTick, watch } from "vue";
import { HelpCircle, AlertTriangle, Info } from "lucide-vue-next";
import { useOverlay } from "../../composables/useOverlay";
import { useAuthStore } from "../../stores/auth";
import { message } from "../../utils/message";
import { getErrorMessage } from "../../utils/helpers";
import Modal from "../ui/Modal.vue";
import Input from "../ui/Input.vue";
import Button from "../ui/Button.vue";

const confirmationText = ref("");
const errorMessage = ref("");
const isLoading = ref(false);
const confirmationInput = ref<InstanceType<typeof Input>>();

const { overlayStore, closeOverlay } = useOverlay();
const { resetMasterPassword } = useAuthStore();

/**
 * Handle reset confirmation
 */
const handleReset = async () => {
  if (confirmationText.value !== "RESET") {
    errorMessage.value = "Please type RESET to confirm";
    return;
  }

  try {
    isLoading.value = true;
    await resetMasterPassword();

    message.success("Master password has been reset successfully!");
    closeOverlay("forgot-password-modal");
    closeOverlay("master-password-unlock");

    confirmationText.value = "";
    errorMessage.value = "";
  } catch (error) {
    console.error("Error resetting master password:", error);
    message.error(
      getErrorMessage(
        error,
        "Failed to reset master password. Please try again.",
      ),
    );
  } finally {
    isLoading.value = false;
  }
};

/**
 * Handle cancel action
 */
const handleCancel = () => {
  closeOverlay("forgot-password-modal");
  confirmationText.value = "";
  errorMessage.value = "";
};

/**
 * Focus input when modal opens
 */
watch(
  () => overlayStore.isVisible("forgot-password-modal"),
  (isVisible) => {
    if (isVisible) {
      nextTick(() => {
        confirmationInput.value?.focus();
      });
    }
  },
);

/**
 * Clear error when user types
 */
watch(confirmationText, () => {
  if (errorMessage.value) {
    errorMessage.value = "";
  }
});
</script>
