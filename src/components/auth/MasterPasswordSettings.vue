<template>
  <Modal
    id="master-password-settings"
    :show-close-button="true"
    title="Master Password Settings"
    size="md"
  >
    <div class="flex flex-col gap-6">
      <Form class="flex flex-col gap-6">
        <div class="flex flex-col gap-2">
          <h4
            class="text-sm font-medium text-gray-100 border-b border-gray-700 pb-2"
          >
            Security Status
          </h4>
          <div class="grid grid-cols-1 sm:grid-cols-2 gap-4">
            <Card>
              <div class="flex flex-col gap-1">
                <div
                  class="text-xs font-medium text-gray-400 uppercase tracking-wide"
                >
                  Status
                </div>
                <div
                  class="flex items-center gap-2 text-sm font-medium text-gray-100"
                >
                  <span
                    class="w-2 h-2 rounded-full"
                    :class="
                      authStore.isAuthenticated ? 'bg-green-500' : 'bg-red-500'
                    "
                  ></span>
                  {{ authStore.isAuthenticated ? "Unlocked" : "Locked" }}
                </div>
              </div>
            </Card>

            <Card>
              <div class="flex flex-col gap-1">
                <div
                  class="text-xs font-medium text-gray-400 uppercase tracking-wide"
                >
                  Auto-unlock
                </div>
                <div class="text-sm font-medium text-gray-100">
                  {{
                    authStore.status.autoUnlockEnabled ? "Enabled" : "Disabled"
                  }}
                </div>
              </div>
            </Card>
          </div>
        </div>

        <div class="flex flex-col gap-2">
          <h4
            class="text-sm font-medium text-gray-100 border-b border-gray-700 pb-2"
          >
            Security Configuration
          </h4>

          <Checkbox
            id="auto-unlock-enabled"
            v-model="autoUnlockEnabled"
            @change="handleAutoUnlockToggle"
            :disabled="isLoading"
            label="Auto-unlock on startup"
            helper-text="Automatically unlock master password when the application starts using system keychain"
          />

          <Select
            id="auto-lock-timeout"
            v-model="autoLockTimeout"
            label="Auto-lock Timeout"
            :options="timeoutOptions"
            @change="handleTimeoutChange"
            :disabled="isLoading"
            helper-text="Automatically lock the session after period of inactivity"
          />

          <Button
            variant="danger"
            :disabled="isLoading"
            :icon="Trash2"
            @click="handleResetPassword"
          >
            Reset Master Password
          </Button>

          <Button
            variant="primary"
            :disabled="isLoading"
            :icon="Key"
            @click="openChangePasswordModal"
          >
            Change Master Password
          </Button>

          <Button
            variant="secondary"
            @click="handleLock"
            :disabled="isLoading"
            :icon="Lock"
          >
            Lock Session
          </Button>
        </div>

        <div class="flex flex-col gap-2" v-if="authStore.currentDevice">
          <h4
            class="text-sm font-medium text-gray-100 border-b border-gray-700 pb-2"
          >
            Current Device Information
          </h4>
          <div class="flex flex-col gap-4">
            <Card>
              <div class="flex justify-between items-center">
                <div class="text-sm font-medium text-gray-400">Device Name</div>
                <div class="text-sm text-gray-100">
                  {{ authStore.currentDevice.deviceName }}
                </div>
              </div>
            </Card>

            <Card>
              <div class="flex justify-between items-center">
                <div class="text-sm font-medium text-gray-400">Device Type</div>
                <div class="text-sm text-gray-100">
                  {{ authStore.currentDevice.deviceType }}
                </div>
              </div>
            </Card>

            <Card>
              <div class="flex justify-between items-center">
                <div class="text-sm font-medium text-gray-400">
                  Operating System
                </div>
                <div class="text-sm text-gray-100">
                  {{ authStore.currentDevice.osName }}
                </div>
              </div>
            </Card>

            <Card>
              <div class="flex justify-between items-center">
                <div class="text-sm font-medium text-gray-400">
                  Operating System Version
                </div>
                <div class="text-sm text-gray-100">
                  {{ authStore.currentDevice.osVersion }}
                </div>
              </div>
            </Card>

            <Card>
              <div class="flex justify-between items-center">
                <div class="text-sm font-medium text-gray-400">Created At</div>
                <div class="text-sm text-gray-100">
                  {{
                    formatRelativeTime(
                      new Date(authStore.currentDevice.createdAt),
                    )
                  }}
                </div>
              </div>
            </Card>
          </div>

          <Button variant="secondary" :icon="Book" class="mt-2">
            Device List
          </Button>
        </div>
      </Form>
    </div>
  </Modal>
</template>

<script setup lang="ts">
import { ref, watch } from "vue";
import { useAuthStore } from "../../stores/auth";
import { Key, Lock, Trash2, Book } from "lucide-vue-next";
import { message } from "../../utils/message";
import { getErrorMessage } from "../../utils/helpers";
import { formatRelativeTime } from "../../utils/formatter";
import { useOverlay } from "../../composables/useOverlay";
import Modal from "../ui/Modal.vue";
import Form from "../ui/Form.vue";
import Button from "../ui/Button.vue";
import Checkbox from "../ui/Checkbox.vue";
import Select from "../ui/Select.vue";
import Card from "../ui/Card.vue";

