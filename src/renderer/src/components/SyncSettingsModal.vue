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
        <div class="status-card" :class="statusClass">
          <div class="status-info">
            <span class="status-label">{{ statusLabel }}</span>
            <span v-if="syncStatus.lastSync" class="last-sync">
              Last sync: {{ formatDate(syncStatus.lastSync) }}
            </span>
            <span v-if="syncStatus.lastError" class="error-message">
              {{ syncStatus.lastError }}
            </span>
          </div>
          <div class="status-actions">
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

      <!-- Configuration Form -->
      <div class="space-y-1">
        <h3 class="text-lg font-medium text-white">MongoDB Configuration</h3>
        <form @submit.prevent="handleSave">
          <Input
            v-model="formData.mongoUri"
            label="MongoDB URI"
            class="mb-4"
            placeholder="mongodb://username:password@hostname:port"
            :disabled="isLoading"
            required
            helper-text="Include credentials in the URI. Example: mongodb://user:pass@cluster.mongodb.net"
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

          <div class="flex justify-between space-x-3 mt-2">
            <Button
              type="button"
              variant="secondary"
              :loading="isTestingConnection"
              @click="testConnection"
            >
              Test Connection
            </Button>
            <Button type="submit" variant="primary" :loading="isLoading" :icon="Save">
              Save
            </Button>
          </div>
        </form>
      </div>

      <!-- Migration Section -->
      <div v-if="hasExistingData && !syncStatus.isConnected" class="space-y-1">
        <h3 class="text-lg font-medium text-white">Data Migration</h3>
        <p class="text-sm text-gray-400 mb-4">
          You have existing local data. When you enable sync, your data will be migrated to MongoDB.
        </p>
        <Button variant="secondary" :loading="isMigrating" @click="migrateData">
          Migrate Data Now
        </Button>
      </div>

      <!-- Danger Zone -->
      <div v-if="currentConfig" class="space-y-1">
        <h3 class="text-lg font-medium text-red-400">Danger Zone</h3>
        <p class="text-sm text-gray-400 mb-4">
          This will permanently delete your sync configuration and disable synchronization.
        </p>
        <Button variant="danger" :loading="isDeleting" @click="deleteSyncConfig">
          Delete Sync Configuration
        </Button>
      </div>
    </div>
  </Modal>
</template>

<script setup lang="ts">
import { ref, reactive, computed, onMounted, watch } from 'vue'
import { Database, Save } from 'lucide-vue-next'
import Modal from './ui/Modal.vue'
import Button from './ui/Button.vue'
import Input from './ui/Input.vue'
import { message } from '../utils/message'
import type { SyncConfig, SyncStatus } from '../types/sync'

interface Props {
  visible: boolean
}

const props = defineProps<Props>()

const emit = defineEmits<{
  close: []
  configUpdated: [config: SyncConfig | null]
}>()

// State
const isLoading = ref(false)
const isTestingConnection = ref(false)
const isMigrating = ref(false)
const isDeleting = ref(false)
const hasExistingData = ref(false)
const currentConfig = ref<SyncConfig | null>(null)
const syncStatus = ref<SyncStatus>({
  isConnected: false,
  isLoading: false
})

// Form data
const formData = reactive<SyncConfig>({
  mongoUri: '',
  databaseName: 'kerminal',
  enabled: true, // Always enabled
  autoSync: true, // Always auto sync
  syncInterval: 30 // seconds instead of minutes
})

// Computed
const statusClass = computed(() => {
  if (syncStatus.value.isLoading) return 'loading'
  if (syncStatus.value.isConnected) return 'connected'
  if (syncStatus.value.lastError) return 'error'
  return 'disconnected'
})

const statusLabel = computed(() => {
  if (syncStatus.value.isLoading) return 'Syncing...'
  if (syncStatus.value.isConnected) return 'Connected'
  if (syncStatus.value.lastError) return 'Error'
  return 'Disconnected'
})

// Methods
async function loadConfig(): Promise<void> {
  try {
    const config = (await window.api.invoke('sync.getConfig')) as SyncConfig | null
    if (config) {
      currentConfig.value = config
      Object.assign(formData, config)
    }

    // Load sync status
    syncStatus.value = (await window.api.invoke('sync.getStatus')) as SyncStatus
  } catch (error) {
    console.error('Failed to load sync config:', error)
  }
}

