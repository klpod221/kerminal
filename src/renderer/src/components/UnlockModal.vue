<template>
  <Modal
    :visible="props.visible"
    title="Unlock Application"
    :icon="Lock"
    :close-on-backdrop="false"
    :show-close-button="false"
  >
    <Form class="space-y-2" @submit="handleUnlockSubmit">
      <!-- Description -->
      <div class="text-sm text-gray-300 leading-relaxed">
        <p>Enter your master password to unlock the application and access your encrypted data.</p>
      </div>

      <!-- Password Field -->
      <div class="space-y-4">
        <Input
          id="unlock-password"
          v-model="form.password"
          rules="required|min:8"
          label="Master Password"
          type="password"
          placeholder="Enter master password"
        />
      </div>
    </Form>

    <!-- Footer -->
    <template #footer>
      <div class="flex justify-end w-full">
        <Button
          variant="primary"
          size="sm"
          :loading="loading"
          :icon="Unlock"
          @click="handleUnlockSubmit"
        >
          Unlock Application
        </Button>
      </div>
    </template>
  </Modal>
</template>

<script setup lang="ts">
import { ref } from 'vue'
import Modal from './ui/Modal.vue'
import Form from './ui/Form.vue'
import Input from './ui/Input.vue'
import Button from './ui/Button.vue'
import { message } from '../utils/message'
import { Lock, Unlock } from 'lucide-vue-next'
import { UnlockModalProps } from '../types/modals'

// Props
const props = withDefaults(defineProps<UnlockModalProps>(), {
  visible: false
})

// State
const form = ref({
  password: ''
})
const loading = ref(false)

const emit = defineEmits<{
  unlocked: []
}>()

const handleUnlockSubmit = async (): Promise<void> => {
  loading.value = true
  try {
    const unlocked = await window.electron.ipcRenderer.invoke(
      'auth:unlock-with-password',
      form.value.password
    )

    if (unlocked) {
      emit('unlocked')
    } else {
      // Reset password field on incorrect password
      form.value.password = ''
      message.error('Incorrect master password. Please try again.')
    }
  } catch {
    message.error('Unlock error: An unexpected error occurred. Please try again.')
  } finally {
    loading.value = false
  }
}
</script>
