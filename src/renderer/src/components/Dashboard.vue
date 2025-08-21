<template>
  <div class="h-full bg-[#171717] text-white p-6 overflow-y-auto">
    <div class="max-w-6xl mx-auto animate-fade-in">
      <!-- Quick Actions -->
      <div class="mb-12">
        <h2 class="text-2xl font-bold mb-6 text-center">Quick Actions</h2>
        <div class="grid grid-cols-2 md:grid-cols-4 gap-6 max-w-4xl mx-auto">
          <Card
            class="text-center transition-all duration-300 transform hover:scale-105 hover:shadow-lg cursor-pointer"
            @click="createNewTerminal"
          >
            <div
              class="bg-blue-500/20 rounded-lg p-3 w-fit mx-auto mb-3 group-hover:bg-blue-500/30 transition-colors"
            >
              <Terminal class="w-8 h-8 text-blue-400" />
            </div>
            <span class="text-sm font-medium">New Terminal</span>
          </Card>
          <Card
            class="text-center transition-all duration-300 transform hover:scale-105 hover:shadow-lg cursor-pointer"
          >
            <div
              class="bg-orange-500/20 rounded-lg p-3 w-fit mx-auto mb-3 group-hover:bg-orange-500/30 transition-colors"
            >
              <Server class="w-8 h-8 text-orange-400" />
            </div>
            <span class="text-sm font-medium">SSH Profile</span>
          </Card>
          <Card
            class="text-center transition-all duration-300 transform hover:scale-105 hover:shadow-lg cursor-pointer"
          >
            <div
              class="bg-green-500/20 rounded-lg p-3 w-fit mx-auto mb-3 group-hover:bg-green-500/30 transition-colors"
            >
              <Settings class="w-8 h-8 text-green-400" />
            </div>
            <span class="text-sm font-medium">Settings</span>
          </Card>
          <Card
            class="text-center transition-all duration-300 transform hover:scale-105 hover:shadow-lg cursor-pointer"
            @click="showAbout"
          >
            <div
              class="bg-purple-500/20 rounded-lg p-3 w-fit mx-auto mb-3 group-hover:bg-purple-500/30 transition-colors"
            >
              <Info class="w-8 h-8 text-purple-400" />
            </div>
            <span class="text-sm font-medium">About</span>
          </Card>
        </div>
      </div>

      <!-- System Information -->
      <div class="mb-8">
        <h2 class="text-2xl font-bold mb-6 text-center">System Information</h2>

        <!-- Loading State -->
        <div v-if="isLoading" class="text-center py-16">
          <div class="relative">
            <div
              class="animate-spin rounded-full h-16 w-16 border-4 border-gray-600 border-t-blue-400 mx-auto"
            ></div>
            <div
              class="animate-ping absolute inset-0 rounded-full h-16 w-16 border-4 border-blue-400 opacity-20 mx-auto"
            ></div>
          </div>
          <p class="text-gray-400 mt-6 text-lg">Loading system information...</p>
        </div>

        <!-- System Info Grid -->
        <div v-else class="grid grid-cols-1 md:grid-cols-2 lg:grid-cols-3 gap-6">
          <!-- Computer Info -->
          <Card
            title="Computer"
            :icon="Computer"
            icon-background="bg-blue-500/20"
            icon-color="text-blue-400"
            :hover="true"
          >
            <div class="flex justify-between items-center">
              <span class="text-gray-400 font-medium mr-1">OS:</span>
              <span class="text-white font-mono text-sm bg-gray-800 px-2 py-1 rounded truncate">{{
                systemInfo.osName + ' ' + systemInfo.arch
              }}</span>
            </div>
            <div class="flex justify-between items-center">
              <span class="text-gray-400 font-medium mr-1">Kernel:</span>
              <span class="text-white font-mono text-sm bg-gray-800 px-2 py-1 rounded truncate">{{
                systemInfo.kernel
              }}</span>
            </div>
            <div class="flex justify-between items-center">
              <span class="text-gray-400 font-medium mr-1">Hostname:</span>
              <span class="text-white font-mono text-sm bg-gray-800 px-2 py-1 rounded truncate">{{
                systemInfo.hostname
              }}</span>
            </div>
          </Card>

          <!-- CPU Info -->
          <Card
            title="CPU"
            :icon="Cpu"
            icon-background="bg-green-500/20"
            icon-color="text-green-400"
            :hover="true"
          >
            <div class="flex justify-between items-center">
              <span class="text-gray-400 font-medium mr-1">Model:</span>
              <span
                class="text-white font-mono text-sm bg-gray-800 px-2 py-1 rounded truncate"
                :title="systemInfo.cpuModel"
                >{{ systemInfo.cpuModel }}</span
              >
            </div>
            <div class="flex justify-between items-center">
              <span class="text-gray-400 font-medium mr-1">Cores:</span>
              <span class="text-white font-mono text-sm bg-gray-800 px-2 py-1 rounded truncate">{{
                systemInfo.cpuCores
              }}</span>
            </div>
            <div class="flex justify-between items-center">
              <span class="text-gray-400 font-medium mr-1">Speed:</span>
              <span class="text-white font-mono text-sm bg-gray-800 px-2 py-1 rounded truncate">{{
                systemInfo.cpuSpeed
              }}</span>
            </div>
          </Card>

          <!-- Memory Info -->
          <Card
            title="Memory"
            :icon="MemoryStick"
            icon-background="bg-purple-500/20"
            icon-color="text-purple-400"
            :hover="true"
          >
            <div class="flex justify-between items-center">
              <span class="text-gray-400 font-medium mr-1">Total:</span>
              <span class="text-white font-mono text-sm bg-gray-800 px-2 py-1 rounded truncate">{{
                systemInfo.totalMemory
              }}</span>
            </div>
            <div class="flex justify-between items-center">
              <span class="text-gray-400 font-medium mr-1">Used:</span>
              <span class="text-white font-mono text-sm bg-gray-800 px-2 py-1 rounded truncate">{{
                systemInfo.usedMemory
              }}</span>
            </div>
            <div class="flex justify-between items-center">
              <span class="text-gray-400 font-medium mr-1">Usage:</span>
              <div class="flex items-center space-x-2">
                <div class="w-16 h-2 bg-gray-700 rounded-full overflow-hidden">
                  <div
                    class="h-full transition-all duration-500 rounded-full"
                    :class="getMemoryUsageColor(true)"
                    :style="{ width: `${systemInfo.memoryUsage}%` }"
                  ></div>
                </div>
                <span :class="getMemoryUsageColor()" class="font-mono text-sm font-bold"
                  >{{ systemInfo.memoryUsage }}%</span
                >
              </div>
            </div>
          </Card>

          <!-- GPU Info -->
          <Card
            title="GPU"
            :icon="Monitor"
            icon-background="bg-orange-500/20"
            icon-color="text-orange-400"
            :hover="true"
          >
            <div class="flex justify-between items-center">
              <span class="text-gray-400 font-medium mr-1">Model:</span>
              <span
                class="text-white font-mono text-sm bg-gray-800 px-2 py-1 rounded truncate"
                :title="systemInfo.gpuModel"
                >{{ systemInfo.gpuModel }}</span
              >
            </div>
            <div class="flex justify-between items-center">
              <span class="text-gray-400 font-medium mr-1">Vendor:</span>
              <span class="text-white font-mono text-sm bg-gray-800 px-2 py-1 rounded truncate">{{
                systemInfo.gpuVendor
              }}</span>
            </div>
          </Card>

          <!-- Network Info -->
          <Card
            title="Network"
            :icon="Wifi"
            icon-background="bg-cyan-500/20"
            icon-color="text-cyan-400"
            :hover="true"
          >
            <div class="flex justify-between items-center">
              <span class="text-gray-400 font-medium mr-1">IP:</span>
              <span class="text-white font-mono text-sm bg-gray-800 px-2 py-1 rounded truncate">{{
                systemInfo.ipAddress
              }}</span>
            </div>
            <div class="flex justify-between items-center">
              <span class="text-gray-400 font-medium mr-1">Status:</span>
              <span
                v-if="networkStatus.isConnected"
                class="inline-flex items-center px-2 py-1 rounded text-xs font-medium bg-green-100 text-green-800"
              >
                <span class="w-2 h-2 bg-green-400 rounded-full mr-1 animate-pulse"></span>
                Connected
              </span>
              <span
                v-else
                class="inline-flex items-center px-2 py-1 rounded text-xs font-medium bg-red-100 text-red-800"
              >
                <span class="w-2 h-2 bg-red-400 rounded-full mr-1"></span>
                Disconnected
              </span>
            </div>
          </Card>

          <!-- Display Info -->
          <Card
            title="Display"
            :icon="Monitor"
            icon-background="bg-pink-500/20"
            icon-color="text-pink-400"
            :hover="true"
          >
            <div class="flex justify-between items-center">
              <span class="text-gray-400 font-medium">Resolution:</span>
              <span class="text-white font-mono text-sm bg-gray-800 px-2 py-1 rounded">{{
                systemInfo.resolution
              }}</span>
            </div>
            <div class="flex justify-between items-center">
              <span class="text-gray-400 font-medium">Refresh Rate:</span>
              <span class="text-white font-mono text-sm bg-gray-800 px-2 py-1 rounded">{{
                systemInfo.refreshRate
              }}</span>
            </div>
          </Card>
        </div>

        <!-- Auto-refresh Status -->
        <div class="text-center mt-8">
          <div class="text-gray-500 text-sm">
            <RefreshCw
              class="w-4 h-4 inline mr-2 transition-transform duration-300"
              :class="{ 'animate-spin': isRefreshing }"
            />
            {{ isRefreshing ? 'Refreshing...' : `Auto-refresh in ${refreshCountdown}s` }}
          </div>
        </div>
      </div>
    </div>

    <!-- About Modal -->
    <Modal
      v-model:visible="showAboutModal"
      title="About Kerminal"
      :icon="Info"
      icon-background="bg-purple-500/20"
      icon-color="text-purple-400"
      @close="showAboutModal = false"
    >
      <div class="space-y-6">
        <!-- App Icon and Name -->
        <div class="text-center">
          <div class="bg-purple-500/20 rounded-full p-4 w-fit mx-auto mb-4">
            <Terminal class="w-12 h-12 text-purple-400" />
          </div>
          <h3 class="text-2xl font-bold text-white mb-2">Kerminal</h3>
          <p class="text-gray-400">Version 1.0.0</p>
        </div>

        <!-- Description -->
        <div class="bg-[#171717] border border-gray-700 rounded-lg p-4">
          <p class="text-gray-300 leading-relaxed">
            A modern terminal application built with Electron and Vue. Designed for developers who
            want a powerful and beautiful terminal experience.
          </p>
        </div>

        <!-- Features -->
        <div class="space-y-3">
          <h4 class="text-lg font-semibold text-white">Features</h4>
          <ul class="space-y-2 text-gray-300">
            <li class="flex items-center space-x-2">
              <div class="w-2 h-2 bg-purple-400 rounded-full"></div>
              <span>Modern and intuitive interface</span>
            </li>
            <li class="flex items-center space-x-2">
              <div class="w-2 h-2 bg-purple-400 rounded-full"></div>
              <span>Real-time system information</span>
            </li>
            <li class="flex items-center space-x-2">
              <div class="w-2 h-2 bg-purple-400 rounded-full"></div>
              <span>Cross-platform compatibility</span>
            </li>
            <li class="flex items-center space-x-2">
              <div class="w-2 h-2 bg-purple-400 rounded-full"></div>
              <span>Built with modern web technologies</span>
            </li>
          </ul>
        </div>

        <!-- Developer Info -->
        <div class="bg-[#171717] border border-gray-700 rounded-lg p-4">
          <h4 class="text-lg font-semibold text-white mb-3">Developer</h4>
          <div class="space-y-2">
            <div class="flex justify-between items-center">
              <span class="text-gray-400">Name:</span>
              <span class="text-white font-medium">klpod221</span>
            </div>
            <div class="flex justify-between items-center">
              <span class="text-gray-400">Website:</span>
              <a
                href="https://klpod221.com"
                target="_blank"
                class="text-purple-400 hover:text-purple-300 transition-colors"
              >
                klpod221.com
              </a>
            </div>
            <div class="flex justify-between items-center">
              <span class="text-gray-400">GitHub:</span>
              <a
                href="https://github.com/klpod221"
                target="_blank"
                class="text-purple-400 hover:text-purple-300 transition-colors"
              >
                @klpod221
              </a>
            </div>
          </div>
        </div>
      </div>

      <template #footer>
        <button
          class="px-4 py-2 bg-purple-600 hover:bg-purple-700 text-white font-medium rounded-lg transition-colors"
          @click="showAboutModal = false"
        >
          Close
        </button>
      </template>
    </Modal>
  </div>
