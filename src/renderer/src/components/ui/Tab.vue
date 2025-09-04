<template>
  <div
    class="no-drag flex items-center px-2 h-full max-h-[30px] border-r border-gray-800 cursor-pointer group transition-all duration-200 flex-1"
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
      class="text-gray-500 hover:text-red-400 ml-2 opacity-0 group-hover:opacity-100 transition-opacity flex-shrink-0"
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
