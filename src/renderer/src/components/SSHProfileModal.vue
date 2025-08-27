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
    <form class="space-y-6" @submit.prevent="handleSubmit">
      <!-- Basic Information -->
      <div class="space-y-4">
        <h3 class="text-lg font-medium text-white">Basic Information</h3>

        <!-- Profile Name -->
        <div>
          <label for="profile-name" class="block text-sm font-medium text-gray-300 mb-2"
            >Profile Name *</label
          >
          <input
            id="profile-name"
            v-model="form.name"
            type="text"
            required
            placeholder="My Server"
            class="w-full px-3 py-2 bg-[#2a2a2a] border border-gray-600 rounded-lg text-white placeholder-gray-400 focus:outline-none focus:border-orange-500 focus:ring-1 focus:ring-orange-500"
          />
        </div>

        <!-- Description -->
        <div>
          <label for="profile-description" class="block text-sm font-medium text-gray-300 mb-2"
            >Description</label
          >
          <textarea
            id="profile-description"
            v-model="form.description"
            rows="2"
            placeholder="Optional description for this SSH profile"
            class="w-full px-3 py-2 bg-[#2a2a2a] border border-gray-600 rounded-lg text-white placeholder-gray-400 focus:outline-none focus:border-orange-500 focus:ring-1 focus:ring-orange-500 resize-none"
          ></textarea>
        </div>

        <!-- Group -->
        <div>
          <label for="profile-group" class="block text-sm font-medium text-gray-300 mb-2"
            >Group</label
          >
          <select
            id="profile-group"
            v-model="form.groupId"
            class="w-full px-3 py-2 bg-[#2a2a2a] border border-gray-600 rounded-lg text-white focus:outline-none focus:border-orange-500 focus:ring-1 focus:ring-orange-500"
            @change="handleGroupChange"
          >
            <option value="">No Group</option>
            <option v-for="group in groups" :key="group.id" :value="group.id">
              {{ group.name }}
            </option>
          </select>
        </div>

        <!-- Color -->
        <div>
          <label for="profile-color" class="block text-sm font-medium text-gray-300 mb-2"
            >Color</label
          >
          <div class="flex items-center space-x-3">
            <input
              id="profile-color"
              v-model="form.color"
              type="color"
              class="w-12 h-10 bg-[#2a2a2a] border border-gray-600 rounded-lg cursor-pointer"
            />
            <input
              v-model="form.color"
              type="text"
              placeholder="#6b7280"
              class="flex-1 px-3 py-2 bg-[#2a2a2a] border border-gray-600 rounded-lg text-white placeholder-gray-400 focus:outline-none focus:border-orange-500 focus:ring-1 focus:ring-orange-500"
            />
          </div>
        </div>
      </div>

      <!-- Connection Settings -->
      <div class="space-y-4">
        <h3 class="text-lg font-medium text-white">Connection Settings</h3>

        <!-- Host -->
        <div>
          <label for="profile-host" class="block text-sm font-medium text-gray-300 mb-2"
            >Host *</label
          >
          <input
            id="profile-host"
            v-model="form.host"
            type="text"
            required
            placeholder="example.com or 192.168.1.100"
            class="w-full px-3 py-2 bg-[#2a2a2a] border border-gray-600 rounded-lg text-white placeholder-gray-400 focus:outline-none focus:border-orange-500 focus:ring-1 focus:ring-orange-500"
          />
        </div>

        <!-- Port and User -->
        <div class="grid grid-cols-1 md:grid-cols-2 gap-4">
          <div>
            <label for="profile-port" class="block text-sm font-medium text-gray-300 mb-2"
              >Port</label
            >
            <input
              id="profile-port"
              v-model.number="form.port"
              type="number"
              min="1"
              max="65535"
              placeholder="22"
              class="w-full px-3 py-2 bg-[#2a2a2a] border border-gray-600 rounded-lg text-white placeholder-gray-400 focus:outline-none focus:border-orange-500 focus:ring-1 focus:ring-orange-500"
            />
          </div>
          <div>
            <label for="profile-username" class="block text-sm font-medium text-gray-300 mb-2"
              >Username *</label
            >
            <input
              id="profile-username"
              v-model="form.user"
              type="text"
              required
              placeholder="root or your username"
              class="w-full px-3 py-2 bg-[#2a2a2a] border border-gray-600 rounded-lg text-white placeholder-gray-400 focus:outline-none focus:border-orange-500 focus:ring-1 focus:ring-orange-500"
            />
          </div>
        </div>
      </div>

      <!-- Authentication -->
      <div class="space-y-4">
        <h3 class="text-lg font-medium text-white">Authentication</h3>

        <!-- Auth Type -->
        <div>
          <label for="profile-auth-type" class="block text-sm font-medium text-gray-300 mb-2"
            >Authentication Method</label
          >
          <select
            id="profile-auth-type"
            v-model="form.authType"
            class="w-full px-3 py-2 bg-[#2a2a2a] border border-gray-600 rounded-lg text-white focus:outline-none focus:border-orange-500 focus:ring-1 focus:ring-orange-500"
          >
            <option value="password">Password</option>
            <option value="key">SSH Key</option>
            <option value="agent">SSH Agent</option>
          </select>
        </div>

        <!-- Password -->
        <div v-if="form.authType === 'password'">
          <label for="profile-password" class="block text-sm font-medium text-gray-300 mb-2"
            >Password</label
          >
          <div class="relative">
            <input
              id="profile-password"
              v-model="form.password"
              :type="showPassword ? 'text' : 'password'"
              placeholder="Enter password (optional - can prompt when connecting)"
              class="w-full px-3 py-2 pr-10 bg-[#2a2a2a] border border-gray-600 rounded-lg text-white placeholder-gray-400 focus:outline-none focus:border-orange-500 focus:ring-1 focus:ring-orange-500"
            />
            <button
              type="button"
              class="absolute right-3 top-1/2 transform -translate-y-1/2 text-gray-400 hover:text-white"
              @click="showPassword = !showPassword"
            >
              <Eye v-if="!showPassword" :size="16" />
              <EyeOff v-else :size="16" />
            </button>
          </div>
          <p class="text-xs text-gray-500 mt-1">
            Leave empty to be prompted for password when connecting
          </p>
        </div>

        <!-- SSH Key -->
        <div v-if="form.authType === 'key'" class="space-y-3">
          <div>
            <label for="profile-key-path" class="block text-sm font-medium text-gray-300 mb-2"
              >Private Key Path</label
            >
            <div class="flex space-x-2">
              <input
                id="profile-key-path"
                v-model="form.privateKeyPath"
                type="text"
                placeholder="~/.ssh/id_rsa"
                class="flex-1 px-3 py-2 bg-[#2a2a2a] border border-gray-600 rounded-lg text-white placeholder-gray-400 focus:outline-none focus:border-orange-500 focus:ring-1 focus:ring-orange-500"
              />
              <button
                type="button"
                class="px-3 py-2 bg-gray-600 hover:bg-gray-700 text-white rounded-lg transition-colors"
                @click="selectKeyFile"
              >
                <Folder :size="16" />
              </button>
            </div>
          </div>

          <div>
            <label for="profile-passphrase" class="block text-sm font-medium text-gray-300 mb-2"
              >Key Passphrase</label
            >
            <div class="relative">
              <input
                id="profile-passphrase"
                v-model="form.passphrase"
                :type="showPassphrase ? 'text' : 'password'"
                placeholder="Enter passphrase if key is encrypted"
                class="w-full px-3 py-2 pr-10 bg-[#2a2a2a] border border-gray-600 rounded-lg text-white placeholder-gray-400 focus:outline-none focus:border-orange-500 focus:ring-1 focus:ring-orange-500"
              />
              <button
                type="button"
                class="absolute right-3 top-1/2 transform -translate-y-1/2 text-gray-400 hover:text-white"
                @click="showPassphrase = !showPassphrase"
              >
                <Eye v-if="!showPassphrase" :size="16" />
                <EyeOff v-else :size="16" />
              </button>
            </div>
          </div>
        </div>
      </div>

      <!-- Options -->
      <div class="space-y-4">
        <h3 class="text-lg font-medium text-white">Options</h3>

        <div class="grid grid-cols-1 md:grid-cols-2 gap-4">
          <!-- Favorite -->
          <label class="flex items-center space-x-2 cursor-pointer">
            <input
              v-model="form.favorite"
              type="checkbox"
              class="w-4 h-4 text-orange-500 bg-gray-600 border-gray-500 rounded focus:ring-orange-500 focus:ring-2"
            />
            <span class="text-sm text-gray-300">Mark as favorite</span>
          </label>

          <!-- Keep Alive -->
          <label class="flex items-center space-x-2 cursor-pointer">
            <input
              v-model="form.keepAlive"
              type="checkbox"
              class="w-4 h-4 text-orange-500 bg-gray-600 border-gray-500 rounded focus:ring-orange-500 focus:ring-2"
            />
            <span class="text-sm text-gray-300">Keep connection alive</span>
          </label>
        </div>
      </div>
    </form>

    <template #footer>
      <div class="flex justify-between">
        <button
          type="button"
          class="px-4 py-2 text-gray-400 hover:text-white transition-colors rounded-lg"
          @click="handleClose"
        >
          Cancel
        </button>

        <div class="flex space-x-3">
          <button
            type="button"
            :disabled="!canSubmit || isSaving"
            class="px-4 py-2 bg-orange-600 hover:bg-orange-700 text-white font-medium rounded-lg transition-colors"
            @click="handleSubmit"
          >
            <div v-if="isSaving" class="flex items-center space-x-2">
              <div
                class="animate-spin rounded-full h-4 w-4 border-2 border-white border-t-transparent"
              ></div>
              <span>Saving...</span>
            </div>
            <div v-else class="flex items-center space-x-2">
              <Save :size="16" />
              <span>{{ isEditing ? 'Update' : 'Create' }}</span>
            </div>
          </button>
        </div>
      </div>
    </template>
  </Modal>