</template>

<script setup lang="ts">
import { ref, onMounted, onUnmounted, computed } from 'vue'
import {
  Computer,
  Monitor,
  Cpu,
  MemoryStick,
  Wifi,
  Terminal,
  Settings,
  Server,
  RefreshCw,
  Info
} from 'lucide-vue-next'
import Card from './ui/Card.vue'
import Modal from './ui/Modal.vue'

// Type definitions
interface SystemInfo {
  platform: string
  release: string
  arch: string
  hostname: string
  uptime: number
  totalMemory: number
  freeMemory: number
  loadAverage: number[]
  cpus: Array<{
    model: string
    speed: number
    times: {
      user: number
      nice: number
      sys: number
      idle: number
      irq: number
    }
  }>
  osRelease?: string
  cpuInfo?: string
  memInfo?: string
  gpuInfo?: string
  resolution?: string
}

interface NetworkInterface {
  name: string
  address: string
  netmask: string
  mac: string
  isConnected?: boolean
}

interface NetworkStatus {
  isConnected: boolean
  primaryInterface: NetworkInterface | null
  interfaces: NetworkInterface[]
}

// Reactive state
const isLoading = ref(true)
const isRefreshing = ref(false)
const showAboutModal = ref(false)
const rawSystemInfo = ref<SystemInfo | null>(null)
const networkStatus = ref<NetworkStatus>({
  isConnected: false,
  primaryInterface: null,
  interfaces: []
})
const refreshCountdown = ref(5) // Countdown timer in seconds

