<template>
  <Modal
    :visible="visible"
    title="MongoDB Sync Settings"
    :icon="Database"
    icon-background="bg-green-500/20"
    icon-color="text-green-400"
    size="lg"
    @close="$emit('close')"
  >
    <div class="space-y-6">
      <!-- Connection Status -->
      <div class="space-y-1">
        <h3 class="text-lg font-medium text-white">Sync Status</h3>
        <div
          class="flex items-center justify-between p-4 rounded-lg border transition-colors"
          :class="{
            'bg-gray-800 border-gray-600':
              !syncStatus.isConnected && !syncStatus.lastError && !syncStatus.isLoading,
            'bg-green-900/50 border-green-500': syncStatus.isConnected && !syncStatus.isLoading,
            'bg-red-900/50 border-red-500': syncStatus.lastError,
            'bg-amber-900/50 border-amber-500': syncStatus.isLoading
          }"
        >
          <div class="flex flex-col gap-1">
            <div class="flex items-center gap-2">
              <span class="font-medium text-white">{{ statusLabel }}</span>
              <span
                v-if="currentConfig?.autoSync && syncStatus.isConnected"
                class="inline-flex items-center px-2 py-0.5 text-xs font-medium text-green-400 bg-green-900/50 border border-green-500 rounded-full"
              >
                Auto Sync
              </span>
            </div>
            <span v-if="syncStatus.lastSync" class="text-sm text-gray-400">
              Last sync: {{ formatRelativeTime(syncStatus.lastSync) }}
            </span>
            <span v-else-if="syncStatus.isConnected" class="text-sm text-gray-400">
              Never synced
            </span>
            <span
              v-if="currentConfig?.autoSync && syncStatus.isConnected"
              class="text-xs text-gray-400 italic"
            >
              Syncing every {{ currentConfig.syncInterval }} seconds
            </span>
            <span v-if="syncStatus.lastError" class="text-sm text-red-400">
              {{ syncStatus.lastError }}
            </span>
          </div>
          <div>
            <Button
              v-if="syncStatus.isConnected"
              variant="secondary"
              size="sm"
              :loading="syncStatus.isLoading"
              @click="performSync"
            >
              Sync Now
            </Button>
          </div>
        </div>
      </div>

      <!-- MongoDB Configuration -->
      <form class="space-y-1">
        <h3 class="text-lg font-medium text-white">MongoDB Configuration</h3>
        <div>
          <Input
            v-model="formData.mongoUri"
            label="MongoDB URI"
            class="mb-4"
            placeholder="mongodb://username:password@hostname:port"
            :disabled="isLoading"
            required
            helper-text="Include credentials in the URI"
          />

          <Input
            v-model="formData.databaseName"
            label="Database Name"
            placeholder="kerminal"
            :disabled="isLoading"
            required
          />

          <Input
            v-model.number="formData.syncInterval"
            label="Sync Interval (seconds)"
            type="number"
            min="5"
            max="3600"
            :disabled="isLoading"
            required
            helper-text="Minimum 5 seconds, maximum 1 hour (3600 seconds)"
          />
        </div>
      </form>

      <!-- Master Password Verification -->
      <div v-if="showPasswordVerification && connectionTestPassed" class="space-y-1">
        <h3 class="text-lg font-medium text-white">MongoDB Master Password</h3>
        <div class="bg-blue-900/20 border border-blue-500 rounded-lg p-4">
          <p class="text-sm text-blue-200 mb-4">
            MongoDB database contains existing data. Please enter the master password to verify and
            proceed with sync setup.
          </p>

          <div class="space-y-4">
            <div
              v-if="passwordVerified"
              class="bg-green-900/20 border border-green-500 rounded-lg p-3"
            >
              <p class="text-sm text-green-200">
                ✅ MongoDB master password verified successfully! You can now setup sync.
              </p>
            </div>

            <Input
              v-model="masterPassword"
              label="MongoDB Master Password"
              :type="showMasterPassword ? 'text' : 'password'"
              placeholder="Enter existing MongoDB master password"
              :right-icon="showMasterPassword ? EyeOff : Eye"
              :disabled="passwordVerified"
              @right-icon-click="showMasterPassword = !showMasterPassword"
            />
            <p class="text-xs text-gray-400">
              {{
                passwordVerified
                  ? 'Password verified! Click "Setup Sync" to continue.'
                  : 'This is the master password that was used to encrypt data in your MongoDB database.'
              }}
            </p>
          </div>
        </div>
      </div>

      <!-- Data Migration Warning -->
      <div
        v-if="hasExistingData && !syncStatus.isConnected && connectionTestPassed"
        class="space-y-1"
      >
        <h3 class="text-lg font-medium text-white">Data Migration</h3>
        <div class="bg-yellow-900/20 border border-yellow-500 rounded-lg p-4">
          <p class="text-sm text-yellow-200 mb-3">
            <strong>Warning:</strong> You have existing local data. When you setup sync:
          </p>
          <ul class="text-sm text-yellow-200 list-disc list-inside space-y-1">
            <li v-if="mongoHasMasterPassword">
              MongoDB already contains data - password verification required
            </li>
            <li v-else>Your local data will be uploaded to MongoDB</li>
            <li>All data will be encrypted using the master password</li>
            <li>A backup will be created automatically</li>
          </ul>
        </div>
      </div>

      <!-- Existing Configuration Management -->
      <div v-if="currentConfig" class="space-y-1">
        <h3 class="text-lg font-medium text-white">Manage Configuration</h3>
        <PopConfirm
          title="Delete sync configuration?"
          content="Are you sure you want to delete the sync configuration? This action cannot be undone."
          placement="top"
          @confirm="deleteSyncConfig"
        >
          <Button variant="danger" :loading="isDeleting"> Delete Configuration </Button>
        </PopConfirm>
      </div>

      <div class="flex justify-between space-x-3 mt-4">
        <Button type="button" variant="secondary" @click="$emit('close')">Close</Button>
        <Button
          type="button"
          variant="primary"
          :loading="isTestingConnection || isLoading"
          :icon="Save"
          @click="handleRightButtonClick"
        >
          {{ getRightButtonLabel }}
        </Button>
      </div>
    </div>
  </Modal>
