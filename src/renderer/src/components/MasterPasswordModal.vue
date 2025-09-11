<template>
  <Modal
    :visible="visible"
    title="Setup Master Password"
    :icon="Shield"
    icon-background="bg-blue-500/20"
    icon-color="text-blue-400"
    size="lg"
    :close-on-backdrop="false"
    :show-close-button="false"
  >
    <!-- Tab Navigation for Create Mode -->
    <div class="mb-2">
      <NavigationTabs v-model="activeCreateTab" :tabs="createTabs" />
    </div>

    <!-- Local Password Creation Tab -->
    <div v-if="activeCreateTab === 'local'">
      <LocalPasswordForm
        ref="localFormRef"
        :loading="isProcessing"
        @submit="handleLocalPasswordSubmit"
      />
    </div>

    <!-- MongoDB Connection Tab -->
    <div v-if="activeCreateTab === 'mongodb'">
      <MongoPasswordForm
        ref="mongoFormRef"
        :loading="isProcessing"
        @submit="handleMongoPasswordSubmit"
      />
    </div>
  </Modal>
</template>

<script setup lang="ts">
import { ref, watch, onMounted } from 'vue'
import { Shield, Database, Monitor } from 'lucide-vue-next'
import Modal from './ui/Modal.vue'
import LocalPasswordForm from './auth/LocalPasswordForm.vue'
import MongoPasswordForm from './auth/MongoPasswordForm.vue'
import { message } from '../utils/message'
import NavigationTabs from './ui/NavigationTabs.vue'
import type {
  MasterPasswordModalProps,
  SecuritySettings,
  MongoConnectionConfig
} from '../types/auth'

const props = withDefaults(defineProps<MasterPasswordModalProps>(), {
  visible: false,
  error: undefined
})

const emit = defineEmits<{
  localCreated: [password: string, settings: SecuritySettings]
  mongoConnected: [config: MongoConnectionConfig, hasExistingMasterPassword: boolean]
  unlocked: []
}>()

// Create mode tabs
const createTabs = [
  { id: 'local', label: 'Local Only', icon: Monitor },
  { id: 'mongodb', label: 'Connect to MongoDB', icon: Database }
]

// State
const activeCreateTab = ref<'local' | 'mongodb'>('local')
const isProcessing = ref(false)

// Component refs
const localFormRef = ref<InstanceType<typeof LocalPasswordForm> | null>(null)
const mongoFormRef = ref<InstanceType<typeof MongoPasswordForm> | null>(null)

// Methods
const handleLocalPasswordSubmit = async (
  password: string,
  settings: SecuritySettings
): Promise<void> => {
  isProcessing.value = true
  try {
    emit('localCreated', password, settings)
  } finally {
    isProcessing.value = false
  }
}

const handleMongoPasswordSubmit = async (
  config: MongoConnectionConfig,
  hasExistingMasterPassword: boolean
): Promise<void> => {
  isProcessing.value = true
  try {
    let connected = false

    if (hasExistingMasterPassword) {
      // Import existing master password from MongoDB
      connected = (await window.api.invoke('auth:connect-mongo-master-password', config)) as boolean
    } else {
      // Create new master password in MongoDB
      connected = (await window.api.invoke('auth:create-mongo-master-password', config)) as boolean
    }

    if (connected) {
      // Only emit when successful
      emit('mongoConnected', config, hasExistingMasterPassword)
    } else {
      // Show error message for incorrect password
      message.error(
        hasExistingMasterPassword
          ? 'Failed to connect to MongoDB. Please check your master password and try again.'
          : 'Failed to create new master password in MongoDB. Please check your connection and try again.'
      )
    }
  } catch (error) {
    // Show error message for exceptions
    message.error(
      hasExistingMasterPassword
        ? 'An error occurred while connecting to MongoDB. Please try again.'
        : 'An error occurred while creating master password in MongoDB. Please try again.'
    )
    console.error('MongoDB connection error:', error)
  } finally {
    isProcessing.value = false
  }
}

const resetForms = (): void => {
  // Reset child component forms
  if (localFormRef.value) {
    localFormRef.value.resetForm()
  }
  if (mongoFormRef.value) {
    mongoFormRef.value.resetForm()
  }

  // Reset tab selection
  activeCreateTab.value = 'local'
  isProcessing.value = false
}

const focusPasswordInput = (): void => {
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
      resetForms()
      focusPasswordInput()
    }
  }
)

// Handle keyboard shortcuts
onMounted(() => {
  const handleKeyDown = (event: KeyboardEvent): void => {
    if (!props.visible) return

    // Prevent Escape key from closing modal (since it's persistent)
    if (event.key === 'Escape') {
      event.preventDefault()
      event.stopPropagation()
    }
  }

  document.addEventListener('keydown', handleKeyDown)

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