// Auto-refresh configuration
const REFRESH_INTERVAL = 5000 // 5 seconds in milliseconds
let refreshTimer: number | null = null
let countdownTimer: number | null = null

// Define emits
const emit = defineEmits<{
  'create-terminal': []
}>()

// Computed properties for formatted system information
const systemInfo = computed(() => {
  if (!rawSystemInfo.value) {
    return {
      osName: 'Loading...',
      kernel: 'Loading...',
      arch: 'Loading...',
      hostname: 'Loading...',
      cpuModel: 'Loading...',
      cpuCores: 'Loading...',
      cpuSpeed: 'Loading...',
      totalMemory: 'Loading...',
      usedMemory: 'Loading...',
      freeMemory: 'Loading...',
      memoryUsage: 0,
      gpuModel: 'Loading...',
      gpuVendor: 'Loading...',
      ipAddress: 'Loading...',
      networkInterface: 'Loading...',
      resolution: 'Loading...',
      refreshRate: 'Loading...'
    }
  }

  const data = rawSystemInfo.value

  // Parse OS information
  const osName = parseOSName(data.osRelease || '', data.platform)
  const kernel = `${data.platform} ${data.release}`

  // Parse CPU information
  const cpuModel = data.cpus?.[0]?.model || 'Unknown CPU'
  const cpuCores = data.cpus ? `${data.cpus.length} cores` : 'Unknown'
  const cpuSpeed = data.cpus?.[0]?.speed
    ? `${(data.cpus[0].speed / 1000).toFixed(2)} GHz`
    : 'Unknown'

  // Calculate memory information
  const totalMemoryMB = Math.round(data.totalMemory / 1024 / 1024)
  const freeMemoryMB = Math.round(data.freeMemory / 1024 / 1024)
  const usedMemoryMB = totalMemoryMB - freeMemoryMB
  const memoryUsage = Math.round((usedMemoryMB / totalMemoryMB) * 100)

  // Parse GPU information
  const gpuInfo = parseGPUInfo(data.gpuInfo || '')

  // Get network information
  const primaryNetwork = networkStatus.value.primaryInterface
  const ipAddress = primaryNetwork?.address || 'Not connected'
  const networkInterface = primaryNetwork?.name || 'None'

  // Parse display information
  const displayInfo = parseDisplayInfo(data.resolution || '')

  return {
    osName,
    kernel,
    arch: data.arch,
    hostname: data.hostname,
    cpuModel,
    cpuCores,
    cpuSpeed,
    totalMemory: `${totalMemoryMB} MB`,
    usedMemory: `${usedMemoryMB} MB`,
    freeMemory: `${freeMemoryMB} MB`,
    memoryUsage,
    gpuModel: gpuInfo.model,
    gpuVendor: gpuInfo.vendor,
    ipAddress,
    networkInterface,
    resolution: displayInfo.resolution,
    refreshRate: displayInfo.refreshRate
  }
})

