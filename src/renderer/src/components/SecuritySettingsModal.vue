<template>
  <Modal
    :visible="visible"
    title="Security Settings"
    :icon="Shield"
    icon-background="bg-green-500/20"
    icon-color="text-green-400"
    size="lg"
    @close="handleClose"
  >
    <form class="space-y-6" @submit.prevent="handleSave">
      <!-- Description -->
      <div class="text-sm text-gray-300 leading-relaxed">
        <p>
          Configure when and how the application should lock itself to protect your encrypted data.
        </p>
      </div>

      <!-- Error/Success Messages -->
      <Message
        v-if="errorMessage"
        type="error"
        title="Error"
        :content="errorMessage"
        :closable="true"
        @close="errorMessage = ''"
      />

      <Message
        v-if="successMessage"
        type="success"
        title="Success"
        :content="successMessage"
        :closable="true"
        @close="successMessage = ''"
      />

      <!-- Security Settings -->
      <div class="space-y-1">
        <h3 class="text-lg font-medium text-white">Security Options</h3>

        <div class="space-y-4 bg-gray-800/50 rounded-lg p-4">
          <!-- Require Password on Start -->
          <div class="flex items-start space-x-3">
            <Checkbox
              v-model="settings.requirePasswordOnStart"
              :helper="false"
              @change="handleRequirePasswordChange"
            />
            <div class="flex-1">
              <label
                class="block text-sm font-medium text-gray-300 cursor-pointer"
                @click="settings.requirePasswordOnStart = !settings.requirePasswordOnStart"
              >
                Require Master Password on application start
              </label>
              <p class="text-xs text-gray-400 mt-1">
                When enabled, the application will always ask for your master password on startup.
                When disabled, your encrypted key will be stored securely in the system keychain for
                automatic unlock.
              </p>
            </div>
          </div>

          <!-- Auto-lock Timeout -->
          <div class="space-y-2">
            <label class="block text-sm font-medium text-gray-300">
              Automatically lock after inactivity
            </label>
            <Select
              v-model="settings.autoLockTimeout"
              placeholder="Select timeout"
              @change="handleAutoLockTimeoutChange"
            >
              <option :value="0">Never</option>
              <option :value="5">5 minutes</option>
              <option :value="15">15 minutes</option>
              <option :value="30">30 minutes</option>
              <option :value="60">1 hour</option>
            </Select>
            <p class="text-xs text-gray-400">
              The application will automatically lock and require your master password after the
              specified period of inactivity.
            </p>
          </div>

          <!-- Biometric Authentication (Future Feature) -->
          <div class="opacity-50">
            <div class="flex items-start space-x-3">
              <Checkbox v-model="settings.useBiometrics" :disabled="true" :helper="false" />
              <div class="flex-1">
                <label class="block text-sm font-medium text-gray-500">
                  Use Biometrics to unlock (Touch ID / Windows Hello)
                </label>
                <p class="text-xs text-gray-500 mt-1">
                  Coming soon - Use biometric authentication for quick unlock
                </p>
              </div>
            </div>
          </div>
        </div>
      </div>

      <!-- Current Status -->
      <div class="space-y-1">
        <h3 class="text-lg font-medium text-white">Current Status</h3>

        <div class="bg-gray-800/50 rounded-lg p-4">
          <div class="grid grid-cols-1 md:grid-cols-2 gap-4">
            <div>
              <dt class="text-sm font-medium text-gray-400">Application Status</dt>
              <dd class="text-sm text-green-400">
                <div class="flex items-center space-x-1">
                  <div class="w-2 h-2 bg-green-400 rounded-full"></div>
                  <span>Unlocked</span>
                </div>
              </dd>
            </div>

            <div>
              <dt class="text-sm font-medium text-gray-400">Auto-unlock</dt>
              <dd class="text-sm text-white">
                {{ settings.requirePasswordOnStart ? 'Disabled' : 'Enabled' }}
              </dd>
            </div>

            <div>
              <dt class="text-sm font-medium text-gray-400">Auto-lock Timer</dt>
              <dd class="text-sm text-white">
                {{ autoLockTimeoutText }}
              </dd>
            </div>

            <div>
              <dt class="text-sm font-medium text-gray-400">Keychain Storage</dt>
              <dd class="text-sm text-white">
                {{ settings.requirePasswordOnStart ? 'Not Used' : 'Active' }}
              </dd>
            </div>
          </div>
        </div>
      </div>

      <!-- Advanced Actions -->
      <div class="space-y-1">
        <h3 class="text-lg font-medium text-white">Advanced Actions</h3>

        <div class="bg-gray-800/50 rounded-lg p-4 space-y-3">
          <Button
            variant="secondary"
            size="sm"
            :icon="Lock"
            :disabled="isSaving"
            @click="handleLockNow"
          >
            Lock Application Now
          </Button>

          <Button
            variant="secondary"
            size="sm"
            :icon="Key"
            :disabled="isSaving"
            @click="handleChangeMasterPassword"
          >
            Change Master Password
          </Button>
        </div>
      </div>

      <!-- Loading State -->
      <Message
        v-if="isSaving"
        type="loading"
        title="Updating Settings"
        content="Applying security configuration..."
        :closable="false"
      />
    </form>

    <template #footer>
      <div class="flex justify-between w-full">
        <Button variant="ghost" size="sm" :disabled="isSaving" @click="handleClose">
          Cancel
        </Button>

        <div class="flex space-x-3">
          <Button variant="ghost" size="sm" :disabled="isSaving" @click="handleReset">
            Reset to Defaults
          </Button>
          <Button
            variant="primary"
            size="sm"
            :disabled="!hasChanges || isSaving"
            :loading="isSaving"
            :icon="Save"
            @click="handleSave"
          >
            Save Settings
          </Button>
        </div>
      </div>
    </template>
  </Modal>
