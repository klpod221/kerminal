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
      <div class="space-y-1">
        <h3 class="text-lg font-medium text-white">Basic Information</h3>

        <!-- Profile Name -->
        <Input
          v-model="form.name"
          label="Profile Name"
          placeholder="My Server"
          :rules="['required']"
          :error-message="validation.fields.value.name?.error.value || undefined"
          @blur="validation.validateField('name')"
        />

        <!-- Description -->
        <Textarea
          v-model="form.description"
          label="Description"
          placeholder="Optional description for this SSH profile"
          :rows="2"
          :error-message="validation.fields.value.description?.error.value || undefined"
          @blur="validation.validateField('description')"
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
      <div class="space-y-1">
        <h3 class="text-lg font-medium text-white">Connection Settings</h3>

        <!-- Host -->
        <Input
          v-model="form.host"
          label="Host"
          placeholder="example.com or 192.168.1.100"
          :rules="['required']"
          :error-message="validation.fields.value.host?.error.value || undefined"
          @blur="validation.validateField('host')"
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
            :error-message="validation.fields.value.port?.error.value || undefined"
            @blur="validation.validateField('port')"
          />
          <Input
            v-model="form.user"
            label="Username"
            placeholder="root or your username"
            :rules="['required']"
            :error-message="validation.fields.value.user?.error.value || undefined"
            @blur="validation.validateField('user')"
          />
        </div>
      </div>

      <!-- Authentication -->
      <div class="space-y-1">
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
            placeholder="Enter password (optional - can prompt when connecting)"
            helper-text="Leave empty to be prompted for password when connecting"
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
              :error-message="validation.fields.value.privateKeyPath?.error.value || undefined"
              @right-icon-click="selectKeyFile"
              @blur="validation.validateField('privateKeyPath')"
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
      <div class="space-y-1">
        <h3 class="text-lg font-medium text-white">Options</h3>

        <div class="flex flex-col space-y-2">
          <!-- Favorite -->
          <Checkbox v-model="form.favorite" label="Mark as favorite" />

          <!-- Keep Alive -->
          <Checkbox v-model="form.keepAlive" label="Keep connection alive" />
        </div>
      </div>
    </form>

    <template #footer>
      <div class="flex justify-between w-full">
        <Button variant="ghost" size="sm" @click="handleClose">Cancel</Button>

        <div class="flex space-x-3">
          <Button
            variant="primary"
            size="sm"
            :disabled="!canSubmit || isSaving"
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
import { useValidation, validationRules } from '../composables/useValidation'
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

// Validation setup
const validation = useValidation()

// Create reactive refs for validation
const nameRef = computed({
  get: () => form.value.name,
  set: (value) => {
    form.value.name = value
  }
})

const descriptionRef = computed({
  get: () => form.value.description,
  set: (value) => {
    form.value.description = value
  }
})

const hostRef = computed({
  get: () => form.value.host,
  set: (value) => {
    form.value.host = value
  }
})

const portRef = computed({
  get: () => form.value.port,
  set: (value) => {
    form.value.port = value
  }
})

const userRef = computed({
  get: () => form.value.user,
  set: (value) => {
    form.value.user = value
  }
})

const privateKeyPathRef = computed({
  get: () => form.value.privateKeyPath,
  set: (value) => {
    form.value.privateKeyPath = value
  }
})

// Register validation fields
validation.registerField('name', nameRef, [
  validationRules.required('Profile name is required'),
  validationRules.profileName()
])

validation.registerField('description', descriptionRef, [
  validationRules.maxLength(200, 'Description must be less than 200 characters')
])

validation.registerField('host', hostRef, [
  validationRules.required('Host is required'),
  validationRules.hostname()
])

validation.registerField('port', portRef, [validationRules.port()])

validation.registerField('user', userRef, [
  validationRules.required('Username is required'),
  validationRules.username()
])

validation.registerField('privateKeyPath', privateKeyPathRef, [validationRules.sshKeyPath()])

// Computed
const isEditing = computed(() => !!props.profile)

const canSubmit = computed(() => {
  return (
    form.value.name &&
    form.value.host &&
    form.value.user &&
    !validation.fields.value.name?.error.value &&
    !validation.fields.value.host?.error.value &&
    !validation.fields.value.user?.error.value &&
    !validation.fields.value.port?.error.value &&
    !validation.fields.value.privateKeyPath?.error.value
  )
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
  validation.resetValidation()
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
    keepAlive: true
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

  // Validate all fields before submitting
  if (!validation.validateAll()) {
    return
  }

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
