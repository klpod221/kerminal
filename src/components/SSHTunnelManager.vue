<template>
  <div class="space-y-6">
    <!-- Header -->
    <div class="flex justify-between items-center">
      <div>
        <h2 class="text-2xl font-bold text-white">SSH Tunnels</h2>
        <p class="text-gray-400 mt-1">Manage your SSH port forwarding</p>
      </div>
      <Button variant="primary" :icon="Plus" @click="handleCreateTunnel">Create Tunnel</Button>
    </div>

    <!-- Tunnel List -->
    <div v-if="tunnels.length > 0" class="space-y-4">
      <div
        v-for="tunnel in tunnels"
        :key="tunnel.id"
        class="bg-gray-800/50 border border-gray-700 rounded-xl p-4 hover:border-gray-600 hover:bg-gray-800/70 transition-all duration-200"
      >
        <!-- Header -->
        <div class="flex items-center justify-between mb-2">
          <div>
            <h3 class="text-lg font-semibold text-white">{{ tunnel.name }}</h3>
            <div
              :class="[
                'px-3 py-1 rounded-full text-xs font-medium w-fit',
                getTunnelTypeColor(tunnel.type)
              ]"
            >
              {{ getTunnelTypeText(tunnel.type) }}
            </div>
          </div>

          <!-- Actions -->
          <div class="flex items-center space-x-2">
            <!-- Start/Stop Button -->
            <Button
              v-if="tunnel.status === 'stopped'"
              variant="success"
              size="sm"
              :icon="Play"
              @click="handleStartTunnel(tunnel)"
            />
            <Button
              v-else-if="tunnel.status === 'running'"
              variant="danger"
              size="sm"
              :icon="Square"
              @click="handleStopTunnel(tunnel)"
            />
            <Button v-else variant="ghost" size="sm" :disabled="true">
              {{ tunnel.status === 'starting' ? 'Starting...' : 'Stopping...' }}
            </Button>

            <!-- Edit Button -->
            <Button variant="secondary" size="sm" :icon="Edit2" @click="handleEditTunnel(tunnel)" />

            <!-- Delete Button -->
            <PopConfirm
              :content="`Are you sure you want to delete '${tunnel.name}'?`"
              @confirm="handleDeleteTunnel(tunnel)"
            >
              <Button variant="danger" size="sm" :icon="Trash2" />
            </PopConfirm>
          </div>
        </div>

        <!-- Description -->
        <p v-if="tunnel.description" class="text-gray-400 mb-2 text-xs">{{ tunnel.description }}</p>

        <!-- Connection Details -->
        <div class="bg-gray-900/50 rounded-lg p-4 space-y-2">
          <div class="grid grid-cols-1 sm:grid-cols-2 gap-4">
            <!-- Profile Info -->
            <div class="flex flex-col space-y-1">
              <span class="text-xs font-medium text-gray-500 uppercase tracking-wider">
                SSH Profile
              </span>
              <div class="flex items-center space-x-2">
                <div class="w-2 h-2 rounded-full bg-blue-500"></div>
                <span class="text-white text-sm font-medium">{{ tunnel.profile.name }}</span>
              </div>
            </div>

            <!-- Auto Start -->
            <div class="flex flex-col space-y-1">
              <span class="text-xs font-medium text-gray-500 uppercase tracking-wider">
                Auto Start
              </span>
              <div class="flex items-center space-x-2">
                <div
                  :class="[
                    'w-2 h-2 rounded-full',
                    tunnel.autoStart ? 'bg-green-500' : 'bg-gray-500'
                  ]"
                ></div>
                <span :class="tunnel.autoStart ? 'text-green-400' : 'text-gray-500'">
                  {{ tunnel.autoStart ? 'Enabled' : 'Disabled' }}
                </span>
              </div>
            </div>

            <!-- Local Port -->
            <div class="flex flex-col space-y-1">
              <span class="text-xs font-medium text-gray-500 uppercase tracking-wider">
                Local Port
              </span>
              <code class="text-blue-400 font-mono bg-gray-800 px-2 py-1 rounded text-sm w-fit">
                {{ tunnel.localPort }}
              </code>
            </div>

            <!-- Remote Info (for non-dynamic tunnels) -->
            <div v-if="tunnel.type !== 'dynamic'" class="flex flex-col space-y-1">
              <span class="text-xs font-medium text-gray-500 uppercase tracking-wider">
                Remote Port
              </span>
              <code class="text-green-400 font-mono bg-gray-800 px-2 py-1 rounded text-sm w-fit">
                {{ tunnel.remotePort }}
              </code>
            </div>

            <!-- SOCKS Proxy Info (for dynamic tunnels) -->
            <div v-if="tunnel.type === 'dynamic'" class="flex flex-col space-y-1">
              <span class="text-xs font-medium text-gray-500 uppercase tracking-wider">
                SOCKS Proxy
              </span>
              <code class="text-purple-400 font-mono bg-gray-800 px-2 py-1 rounded text-sm w-fit">
                localhost:{{ tunnel.localPort }}
              </code>
            </div>
          </div>

          <!-- Tunnel Flow Visualization -->
          <div class="mt-4 p-3 bg-gray-800/50 rounded-lg">
            <div class="text-xs font-medium text-gray-500 mb-2">TUNNEL FLOW</div>
            <div class="flex items-center justify-center space-x-3 text-sm">
              <div v-if="tunnel.type === 'local'" class="flex items-center space-x-3">
                <div
                  class="bg-blue-500/20 text-blue-400 px-2 py-1 rounded border border-blue-500/30"
                >
                  localhost:{{ tunnel.localPort }}
                </div>
                <div class="text-gray-400">→</div>
                <div
                  class="bg-green-500/20 text-green-400 px-2 py-1 rounded border border-green-500/30"
                >
                  {{ tunnel.remoteHost }}:{{ tunnel.remotePort }}
                </div>
              </div>
              <div v-else-if="tunnel.type === 'remote'" class="flex items-center space-x-3">
                <div
                  class="bg-green-500/20 text-green-400 px-2 py-1 rounded border border-green-500/30"
                >
                  {{ tunnel.remoteHost }}:{{ tunnel.remotePort }}
                </div>
                <div class="text-gray-400">→</div>
                <div
                  class="bg-blue-500/20 text-blue-400 px-2 py-1 rounded border border-blue-500/30"
                >
                  localhost:{{ tunnel.localPort }}
                </div>
              </div>
              <div v-else class="flex items-center space-x-3">
                <div
                  class="bg-purple-500/20 text-purple-400 px-2 py-1 rounded border border-purple-500/30"
                >
                  SOCKS Proxy
                </div>
                <div class="text-gray-400">@</div>
                <div
                  class="bg-blue-500/20 text-blue-400 px-2 py-1 rounded border border-blue-500/30"
                >
                  localhost:{{ tunnel.localPort }}
                </div>
              </div>
            </div>
          </div>
        </div>
      </div>
    </div>

    <!-- Empty State -->
    <div v-else class="text-center py-12">
      <Wifi class="mx-auto h-12 w-12 text-gray-400 mb-4" />
      <h3 class="text-lg font-medium text-white mb-2">No SSH Tunnels</h3>
      <p class="text-gray-400 mb-6">
        Create your first SSH tunnel to get started with port forwarding.
      </p>
      <Button variant="primary" :icon="Plus" @click="handleCreateTunnel">
        Create Your First Tunnel
      </Button>
    </div>
  </div>
