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
        <Input v-model="form.name" label="Tunnel Name" placeholder="My Web App Tunnel" />

        <!-- Description -->
        <Textarea
          v-model="form.description"
          label="Description"
          placeholder="Optional description for this tunnel"
          :rows="2"
        />

        <!-- SSH Profile -->
        <Select
          v-model="form.profileId"
          label="SSH Profile"
          placeholder="Select SSH Profile"
          :rules="['required']"
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
        />

        <!-- Remote Configuration (for local/remote tunnels) -->
        <div v-if="form.type !== 'dynamic'" class="space-y-3">
          <div class="grid grid-cols-1 md:grid-cols-2 gap-4">
            <Input
              v-model="form.remoteHost"
              label="Remote Host"
              placeholder="localhost or internal.example.com"
            />
            <Input
              v-model.number="form.remotePort"
              label="Remote Port"
              type="number"
              placeholder="8080"
              min="1"
              max="65535"
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

// Computed
const isEditing = computed(() => !!props.tunnel)

const canSubmit = computed(() => {
  const baseValidation = form.value.name && form.value.profileId && form.value.localPort

  if (form.value.type === 'dynamic') {
    return baseValidation
  }

  return baseValidation && form.value.remoteHost && form.value.remotePort
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
