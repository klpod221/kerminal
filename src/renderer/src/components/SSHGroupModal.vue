<template>
  <Modal :visible="show" title="SSH Group" @close="closeModal">
    <div class="space-y-4">
      <!-- Group Name -->
      <div>
        <label for="group-name" class="block text-sm font-medium text-gray-300 mb-2"
          >Group Name</label
        >
        <input
          id="group-name"
          v-model="groupData.name"
          type="text"
          placeholder="Enter group name"
          class="w-full px-3 py-2 bg-gray-800 border border-gray-600 rounded-md text-white focus:outline-none focus:ring-2 focus:ring-blue-500 focus:border-transparent"
          required
        />
      </div>

      <!-- Description -->
      <div>
        <label for="group-description" class="block text-sm font-medium text-gray-300 mb-2"
          >Description</label
        >
        <textarea
          id="group-description"
          v-model="groupData.description"
          placeholder="Enter group description (optional)"
          rows="3"
          class="w-full px-3 py-2 bg-gray-800 border border-gray-600 rounded-md text-white focus:outline-none focus:ring-2 focus:ring-blue-500 focus:border-transparent resize-none"
        ></textarea>
      </div>

      <!-- Default Settings -->
      <div class="border-t border-gray-600 pt-4">
        <h3 class="text-lg font-medium text-white mb-3">Default Settings</h3>
        <p class="text-sm text-gray-400 mb-4">
          These settings will be used as defaults for new profiles in this group
        </p>

        <!-- Default User -->
        <div class="mb-4">
          <label for="default-user" class="block text-sm font-medium text-gray-300 mb-2"
            >Default User</label
          >
          <input
            id="default-user"
            v-model="groupData.defaultUser"
            type="text"
            placeholder="e.g., root, admin"
            class="w-full px-3 py-2 bg-gray-800 border border-gray-600 rounded-md text-white focus:outline-none focus:ring-2 focus:ring-blue-500 focus:border-transparent"
          />
        </div>

        <!-- Default Host -->
        <div class="mb-4">
          <label for="default-host" class="block text-sm font-medium text-gray-300 mb-2"
            >Default Host</label
          >
          <input
            id="default-host"
            v-model="groupData.defaultHost"
            type="text"
            placeholder="e.g., server.example.com"
            class="w-full px-3 py-2 bg-gray-800 border border-gray-600 rounded-md text-white focus:outline-none focus:ring-2 focus:ring-blue-500 focus:border-transparent"
          />
        </div>

        <!-- Default Port -->
        <div class="mb-4">
          <label for="default-port" class="block text-sm font-medium text-gray-300 mb-2"
            >Default Port</label
          >
          <input
            id="default-port"
            v-model.number="groupData.defaultPort"
            type="number"
            placeholder="22"
            min="1"
            max="65535"
            class="w-full px-3 py-2 bg-gray-800 border border-gray-600 rounded-md text-white focus:outline-none focus:ring-2 focus:ring-blue-500 focus:border-transparent"
          />
        </div>

        <!-- Default SSH Key Path -->
        <div class="mb-4">
          <label for="default-key-path" class="block text-sm font-medium text-gray-300 mb-2"
            >Default SSH Key Path</label
          >
          <div class="flex space-x-2">
            <input
              id="default-key-path"
              v-model="groupData.defaultKeyPath"
              type="text"
              placeholder="Path to SSH private key"
              class="flex-1 px-3 py-2 bg-gray-800 border border-gray-600 rounded-md text-white focus:outline-none focus:ring-2 focus:ring-blue-500 focus:border-transparent"
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

        <!-- Default Password -->
        <div class="mb-4">
          <label for="default-password" class="block text-sm font-medium text-gray-300 mb-2"
            >Default Password</label
          >
          <div class="relative">
            <input
              id="default-password"
              v-model="groupData.defaultPassword"
              :type="showPassword ? 'text' : 'password'"
              placeholder="Default password for this group"
              class="w-full px-3 py-2 pr-10 bg-gray-800 border border-gray-600 rounded-md text-white focus:outline-none focus:ring-2 focus:ring-blue-500 focus:border-transparent"
            />
            <button
              type="button"
              class="absolute inset-y-0 right-0 pr-3 flex items-center text-gray-400 hover:text-white"
              @click="showPassword = !showPassword"
            >
              <Eye v-if="!showPassword" class="w-5 h-5" />
              <EyeOff v-else class="w-5 h-5" />
            </button>
          </div>
        </div>

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
            <input
              v-model="groupData.color"
              type="text"
              placeholder="#6b7280"
              class="flex-1 px-3 py-2 bg-gray-800 border border-gray-600 rounded-md text-white placeholder-gray-400 focus:outline-none focus:ring-2 focus:ring-blue-500 focus:border-transparent"
            />
          </div>
        </div>
      </div>
    </div>

    <template #footer>
      <div class="flex justify-end space-x-3">
        <button
          type="button"
          class="px-4 py-2 bg-gray-700 text-white rounded-md hover:bg-gray-600 focus:outline-none focus:ring-2 focus:ring-gray-500"
          @click="closeModal"
        >
          Cancel
        </button>
        <button
          type="button"
          :disabled="!groupData.name.trim()"
          class="px-4 py-2 bg-blue-600 text-white rounded-md hover:bg-blue-700 focus:outline-none focus:ring-2 focus:ring-blue-500 disabled:opacity-50 disabled:cursor-not-allowed"
          @click="saveGroup"
        >
          {{ isEditing ? 'Update' : 'Create' }} Group
        </button>
      </div>
    </template>
  </Modal>
</template>

<script setup lang="ts">
import { ref, watch, computed } from 'vue'
import { Eye, EyeOff, Folder } from 'lucide-vue-next'
import Modal from './ui/Modal.vue'
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