</template>

<script setup lang="ts">
import { ref, reactive, computed, watch, onMounted, onUnmounted } from 'vue'
import { Database, Save, Eye, EyeOff } from 'lucide-vue-next'
import Modal from './ui/Modal.vue'
import Input from './ui/Input.vue'
import Button from './ui/Button.vue'
import PopConfirm from './ui/PopConfirm.vue'
import { message } from '../utils/message'
import type { SyncConfig, SyncStatus } from '../types/sync'

// Props & Emits
interface Props {
  visible?: boolean
  refreshTrigger?: number
}

const props = withDefaults(defineProps<Props>(), {
  visible: false,
  refreshTrigger: 0
})

const emit = defineEmits<{
  close: []
  configUpdated: [config: SyncConfig | null]
}>()

// State Management
const currentConfig = ref<SyncConfig | null>(null)
const syncStatus = ref<SyncStatus>({ isConnected: false, isLoading: false })
const hasExistingData = ref(false)
const mongoHasMasterPassword = ref(false)
const connectionTestPassed = ref(false)
const statusRefreshInterval = ref<number | null>(null)

// Password verification state
const showPasswordVerification = ref(false)
const masterPassword = ref('')
const showMasterPassword = ref(false)
const passwordVerified = ref(false)

// Loading states
const isLoading = ref(false)
const isTestingConnection = ref(false)
const isDeleting = ref(false)

// Form data
const formData = reactive({
  mongoUri: '',
  databaseName: 'kerminal',
  enabled: true,
  autoSync: true,
  syncInterval: 30
})

