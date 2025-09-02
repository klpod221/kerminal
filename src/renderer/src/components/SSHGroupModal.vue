<template>
  <Modal :visible="show" title="SSH Group" @close="closeModal">
    <!-- Group Name -->
    <Input
      v-model="groupData.name"
      label="Group Name"
      placeholder="Enter group name"
      required
      :error-message="nameError"
      @blur="validation.validateField('name')"
    />

    <!-- Description -->
    <Textarea
      v-model="groupData.description"
      label="Description"
      placeholder="Enter group description (optional)"
      :rows="3"
      :error-message="descriptionError"
      @blur="validation.validateField('description')"
    />

    <!-- Default Settings -->
    <div class="border-t border-gray-600 pt-4">
      <h3 class="text-lg font-medium text-white mb-3">Default Settings</h3>
      <p class="text-sm text-gray-400 mb-4">
        These settings will be used as defaults for new profiles in this group
      </p>

      <!-- Default User -->
      <Input
        v-model="groupData.defaultUser"
        label="Default User"
        placeholder="e.g., root, admin"
        :error-message="defaultUserError"
        @blur="validation.validateField('defaultUser')"
      />

      <!-- Default Host -->
      <Input
        v-model="groupData.defaultHost"
        label="Default Host"
        placeholder="e.g., server.example.com"
        :error-message="defaultHostError"
        @blur="validation.validateField('defaultHost')"
      />

      <!-- Default Port -->
      <Input
        v-model.number="groupData.defaultPort"
        label="Default Port"
        type="number"
        placeholder="22"
        min="1"
        max="65535"
        :error-message="defaultPortError"
        @blur="validation.validateField('defaultPort')"
      />

      <!-- Default SSH Key Path -->
      <Input
        v-model="groupData.defaultKeyPath"
        label="Default SSH Key Path"
        placeholder="Path to SSH private key"
        :right-icon="Folder"
        :error-message="defaultKeyPathError"
        @right-icon-click="selectKeyFile"
        @blur="validation.validateField('defaultKeyPath')"
      />

      <!-- Default Password -->
      <Input
        v-model="groupData.defaultPassword"
        label="Default Password"
        :type="showPassword ? 'text' : 'password'"
        placeholder="Default password for this group"
        :right-icon="showPassword ? EyeOff : Eye"
        @right-icon-click="showPassword = !showPassword"
      />

      <ColorPicker v-model="groupData.color" label="Group Color" />
    </div>

    <template #footer>
      <div class="flex justify-end space-x-3">
        <Button variant="secondary" @click="closeModal"> Cancel </Button>
        <Button variant="primary" :disabled="!canSubmit" :icon="Save" @click="saveGroup">
          {{ isEditing ? 'Update' : 'Create' }}
        </Button>
      </div>
    </template>
  </Modal>
</template>

<script setup lang="ts">
import { ref, watch, computed } from 'vue'
import { Eye, EyeOff, Folder, Save } from 'lucide-vue-next'
import Modal from './ui/Modal.vue'
import Input from './ui/Input.vue'
import ColorPicker from './ui/ColorPicker.vue'
import Textarea from './ui/Textarea.vue'
import Button from './ui/Button.vue'
import { useValidation, validationRules } from '../composables/useValidation'
import type { SSHGroup } from '../types/ssh'

interface Props {
  show: boolean
  group?: SSHGroup | null
}

const props = defineProps<Props>()

// Debug log
watch(
  () => props.show,
  (newShow) => {
    console.log('SSHGroupModal show prop changed:', newShow)
  }
)

const emit = defineEmits(['close', 'save', 'update'])

const showPassword = ref(false)

// Form data
const groupData = ref({
  name: '',
  description: '',
  defaultUser: '',
  defaultHost: '',
  defaultPort: undefined as number | undefined,
  defaultKeyPath: '',
  defaultPassword: '',
  color: '#6b7280'
})

// Validation setup
const validation = useValidation()

// Create reactive refs for validation
const nameRef = computed({
  get: () => groupData.value.name,
  set: (value) => {
    groupData.value.name = value
  }
})

const descriptionRef = computed({
  get: () => groupData.value.description,
  set: (value) => {
    groupData.value.description = value
  }
})

const defaultUserRef = computed({
  get: () => groupData.value.defaultUser,
  set: (value) => {
    groupData.value.defaultUser = value
  }
})

const defaultHostRef = computed({
  get: () => groupData.value.defaultHost,
  set: (value) => {
    groupData.value.defaultHost = value
  }
})

