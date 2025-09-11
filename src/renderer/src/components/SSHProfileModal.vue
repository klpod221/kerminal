<template>
  <Modal
    :visible="visible"
    :title="isEditing ? 'Edit SSH Profile' : 'Create SSH Profile'"
    :icon="Server"
    icon-background="bg-orange-500/20"
    icon-color="text-orange-400"
    size="lg"
    @close="handleClose"
  >
    <form class="space-y-2" @submit.prevent="handleSubmit">
      <!-- Basic Information -->
      <div>
        <h3 class="text-lg font-medium text-white">Basic Information</h3>

        <!-- Profile Name -->
        <Input
          v-model="form.name"
          label="Profile Name"
          placeholder="My Server"
        />

        <!-- Description -->
        <Textarea
          v-model="form.description"
          label="Description"
          placeholder="Optional description for this SSH profile"
          :rows="2"
        />

        <!-- Group -->
        <Select
          v-model="form.groupId"
          label="Group"
          placeholder="No Group"
          @change="handleGroupChange"
        >
          <option value="">No Group</option>
          <option v-for="group in groups" :key="group.id" :value="group.id">
            {{ group.name }}
          </option>
        </Select>

        <!-- Color -->
        <ColorPicker v-model="form.color" label="Profile Color" />
      </div>

      <!-- Connection Settings -->
      <div>
        <h3 class="text-lg font-medium text-white">Connection Settings</h3>

        <!-- Host -->
        <Input
          v-model="form.host"
          label="Host"
          placeholder="example.com or 192.168.1.100"
        />

        <!-- Port and User -->
        <div class="grid grid-cols-1 md:grid-cols-2 gap-4">
          <Input
            v-model.number="form.port"
            label="Port"
            type="number"
            placeholder="22"
            min="1"
            max="65535"
          />
          <Input
            v-model="form.user"
            label="Username"
            placeholder="root or your username"
          />
        </div>
      </div>

      <!-- Authentication -->
      <div>
        <h3 class="text-lg font-medium text-white">Authentication</h3>

        <!-- Auth Type -->
        <Select v-model="form.authType" label="Authentication Method">
          <option value="password">Password</option>
          <option value="key">SSH Key</option>
          <option value="agent">SSH Agent</option>
        </Select>

        <!-- Password -->
        <div v-if="form.authType === 'password'">
          <Input
            v-model="form.password"
            label="Password"
            :type="showPassword ? 'text' : 'password'"
            placeholder="Enter password"
            :right-icon="showPassword ? EyeOff : Eye"
            @right-icon-click="showPassword = !showPassword"
          />
        </div>

        <!-- SSH Key -->
        <div v-if="form.authType === 'key'" class="space-y-3">
          <div>
            <Input
              v-model="form.privateKeyPath"
              label="Private Key Path"
              placeholder="~/.ssh/id_rsa"
              :right-icon="Folder"
              @right-icon-click="selectKeyFile"
            />
          </div>

          <div>
            <Input
              v-model="form.passphrase"
              label="Key Passphrase"
              :type="showPassphrase ? 'text' : 'password'"
              placeholder="Enter passphrase if key is encrypted"
              :right-icon="showPassphrase ? EyeOff : Eye"
              @right-icon-click="showPassphrase = !showPassphrase"
            />
          </div>
        </div>
      </div>

      <!-- Options -->
      <div>
        <h3 class="text-lg font-medium text-white">Options</h3>

        <div class="flex flex-col space-y-2">
          <!-- Favorite -->
          <Checkbox v-model="form.favorite" label="Mark as favorite" :helper="false" />

          <!-- Keep Alive -->
          <Checkbox v-model="form.keepAlive" label="Keep connection alive" />
        </div>
      </div>

      <!-- Proxy Settings -->
      <div>
        <h3 class="text-lg font-medium text-white">Proxy Settings</h3>
        <ProxySettings v-model:proxy="form.proxy" />
      </div>
    </form>

    <template #footer>
      <div class="flex justify-between w-full">
        <Button variant="ghost" size="sm" @click="handleClose">Cancel</Button>

        <div class="flex space-x-3">
          <Button
            variant="secondary"
            size="sm"
            :disabled="isTestingConnection"
            :loading="isTestingConnection"
            @click="handleTestConnection"
          >
            Test Connection
          </Button>
          <Button
            variant="primary"
            size="sm"
            :disabled="isSaving"
            :loading="isSaving"
            :icon="Save"
            @click="handleSubmit"
          >
            <template v-if="!isSaving">
              {{ isEditing ? 'Update' : 'Create' }}
            </template>
          </Button>
        </div>
      </div>
    </template>
  </Modal>
