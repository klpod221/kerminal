<template>
  <div class="space-y-2">
    <!-- Description -->
    <div class="text-sm text-gray-300 leading-relaxed">
      <p class="mb-2">
        Create a master password to secure your data locally. This password will encrypt your SSH
        profiles, saved commands, and tunnel configurations on this device only.
      </p>
      <p class="text-xs text-yellow-400">
        <strong>Important:</strong> This password cannot be recovered. Please store it safely.
      </p>
    </div>

    <!-- Form -->
    <form @submit.prevent="handleSubmit">
      <!-- Master Password -->
      <Input
        v-model="form.password"
        label="Master Password"
        :type="showPassword ? 'text' : 'password'"
        placeholder="Enter master password"
        :error-message="passwordError"
        :right-icon="showPassword ? EyeOff : Eye"
        @right-icon-click="showPassword = !showPassword"
        @blur="validation.validateField('password')"
      />

      <!-- Confirm Password -->
      <Input
        v-model="form.confirmPassword"
        label="Confirm Master Password"
        :type="showConfirmPassword ? 'text' : 'password'"
        placeholder="Confirm master password"
        :error-message="confirmPasswordError"
        :right-icon="showConfirmPassword ? EyeOff : Eye"
        @right-icon-click="showConfirmPassword = !showConfirmPassword"
        @blur="validation.validateField('confirmPassword')"
      />

      <!-- Security Settings -->
      <div class="space-y-1">
        <h3 class="text-sm font-medium text-white">Security Settings</h3>
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
        title="Creating Master Password"
        content="Setting up local encryption..."
        :closable="false"
      />

      <!-- Submit Button -->
      <div class="flex justify-end w-full pt-4">
        <Button
          variant="primary"
          size="sm"
          :disabled="!canSubmit || isProcessing"
          :loading="isProcessing"
          :icon="Shield"
          type="submit"
        >
          Create Local Master Password
        </Button>
      </div>
    </form>
  </div>
</template>

<script setup lang="ts">
import { ref, computed } from 'vue'
import { Shield, Eye, EyeOff } from 'lucide-vue-next'
import Input from '../ui/Input.vue'
import Button from '../ui/Button.vue'
import Checkbox from '../ui/Checkbox.vue'
import Select from '../ui/Select.vue'
import Message from '../ui/Message.vue'
import { useValidation, validationRules } from '../../composables/useValidation'
import type { SecuritySettings } from '../../types/auth'

const emit = defineEmits<{
  submit: [password: string, settings: SecuritySettings]
}>()

defineProps<{
  isProcessing: boolean
}>()

// State
const showPassword = ref(false)
const showConfirmPassword = ref(false)

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
    if (value !== form.value.password) {
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
  const hasConfirmPassword = form.value.confirmPassword.trim().length > 0
  const passwordsMatch = form.value.password === form.value.confirmPassword
  const isValid = !passwordError.value && !confirmPasswordError.value

  return hasPassword && hasConfirmPassword && passwordsMatch && isValid
})

// Methods
const handleSubmit = async (): Promise<void> => {
  if (!canSubmit.value) {
    validation.validateAll()
    return
  }

  emit('submit', form.value.password, form.value.settings)
}

// Reset form function (exposed for parent component)
defineExpose({
  resetForm: () => {
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
    if (validation.fields.value && Object.keys(validation.fields.value).length > 0) {
      validation.resetValidation()
    }
  }
})
</script>
