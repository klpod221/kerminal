<template>
  <div
    class="title-bar flex items-center h-[30px] min-h-[30px] max-h-[30px] text-white font-sans select-none bg-[#0D0D0D] border-b border-gray-800 flex-shrink-0 relative z-50"
  >
    <!-- Dashboard Icon -->
    <div
      class="no-drag flex items-center px-3 hover:bg-gray-800 cursor-pointer h-full max-h-[30px] transition-colors duration-200 flex-shrink-0"
      :class="{ 'bg-gray-800': isDashboardActive }"
      @click="openDashboard"
    >
      <LayoutDashboard
        :size="16"
        class="transition-colors duration-200"
        :class="isDashboardActive ? 'text-blue-400' : 'text-gray-400 hover:text-white'"
      />
    </div>

    <!-- SSH Profiles Icon -->
    <div
      class="no-drag flex items-center px-3 hover:bg-gray-800 cursor-pointer h-full max-h-[30px] transition-colors duration-200 flex-shrink-0"
      @click="toggleSSHDrawer"
    >
      <PanelLeft :size="16" class="transition-colors duration-200 text-gray-400 hover:text-white" />
    </div>

    <!-- Tabs Container -->
    <div class="flex items-center flex-1 h-full max-h-[30px] min-w-0">
      <div class="flex items-center h-full max-h-[30px] max-w-[calc(100%-80px)]">
        <div
          v-for="tab in tabs"
          :key="tab.id"
          class="no-drag flex items-center px-2 h-full max-h-[30px] border-r border-gray-800 cursor-pointer group transition-all duration-200 flex-1"
          :class="{
            'bg-[#171717] border-b-2 border-b-blue-500': tab.active,
            'hover:bg-gray-800': !tab.active
          }"
          :style="{ minWidth: tabMinWidth + 'px', maxWidth: tabMaxWidth + 'px' }"
          @click="selectTab(tab.id)"
        >
          <Terminal
            v-if="tabMinWidth >= 80"
            :size="14"
            class="mr-2 transition-colors duration-200 flex-shrink-0"
            :class="tab.active ? 'text-blue-400' : 'text-gray-400'"
          />
          <div
            v-if="tab.color && tabMinWidth >= 60"
            class="w-2 h-2 rounded-full mr-2 flex-shrink-0"
            :style="{ backgroundColor: tab.color }"
          ></div>
          <span
            class="text-sm truncate flex-1 transition-colors duration-200"
            :class="tab.active ? 'text-white' : 'text-gray-300'"
          >
            {{ tab.title }}
          </span>
          <X
            v-if="tabMinWidth >= 100"
            :size="14"
            class="text-gray-500 hover:text-red-400 ml-2 opacity-0 group-hover:opacity-100 transition-opacity flex-shrink-0"
            @click.stop="closeTab(tab.id)"
          />
        </div>

        <!-- Add Tab Button -->
        <div
          class="no-drag flex items-center justify-center w-8 h-full hover:bg-gray-800 cursor-pointer flex-shrink-0"
          :title="`Add new tab (${tabs.length} tabs)`"
          @click="addTab"
        >
          <Plus :size="14" class="text-gray-400 hover:text-white" />
        </div>
      </div>

      <!-- Draggable space - Always visible with minimum 80px width -->
      <div class="draggable flex-1 h-full" style="min-width: 80px"></div>
    </div>

    <!-- Window Controls -->
    <div class="flex items-center h-full max-h-[30px] flex-shrink-0">
      <div
        class="no-drag flex items-center justify-center w-8 h-full max-h-[30px] hover:bg-gray-800 cursor-pointer"
        @click="minimizeWindow"
      >
        <Minus :size="14" class="text-gray-400 hover:text-white" />
      </div>
      <div
        class="no-drag flex items-center justify-center w-8 h-full max-h-[30px] hover:bg-gray-800 cursor-pointer"
        @click="maximizeWindow"
      >
        <component
          :is="isMaximized ? Minimize2 : Maximize2"
          :size="14"
          class="text-gray-400 hover:text-white"
        />
      </div>
      <div
        class="no-drag flex items-center justify-center w-8 h-full max-h-[30px] hover:bg-red-600 cursor-pointer"
        @click="closeWindow"
      >
        <X :size="14" class="text-gray-400 hover:text-white" />
      </div>
    </div>
  </div>