</template>

<script setup lang="ts">
import { ref, computed, watch } from 'vue'
import { Server, Eye, EyeOff, Folder, Save } from 'lucide-vue-next'
import Modal from './ui/Modal.vue'
import Input from './ui/Input.vue'
import ColorPicker from './ui/ColorPicker.vue'
import Select from './ui/Select.vue'
import Textarea from './ui/Textarea.vue'
import Checkbox from './ui/Checkbox.vue'
import Button from './ui/Button.vue'
import ProxySettings from './ui/ProxySettings.vue'
import type { SSHProfileModalProps } from '../types/modals'
import type { SSHGroup, SSHProfile, SSHProfileWithConfig, SSHProxy } from '../types/ssh'

const props = withDefaults(defineProps<SSHProfileModalProps>(), {
  visible: false,
  profile: null,
  groups: () => [],
  preselectedGroup: null
})

const emit = defineEmits(['update:visible', 'save', 'update', 'close'])

// State
const showPassword = ref(false)
const showPassphrase = ref(false)
const isSaving = ref(false)
const isTestingConnection = ref(false)
const testConnectionResult = ref<{
  success: boolean
  message: string
  duration?: number
} | null>(null)
const testConnectionMessageKey = ref(0)

// Form data
const form = ref({
  name: '',
  description: '',
  host: '',
  port: 22,
  user: '',
  authType: 'password' as 'password' | 'key' | 'agent',
  password: '',
  privateKeyPath: '',
  passphrase: '',
  groupId: '',
  color: '#6b7280',
  favorite: false,
  keepAlive: true,
  proxy: null as SSHProxy | null
})


// Computed
const isEditing = computed(() => !!props.profile)

// Methods
const resetForm = (): void => {
  form.value = {
    name: '',
    description: '',
    host: '',
    port: 22,
    user: '',
    authType: 'password',
    password: '',
    privateKeyPath: '',
    passphrase: '',
    groupId: '',
    color: '#6b7280',
    favorite: false,
    keepAlive: true,
    proxy: null
  }

  // Clear test results
  clearTestResult()
}

const loadProfile = (profile: SSHProfileWithConfig): void => {
  form.value = {
    name: profile.name,
    description: profile.description || '',
    host: profile.host,
    port: profile.port || 22,
    user: profile.user,
    authType: 'password', // Default to password since keyPath maps to authType key
    password: profile.password || '',
    privateKeyPath: profile.keyPath || '',
    passphrase: '', // Not stored in profile for security
    groupId: profile.groupId || '',
    color: profile.color || '#6b7280',
    favorite: profile.favorite || false,
    keepAlive: true,
    proxy: profile.proxy || null
  }
}

const selectKeyFile = async (): Promise<void> => {
  try {
    const selectedPath = (await window.api.invoke('dialog.selectFile')) as string | null
    if (selectedPath) {
      form.value.privateKeyPath = selectedPath
    }
  } catch (error) {
    console.error('Failed to select key file:', error)
  }
}

const handleGroupChange = (): void => {
  if (!form.value.groupId || isEditing.value) return

  const selectedGroup = props.groups.find((group) => group.id === form.value.groupId)
  if (!selectedGroup) return

  applyGroupDefaults(selectedGroup)
}

const applyGroupDefaults = (group: SSHGroup): void => {
  // Auto-fill default values from group
  if (group.defaultHost && !form.value.host) {
    form.value.host = group.defaultHost
  }
  if (group.defaultPort && !form.value.port) {
    form.value.port = group.defaultPort
  }
  if (group.defaultUser && !form.value.user) {
    form.value.user = group.defaultUser
  }
  if (group.defaultKeyPath && !form.value.privateKeyPath) {
    form.value.privateKeyPath = group.defaultKeyPath
    form.value.authType = 'key'
  }
  if (group.defaultPassword && !form.value.password && form.value.authType === 'password') {
    form.value.password = group.defaultPassword
  }
  if (group.color && (!form.value.color || form.value.color === '#6b7280')) {
    form.value.color = group.color
  }
  if (group.defaultProxy && !form.value.proxy) {
    form.value.proxy = group.defaultProxy
  }
}