</template>

<script setup lang="ts">
import { ref, computed, watch, onMounted } from 'vue'
import { Shield, Lock, Key, Save } from 'lucide-vue-next'
import Modal from './ui/Modal.vue'
import Button from './ui/Button.vue'
import Checkbox from './ui/Checkbox.vue'
import Select from './ui/Select.vue'
import Message from './ui/Message.vue'
import type { SecuritySettingsModalProps, SecuritySettings } from '../types/auth'

const props = withDefaults(defineProps<SecuritySettingsModalProps>(), {
  visible: false
})

const emit = defineEmits<{
  'update:visible': [visible: boolean]
  close: []
  'lock-now': []
  'change-master-password': []
}>()

// State
const isSaving = ref(false)
const errorMessage = ref('')
const successMessage = ref('')
const originalSettings = ref<SecuritySettings>({
  requirePasswordOnStart: true,
  autoLockTimeout: 15,
  useBiometrics: false
})

// Form settings (copy of original for comparison)
const settings = ref<SecuritySettings>({
  requirePasswordOnStart: true,
  autoLockTimeout: 15,
  useBiometrics: false
})

// Computed
const autoLockTimeoutText = computed(() => {
  const timeout = settings.value.autoLockTimeout
  if (timeout === 0) return 'Never'
  if (timeout < 60) return `${timeout} minutes`
  return `${timeout / 60} hour${timeout > 60 ? 's' : ''}`
})

const hasChanges = computed(() => {
  return (
    settings.value.requirePasswordOnStart !== originalSettings.value.requirePasswordOnStart ||
    settings.value.autoLockTimeout !== originalSettings.value.autoLockTimeout ||
    settings.value.useBiometrics !== originalSettings.value.useBiometrics
  )
})

// Methods
const loadCurrentSettings = async (): Promise<void> => {
  try {
    const currentSettings = (await window.electron.ipcRenderer.invoke(
      'auth:get-security-settings'
    )) as SecuritySettings | null

    if (currentSettings) {
      settings.value = { ...currentSettings }
      originalSettings.value = { ...currentSettings }
    }
  } catch (error) {
    console.error('Failed to load security settings:', error)
    errorMessage.value = 'Failed to load current security settings'
  }
}

const handleSave = async (): Promise<void> => {
  if (!hasChanges.value) return

  try {
    isSaving.value = true
    errorMessage.value = ''
    successMessage.value = ''

    // Create plain object to avoid Vue reactivity issues
    const plainSettings = {
      requirePasswordOnStart: settings.value.requirePasswordOnStart,
      autoLockTimeout: settings.value.autoLockTimeout,
      useBiometrics: settings.value.useBiometrics
    }

    await window.electron.ipcRenderer.invoke('auth:update-security-settings', plainSettings)

    // Update original settings to reflect saved state
    originalSettings.value = { ...settings.value }

    successMessage.value = 'Security settings updated successfully'
  } catch (error) {
    console.error('Failed to save security settings:', error)
    errorMessage.value = 'Failed to save security settings'
  } finally {
    isSaving.value = false
  }
}

const handleReset = (): void => {
  settings.value = {
    requirePasswordOnStart: true,
    autoLockTimeout: 15,
    useBiometrics: false
  }
}

const handleClose = (): void => {
  // Reset to original settings if there are unsaved changes
  if (hasChanges.value) {
    settings.value = { ...originalSettings.value }
  }

  // Clear messages
  errorMessage.value = ''
  successMessage.value = ''

  emit('update:visible', false)
  emit('close')
}

const handleRequirePasswordChange = (): void => {
  // Clear any previous messages when settings change
  errorMessage.value = ''
  successMessage.value = ''
}

const handleAutoLockTimeoutChange = (): void => {
  // Clear any previous messages when settings change
  errorMessage.value = ''
  successMessage.value = ''
}

const handleLockNow = async (): Promise<void> => {
  try {
    await window.electron.ipcRenderer.invoke('auth:lock')
    emit('lock-now')
    handleClose()
  } catch (error) {
    console.error('Failed to lock application:', error)
    errorMessage.value = 'Failed to lock application'
  }
}

const handleChangeMasterPassword = (): void => {
  emit('change-master-password')
  handleClose()
}

// Load settings when modal opens
watch(
  () => props.visible,
  async (visible) => {
    if (visible) {
      await loadCurrentSettings()
    }
  }
)

// Load initial settings
onMounted(() => {
  if (props.visible) {
    loadCurrentSettings()
  }
})
</script>

<style scoped>
/* Enhanced checkbox styling within this modal */
:deep(.checkbox) {
  margin-right: 0.75rem;
}

/* Status indicator pulse animation */
.bg-green-400.rounded-full {
  animation: statusPulse 2s infinite;
}

@keyframes statusPulse {
  0%,
  100% {
    opacity: 1;
  }
  50% {
    opacity: 0.5;
  }
}

/* Smooth transitions for form elements */
.space-y-4 > * {
  transition: all 0.2s ease-in-out;
}

/* Disabled state styling */
.opacity-50 {
  transition: opacity 0.2s ease-in-out;
}
</style>
