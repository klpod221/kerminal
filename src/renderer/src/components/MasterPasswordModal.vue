<template>
  <Modal
    :visible="visible"
    :title="mode === 'create' ? 'Create Master Password' : 'Unlock Application'"
    :icon="mode === 'create' ? Shield : Lock"
    icon-background="bg-blue-500/20"
    icon-color="text-blue-400"
    size="md"
    :close-on-backdrop="false"
    :show-close-button="false"
  >
    <form class="space-y-6" @submit.prevent="handleSubmit">
      <!-- Description -->
      <div class="text-sm text-gray-300 leading-relaxed">
        <template v-if="mode === 'create'">
          <p class="mb-3">
            Secure all your sensitive data with a master password. This password will encrypt your
            SSH profiles, saved commands, and tunnel configurations.
          </p>
          <p class="text-xs text-yellow-400">
            <strong>Important:</strong> This password cannot be recovered. Please store it safely.
          </p>
        </template>
        <template v-else>
          <p>
            Enter your master password to unlock the application and access your encrypted data.
          </p>
        </template>
      </div>

      <!-- Error Message -->
      <Message
        v-if="error"
        type="error"
        title="Authentication Error"
        :content="error"
        :closable="false"
      />

      <!-- Password Fields -->
      <div class="space-y-4">
        <!-- Master Password -->
        <Input
          v-model="form.password"
          label="Master Password"
          :type="showPassword ? 'text' : 'password'"
          placeholder="Enter master password"
          :rules="['required']"
          :error-message="passwordError"
          :right-icon="showPassword ? EyeOff : Eye"
          @right-icon-click="showPassword = !showPassword"
          @blur="validation.validateField('password')"
        />

        <!-- Confirm Password (Create mode only) -->
        <div v-if="mode === 'create'">
          <Input
            v-model="form.confirmPassword"
            label="Confirm Master Password"
            :type="showConfirmPassword ? 'text' : 'password'"
            placeholder="Confirm master password"
            :rules="['required']"
            :error-message="confirmPasswordError"
            :right-icon="showConfirmPassword ? EyeOff : Eye"
            @right-icon-click="showConfirmPassword = !showConfirmPassword"
            @blur="validation.validateField('confirmPassword')"
          />
        </div>
      </div>

      <!-- Initial Security Settings (Create mode only) -->
      <div v-if="mode === 'create'" class="space-y-1">
        <h3 class="text-sm font-medium text-white">Initial Security Settings</h3>
        <div class="space-y-3 bg-gray-800/50 rounded-lg p-4">
          <Checkbox
            v-model="form.settings.requirePasswordOnStart"
            label="Require master password on application start"
            :helper="false"
          />
          <p class="text-xs text-gray-400 mt-1 ml-6">
            When enabled, you'll need to enter your password every time you open the app
          </p>

          <div class="space-y-2">
            <label class="block text-sm font-medium text-gray-300">
              Auto-lock after inactivity
            </label>
            <Select v-model="form.settings.autoLockTimeout">
              <option :value="0">Never</option>
              <option :value="5">5 minutes</option>
              <option :value="15">15 minutes</option>
              <option :value="30">30 minutes</option>
              <option :value="60">1 hour</option>
            </Select>
          </div>
        </div>
      </div>

      <!-- Loading State -->
      <Message
        v-if="isProcessing"
        type="loading"
        :title="mode === 'create' ? 'Creating Master Password' : 'Unlocking Application'"
        :content="mode === 'create' ? 'Setting up encryption...' : 'Verifying password...'"
        :closable="false"
      />
    </form>

    <template #footer>
      <div class="flex justify-end w-full">
        <Button
          variant="primary"
          size="sm"
          :disabled="!canSubmit || isProcessing"
          :loading="isProcessing"
          :icon="mode === 'create' ? Shield : Unlock"
          @click="handleSubmit"
        >
          {{ mode === 'create' ? 'Create Master Password' : 'Unlock Application' }}
        </Button>
      </div>
    </template>
  </Modal>
</template>

<script setup lang="ts">
import { ref, computed, watch, onMounted } from 'vue'
import { Shield, Lock, Unlock, Eye, EyeOff } from 'lucide-vue-next'
import Modal from './ui/Modal.vue'
import Input from './ui/Input.vue'
import Button from './ui/Button.vue'
import Checkbox from './ui/Checkbox.vue'
import Select from './ui/Select.vue'
import Message from './ui/Message.vue'
import { useValidation, validationRules } from '../composables/useValidation'
import type { MasterPasswordModalProps, SecuritySettings } from '../types/auth'

const props = withDefaults(defineProps<MasterPasswordModalProps>(), {
  visible: false,
  mode: 'unlock',
  error: undefined
})

const emit = defineEmits<{
  created: [password: string, settings: SecuritySettings]
  unlocked: []
}>()

// State
const showPassword = ref(false)
const showConfirmPassword = ref(false)
const isProcessing = ref(false)

