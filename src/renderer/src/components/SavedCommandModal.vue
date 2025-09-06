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
          v-model="commandForm.name"
          label="Command Name"
          placeholder="My Command"
          :rules="['required']"
          :error-message="nameError"
          @blur="validation.validateField('name')"
        />

        <!-- Description -->
        <Textarea
          v-model="commandForm.description"
          label="Description"
          placeholder="Optional description for this command"
          :rows="2"
          :error-message="descriptionError"
          @blur="validation.validateField('description')"
        />

        <!-- Command -->
        <Textarea
          v-model="commandForm.command"
          label="Command"
          placeholder="Enter your command here"
          :rows="4"
          :rules="['required']"
          :error-message="commandError"
          @blur="validation.validateField('command')"
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
            :disabled="!canSubmit || isSaving"
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
import { ref, computed, watch } from 'vue'
import { BookOpen, Save } from 'lucide-vue-next'
import Modal from './ui/Modal.vue'
import Input from './ui/Input.vue'
import Textarea from './ui/Textarea.vue'
import Button from './ui/Button.vue'
import { useValidation, validationRules } from '../composables/useValidation'
import { message } from '../utils/message'
import type { SavedCommand } from '../types/ssh'

interface Props {
  visible?: boolean
  editingCommand?: SavedCommand | null
}

interface CommandForm {
  name: string
  command: string
  description: string
}

interface CommandFormErrors {
  name?: string
  command?: string
}

const props = withDefaults(defineProps<Props>(), {
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

// Validation setup
const validation = useValidation()

// Create reactive refs for validation
const nameRef = computed({
  get: () => commandForm.value.name,
  set: (value) => {
    commandForm.value.name = value
  }
})

const descriptionRef = computed({
  get: () => commandForm.value.description,
  set: (value) => {
    commandForm.value.description = value
  }
})

const commandRef = computed({
  get: () => commandForm.value.command,
  set: (value) => {
    commandForm.value.command = value
  }
})

// Register validation fields
validation.registerField('name', nameRef, [
  validationRules.required('Command name is required'),
  validationRules.maxLength(50, 'Command name must be less than 50 characters')
])

validation.registerField('description', descriptionRef, [
  validationRules.maxLength(200, 'Description must be less than 200 characters')
])

validation.registerField('command', commandRef, [
  validationRules.required('Command is required'),
  validationRules.maxLength(1000, 'Command must be less than 1000 characters')
])

// Computed
// Error message computed properties for safe access
const nameError = computed(() => validation.fields.value?.name?.error?.value || undefined)
const descriptionError = computed(
  () => validation.fields.value?.description?.error?.value || undefined
)
const commandError = computed(() => validation.fields.value?.command?.error?.value || undefined)

const canSubmit = computed(() => {
  return (
    commandForm.value.name && commandForm.value.command && !nameError.value && !commandError.value
  )
})

// Methods
const resetForm = (): void => {
  commandForm.value = {
    name: '',
    command: '',
    description: ''
  }
  commandFormErrors.value = {}

  // Only reset validation if fields are properly initialized
  if (validation.fields.value && Object.keys(validation.fields.value).length > 0) {
    validation.resetValidation()
  }
}

const handleClose = (): void => {
  emit('update:visible', false)
  resetForm()
}

const handleSubmit = async (): Promise<void> => {
  if (!canSubmit.value || isSaving.value) return

  // Validate all fields before submitting
  if (!validation.validateAll()) {
    return
  }

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