async function testConnection(): Promise<void> {
  if (!formData.mongoUri || !formData.databaseName) {
    message.error('Please fill in MongoDB URI and database name')
    return
  }

  isTestingConnection.value = true
  try {
    const success = await window.api.invoke(
      'sync.testConnection',
      formData.mongoUri,
      formData.databaseName
    )
    if (success) {
      message.success('Connection test successful!')
    } else {
      message.error('Failed to connect to MongoDB')
    }
  } catch (error) {
    message.error('Connection test failed')
    console.error('Connection test error:', error)
  } finally {
    isTestingConnection.value = false
  }
}

async function handleSave(): Promise<void> {
  isLoading.value = true
  try {
    // Create a plain object from reactive formData
    const configData = {
      mongoUri: formData.mongoUri,
      databaseName: formData.databaseName,
      enabled: formData.enabled,
      autoSync: formData.autoSync,
      syncInterval: formData.syncInterval
    }

    const success = await window.api.invoke('sync.setup', configData)
    if (success) {
      message.success('Sync configuration saved successfully!')
      currentConfig.value = { ...configData }

      // Wait a bit for sync service to fully initialize before getting status
      await new Promise((resolve) => setTimeout(resolve, 100))
      syncStatus.value = (await window.api.invoke('sync.getStatus')) as SyncStatus

      emit('configUpdated', currentConfig.value)
    } else {
      message.error('Failed to save sync configuration')
    }
  } catch (error) {
    message.error('Failed to save sync configuration')
    console.error('Save config error:', error)
  } finally {
    isLoading.value = false
  }
}

async function performSync(): Promise<void> {
  try {
    syncStatus.value.isLoading = true
    const success = await window.api.invoke('sync.performSync')
    if (success) {
      message.success('Sync completed successfully!')
      syncStatus.value = (await window.api.invoke('sync.getStatus')) as SyncStatus
    } else {
      message.error('Sync failed')
    }
  } catch (error) {
    message.error('Sync failed')
    console.error('Sync error:', error)
  }
}

async function migrateData(): Promise<void> {
  isMigrating.value = true
  try {
    const success = await window.api.invoke('sync.migrateData')
    if (success) {
      message.success('Data migration completed successfully!')
      hasExistingData.value = false
      syncStatus.value = (await window.api.invoke('sync.getStatus')) as SyncStatus
    } else {
      message.error('Data migration failed')
    }
  } catch (error) {
    message.error('Data migration failed')
    console.error('Migration error:', error)
  } finally {
    isMigrating.value = false
  }
}

async function deleteSyncConfig(): Promise<void> {
  const confirmed = confirm(
    'Are you sure you want to delete the sync configuration? This action cannot be undone.'
  )
  if (!confirmed) return

  isDeleting.value = true
  try {
    const success = await window.api.invoke('sync.deleteConfig')
    if (success) {
      message.success('Sync configuration deleted successfully!')
      currentConfig.value = null
      Object.assign(formData, {
        mongoUri: '',
        databaseName: 'kerminal',
        enabled: true,
        autoSync: true,
        syncInterval: 30
      })
      syncStatus.value = {
        isConnected: false,
        isLoading: false
      }
      emit('configUpdated', null)
    } else {
      message.error('Failed to delete sync configuration')
    }
  } catch (error) {
    message.error('Failed to delete sync configuration')
    console.error('Delete config error:', error)
  } finally {
    isDeleting.value = false
  }
}

function formatDate(date: Date | string): string {
  const d = new Date(date)
  return d.toLocaleString()
}

// Watchers
watch(
  () => props.visible,
  (visible) => {
    if (visible) {
      loadConfig()
    }
  }
)

// Lifecycle
onMounted(() => {
  if (props.visible) {
    loadConfig()
  }
})
</script>

<style scoped>
.status-card {
  display: flex;
  align-items: center;
  justify-content: space-between;
  padding: 1rem;
  border-radius: 0.5rem;
  border: 1px solid rgb(75 85 99);
  background: rgb(31 41 55);
}

.status-card.connected {
  background: rgb(6 78 59);
  border-color: rgb(34 197 94);
}

.status-card.error {
  background: rgb(127 29 29);
  border-color: rgb(239 68 68);
}

.status-card.loading {
  background: rgb(120 53 15);
  border-color: rgb(245 158 11);
}

.status-info {
  display: flex;
  flex-direction: column;
  gap: 0.25rem;
}

.status-label {
  font-weight: 500;
  color: white;
}

.last-sync {
  font-size: 0.875rem;
  color: rgb(156 163 175);
}

.error-message {
  font-size: 0.875rem;
  color: rgb(239 68 68);
}
</style>
