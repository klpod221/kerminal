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

    <!-- Tabs Container -->
    <div class="flex items-center flex-1 h-full max-h-[30px] min-w-0">
      <div class="flex items-center h-full max-h-[30px] overflow-hidden">
        <div
          v-for="tab in tabs"
          :key="tab.id"
          class="no-drag flex items-center px-3 h-full max-h-[30px] border-r border-gray-800 cursor-pointer w-[180px] group transition-colors duration-200 flex-shrink-0"
          :class="{
            'bg-[#171717] border-b-2 border-b-blue-500': tab.active,
            'hover:bg-gray-800': !tab.active
          }"
          @click="selectTab(tab.id)"
        >
          <Terminal
            :size="14"
            class="mr-2 transition-colors duration-200 flex-shrink-0"
            :class="tab.active ? 'text-blue-400' : 'text-gray-400'"
          />
          <span
            class="text-sm truncate flex-1 transition-colors duration-200"
            :class="tab.active ? 'text-white' : 'text-gray-300'"
          >
            {{ tab.title }}
          </span>
          <X
            :size="14"
            class="text-gray-500 hover:text-red-400 ml-2 opacity-0 group-hover:opacity-100 transition-opacity flex-shrink-0"
            @click.stop="closeTab(tab.id)"
          />
        </div>

        <!-- Add Tab Button -->
        <div
          class="no-drag flex items-center justify-center w-8 h-full hover:bg-gray-800 cursor-pointer flex-shrink-0"
          @click="addTab"
        >
          <Plus :size="14" class="text-gray-400 hover:text-white" />
        </div>
      </div>

      <!-- Draggable space -->
      <div class="draggable flex-1 h-full min-w-4"></div>
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
import { ref, onMounted, onBeforeUnmount } from 'vue'
import { LayoutDashboard, Terminal, Plus, X, Minus, Minimize2, Maximize2 } from 'lucide-vue-next'

interface Tab {
  id: string
  title: string
  active: boolean
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
}>()

const isMaximized = ref(false)

let removeMaximizedListener: (() => void) | null = null

onMounted(() => {
  // Listen for maximize state changes from main process
  if (window.api?.on) {
    removeMaximizedListener = window.api.on('window-maximized', (...args: unknown[]) => {
      const maximized = args[0] as boolean
      isMaximized.value = maximized
    })
  }
})

onBeforeUnmount(() => {
  if (removeMaximizedListener) {
    removeMaximizedListener()
  }
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
