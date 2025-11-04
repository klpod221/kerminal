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
            :disabled="authStore.isLoading"
            label="Auto-unlock on startup"
            helper-text="Automatically unlock master password when the application starts using system keychain"
          />

          <Select
            id="auto-lock-timeout"
            v-model="autoLockTimeout"
            label="Auto-lock Timeout"
            :options="timeoutOptions"
            @change="handleTimeoutChange"
            :disabled="authStore.isLoading"
            helper-text="Automatically lock the session after period"
          />

          <Button
            variant="danger"
            :disabled="authStore.isLoading"
            :icon="Trash2"
            @click="handleResetPassword"
          >
            Reset Master Password
          </Button>

          <Button
            variant="primary"
            :disabled="authStore.isLoading"
            :icon="Key"
            @click="openChangePasswordModal"
          >
            Change Master Password
          </Button>

          <Button
            variant="secondary"
            @click="handleLock"
            :disabled="authStore.isLoading"
            :icon="Lock"
          >
            Lock Session
          </Button>
        </div>
      </Form>
    </div>
  </Modal>

  <!-- Password Confirmation Modal -->
  <PasswordConfirmModal
    message="To enable auto-unlock, please confirm your master password:"
    @confirm="onPasswordConfirmed"
    @cancel="onPasswordCancelled"
  />

  <!-- Reset Confirmation Modal -->
  <ResetConfirmModal @confirm="onResetConfirmed" @cancel="onResetCancelled" />
</template>

<script setup lang="ts">
import { ref, watch } from "vue";
import { useAuthStore } from "../../stores/auth";
import { Key, Lock, Trash2 } from "lucide-vue-next";
import { message } from "../../utils/message";
import { useOverlay } from "../../composables/useOverlay";
import Modal from "../ui/Modal.vue";
import Form from "../ui/Form.vue";
import Button from "../ui/Button.vue";
import Checkbox from "../ui/Checkbox.vue";
import Select from "../ui/Select.vue";
import Card from "../ui/Card.vue";
import PasswordConfirmModal from "./PasswordConfirmModal.vue";
import ResetConfirmModal from "./ResetConfirmModal.vue";

const autoUnlockEnabled = ref(false);
const autoLockTimeout = ref(0);

const timeoutOptions = [
  { value: 0, label: "Never" },
  { value: 5, label: "5 minutes" },
  { value: 15, label: "15 minutes" },
  { value: 30, label: "30 minutes" },
  { value: 60, label: "1 hour" },
  { value: 120, label: "2 hours" },
  { value: 240, label: "4 hours" },
];

const authStore = useAuthStore();
const { openOverlay, closeOverlay } = useOverlay();

const handleAutoUnlockToggle = async () => {
  let password = null;

  if (autoUnlockEnabled.value) {
    const result = await showPasswordConfirmationModal();
    if (!result.confirmed) {
      autoUnlockEnabled.value = false;
      return;
    }
    password = result.password;
  }

  const config = {
    autoUnlock: autoUnlockEnabled.value,
    autoLockTimeout: autoLockTimeout.value,
    ...(password && { password }),
  };

  await authStore.updateMasterPasswordConfig(config);
  message.success(
    `Auto-unlock has been ${autoUnlockEnabled.value ? "enabled" : "disabled"}.`,
  );
};

let passwordConfirmResolver:
  | ((result: { confirmed: boolean; password?: string }) => void)
  | null = null;

const showPasswordConfirmationModal = (): Promise<{
  confirmed: boolean;
  password?: string;
}> => {
  return new Promise((resolve) => {
    passwordConfirmResolver = resolve;
    openOverlay("password-confirm-modal");
  });
};

const onPasswordConfirmed = (password: string) => {
  if (passwordConfirmResolver) {
    passwordConfirmResolver({ confirmed: true, password });
    passwordConfirmResolver = null;
  }
};

const onPasswordCancelled = () => {
  if (passwordConfirmResolver) {
    passwordConfirmResolver({ confirmed: false });
    passwordConfirmResolver = null;
  }
};

const handleLock = async () => {
  await authStore.lock();
  closeOverlay("master-password-settings");
  message.success("Session locked.");
};

const openChangePasswordModal = () => {
  openOverlay("master-password-change");
};

let resetConfirmResolver: ((confirmed: boolean) => void) | null = null;

const handleResetPassword = async () => {
  const confirmed = await showResetConfirmationModal();
  if (!confirmed) return;

  await authStore.resetMasterPassword();
  closeOverlay("master-password-settings");
  message.success(
    "Master password has been reset successfully. You can now set up a new master password.",
  );
};

const showResetConfirmationModal = (): Promise<boolean> => {
  return new Promise((resolve) => {
    resetConfirmResolver = resolve;
    openOverlay("reset-confirm-modal");
  });
};

const onResetConfirmed = () => {
  if (resetConfirmResolver) {
    resetConfirmResolver(true);
    resetConfirmResolver = null;
  }
};

const onResetCancelled = () => {
  if (resetConfirmResolver) {
    resetConfirmResolver(false);
    resetConfirmResolver = null;
  }
};

const handleTimeoutChange = async () => {
  await authStore.updateMasterPasswordConfig({
    autoUnlock: autoUnlockEnabled.value,
    autoLockTimeout: autoLockTimeout.value,
  });

  authStore.securitySettings.autoLockTimeout = Number(autoLockTimeout.value);

  message.success(
    `Auto-lock timeout updated to ${
      Number(autoLockTimeout.value) === 0
        ? "never"
        : `${autoLockTimeout.value} minutes`
    }.`,
  );
};

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
