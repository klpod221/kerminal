<template>
  <Modal
    :visible="visible"
    :title="isEditing ? 'Edit SSH Tunnel' : 'Create SSH Tunnel'"
    :icon="Wifi"
    icon-background="bg-purple-500/20"
    icon-color="text-purple-400"
    size="lg"
    @close="handleClose"
  >
    <form class="space-y-6" @submit.prevent="handleSubmit">
      <!-- Basic Information -->
      <div class="space-y-1">
        <h3 class="text-lg font-medium text-white">Basic Information</h3>

        <!-- Tunnel Name -->
        <Input
          v-model="form.name"
          label="Tunnel Name"
          placeholder="My Web App Tunnel"
          :rules="['required']"
          :error-message="nameError"
          @blur="validation.validateField('name')"
        />

        <!-- Description -->
        <Textarea
          v-model="form.description"
          label="Description"
          placeholder="Optional description for this tunnel"
          :rows="2"
          :error-message="descriptionError"
          @blur="validation.validateField('description')"
        />

        <!-- SSH Profile -->
        <Select
          v-model="form.profileId"
          label="SSH Profile"
          placeholder="Select SSH Profile"
          :rules="['required']"
          :error-message="profileError"
          @blur="validation.validateField('profileId')"
        >
          <option value="">Select SSH Profile</option>
          <option v-for="profile in profiles" :key="profile.id" :value="profile.id">
            {{ profile.name }} ({{ profile.user }}@{{ profile.host }})
          </option>
        </Select>
      </div>

      <!-- Tunnel Configuration -->
      <div class="space-y-1">
        <h3 class="text-lg font-medium text-white">Tunnel Configuration</h3>

        <!-- Tunnel Type -->
        <Select v-model="form.type" label="Tunnel Type" @change="handleTypeChange">
          <option value="local">Local Port Forwarding</option>
          <option value="remote">Remote Port Forwarding</option>
          <option value="dynamic">Dynamic Port Forwarding (SOCKS)</option>
        </Select>

        <!-- Local Port -->
        <Input
          v-model.number="form.localPort"
          label="Local Port"
          type="number"
          placeholder="8080"
          min="1"
          max="65535"
          :rules="['required']"
          :error-message="localPortError"
          @blur="validation.validateField('localPort')"
        />

        <!-- Remote Configuration (for local/remote tunnels) -->
        <div v-if="form.type !== 'dynamic'" class="space-y-3">
          <div class="grid grid-cols-1 md:grid-cols-2 gap-4">
            <Input
              v-model="form.remoteHost"
              label="Remote Host"
              placeholder="localhost or internal.example.com"
              :rules="form.type === 'local' || form.type === 'remote' ? ['required'] : []"
              :error-message="remoteHostError"
              @blur="validation.validateField('remoteHost')"
            />
            <Input
              v-model.number="form.remotePort"
              label="Remote Port"
              type="number"
              placeholder="8080"
              min="1"
              max="65535"
              :rules="form.type === 'local' || form.type === 'remote' ? ['required'] : []"
              :error-message="remotePortError"
              @blur="validation.validateField('remotePort')"
            />
          </div>

          <!-- Helper text based on tunnel type -->
          <div class="text-sm text-gray-400">
            <div v-if="form.type === 'local'">
              <strong>Local Port Forwarding:</strong>
              Access remote service at
              <code>localhost:{{ form.localPort || 'XXXX' }}</code>
              →
              <code>{{ form.remoteHost || 'remote' }}:{{ form.remotePort || 'YYYY' }}</code>
            </div>
            <div v-if="form.type === 'remote'">
              <strong>Remote Port Forwarding:</strong>
              Remote server can access your local service
              <code>{{ form.remoteHost || 'remote' }}:{{ form.remotePort || 'YYYY' }}</code>
              →
              <code>localhost:{{ form.localPort || 'XXXX' }}</code>
            </div>
          </div>
        </div>

        <!-- Dynamic tunnel info -->
        <div v-if="form.type === 'dynamic'" class="text-sm text-gray-400">
          <strong>Dynamic Port Forwarding (SOCKS):</strong>
          Use
          <code>localhost:{{ form.localPort || 'XXXX' }}</code> as SOCKS proxy in your applications
        </div>
      </div>

      <!-- Options -->
      <div class="space-y-1">
        <h3 class="text-lg font-medium text-white">Options</h3>

        <div class="flex flex-col space-y-2">
          <!-- Auto Start -->
          <Checkbox
            v-model="form.autoStart"
            label="Auto-start tunnel when app opens"
            :helper="false"
          />
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
import { Wifi, Save } from 'lucide-vue-next'
import Modal from './ui/Modal.vue'
import Input from './ui/Input.vue'
import Select from './ui/Select.vue'
import Textarea from './ui/Textarea.vue'
import Checkbox from './ui/Checkbox.vue'
import Button from './ui/Button.vue'
import { useValidation, validationRules } from '../composables/useValidation'
import type { ValidationValue } from '../types/ui'
import type { SSHTunnel, SSHTunnelWithProfile } from '../types/ssh'
import type { SSHTunnelModalProps } from '../types/modals'

const props = withDefaults(defineProps<SSHTunnelModalProps>(), {
  visible: false,
  tunnel: null,
  profiles: () => [],
  preselectedProfile: null
})

const emit = defineEmits(['update:visible', 'save', 'update', 'close'])