const handleSubmit = async (): Promise<void> => {
  try {
    isSaving.value = true

    // Clean proxy object to ensure it's serializable
    const cleanProxy = form.value.proxy
      ? {
          type: form.value.proxy.type,
          host: form.value.proxy.host,
          port: form.value.proxy.port,
          ...(form.value.proxy.username && { username: form.value.proxy.username }),
          ...(form.value.proxy.password && { password: form.value.proxy.password }),
          ...(form.value.proxy.jumpHost && { jumpHost: form.value.proxy.jumpHost }),
          ...(form.value.proxy.jumpPort && { jumpPort: form.value.proxy.jumpPort }),
          ...(form.value.proxy.jumpUser && { jumpUser: form.value.proxy.jumpUser }),
          ...(form.value.proxy.jumpKeyPath && {
            jumpKeyPath: form.value.proxy.jumpKeyPath
          }),
          ...(form.value.proxy.jumpPassword && {
            jumpPassword: form.value.proxy.jumpPassword
          })
        }
      : undefined

    const profileData: Partial<SSHProfile> = {
      name: form.value.name,
      description: form.value.description || undefined,
      host: form.value.host,
      port: form.value.port,
      user: form.value.user,
      password: form.value.password || undefined,
      keyPath: form.value.authType === 'key' ? form.value.privateKeyPath || undefined : undefined,
      groupId: form.value.groupId || undefined,
      color: form.value.color,
      favorite: form.value.favorite,
      proxy: cleanProxy
    }

    if (isEditing.value && props.profile) {
      emit('update', props.profile.id, profileData)
    } else {
      emit('save', profileData)
    }

    handleClose()
  } catch (error) {
    console.error('Failed to save profile:', error)
  } finally {
    isSaving.value = false
  }
}

const handleTestConnection = async (): Promise<void> => {
  try {
    isTestingConnection.value = true
    testConnectionResult.value = null

    // Create form data for testing
    const formData = {
      host: form.value.host,
      port: form.value.port,
      user: form.value.user,
      authType: form.value.authType,
      password: form.value.password,
      privateKeyPath: form.value.privateKeyPath,
      proxy: form.value.proxy
    }

    // Create resolved config from form data
    const config = await window.api.invoke('ssh.createResolvedConfigFromFormData', formData)

    // Test the connection
    const result = (await window.api.invoke('ssh.testConnection', config)) as {
      success: boolean
      message: string
      duration?: number
      error?: string
    }
    testConnectionResult.value = result

    // Increment key to force re-render of Message component
    testConnectionMessageKey.value++
  } catch (error) {
    console.error('Failed to test connection:', error)
    testConnectionResult.value = {
      success: false,
      message: error instanceof Error ? error.message : 'Unknown error occurred'
    }
    testConnectionMessageKey.value++
  } finally {
    isTestingConnection.value = false
  }
}

const handleClose = (): void => {
  emit('update:visible', false)
  emit('close')
  resetForm()
}

const getTestConnectionMessage = (result: { message: string; duration?: number }): string => {
  let message = result.message
  if (result.duration) {
    message += ` (${result.duration}ms)`
  }
  return message
}

const clearTestResult = (): void => {
  testConnectionResult.value = null
}

// Watch for profile changes
watch(
  () => props.profile,
  (profile) => {
    if (profile) {
      loadProfile(profile)
    } else {
      resetForm()
    }
  },
  { immediate: true }
)

// Watch for preselected group changes
watch(
  () => props.preselectedGroup,
  (group) => {
    if (group && !isEditing.value) {
      form.value.groupId = group.id
      // Trigger auto-fill from group defaults
      handleGroupChange()
    }
  },
  { immediate: true }
)

// Watch for visibility changes
watch(
  () => props.visible,
  (visible) => {
    if (!visible) {
      showPassword.value = false
      showPassphrase.value = false
      clearTestResult()
    }
  }
)

// Clear test results when connection settings change
watch(
  () => [form.value.host, form.value.port, form.value.user, form.value.authType],
  () => {
    clearTestResult()
  }
)
</script>