</template>

<script setup lang="ts">
import { ref, onMounted, onUnmounted } from 'vue'
import { Plus, Play, Square, Edit2, Trash2, Wifi } from 'lucide-vue-next'
import Button from './ui/Button.vue'
import PopConfirm from './ui/PopConfirm.vue'
import { message } from '../utils/message'
import type { SSHTunnelWithProfile, SSHProfile } from '../types/ssh'
import type { SSHTunnelManagerProps } from '../types/components'

const props = withDefaults(defineProps<SSHTunnelManagerProps>(), {
  onHideManager: undefined,
  onShowManager: undefined
})

// Emits
const emit = defineEmits<{
  'create-tunnel': []
  'edit-tunnel': [tunnel: SSHTunnelWithProfile]
}>()

// State
const tunnels = ref<SSHTunnelWithProfile[]>([])
const profiles = ref<SSHProfile[]>([])
const statusUpdateInterval = ref<ReturnType<typeof setInterval>>()
let removeDataChangeListener: (() => void) | null = null

// Load data on mount
onMounted(async () => {
  await loadTunnels()
  await loadProfiles()
  startStatusUpdates()

  // Listen for sync events to reload data
  if (window.api?.on) {
    removeDataChangeListener = window.api.on('sync.dataChanged', (...args: unknown[]) => {
      const data = args[1] as { collection: string }
      if (data?.collection === 'ssh-tunnels') {
        loadTunnels()
      }
      if (data?.collection === 'ssh-profiles') {
        loadProfiles()
      }
    })
  }
})

