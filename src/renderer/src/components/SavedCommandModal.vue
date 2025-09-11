<template>
  <Modal
    :visible="visible"
    :title="editingCommand ? 'Edit Command' : 'Create Command'"
    :icon="BookOpen"
    icon-background="bg-blue-500/20"
    icon-color="text-blue-400"
    size="lg"
    @close="handleClose"
  >
    <form class="space-y-6" @submit.prevent="handleSubmit">
      <!-- Basic Information -->
      <div class="space-y-1">
        <!-- Command Name -->
        <Input
          id="command-name"
          v-model="commandForm.name"
          type="text"
          label="Command Name"
          placeholder="My Command"
        />

        <!-- Description -->
        <Textarea
          v-model="commandForm.description"
          label="Description"
          placeholder="Optional description for this command"
          :rows="2"
        />

        <!-- Command -->
        <Textarea
          v-model="commandForm.command"
          label="Command"
          placeholder="Enter your command here"
          :rows="4"
          :rules="['required']"
        />
      </div>
    </form>

    <template #footer>
      <div class="flex justify-between w-full">
        <Button variant="ghost" size="sm" @click="handleClose">Cancel</Button>

        <div class="flex space-x-3">
          <Button
            variant="primary"
            size="sm"
            :disabled="isSaving"
            :loading="isSaving"
            :icon="Save"
            @click="handleSubmit"
          >
            <template v-if="!isSaving">
              {{ editingCommand ? 'Update' : 'Create' }}
            </template>
          </Button>
        </div>
      </div>
    </template>
  </Modal>
</template>

<script setup lang="ts">
import { ref, watch } from 'vue'
import { BookOpen, Save } from 'lucide-vue-next'
import Modal from './ui/Modal.vue'
import Input from './ui/Input.vue'
import Textarea from './ui/Textarea.vue'
import Button from './ui/Button.vue'
import { message } from '../utils/message'
import type { SavedCommandModalProps, CommandForm, CommandFormErrors } from '../types/modals'

const props = withDefaults(defineProps<SavedCommandModalProps>(), {
  visible: false,
  editingCommand: null
})

const emit = defineEmits<{
  'update:visible': [visible: boolean]
  'command-saved': []
}>()

// State
const isSaving = ref(false)

// Form state
const commandForm = ref<CommandForm>({
  name: '',
  command: '',
  description: ''
})

const commandFormErrors = ref<CommandFormErrors>({})

// Methods
const resetForm = (): void => {
  commandForm.value = {
    name: '',
    command: '',
    description: ''
  }
  commandFormErrors.value = {}
}

const handleClose = (): void => {
  emit('update:visible', false)
  resetForm()
}

const handleSubmit = async (): Promise<void> => {
  try {
    isSaving.value = true

    const commandData = {
      name: commandForm.value.name.trim(),
      command: commandForm.value.command.trim(),
      description: commandForm.value.description.trim() || undefined
    }

    if (props.editingCommand) {
      await window.api.invoke('saved-commands.update', props.editingCommand.id, commandData)
      message.success('Command updated successfully')
    } else {
      await window.api.invoke('saved-commands.create', commandData)
      message.success('Command created successfully')
    }

    emit('command-saved')
    handleClose()
  } catch {
    message.error('Failed to save command')
  } finally {
    isSaving.value = false
  }
}

// Watchers
watch(
  () => props.visible,
  (newVisible) => {
    if (newVisible && props.editingCommand) {
      // Populate form with editing command data
      commandForm.value = {
        name: props.editingCommand.name,
        command: props.editingCommand.command,
        description: props.editingCommand.description || ''
      }
    } else if (newVisible) {
      // Reset form for new command
      resetForm()
    }
  }
)

watch(
  () => props.editingCommand,
  (newCommand) => {
    if (newCommand && props.visible) {
      commandForm.value = {
        name: newCommand.name,
        command: newCommand.command,
        description: newCommand.description || ''
      }
    }
  }
)
</script>
