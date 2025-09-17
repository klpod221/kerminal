<template>
  <Modal
    id="master-password-change"
    parent-id="master-password-settings"
    :show-close-button="true"
    title="Change Master Password"
    size="md"
  >
    <div class="flex flex-col gap-6">
      <Card>
        <div class="flex items-start gap-4">
          <Key :size="80" class="text-blue-400" />
          <div>
            <h3 class="text-lg font-semibold text-gray-100 mb-1">
              Update Security
            </h3>
            <p class="text-sm text-gray-400">
              Change your master password regularly to keep your data secure.
            </p>
          </div>
        </div>
      </Card>

      <Form @submit="" class="flex flex-col gap-6">
        <div class="flex flex-col gap-2">
          <h4
            class="text-sm font-medium text-gray-100 border-b border-gray-700 pb-2"
          >
            Current Password
          </h4>
          <Input
            id="current-password"
            v-model="changeForm.oldPassword"
            label="Current Master Password"
            type="password"
            placeholder="Enter your current password"
            rules="required"
            :disabled="isLoading"
          />
        </div>

        <div class="flex flex-col gap-2">
          <h4
            class="text-sm font-medium text-gray-100 border-b border-gray-700 pb-2"
          >
            New Password
          </h4>
          <Input
            id="new-password"
            v-model="changeForm.newPassword"
            label="New Master Password"
            type="password"
            placeholder="Enter a strong new password"
            rules="required"
            :disabled="isLoading"
          />

          <Input
            id="confirm-new-password"
            v-model="changeForm.confirmNewPassword"
            label="Confirm New Password"
            type="password"
            placeholder="Confirm your new password"
            rules="required"
            :disabled="isLoading"
          />
        </div>

        <Card>
          <div class="flex items-start gap-3">
            <AlertTriangle
              :size="20"
              class="text-yellow-500 mt-0.5 flex-shrink-0"
            />
            <div>
              <h5 class="text-sm font-semibold text-yellow-400 mb-1">
                Important Security Notice
              </h5>
              <p class="text-sm text-gray-300 mb-2">
                Changing your master password will:
              </p>
              <ul class="text-sm text-gray-400 list-disc pl-4 space-y-1">
                <li>Re-encrypt all your stored SSH credentials</li>
                <li>Invalidate auto-unlock on other devices</li>
                <li>Require you to re-enter the new password on all devices</li>
              </ul>
            </div>
          </div>
        </Card>

        <div class="flex justify-end gap-3 pt-4 border-t border-gray-700">
          <Button type="button" variant="secondary" :disabled="isLoading">
            Cancel
          </Button>
          <Button type="submit" variant="primary" :loading="isLoading">
            Change Password
          </Button>
        </div>
      </Form>
    </div>
  </Modal>
</template>

<script setup lang="ts">
import { ref } from "vue";
import { Key, AlertTriangle } from "lucide-vue-next";
import Modal from "./ui/Modal.vue";
import Form from "./ui/Form.vue";
import Input from "./ui/Input.vue";
import Button from "./ui/Button.vue";
import Card from "./ui/Card.vue";

// State
const isLoading = ref(false);
const changeForm = ref({
  oldPassword: "",
  newPassword: "",
  confirmNewPassword: "",
});
</script>
