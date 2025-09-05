<template>
  <div
    class="no-drag flex items-center px-2 h-full max-h-[30px] border-r border-gray-800 cursor-pointer group transition-all duration-300 ease-out flex-1 relative overflow-hidden"
    :class="{
      'bg-[#171717] border-b-2 border-b-blue-500': isActive,
      'hover:bg-gray-800': !isActive
    }"
    :style="{ minWidth: minWidth + 'px', maxWidth: maxWidth + 'px' }"
    draggable="true"
    @click="$emit('select')"
    @dragstart="onDragStart"
    @dragover="onDragOver"
    @drop="onDrop"
  >
    <Terminal
      v-if="minWidth >= 80"
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
      {{ tab.title }}
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
import { Terminal, X } from 'lucide-vue-next'
import type { Tab } from '../../types/panel'

interface Props {
  tab: Tab
  isActive: boolean
  minWidth: number
  maxWidth: number
}

const props = defineProps<Props>()

const emit = defineEmits<{
  select: []
  close: []
  dragStart: [tab: Tab]
  drop: [draggedTab: Tab, targetTab: Tab]
}>()

const onDragStart = (event: DragEvent): void => {
  if (event.dataTransfer) {
    event.dataTransfer.setData('application/json', JSON.stringify(props.tab))
    event.dataTransfer.effectAllowed = 'move'
  }
  emit('dragStart', props.tab)
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
      const draggedTab = JSON.parse(draggedTabData) as Tab
      emit('drop', draggedTab, props.tab)
    }
  }
}
</script>

<style scoped>
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