// State
const isLoading = ref(false);
const autoUnlockEnabled = ref(false);
const autoLockTimeout = ref(0);

// Timeout options
const timeoutOptions = [
  { value: 0, label: "Never" },
  { value: 5, label: "5 minutes" },
  { value: 15, label: "15 minutes" },
  { value: 30, label: "30 minutes" },
  { value: 60, label: "1 hour" },
  { value: 120, label: "2 hours" },
  { value: 240, label: "4 hours" },
];

// Stores and composables
const authStore = useAuthStore();
const { openOverlay, closeOverlay } = useOverlay();

// Event handlers
const handleAutoUnlockToggle = async () => {
  let password = null;

  // If enabling auto-unlock, we need to confirm with password first
  if (autoUnlockEnabled.value) {
    // Show confirmation modal with password input
    const result = await showPasswordConfirmationModal();
    if (!result.confirmed) {
      // Revert the toggle if user cancels
      autoUnlockEnabled.value = false;
      return;
    }
    password = result.password;
  }

  isLoading.value = true;
  try {
    const config = {
      autoUnlock: autoUnlockEnabled.value,
      autoLockTimeout: autoLockTimeout.value,
      ...(password && { password }),
    };

    await authStore.updateMasterPasswordConfig(config);
    message.success(
      `Auto-unlock has been ${
        autoUnlockEnabled.value ? "enabled" : "disabled"
      }.`,
    );
  } catch (error) {
    message.error(
      getErrorMessage(error, "Failed to update auto-unlock setting."),
    );
    // Revert the toggle state on error
    autoUnlockEnabled.value = !autoUnlockEnabled.value;
  } finally {
    isLoading.value = false;
  }
};

// Show password confirmation modal for enabling auto-unlock
const showPasswordConfirmationModal = (): Promise<{ confirmed: boolean; password?: string }> => {
  return new Promise((resolve) => {
    const password = prompt(
      "To enable auto-unlock, please confirm your master password:"
    );

    if (!password) {
      resolve({ confirmed: false });
      return;
    }

    // Verify password by attempting unlock (this won't change auth state if already unlocked)
    authStore.unlock({ password })
      .then((isValid: boolean) => {
        if (isValid) {
          resolve({ confirmed: true, password });
        } else {
          message.error("Invalid master password. Auto-unlock not enabled.");
          resolve({ confirmed: false });
        }
      })
      .catch((error: any) => {
        message.error(getErrorMessage(error, "Failed to verify password."));
        resolve({ confirmed: false });
      });
  });
};

const handleLock = async () => {
  isLoading.value = true;
  try {
    await authStore.lock();
    closeOverlay("master-password-settings");
    message.success("Session locked.");
  } catch (error) {
    message.error(getErrorMessage(error, "Failed to lock session."));
  } finally {
    isLoading.value = false;
  }
};

const openChangePasswordModal = () => {
  openOverlay("master-password-change");
};

const handleResetPassword = async () => {
  // Show confirmation dialog
  const confirmed = confirm(
    "Are you sure you want to reset your master password?\n\n" +
      "This will permanently delete:\n" +
      "• All SSH profiles and connections\n" +
      "• All SSH groups and configurations\n" +
      "• Your master password and encryption keys\n\n" +
      "This action cannot be undone!",
  );

  if (!confirmed) return;

  isLoading.value = true;
  try {
    await authStore.resetMasterPassword();
    closeOverlay("master-password-settings");
    message.success(
      "Master password has been reset successfully. You can now set up a new master password.",
    );
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

const handleTimeoutChange = async () => {
  isLoading.value = true;
  try {
    // Update master password config with both settings
    await authStore.updateMasterPasswordConfig({
      autoUnlock: autoUnlockEnabled.value,
      autoLockTimeout: autoLockTimeout.value,
    });

    // Update local security settings with converted number
    authStore.securitySettings.autoLockTimeout = Number(autoLockTimeout.value);

    // Reset auto-lock timer with new timeout
    authStore.resetAutoLockTimer();

    message.success(
      `Auto-lock timeout updated to ${
        Number(autoLockTimeout.value) === 0
          ? "never"
          : `${autoLockTimeout.value} minutes`
      }.`,
    );
  } catch (error) {
    message.error(
      getErrorMessage(error, "Failed to update auto-lock timeout."),
    );
  } finally {
    isLoading.value = false;
  }
};

// Watchers
watch(
  () => authStore.status.autoUnlockEnabled,
  (newVal) => {
    autoUnlockEnabled.value = newVal;
  },
  { immediate: true },
);

watch(
  () => authStore.securitySettings.autoLockTimeout,
  (newVal) => {
    autoLockTimeout.value = newVal;
  },
  { immediate: true },
);
</script>