/**
 * Parse OS name from /etc/os-release
 */
function parseOSName(osRelease: string, platform: string): string {
  if (!osRelease || platform !== 'linux') {
    return platform.charAt(0).toUpperCase() + platform.slice(1)
  }

  const lines = osRelease.split('\n')
  for (const line of lines) {
    if (line.startsWith('PRETTY_NAME=')) {
      return line.split('=')[1]?.replace(/"/g, '') || platform
    }
  }
  return platform
}

/**
 * Parse GPU information from lspci output
 */
function parseGPUInfo(gpuInfo: string): { model: string; vendor: string } {
  if (!gpuInfo) {
    return { model: 'Unknown GPU', vendor: 'Unknown' }
  }

  const lines = gpuInfo.split('\n')
  for (const line of lines) {
    if (
      line.toLowerCase().includes('vga compatible controller') ||
      line.toLowerCase().includes('3d controller')
    ) {
      const parts = line.split(': ')
      if (parts.length > 1) {
        const gpuDesc = parts[1]
        const vendor = gpuDesc.split(' ')[0]
        return {
          model: gpuDesc,
          vendor: vendor
        }
      }
    }
  }
  return { model: 'Unknown GPU', vendor: 'Unknown' }
}

/**
 * Parse display information from xrandr output
 */
function parseDisplayInfo(resolution: string): { resolution: string; refreshRate: string } {
  if (!resolution) {
    return { resolution: 'Unknown', refreshRate: 'Unknown' }
  }

  const lines = resolution.split('\n')
  for (const line of lines) {
    if (line.includes('*')) {
      const match = line.match(/(\d+x\d+).*?(\d+\.\d+)\*/)
      if (match) {
        return {
          resolution: match[1],
          refreshRate: `${Math.round(parseFloat(match[2]))} Hz`
        }
      }
    }
  }
  return { resolution: 'Unknown', refreshRate: 'Unknown' }
}

/**
 * Get memory usage color based on percentage
 */
function getMemoryUsageColor(isBackground = false): string {
  const usage = systemInfo.value.memoryUsage
  if (isBackground) {
    if (usage > 80) return 'bg-red-500'
    if (usage > 60) return 'bg-yellow-500'
    return 'bg-green-500'
  } else {
    if (usage > 80) return 'text-red-400'
    if (usage > 60) return 'text-yellow-400'
    return 'text-green-400'
  }
}

/**
 * Load system information from main process
 */
async function loadSystemInfo(): Promise<void> {
  try {
    const [sysInfo, netStatus] = await Promise.all([
      window.api.getSystemInfo(),
      window.api.getNetworkStatus()
    ])

    rawSystemInfo.value = sysInfo
    networkStatus.value = netStatus
  } catch (error) {
    console.error('Failed to load system information:', error)
  }
}

/**
 * Start the countdown timer
 */
function startCountdown(): void {
  refreshCountdown.value = REFRESH_INTERVAL / 1000 // Convert to seconds

  countdownTimer = window.setInterval(() => {
    refreshCountdown.value--
    if (refreshCountdown.value <= 0) {
      refreshCountdown.value = REFRESH_INTERVAL / 1000
    }
  }, 1000)
}

/**
 * Start auto-refresh timer
 */
function startAutoRefresh(): void {
  refreshTimer = window.setInterval(async () => {
    if (!isRefreshing.value) {
      isRefreshing.value = true
      try {
        await loadSystemInfo()
      } finally {
        isRefreshing.value = false
      }
    }
  }, REFRESH_INTERVAL)
}

/**
 * Stop all timers
 */
function stopTimers(): void {
  if (refreshTimer) {
    clearInterval(refreshTimer)
    refreshTimer = null
  }
  if (countdownTimer) {
    clearInterval(countdownTimer)
    countdownTimer = null
  }
}

/**
 * Show about dialog
 */
function showAbout(): void {
  showAboutModal.value = true
}

/**
 * Create a new terminal and switch to it
 */
function createNewTerminal(): void {
  emit('create-terminal')
}

// Load system information on component mount
onMounted(async () => {
  try {
    await loadSystemInfo()
    // Start auto-refresh and countdown timers
    startAutoRefresh()
    startCountdown()
  } finally {
    isLoading.value = false
  }
})

// Cleanup timers when component is unmounted
onUnmounted(() => {
  stopTimers()
})
</script>