// Form data
const form = ref({
  password: '',
  confirmPassword: '',
  settings: {
    requirePasswordOnStart: true,
    autoLockTimeout: 15
  } as SecuritySettings
})

// Validation setup
const validation = useValidation()

// Create reactive refs for validation
const passwordRef = computed({
  get: () => form.value.password,
  set: (value) => {
    form.value.password = value
  }
})

const confirmPasswordRef = computed({
  get: () => form.value.confirmPassword,
  set: (value) => {
    form.value.confirmPassword = value
  }
})

// Register validation fields
validation.registerField('password', passwordRef, [
  validationRules.required('Master password is required'),
  validationRules.minLength(8, 'Master password must be at least 8 characters')
])

validation.registerField('confirmPassword', confirmPasswordRef, [
  validationRules.required('Please confirm your password'),
  validationRules.custom((value) => {
    if (props.mode === 'create' && value !== form.value.password) {
      return false
    }
    return true
  }, 'Passwords do not match')
])

// Error message computed properties
const passwordError = computed(() => validation.fields.value?.password?.error?.value || undefined)
const confirmPasswordError = computed(
  () => validation.fields.value?.confirmPassword?.error?.value || undefined
)

// Computed
const canSubmit = computed(() => {
  const hasPassword = form.value.password.trim().length > 0

  if (props.mode === 'create') {
    const hasConfirmPassword = form.value.confirmPassword.trim().length > 0
    const passwordsMatch = form.value.password === form.value.confirmPassword
    const isValid = !passwordError.value && !confirmPasswordError.value
    return hasPassword && hasConfirmPassword && passwordsMatch && isValid
  }

  return hasPassword && !passwordError.value
})

// Methods
const resetForm = (): void => {
  form.value = {
    password: '',
    confirmPassword: '',
    settings: {
      requirePasswordOnStart: true,
      autoLockTimeout: 15
    }
  }

  showPassword.value = false
  showConfirmPassword.value = false

  // Reset validation if fields are initialized
  if (validation.fields.value && Object.keys(validation.fields.value).length > 0) {
    validation.resetValidation()
  }
}

const handleSubmit = async (): Promise<void> => {
  if (!canSubmit.value) {
    validation.validateAll()
    return
  }

  try {
    isProcessing.value = true

    if (props.mode === 'create') {
      emit('created', form.value.password, form.value.settings)
    } else {
      // Try to unlock with the provided password
      const unlocked = await window.electron.ipcRenderer.invoke(
        'auth:unlock-with-password',
        form.value.password
      )
      if (unlocked) {
        emit('unlocked')
      } else {
        // Show error - password is incorrect
        form.value.password = ''
        // You could add an error message here
        console.error('Incorrect password')
        return
      }
    }
  } catch (error) {
    console.error('Form submission error:', error)
  } finally {
    isProcessing.value = false
  }
}

// Focus password input when modal opens
const focusPasswordInput = (): void => {
  // Wait for next tick to ensure DOM is rendered
  setTimeout(() => {
    const passwordInput = document.querySelector('input[type="password"]') as HTMLInputElement
    if (passwordInput) {
      passwordInput.focus()
    }
  }, 100)
}

// Watch for modal visibility changes
watch(
  () => props.visible,
  (visible) => {
    if (visible) {
      resetForm()
      focusPasswordInput()
    } else {
      isProcessing.value = false
    }
  }
)

// Watch for mode changes
watch(
  () => props.mode,
  () => {
    resetForm()
  }
)

// Handle keyboard shortcuts
onMounted(() => {
  const handleKeyDown = (event: KeyboardEvent): void => {
    if (!props.visible) return

    // Enter key submits the form
    if (event.key === 'Enter' && canSubmit.value && !isProcessing.value) {
      event.preventDefault()
      handleSubmit()
    }

    // Prevent Escape key from closing modal (since it's persistent)
    if (event.key === 'Escape') {
      event.preventDefault()
      event.stopPropagation()
    }
  }

  document.addEventListener('keydown', handleKeyDown)

  // Cleanup on unmount
  return () => {
    document.removeEventListener('keydown', handleKeyDown)
  }
})
</script>

<style scoped>
/* Prevent interaction with background content */
:deep(.modal-overlay) {
  pointer-events: all;
}

/* Enhanced focus styles for password inputs */
:deep(input[type='password']:focus),
:deep(input[type='text']:focus) {
  border-color: #3b82f6;
  box-shadow:
    0 0 0 2px #3b82f6,
    0 0 0 4px #1f2937;
}

/* Subtle animation for the modal */
.modal-content {
  animation: modalSlideIn 0.3s ease-out;
}

@keyframes modalSlideIn {
  from {
    opacity: 0;
    transform: scale(0.95) translateY(-10px);
  }
  to {
    opacity: 1;
    transform: scale(1) translateY(0);
  }
}
</style>