</template>

<script setup lang="ts">
import { ref, computed, watch } from 'vue'
import { Server, Eye, EyeOff, Folder, Save } from 'lucide-vue-next'
import Modal from './ui/Modal.vue'
import type { SSHGroup, SSHProfile, SSHProfileWithConfig } from '../types/ssh'

interface Props {
  visible?: boolean
  profile?: SSHProfileWithConfig | null
  groups?: SSHGroup[]
  preselectedGroup?: SSHGroup | null
}

const props = withDefaults(defineProps<Props>(), {
  visible: false,
  profile: null,
  groups: () => [],
  preselectedGroup: null
})

const emit = defineEmits<{
  'update:visible': [visible: boolean]
  save: [profile: Partial<SSHProfile>]
  update: [id: string, updates: Partial<SSHProfile>]
  close: []
}>()

// State
const showPassword = ref(false)
const showPassphrase = ref(false)
const isSaving = ref(false)

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
  keepAlive: true
})

// Computed
const isEditing = computed(() => !!props.profile)

const canSubmit = computed(() => {
  return form.value.name && form.value.host && form.value.user
})

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
    keepAlive: true
  }
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
    keepAlive: true // Default to true
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

  // Auto-fill default values from group
  if (selectedGroup.defaultHost && !form.value.host) {
    form.value.host = selectedGroup.defaultHost
  }
  if (selectedGroup.defaultPort && !form.value.port) {
    form.value.port = selectedGroup.defaultPort
  }
  if (selectedGroup.defaultUser && !form.value.user) {
    form.value.user = selectedGroup.defaultUser
  }
  if (selectedGroup.defaultKeyPath && !form.value.privateKeyPath) {
    form.value.privateKeyPath = selectedGroup.defaultKeyPath
    form.value.authType = 'key'
  }
  if (selectedGroup.defaultPassword && !form.value.password && form.value.authType === 'password') {
    form.value.password = selectedGroup.defaultPassword
  }
  if (selectedGroup.color && (!form.value.color || form.value.color === '#6b7280')) {
    form.value.color = selectedGroup.color
  }
}

const handleSubmit = async (): Promise<void> => {
  if (!canSubmit.value || isSaving.value) return

  try {
    isSaving.value = true

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
      favorite: form.value.favorite
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

const handleClose = (): void => {
  emit('update:visible', false)
  emit('close')
  resetForm()
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
    }
  }
)
</script>