const defaultPortRef = computed({
  get: () => groupData.value.defaultPort,
  set: (value) => {
    groupData.value.defaultPort = value
  }
})

const defaultKeyPathRef = computed({
  get: () => groupData.value.defaultKeyPath,
  set: (value) => {
    groupData.value.defaultKeyPath = value
  }
})

// Register validation fields
validation.registerField('name', nameRef, [
  validationRules.required('Group name is required'),
  validationRules.groupName()
])

validation.registerField('description', descriptionRef, [
  validationRules.maxLength(200, 'Description must be less than 200 characters')
])

validation.registerField('defaultUser', defaultUserRef, [
  validationRules.custom((value: string | number | undefined) => {
    if (!value) return true // Optional field
    const str = String(value).trim()
    const usernameRegex = /^[a-zA-Z0-9._-]+$/
    return usernameRegex.test(str) && str.length <= 32
  }, 'Please enter a valid username')
])

validation.registerField('defaultHost', defaultHostRef, [validationRules.hostname()])

validation.registerField('defaultPort', defaultPortRef, [validationRules.port()])

validation.registerField('defaultKeyPath', defaultKeyPathRef, [validationRules.sshKeyPath()])

const isEditing = computed(() => !!props.group)

// Error message computed properties for safe access
const nameError = computed(() => validation.fields.value?.name?.error?.value || undefined)
const descriptionError = computed(
  () => validation.fields.value?.description?.error?.value || undefined
)
const defaultUserError = computed(
  () => validation.fields.value?.defaultUser?.error?.value || undefined
)
const defaultHostError = computed(
  () => validation.fields.value?.defaultHost?.error?.value || undefined
)
const defaultPortError = computed(
  () => validation.fields.value?.defaultPort?.error?.value || undefined
)
const defaultKeyPathError = computed(
  () => validation.fields.value?.defaultKeyPath?.error?.value || undefined
)

// Check if form can be submitted
const canSubmit = computed(() => {
  return groupData.value.name.trim() && !nameError.value
})

/**
 * Reset form to default values
 */
const resetForm = (): void => {
  groupData.value = {
    name: '',
    description: '',
    defaultUser: '',
    defaultHost: '',
    defaultPort: undefined,
    defaultKeyPath: '',
    defaultPassword: '',
    color: '#6b7280'
  }
  showPassword.value = false

  // Only reset validation if fields are properly initialized
  if (validation.fields.value && Object.keys(validation.fields.value).length > 0) {
    validation.resetValidation()
  }
}

// Watch for group prop changes (editing mode)
watch(
  () => props.group,
  (newGroup) => {
    if (newGroup) {
      groupData.value = {
        name: newGroup.name,
        description: newGroup.description || '',
        defaultUser: newGroup.defaultUser || '',
        defaultHost: newGroup.defaultHost || '',
        defaultPort: newGroup.defaultPort,
        defaultKeyPath: newGroup.defaultKeyPath || '',
        defaultPassword: newGroup.defaultPassword || '',
        color: newGroup.color || '#6b7280'
      }
    } else {
      resetForm()
    }
  },
  { immediate: true }
)

/**
 * Close modal and reset form
 */
const closeModal = (): void => {
  resetForm()
  emit('close')
}

/**
 * Select SSH key file
 */
const selectKeyFile = async (): Promise<void> => {
  try {
    const selectedPath = (await window.api.invoke('dialog.selectFile')) as string | null
    if (selectedPath) {
      groupData.value.defaultKeyPath = selectedPath
    }
  } catch (error) {
    console.error('Failed to select key file:', error)
  }
}

/**
 * Save or update group
 */
const saveGroup = (): void => {
  // Validate all fields before saving
  if (!validation.validateAll()) {
    return
  }

  if (!groupData.value.name.trim()) {
    return
  }

  const data = {
    name: groupData.value.name.trim(),
    description: groupData.value.description.trim() || undefined,
    defaultUser: groupData.value.defaultUser.trim() || undefined,
    defaultHost: groupData.value.defaultHost.trim() || undefined,
    defaultPort: groupData.value.defaultPort || undefined,
    defaultKeyPath: groupData.value.defaultKeyPath.trim() || undefined,
    defaultPassword: groupData.value.defaultPassword.trim() || undefined,
    color: groupData.value.color
  }

  if (isEditing.value && props.group) {
    emit('update', props.group.id, data)
  } else {
    emit('save', data)
  }

  closeModal()
}
</script>