// Computed Properties
const statusLabel = computed(() => {
  if (syncStatus.value.isLoading) return 'Syncing...'
  if (syncStatus.value.isConnected) return 'Connected'
  if (syncStatus.value.lastError) return 'Error'
  return 'Disconnected'
})

const getRightButtonLabel = computed(() => {
  if (!connectionTestPassed.value) return 'Test Connection'
  if (showPasswordVerification.value && mongoHasMasterPassword.value && !passwordVerified.value) {
    return 'Verify Password'
  }
  return 'Setup Sync'
})

// Helper Functions
function resetPasswordState(): void {
  showPasswordVerification.value = false
  masterPassword.value = ''
  passwordVerified.value = false
}

// Main Methods
async function handleRightButtonClick(): Promise<void> {
  if (!connectionTestPassed.value) {
    await testConnection()
  } else if (
    showPasswordVerification.value &&
    mongoHasMasterPassword.value &&
    !passwordVerified.value
  ) {
    await verifyMasterPassword()
  } else {
    await handleSave()
  }
}

async function loadConfig(): Promise<void> {
  try {
    const config = (await window.api.invoke('sync.getConfig')) as SyncConfig | null
    if (config) {
      currentConfig.value = config
      Object.assign(formData, config)
    }

    syncStatus.value = (await window.api.invoke('sync.getStatus')) as SyncStatus
    hasExistingData.value = (await window.api.invoke('sync.hasExistingData')) as boolean
  } catch {
    message.error('Failed to load sync configuration')
  }
}

async function testConnection(): Promise<void> {
  if (!formData.mongoUri || !formData.databaseName) {
    message.error('Please fill in MongoDB URI and database name')
    return
  }

  isTestingConnection.value = true
  connectionTestPassed.value = false
  mongoHasMasterPassword.value = false

  try {
    const success = await window.api.invoke(
      'sync.testConnection',
      formData.mongoUri,
      formData.databaseName
    )

    if (success) {
      message.success('Connection test successful!')
      connectionTestPassed.value = true

      mongoHasMasterPassword.value = (await window.api.invoke(
        'auth:check-mongo-master-password-exists',
        formData.mongoUri,
        formData.databaseName
      )) as boolean

      if (mongoHasMasterPassword.value) {
        showPasswordVerification.value = true
      }
    } else {
      message.error('Failed to connect to MongoDB')
    }
  } catch {
    message.error('Connection test failed')
  } finally {
    isTestingConnection.value = false
  }
}

async function handleSave(): Promise<void> {
  if (!connectionTestPassed.value) {
    message.error('Please test the connection first')
    return
  }

  if (mongoHasMasterPassword.value && !passwordVerified.value) {
    message.error('Please verify the MongoDB master password first')
    return
  }

  await setupSync()
}

async function setupSync(): Promise<void> {
  isLoading.value = true

  try {
    const configData = {
      mongoUri: formData.mongoUri,
      databaseName: formData.databaseName,
      enabled: formData.enabled,
      autoSync: formData.autoSync,
      syncInterval: formData.syncInterval
    }

    if (showPasswordVerification.value && passwordVerified.value) {
      const success = await window.api.invoke(
        'sync.setupWithPassword',
        configData,
        masterPassword.value
      )

      if (!success) {
        message.error('Failed to setup sync with password verification')
        return
      }

      resetPasswordState()
    } else {
      const success = await window.api.invoke('sync.setup', configData)

      if (!success) {
        message.error('Failed to save sync configuration')
        return
      }
    }

    message.success('Sync configuration saved successfully!')
    currentConfig.value = { ...configData }

    await new Promise((resolve) => setTimeout(resolve, 100))
    syncStatus.value = (await window.api.invoke('sync.getStatus')) as SyncStatus

    emit('configUpdated', currentConfig.value)
  } catch (error) {
    console.error('Error setting up sync:', error)

    if (error instanceof Error) {
      if (error.message.includes('Incorrect master password')) {
        message.error('Incorrect master password')
      } else {
        message.error(`Failed to setup sync: ${error.message}`)
      }
    } else {
      message.error('Failed to setup sync')
    }
  } finally {
    isLoading.value = false
  }
}

