<template>
  <Drawer
    :visible="visible && !showCommandModal"
    title="Saved Commands"
    position="right"
    width="lg"
    :icon="BookOpen"
    icon-background="bg-blue-500/20"
    icon-color="text-blue-400"
    @update:visible="handleDrawerVisibilityChange"
  >
    <!-- Search Bar -->
    <div class="p-4 border-b border-gray-700">
      <Input
        v-model="searchQuery"
        type="text"
        placeholder="Search SSH profiles..."
        :helper="false"
        :left-icon="Search"
      />
    </div>

    <!-- Content -->
    <div class="flex-1 overflow-y-auto">
      <!-- Loading State -->
      <div v-if="loading" class="p-6 text-center">
        <div
          class="animate-spin rounded-full h-8 w-8 border-2 border-gray-600 border-t-blue-400 mx-auto mb-3"
        ></div>
        <p class="text-gray-400">Loading commands...</p>
      </div>

      <!-- Empty State -->
      <div v-else-if="filteredCommands.length === 0" class="p-6 text-center">
        <BookOpen :size="48" class="mx-auto mb-4 text-gray-500" />
        <h3 class="text-lg font-medium text-white mb-2">
          {{ searchQuery ? 'No Matching Commands' : 'No Commands Found' }}
        </h3>
        <p class="text-gray-400 mb-4">
          {{
            searchQuery
              ? 'Try adjusting your search terms.'
              : 'Create your first command to get started.'
          }}
        </p>
        <Button v-if="!searchQuery" variant="primary" @click="openCreateCommandModal">
          <Plus :size="16" class="inline mr-2" />
          Create Command
        </Button>
      </div>

      <!-- Commands List -->
      <div v-else class="p-4 space-y-3">
        <div
          v-for="command in filteredCommands"
          :key="command.id"
          class="group bg-[#2a2a2a] hover:bg-[#333333] hover:border-gray-500 border border-transparent rounded-lg p-2 transition-all duration-300 transform hover:scale-[1.02] hover:shadow-lg"
        >
          <!-- Command Header -->
          <div class="flex items-start justify-between">
            <div class="flex-1 min-w-0">
              <div class="flex items-center space-x-2">
                <h4
                  class="text-sm font-medium text-white group-hover:text-blue-300 truncate transition-colors duration-300"
                >
                  {{ command.name }}
                </h4>
              </div>
              <p
                v-if="command.description"
                class="text-xs text-gray-400 group-hover:text-gray-300 truncate transition-colors duration-300"
              >
                {{ command.description }}
              </p>
            </div>

            <!-- Action Buttons -->
            <div
              class="flex items-center space-x-1 ml-2 opacity-0 group-hover:opacity-100 transition-all duration-300"
            >
              <Button
                title="Run command"
                variant="ghost"
                size="sm"
                :icon="Play"
                @click="executeCommand(command)"
              />
              <Button
                title="Copy to clipboard"
                variant="ghost"
                size="sm"
                :icon="Copy"
                @click="copyCommand(command)"
              />
              <Button
                title="Edit command"
                variant="ghost"
                size="sm"
                :icon="Edit"
                @click="editCommand(command)"
              />
              <PopConfirm
                title="Delete Command"
                content="This action cannot be undone."
                placement="bottom"
                @confirm="deleteCommand(command)"
              >
                <Button title="Delete command" variant="ghost" size="sm" :icon="Trash2" />
              </PopConfirm>
            </div>
          </div>

          <!-- Command Text -->
          <div class="bg-gray-900 rounded-md px-3 py-2 mt-2 border border-gray-700">
            <code
              class="text-sm text-green-400 font-mono break-all group-hover:text-green-300 transition-colors duration-300"
            >
              {{ command.command }}
            </code>
          </div>
        </div>
      </div>
    </div>

    <!-- Footer -->
    <template #footer>
      <div class="flex justify-between items-center">
        <div class="text-xs text-gray-500">
          {{ commands.length }} command{{ commands.length !== 1 ? 's' : '' }}
        </div>

        <Button
          variant="warning"
          size="sm"
          :icon="Plus"
          text="New Command"
          @click="openCreateCommandModal"
        />
      </div>
    </template>
  </Drawer>

  <!-- Create/Edit Command Modal -->
  <SavedCommandModal
    :visible="showCommandModal"
    :editing-command="editingCommand"
    @update:visible="handleModalVisibilityChange"
    @command-saved="handleCommandSaved"
  />
