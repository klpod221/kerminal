<template>
  <Modal :visible="show" title="SSH Group" @close="closeModal">
    <div class="space-y-4">
      <!-- Group Name -->
      <Input v-model="groupData.name" label="Group Name" placeholder="Enter group name" required />

      <!-- Description -->
      <Textarea
        v-model="groupData.description"
        label="Description"
        placeholder="Enter group description (optional)"
        :rows="3"
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
        />

        <!-- Default Host -->
        <Input
          v-model="groupData.defaultHost"
          label="Default Host"
          placeholder="e.g., server.example.com"
        />

        <!-- Default Port -->
        <Input
          v-model.number="groupData.defaultPort"
          label="Default Port"
          type="number"
          placeholder="22"
          min="1"
          max="65535"
        />

        <!-- Default SSH Key Path -->
        <Input
          v-model="groupData.defaultKeyPath"
          label="Default SSH Key Path"
          placeholder="Path to SSH private key"
          :right-icon="Folder"
          @right-icon-click="selectKeyFile"
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

        <!-- Group Color -->
        <div>
          <label for="group-color" class="block text-sm font-medium text-gray-300 mb-2"
            >Group Color</label
          >
          <div class="flex items-center space-x-3">
            <input
              id="group-color"
              v-model="groupData.color"
              type="color"
              class="w-12 h-10 bg-[#2a2a2a] border border-gray-600 rounded-lg cursor-pointer"
            />
            <Input v-model="groupData.color" placeholder="#6b7280" />
          </div>
        </div>
      </div>
    </div>

    <template #footer>
      <div class="flex justify-end space-x-3">
        <Button variant="secondary" @click="closeModal"> Cancel </Button>
        <Button variant="primary" :disabled="!groupData.name.trim()" @click="saveGroup">
          {{ isEditing ? 'Update' : 'Create' }} Group
        </Button>
      </div>
    </template>
  </Modal>
</template>

<script setup lang="ts">
import { ref, watch, computed } from 'vue'
import { Eye, EyeOff, Folder } from 'lucide-vue-next'
import Modal from './ui/Modal.vue'
import Input from './ui/Input.vue'
import Textarea from './ui/Textarea.vue'
import Button from './ui/Button.vue'
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

const emit = defineEmits<{
  close: []
  save: [groupData: Omit<SSHGroup, 'id' | 'created' | 'updated'>]
  update: [id: string, updates: Partial<SSHGroup>]
}>()

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

const isEditing = computed(() => !!props.group)

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