async function performSync(): Promise<void> {
  try {
    syncStatus.value.isLoading = true
    await window.api.invoke('sync.performSync')
    syncStatus.value = (await window.api.invoke('sync.getStatus')) as SyncStatus
    message.success('Sync completed successfully!')
  } catch {
    message.error('Sync failed')
    syncStatus.value.isLoading = false
  }
}

async function deleteSyncConfig(): Promise<void> {
  isDeleting.value = true
  try {
    const success = await window.api.invoke('sync.deleteConfig')
    if (success) {
      message.success('Sync configuration deleted successfully!')
      resetForm()
    } else {
      message.error('Failed to delete sync configuration')
    }
  } catch {
    message.error('Failed to delete sync configuration')
  } finally {
    isDeleting.value = false
  }
}

function formatRelativeTime(date: Date): string {
  const now = new Date()
  const diffInSeconds = Math.floor((now.getTime() - new Date(date).getTime()) / 1000)

  if (diffInSeconds < 60) return `${diffInSeconds} seconds ago`
  if (diffInSeconds < 3600) return `${Math.floor(diffInSeconds / 60)} minutes ago`
  if (diffInSeconds < 86400) return `${Math.floor(diffInSeconds / 3600)} hours ago`
  return `${Math.floor(diffInSeconds / 86400)} days ago`
}

function resetForm(): void {
  currentConfig.value = null
  Object.assign(formData, {
    mongoUri: '',
    databaseName: 'kerminal',
    enabled: true,
    autoSync: true,
    syncInterval: 30
  })
  syncStatus.value = { isConnected: false, isLoading: false }
  connectionTestPassed.value = false
  mongoHasMasterPassword.value = false
  showPasswordVerification.value = false
  masterPassword.value = ''
  passwordVerified.value = false
}

async function verifyMasterPassword(): Promise<void> {
  if (!masterPassword.value.trim()) {
    message.error('Please enter the master password')
    return
  }

  isLoading.value = true
  try {
    const verified = (await window.api.invoke(
      'auth:verify-mongo-master-password',
      formData.mongoUri,
      formData.databaseName,
      masterPassword.value
    )) as boolean

    if (verified) {
      passwordVerified.value = true
      message.success('MongoDB master password verified successfully')
    } else {
      message.error('Incorrect MongoDB master password. Please try again.')
    }
  } catch (error) {
    console.error('MongoDB master password verification failed:', error)
    message.error(
      'Failed to verify MongoDB master password. Please check your connection and try again.'
    )
  } finally {
    isLoading.value = false
  }
}

async function loadSyncStatus(): Promise<void> {
  try {
    syncStatus.value = (await window.api.invoke('sync.getStatus')) as SyncStatus
  } catch {
    // Ignore errors during status loading
  }
}

// Status Management
function startStatusRefresh(): void {
  if (statusRefreshInterval.value) return
  statusRefreshInterval.value = setInterval(async () => {
    await loadSyncStatus()
  }, 5000) as unknown as number
}

function stopStatusRefresh(): void {
  if (statusRefreshInterval.value) {
    clearInterval(statusRefreshInterval.value)
    statusRefreshInterval.value = null
  }
}

// Watchers
watch(
  () => props.visible,
  (visible) => {
    if (visible) {
      loadConfig()
      startStatusRefresh()
    } else {
      stopStatusRefresh()
    }
  }
)

watch(
  () => props.refreshTrigger,
  () => {
    if (props.visible) loadConfig()
  }
)

// Lifecycle Hooks
onMounted(() => {
  if (props.visible) {
    loadConfig()
    startStatusRefresh()
  }
})

onUnmounted(() => {
  stopStatusRefresh()
})
</script>
