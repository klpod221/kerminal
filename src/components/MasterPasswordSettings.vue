<template>
  <Modal
    id="master-password-settings"
    :show-close-button="true"
    title="Master Password Settings"
    size="md"
  >
    <div class="flex flex-col gap-6">
      <Card class="!p-4 !border-blue-400">
        <div class="flex items-start gap-4">
          <Shield class="text-blue-500 w-12 h-12" />
          <div>
            <h3 class="text-lg font-semibold text-gray-100 mb-1">
              Master Password Settings
            </h3>
            <p class="text-sm text-gray-400">
              Manage security and encryption settings for your SSH profiles.
            </p>
          </div>
        </div>
      </Card>

      <Form class="flex flex-col gap-6">
        <div class="flex flex-col gap-2">
          <h4
            class="text-sm font-medium text-gray-100 border-b border-gray-700 pb-2"
          >
            Security Status
          </h4>
          <div class="grid grid-cols-1 md:grid-cols-2 lg:grid-cols-3 gap-4">
            <Card class="!p-3">
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
                    :class="isUnlocked ? 'bg-green-500' : 'bg-red-500'"
                  ></span>
                  {{ isUnlocked ? "Unlocked" : "Locked" }}
                </div>
              </div>
            </Card>

            <Card class="!p-3">
              <div class="flex flex-col gap-1">
                <div
                  class="text-xs font-medium text-gray-400 uppercase tracking-wide"
                >
                  Auto-unlock
                </div>
                <div class="text-sm font-medium text-gray-100">
                  {{ status?.autoUnlockEnabled ? "Enabled" : "Disabled" }}
                </div>
              </div>
            </Card>

            <Card class="!p-3">
              <div class="flex flex-col gap-1">
                <div
                  class="text-xs font-medium text-gray-400 uppercase tracking-wide"
                >
                  Session
                </div>
                <div class="text-sm font-medium text-gray-100">
                  {{ status?.sessionActive ? "Active" : "Inactive" }}
                </div>
              </div>
            </Card>

            <Card class="!p-3">
              <div class="flex flex-col gap-1">
                <div
                  class="text-xs font-medium text-gray-400 uppercase tracking-wide"
                >
                  Device
                </div>
                <div class="text-sm font-medium text-gray-100">
                  {{ currentDevice?.name || "Unknown" }}
                </div>
              </div>
            </Card>

            <Card class="!p-3">
              <div class="flex flex-col gap-1">
                <div
                  class="text-xs font-medium text-gray-400 uppercase tracking-wide"
                >
                  Last unlock
                </div>
                <div class="text-sm font-medium text-gray-100">
                  {{ status?.sessionExpiresAt ? "Recently" : "Unknown" }}
                </div>
              </div>
            </Card>

            <Card class="!p-3">
              <div class="flex flex-col gap-1">
                <div
                  class="text-xs font-medium text-gray-400 uppercase tracking-wide"
                >
                  Created
                </div>
                <div class="text-sm font-medium text-gray-100">Unknown</div>
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
            :disabled="!isUnlocked || isLoading"
            @change="handleAutoUnlockToggle"
            label="Auto-unlock on startup"
            helper-text="Automatically unlock master password when the application starts using system keychain"
          />

          <Checkbox
            id="require-auth-access"
            v-model="requireAuthForAccess"
            :disabled="!isUnlocked || isLoading"
            @change="handleAuthRequirementToggle"
            label="Require authentication for profile access"
            helper-text="Require master password verification before accessing encrypted SSH profiles"
          />
        </div>

        <div class="flex flex-col gap-2" v-if="currentDevice">
          <h4
            class="text-sm font-medium text-gray-100 border-b border-gray-700 pb-2"
          >
            Device Information
          </h4>
          <div class="flex flex-col gap-4">
            <Card class="!p-3">
              <div class="flex justify-between items-center">
                <div class="text-sm font-medium text-gray-400">Device ID</div>
                <div class="text-sm text-gray-100 font-mono">
                  {{ currentDevice.id }}
                </div>
              </div>
            </Card>

            <Card class="!p-3">
              <div class="flex justify-between items-center">
                <div class="text-sm font-medium text-gray-400">Device Name</div>
                <div class="text-sm text-gray-100 font-mono">
                  {{ currentDevice.name }}
                </div>
              </div>
            </Card>

            <Card class="!p-3">
              <div class="flex justify-between items-center">
                <div class="text-sm font-medium text-gray-400">Operating System</div>
                <div class="text-sm text-gray-100 font-mono">{{ "Linux" }}</div>
              </div>
            </Card>

            <Card class="!p-3">
              <div class="flex justify-between items-center">
                <div class="text-sm font-medium text-gray-400">First Seen</div>
                <div class="text-sm text-gray-100">
                  {{
                    currentDevice.created
                      ? formatDateTime(currentDevice.created)
                      : "Unknown"
                  }}
                </div>
              </div>
            </Card>

            <Card class="!p-3">
              <div class="flex justify-between items-center">
                <div class="text-sm font-medium text-gray-400">Last Accessed</div>
                <div class="text-sm text-gray-100">
                  {{
                    currentDevice.lastVerified
                      ? formatDateTime(currentDevice.lastVerified)
                      : "Never"
                  }}
                </div>
              </div>
            </Card>
          </div>
        </div>
      </Form>
    </div>

    <template #footer>
      <div class="flex flex-wrap gap-3">
        <Button
          variant="primary"
          @click="$emit('change-password')"
          :disabled="!isUnlocked || isLoading"
          :icon="Key"
        >
          Change Master Password
        </Button>

        <Button
          variant="secondary"
          @click="handleLock"
          :disabled="!isUnlocked || isLoading"
          :icon="Lock"
        >
          Lock Session
        </Button>

        <Button
          variant="danger"
          @click="$emit('reset-password')"
          :disabled="isLoading"
          :icon="Trash2"
          class="ml-auto"
        >
          Reset Master Password
        </Button>
      </div>
    </template>
  </Modal>
</template>

<script setup lang="ts">
import { ref } from "vue";
import { Shield, Key, Lock, Trash2 } from "lucide-vue-next";
import Modal from "./ui/Modal.vue";
import Form from "./ui/Form.vue";
import Button from "./ui/Button.vue";
import Checkbox from "./ui/Checkbox.vue";
import Card from "./ui/Card.vue";

// State
const isLoading = ref(false);
const isUnlocked = ref(true); // Mock value
const autoUnlockEnabled = ref(false);
const requireAuthForAccess = ref(true);

// Mock data
const status = ref({
  autoUnlockEnabled: false,
  sessionActive: true,
  sessionExpiresAt: new Date(),
});

const currentDevice = ref({
  id: "device-123",
  name: "My Arch Linux",
  created: new Date(),
  lastVerified: new Date(),
});

// Event handlers
const handleAutoUnlockToggle = () => {
  // Implementation for auto unlock toggle
};

const handleAuthRequirementToggle = () => {
  // Implementation for auth requirement toggle
};

const handleLock = () => {
  // Implementation for lock session
};

const formatDateTime = (date: Date) => {
  return date.toLocaleString();
};
</script>
