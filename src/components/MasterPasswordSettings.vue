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

          <Button variant="danger" :disabled="isLoading" :icon="Trash2">
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
                  {{ authStore.currentDevice.device_name }}
                </div>
              </div>
            </Card>

            <Card>
              <div class="flex justify-between items-center">
                <div class="text-sm font-medium text-gray-400">Device Type</div>
                <div class="text-sm text-gray-100">
                  {{ authStore.currentDevice.device_type }}
                </div>
              </div>
            </Card>

            <Card>
              <div class="flex justify-between items-center">
                <div class="text-sm font-medium text-gray-400">
                  Operating System
                </div>
                <div class="text-sm text-gray-100">
                  {{ authStore.currentDevice.os_name }}
                </div>
              </div>
            </Card>

            <Card>
              <div class="flex justify-between items-center">
                <div class="text-sm font-medium text-gray-400">
                  Operating System Version
                </div>
                <div class="text-sm text-gray-100">
                  {{ authStore.currentDevice.os_version }}
                </div>
              </div>
            </Card>

            <Card>
              <div class="flex justify-between items-center">
                <div class="text-sm font-medium text-gray-400">Created At</div>
                <div class="text-sm text-gray-100">
                  {{
                    formatRelativeTime(
                      new Date(authStore.currentDevice.created_at),
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
import { useAuthStore } from "../stores/auth";
import { Key, Lock, Trash2, Book } from "lucide-vue-next";
import { message } from "../utils/message";
import { formatRelativeTime } from "../utils/formatter";
import { useOverlay } from "../composables/useOverlay";
import Modal from "./ui/Modal.vue";
import Form from "./ui/Form.vue";
import Button from "./ui/Button.vue";
import Checkbox from "./ui/Checkbox.vue";
import Card from "./ui/Card.vue";

// State
const isLoading = ref(false);
const autoUnlockEnabled = ref(false);

// Stores and composables
const authStore = useAuthStore();
const { openOverlay, closeOverlay } = useOverlay();

// Event handlers
const handleAutoUnlockToggle = async () => {
  isLoading.value = true;
  try {
    await authStore.updateMasterPasswordConfig({
      autoUnlock: autoUnlockEnabled.value,
    });
    message.success(
      `Auto-unlock has been ${
        autoUnlockEnabled.value ? "enabled" : "disabled"
      }.`,
    );
  } catch (error) {
    message.error("Failed to update auto-unlock setting.");
    // Revert the toggle state on error
    autoUnlockEnabled.value = !autoUnlockEnabled.value;
  } finally {
    isLoading.value = false;
  }
};

const handleLock = async () => {
  isLoading.value = true;
  try {
    await authStore.lock();
    closeOverlay("master-password-settings");
    message.success("Session locked.");
  } catch (error) {
    message.error("Failed to lock session.");
  } finally {
    isLoading.value = false;
  }
};

const openChangePasswordModal = () => {
  openOverlay("master-password-change");
};

// Watchers
watch(
  () => authStore.status.autoUnlockEnabled,
  (newVal) => {
    autoUnlockEnabled.value = newVal;
  },
  { immediate: true },
);
</script>
