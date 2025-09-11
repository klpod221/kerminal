<template>
  <div class="space-y-2">
    <!-- Description -->
    <p class="text-xs text-yellow-400">
      <strong>Important:</strong> This password cannot be recovered. Make sure to remember it!
    </p>

    <!-- Form -->
    <Form @submit="handleSubmit">
      <!-- Master Password -->
      <Input
        id="master-password"
        v-model="form.password"
        rules="required|min:8"
        label="Master Password"
        type="password"
        placeholder="Enter master password"
      />

      <!-- Confirm Password -->
      <Input
        id="confirm-master-password"
        v-model="form.confirmPassword"
        rules="required|same:master-password"
        label="Confirm Master Password"
        type="password"
        placeholder="Confirm master password"
      />

      <!-- Security Settings -->
      <div class="space-y-1">
        <h3 class="text-sm font-medium text-white">Security Settings</h3>
        <div class="space-y-3 bg-gray-800/50 rounded-lg p-4">
          <Checkbox
            id="require-password-on-start"
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
            <Select id="auto-lock-timeout" v-model="form.settings.autoLockTimeout" :helper="false">
              <option :value="0">Never</option>
              <option :value="5">5 minutes</option>
              <option :value="15">15 minutes</option>
              <option :value="30">30 minutes</option>
              <option :value="60">1 hour</option>
            </Select>
          </div>
        </div>
      </div>

      <!-- Submit Button -->
      <div class="flex justify-end w-full pt-4">
        <Button variant="primary" size="sm" :loading="loading" :icon="Save" type="submit">
          Create Master Password
        </Button>
      </div>
    </Form>
  </div>
</template>

<script setup lang="ts">
import { ref } from 'vue'
import { Save } from 'lucide-vue-next'
import Form from '../ui/Form.vue'
import Input from '../ui/Input.vue'
import Checkbox from '../ui/Checkbox.vue'
import Select from '../ui/Select.vue'
import Button from '../ui/Button.vue'
import type { SecuritySettings } from '../../types/auth'

const emit = defineEmits<{
  submit: [password: string, settings: SecuritySettings]
}>()

defineProps<{
  loading: boolean
}>()

// Form data
const form = ref({
  password: '',
  confirmPassword: '',
  settings: {
    requirePasswordOnStart: true,
    autoLockTimeout: 15
  } as SecuritySettings
})

// Methods
const handleSubmit = async (): Promise<void> => {
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
  }
})
</script>