// State
const isSaving = ref(false)

// Form data
const form = ref({
  name: '',
  description: '',
  profileId: '',
  type: 'local' as 'local' | 'remote' | 'dynamic',
  localPort: 8080,
  remoteHost: 'localhost',
  remotePort: 8080,
  autoStart: false
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

const profileIdRef = computed({
  get: () => form.value.profileId,
  set: (value) => {
    form.value.profileId = value
  }
})

const localPortRef = computed({
  get: () => form.value.localPort,
  set: (value) => {
    form.value.localPort = value
  }
})

const remoteHostRef = computed({
  get: () => form.value.remoteHost,
  set: (value) => {
    form.value.remoteHost = value
  }
})

const remotePortRef = computed({
  get: () => form.value.remotePort,
  set: (value) => {
    form.value.remotePort = value
  }
})

// Register validation fields
validation.registerField('name', nameRef, [
  validationRules.required('Tunnel name is required'),
  validationRules.maxLength(100, 'Tunnel name must be less than 100 characters')
])

validation.registerField('description', descriptionRef, [
  validationRules.maxLength(200, 'Description must be less than 200 characters')
])

validation.registerField('profileId', profileIdRef, [
  validationRules.required('SSH Profile is required')
])

validation.registerField('localPort', localPortRef, [validationRules.port()])

validation.registerField('remoteHost', remoteHostRef, [
  validationRules.custom((value: ValidationValue) => {
    if (form.value.type !== 'dynamic' && !value?.toString()?.trim()) {
      return false
    }
    return true
  }, 'Remote host is required for local/remote tunnels')
])

validation.registerField('remotePort', remotePortRef, [
  validationRules.custom((value: ValidationValue) => {
    if (form.value.type !== 'dynamic') {
      const num = Number(value)
      if (!num || num < 1 || num > 65535) {
        return false
      }
    }
    return true
  }, 'Remote port is required and must be between 1 and 65535')
])

// Computed
const isEditing = computed(() => !!props.tunnel)

// Error message computed properties
const nameError = computed(() => validation.fields.value?.name?.error?.value || undefined)
const descriptionError = computed(
  () => validation.fields.value?.description?.error?.value || undefined
)
const profileError = computed(() => validation.fields.value?.profileId?.error?.value || undefined)
const localPortError = computed(() => validation.fields.value?.localPort?.error?.value || undefined)
const remoteHostError = computed(
  () => validation.fields.value?.remoteHost?.error?.value || undefined
)
const remotePortError = computed(
  () => validation.fields.value?.remotePort?.error?.value || undefined
)

const canSubmit = computed(() => {
  const baseValidation =
    form.value.name &&
    form.value.profileId &&
    form.value.localPort &&
    !nameError.value &&
    !profileError.value &&
    !localPortError.value

  if (form.value.type === 'dynamic') {
    return baseValidation
  }

  return (
    baseValidation &&
    form.value.remoteHost &&
    form.value.remotePort &&
    !remoteHostError.value &&
    !remotePortError.value
  )
})

// Methods
const resetForm = (): void => {
  form.value = {
    name: '',
    description: '',
    profileId: '',
    type: 'local',
    localPort: 8080,
    remoteHost: 'localhost',
    remotePort: 8080,
    autoStart: false
  }

  if (validation.fields.value && Object.keys(validation.fields.value).length > 0) {
    validation.resetValidation()
  }
}

const loadTunnel = (tunnel: SSHTunnelWithProfile): void => {
  form.value = {
    name: tunnel.name,
    description: tunnel.description || '',
    profileId: tunnel.profileId,
    type: tunnel.type,
    localPort: tunnel.localPort,
    remoteHost: tunnel.remoteHost || 'localhost',
    remotePort: tunnel.remotePort || 8080,
    autoStart: tunnel.autoStart
  }
}

const handleTypeChange = (): void => {
  // Reset remote fields when switching to dynamic
  if (form.value.type === 'dynamic') {
    form.value.remoteHost = ''
    form.value.remotePort = 0
  } else if (!form.value.remoteHost) {
    form.value.remoteHost = 'localhost'
    form.value.remotePort = 8080
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

    const tunnelData: Partial<SSHTunnel> = {
      name: form.value.name,
      description: form.value.description || undefined,
      profileId: form.value.profileId,
      type: form.value.type,
      localPort: form.value.localPort,
      autoStart: form.value.autoStart
    }

    // Add remote configuration if not dynamic
    if (form.value.type !== 'dynamic') {
      tunnelData.remoteHost = form.value.remoteHost
      tunnelData.remotePort = form.value.remotePort
    }

    if (isEditing.value && props.tunnel) {
      emit('update', props.tunnel.id, tunnelData)
    } else {
      emit('save', tunnelData)
    }

    handleClose()
  } catch {
    // Error handling will be done by the parent component
  } finally {
    isSaving.value = false
  }
}

const handleClose = (): void => {
  emit('update:visible', false)
  emit('close')
  resetForm()
}

// Watch for tunnel changes
watch(
  () => props.tunnel,
  (tunnel) => {
    if (tunnel) {
      loadTunnel(tunnel)
    } else {
      resetForm()
    }
  },
  { immediate: true }
)

// Watch for preselected profile changes
watch(
  () => props.preselectedProfile,
  (profile) => {
    if (profile && !isEditing.value) {
      form.value.profileId = profile.id
    }
  },
  { immediate: true }
)
</script>
