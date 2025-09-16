<template>
  <Modal
    id="master-password-unlock"
    :visible="show"
    :show-close-button="false"
    :close-on-backdrop="false"
    :close-on-escape="false"
    title="Unlock Master Password"
    size="sm"
  >
    <div class="flex flex-col gap-6">
      <div class="flex items-start gap-4 p-4 bg-blue-100 rounded-lg border border-blue-400">
        <Lock :size="32" class="text-blue-500" />
        <div>
          <h3 class="text-lg font-semibold text-gray-100 mb-1">Enter Master Password</h3>
          <p class="text-sm text-gray-400">Your SSH profiles are encrypted and require authentication to access.</p>
        </div>
      </div>

      <Form @submit="handleSubmit" class="flex flex-col gap-4">
        <Input
          id="unlock-password"
          ref="passwordInput"
          v-model="verificationForm.password"
          label="Master Password"
          type="password"
          placeholder="Enter your master password"
          rules="required"
          :disabled="isLoading"
          :error-message="error || undefined"
          @keydown.enter="handleSubmit"
        />

        <div v-if="canAutoUnlock" class="p-3 bg-gray-800 border border-gray-700 rounded">
          <Checkbox
            id="remember-session"
            v-model="rememberSession"
            label="Keep me signed in"
            helper-text="Use keychain to auto-unlock in future sessions"
          />
        </div>

        <div v-if="error" class="flex items-center gap-2 bg-red-100 border border-red-400 text-red-700 rounded px-3 py-2">
          <span class="font-bold">⚠</span>
          <span>{{ error }}</span>
        </div>

        <Button
          type="submit"
          variant="primary"
          :loading="isLoading"
          :disabled="!verificationForm.password"
          class="w-full"
        >
          Unlock
        </Button>
      </Form>

      <div class="text-center pt-2 border-t border-gray-700">
        <button
          type="button"
          class="text-blue-500 text-sm underline hover:text-blue-400 focus:outline-none focus:ring-2 focus:ring-blue-500 rounded"
          @click="$emit('forgot-password')"
        >
          Forgot your master password?
        </button>
      </div>
    </div>
  </Modal>
</template>

<script setup lang="ts">
import { Lock } from 'lucide-vue-next'
import Modal from './ui/Modal.vue'
import Form from './ui/Form.vue'
import Input from './ui/Input.vue'
import Button from './ui/Button.vue'
import Checkbox from './ui/Checkbox.vue'
</script>