onUnmounted(() => {
  if (statusUpdateInterval.value) {
    clearInterval(statusUpdateInterval.value)
  }

  if (removeDataChangeListener) {
    removeDataChangeListener()
  }
})

// Methods
const loadTunnels = async (): Promise<void> => {
  try {
    const result = await window.api.invoke('ssh-tunnels.getAll')
    tunnels.value = result as SSHTunnelWithProfile[]
  } catch {
    message.error('Failed to load tunnels')
  }
}

const loadProfiles = async (): Promise<void> => {
  try {
    const result = await window.api.invoke('ssh-profiles.getAll')
    profiles.value = result as SSHProfile[]
  } catch {
    // Silently handle - profiles not critical for tunnel management
  }
}

const startStatusUpdates = (): void => {
  // Update tunnel status every 5 seconds
  statusUpdateInterval.value = setInterval(async () => {
    // Simply reload tunnels to get fresh status
    await loadTunnels()
  }, 5000)
}

// Tunnel Actions
const handleStartTunnel = async (tunnel: SSHTunnelWithProfile): Promise<void> => {
  try {
    await window.api.invoke('ssh-tunnels.start', tunnel.id)
    message.success(`Tunnel "${tunnel.name}" started successfully`)
    await loadTunnels()
  } catch (error: unknown) {
    const errorMessage = error instanceof Error ? error.message : 'Unknown error'
    message.error(`Failed to start tunnel: ${errorMessage}`)
  }
}

const handleStopTunnel = async (tunnel: SSHTunnelWithProfile): Promise<void> => {
  try {
    await window.api.invoke('ssh-tunnels.stop', tunnel.id)
    message.info(`Tunnel "${tunnel.name}" stopped`)
    await loadTunnels()
  } catch (error) {
    console.error('Failed to stop tunnel:', error)
    message.error(`Failed to stop tunnel: ${error}`)
  }
}

const handleDeleteTunnel = async (tunnel: SSHTunnelWithProfile): Promise<void> => {
  try {
    // Stop tunnel if running
    if (tunnel.status === 'running' || tunnel.status === 'starting') {
      await window.api.invoke('ssh-tunnels.stop', tunnel.id)
    }

    await window.api.invoke('ssh-tunnels.delete', tunnel.id)
    message.success(`Tunnel "${tunnel.name}" deleted`)
    await loadTunnels()
  } catch (error) {
    console.error('Failed to delete tunnel:', error)
    message.error(`Failed to delete tunnel: ${error}`)
  }
}

// Modal Actions
const handleCreateTunnel = (): void => {
  emit('create-tunnel')
  // Hide manager modal when opening tunnel modal
  if (props.onHideManager) {
    props.onHideManager()
  }
}

const handleEditTunnel = (tunnel: SSHTunnelWithProfile): void => {
  emit('edit-tunnel', tunnel)
  // Hide manager modal when opening tunnel modal
  if (props.onHideManager) {
    props.onHideManager()
  }
}

const getTunnelTypeColor = (type: string): string => {
  switch (type) {
    case 'local':
      return 'bg-blue-500/20 text-blue-400'
    case 'remote':
      return 'bg-purple-500/20 text-purple-400'
    case 'dynamic':
      return 'bg-green-500/20 text-green-400'
    default:
      return 'bg-gray-500/20 text-gray-400'
  }
}

const getTunnelTypeText = (type: string): string => {
  switch (type) {
    case 'local':
      return 'Local Forward'
    case 'remote':
      return 'Remote Forward'
    case 'dynamic':
      return 'SOCKS Proxy'
    default:
      return type
  }
}
</script>
