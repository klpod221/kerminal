<template>
  <div
    class="no-drag flex items-center px-2 h-full max-h-[30px] border-r border-gray-800 cursor-pointer group transition-all duration-300 ease-out flex-1 relative overflow-hidden"
    :class="{
      'bg-[#171717] border-b-2 border-b-blue-500': isActive,
      'hover:bg-gray-800': !isActive,
      'opacity-50': isDragging
    }"
    :style="{ minWidth: minWidth + 'px', maxWidth: maxWidth + 'px' }"
    draggable="true"
    @click="$emit('select')"
    @dragstart="onDragStart"
    @dragend="onDragEnd"
    @dragover="onDragOver"
    @drop="onDrop"
  >
    <div
      v-if="isConnecting && minWidth >= 80"
      class="mr-2 transition-colors duration-200 flex-shrink-0"
    >
      <div
        class="animate-spin rounded-full h-3 w-3 border border-blue-400 border-t-transparent"
      ></div>
    </div>
    <Terminal
      v-else-if="minWidth >= 80"
      :size="14"
      class="mr-2 transition-colors duration-200 flex-shrink-0"
      :class="isActive ? 'text-blue-400' : 'text-gray-400'"
    />
    <div
      v-if="tab.color && minWidth >= 60"
      class="w-2 h-2 rounded-full mr-2 flex-shrink-0"
      :style="{ backgroundColor: tab.color }"
    ></div>
    <span
      class="text-sm truncate flex-1 transition-colors duration-200"
      :class="isActive ? 'text-white' : 'text-gray-300'"
    >
      {{ isConnecting ? 'Connecting...' : tab.title }}
    </span>
    <X
      v-if="minWidth >= 100"
      :size="14"
      class="text-gray-500 hover:text-red-400 ml-2 opacity-0 group-hover:opacity-100 transition-all duration-300 ease-out flex-shrink-0 transform hover:scale-110"
      @click.stop="$emit('close')"
    />
  </div>
</template>

<script setup lang="ts">
import { ref } from 'vue'
import { Terminal, X } from 'lucide-vue-next'
import type { TabProps, TabEmits } from '../../types/ui'
import type { Tab } from '../../types/panel'

const props = withDefaults(defineProps<TabProps>(), {
  isConnecting: false
})

const emit = defineEmits<TabEmits>()

// Drag state
const isDragging = ref(false)

const onDragStart = (event: DragEvent): void => {
  isDragging.value = true
  if (event.dataTransfer) {
    // Store both tab data and source panel info
    const dragData = {
      tab: props.tab,
      sourcePanelId: props.panelId
    }
    event.dataTransfer.setData('application/json', JSON.stringify(dragData))
    event.dataTransfer.effectAllowed = 'move'
  }
  emit('dragStart', props.tab)
}

const onDragEnd = (): void => {
  isDragging.value = false
}

const onDragOver = (event: DragEvent): void => {
  event.preventDefault()
  if (event.dataTransfer) {
    event.dataTransfer.dropEffect = 'move'
  }
}

const onDrop = (event: DragEvent): void => {
  event.preventDefault()
  if (event.dataTransfer) {
    const draggedTabData = event.dataTransfer.getData('application/json')
    if (draggedTabData) {
      try {
        const dragData = JSON.parse(draggedTabData)
        const draggedTab = dragData.tab as Tab
        emit('drop', draggedTab, props.tab)
      } catch (error) {
        console.error('Error parsing dragged tab data:', error)
      }
    }
  }
}
</script>

<style scoped>
/* Drag state styles */
.group.opacity-50 {
  transform: rotate(2deg) scale(0.98);
  box-shadow: 0 4px 12px rgba(0, 0, 0, 0.3);
}

/* Add a subtle shimmer effect for new tabs */
@keyframes shimmer {
  0% {
    background-position: -200px 0;
  }
  100% {
    background-position: calc(200px + 100%) 0;
  }
}

.group::before {
  content: '';
  position: absolute;
  top: 0;
  left: -100%;
  width: 100%;
  height: 100%;
  background: linear-gradient(90deg, transparent, rgba(255, 255, 255, 0.1), transparent);
  transition: left 0.5s;
}

.group:hover::before {
  left: 100%;
}

/* Enhanced active tab indicator */
.group.bg-\[#171717\]::after {
  content: '';
  position: absolute;
  bottom: 0;
  left: 0;
  right: 0;
  height: 2px;
  background: linear-gradient(90deg, #3b82f6, #06b6d4, #3b82f6);
  background-size: 200% 100%;
  animation: gradientShift 2s ease-in-out infinite;
}

@keyframes gradientShift {
  0%,
  100% {
    background-position: 0% 50%;
  }
  50% {
    background-position: 100% 50%;
  }
}
</style>