</template>

<script setup lang="ts">
import { ref, computed, watch, onMounted } from 'vue'
import { BookOpen, Plus, Play, Copy, Edit, Trash2, Search } from 'lucide-vue-next'
import Input from './ui/Input.vue'
import Drawer from './ui/Drawer.vue'
import Button from './ui/Button.vue'
import PopConfirm from './ui/PopConfirm.vue'
import SavedCommandModal from './SavedCommandModal.vue'
import { message } from '../utils/message'
import type { SavedCommand } from '../types/ssh'

interface Props {
  visible?: boolean
  activeTerminalId?: string
}

const props = withDefaults(defineProps<Props>(), {
  visible: false,
  activeTerminalId: ''
})

const emit = defineEmits<{
  'update:visible': [visible: boolean]
}>()

// State
const commands = ref<SavedCommand[]>([])
const searchQuery = ref('')
const loading = ref(false)
const showCommandModal = ref(false)
const editingCommand = ref<SavedCommand | null>(null)

// Computed
const filteredCommands = computed(() => {
  if (!searchQuery.value.trim()) {
    return commands.value
  }

  const query = searchQuery.value.toLowerCase()
  return commands.value.filter(
    (command) =>
      command.name.toLowerCase().includes(query) ||
      command.command.toLowerCase().includes(query) ||
      (command.description && command.description.toLowerCase().includes(query))
  )
})

// Methods
const loadCommands = async (): Promise<void> => {
  if (!props.visible) return

  loading.value = true
  try {
    // Load all commands
    commands.value = (await window.api.invoke('saved-commands.getAll')) as SavedCommand[]
  } catch (error) {
    console.error('Failed to load commands:', error)
    message.error('Failed to load commands')
  } finally {
    loading.value = false
  }
}

const handleDrawerVisibilityChange = (visible: boolean): void => {
  if (!showCommandModal.value) {
    emit('update:visible', visible)
  }
}

const handleModalVisibilityChange = (visible: boolean): void => {
  showCommandModal.value = visible
}

const handleCommandSaved = (): void => {
  loadCommands()
}

const openCreateCommandModal = (): void => {
  editingCommand.value = null
  showCommandModal.value = true
}

const editCommand = (command: SavedCommand): void => {
  editingCommand.value = command
  showCommandModal.value = true
}

const executeCommand = async (command: SavedCommand): Promise<void> => {
  if (!props.activeTerminalId) {
    message.error('No active terminal found')
    return
  }

  try {
    window.api.send('saved-commands.execute', {
      terminalId: props.activeTerminalId,
      command: command.command
    })
    message.success('Command executed')

    // Close drawer after executing command
    emit('update:visible', false)
  } catch (error) {
    console.error('Failed to execute command:', error)
    message.error('Failed to execute command')
  }
}

const copyCommand = async (command: SavedCommand): Promise<void> => {
  try {
    await window.api.invoke('saved-commands.copyToClipboard', command.command)
    message.success('Command copied to clipboard')
  } catch (error) {
    console.error('Failed to copy command:', error)
    message.error('Failed to copy command')
  }
}

const deleteCommand = async (command: SavedCommand): Promise<void> => {
  try {
    await window.api.invoke('saved-commands.delete', command.id)
    message.success('Command deleted successfully')
    await loadCommands()
  } catch (error) {
    console.error('Failed to delete command:', error)
    message.error('Failed to delete command')
  }
}

// Watchers
watch(
  () => props.visible,
  (newVisible) => {
    if (newVisible) {
      loadCommands()
    }
  }
)

// Lifecycle
onMounted(() => {
  if (props.visible) {
    loadCommands()
  }
})
</script>