</template>

<script setup lang="ts">
import { ref, onMounted, onBeforeUnmount, computed } from 'vue'
import {
  LayoutDashboard,
  Terminal,
  Plus,
  X,
  Minus,
  Minimize2,
  Maximize2,
  PanelLeft
} from 'lucide-vue-next'

interface Tab {
  id: string
  title: string
  active: boolean
  color?: string
}

interface Props {
  isDashboardActive?: boolean
  tabs?: Tab[]
}

const { isDashboardActive = false, tabs = [] } = defineProps<Props>()

const emit = defineEmits<{
  'open-dashboard': []
  'open-terminal': []
  'add-tab': []
  'close-tab': [tabId: string]
  'select-tab': [tabId: string]
  'toggle-ssh-drawer': []
}>()

const isMaximized = ref(false)
const windowWidth = ref(window.innerWidth)

let removeMaximizedListener: (() => void) | null = null

// Update window width on resize
const updateWindowWidth = (): void => {
  windowWidth.value = window.innerWidth
}

// Computed properties for responsive tab sizing
const tabMinWidth = computed(() => {
  const tabCount = tabs.length
  const addButtonWidth = 32 // w-8 = 32px for the + button
  const sideIconsWidth = 48 // Dashboard + SSH icons (24px each)
  const windowControlsWidth = 96 // 3 window control buttons (32px each)
  const draggableSpaceWidth = 80 // Minimum draggable space width
  const availableWidth =
    windowWidth.value -
    sideIconsWidth -
    windowControlsWidth -
    addButtonWidth -
    draggableSpaceWidth -
    16 // 16px padding

  // Calculate ideal width per tab
  const idealTabWidth = Math.floor(availableWidth / Math.max(tabCount, 1))

  // Apply breakpoints based on available space and tab count
  if (tabCount <= 4 && idealTabWidth >= 180) return 180 // Full width when few tabs
  if (tabCount <= 6 && idealTabWidth >= 150) return 150 // Medium width
  if (tabCount <= 8 && idealTabWidth >= 120) return 120 // Smaller width
  if (tabCount <= 10 && idealTabWidth >= 100) return 100 // Even smaller

  // No minimum limit - let tabs shrink as needed to fit all tabs
  return Math.max(idealTabWidth, 20) // Only ensure some minimal readable space (20px)
})

const tabMaxWidth = computed(() => {
  const tabCount = tabs.length
  if (tabCount <= 4) return 180
  return 200 // Allow some expansion but not too much
})

onMounted(() => {
  // Listen for maximize state changes from main process
  if (window.api?.on) {
    removeMaximizedListener = window.api.on('window-maximized', (...args: unknown[]) => {
      const maximized = args[0] as boolean
      isMaximized.value = maximized
    })
  }

  // Listen for window resize
  window.addEventListener('resize', updateWindowWidth)
})

onBeforeUnmount(() => {
  if (removeMaximizedListener) {
    removeMaximizedListener()
  }
  // Remove resize listener
  window.removeEventListener('resize', updateWindowWidth)
})

const addTab = (): void => {
  emit('add-tab')
}

const closeTab = (tabId: string): void => {
  emit('close-tab', tabId)
}

const selectTab = (tabId: string): void => {
  emit('select-tab', tabId)
}

const openDashboard = (): void => {
  emit('open-dashboard')
}

const toggleSSHDrawer = (): void => {
  emit('toggle-ssh-drawer')
}

const minimizeWindow = (): void => {
  if (window.api?.send) {
    window.api.send('window-minimize', {})
  }
}

const maximizeWindow = (): void => {
  if (window.api?.send) {
    window.api.send('window-maximize', {})
  }
}

const closeWindow = (): void => {
  if (window.api?.send) {
    window.api.send('window-close', {})
  }
}
</script>

<style scoped>
.title-bar {
  -webkit-app-region: no-drag;
}

.draggable {
  -webkit-app-region: drag;
}

.no-drag {
  -webkit-app-region: no-drag;
}
</style>
