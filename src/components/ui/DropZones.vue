<template>
  <div
    v-show="showDropZones"
    class="drop-zones-container"
    @drop="onDrop"
    @dragover="onDragOver"
    @dragenter="onDragEnter"
    @dragleave="onDragLeave"
  >
    <!-- Top Drop Zone -->
    <div
      class="drop-zone drop-zone-top"
      :class="{ 'drop-zone-active': activeZone === 'top' }"
      @dragenter="setActiveZone('top')"
    >
      <div class="drop-zone-indicator">
        <div class="drop-zone-icon">⬆</div>
        <div class="drop-zone-text">Split Top</div>
      </div>
    </div>

    <!-- Bottom Drop Zone -->
    <div
      class="drop-zone drop-zone-bottom"
      :class="{ 'drop-zone-active': activeZone === 'bottom' }"
      @dragenter="setActiveZone('bottom')"
    >
      <div class="drop-zone-indicator">
        <div class="drop-zone-icon">⬇</div>
        <div class="drop-zone-text">Split Bottom</div>
      </div>
    </div>

    <!-- Left Drop Zone -->
    <div
      class="drop-zone drop-zone-left"
      :class="{ 'drop-zone-active': activeZone === 'left' }"
      @dragenter="setActiveZone('left')"
    >
      <div class="drop-zone-indicator">
        <div class="drop-zone-icon">⬅</div>
        <div class="drop-zone-text text-center">Split Left</div>
      </div>
    </div>

    <!-- Right Drop Zone -->
    <div
      class="drop-zone drop-zone-right"
      :class="{ 'drop-zone-active': activeZone === 'right' }"
      @dragenter="setActiveZone('right')"
    >
      <div class="drop-zone-indicator">
        <div class="drop-zone-icon">➡</div>
        <div class="drop-zone-text text-center">Split Right</div>
      </div>
    </div>

    <!-- Center Drop Zone (for moving tab to existing panel) -->
    <div
      class="drop-zone drop-zone-center"
      :class="{ 'drop-zone-active': activeZone === 'center' }"
      @dragenter="setActiveZone('center')"
    >
      <div class="drop-zone-indicator">
        <div class="drop-zone-icon">📁</div>
        <div class="drop-zone-text">Add to Panel</div>
      </div>
    </div>
  </div>
</template>

<script setup lang="ts">
import { ref } from 'vue'
import type { Tab } from '../../types/panel'

interface DropZonesProps {
  showDropZones: boolean
  panelId: string
}

interface DropZonesEmits {
  splitPanel: [direction: 'top' | 'bottom' | 'left' | 'right', draggedTab: Tab, sourcePanelId: string]
  moveTab: [draggedTab: Tab, sourcePanelId: string]
}

const props = defineProps<DropZonesProps>()
const emit = defineEmits<DropZonesEmits>()

// Active drop zone state
const activeZone = ref<'top' | 'bottom' | 'left' | 'right' | 'center' | null>(null)
let hideActiveZoneTimeout: ReturnType<typeof setTimeout> | null = null

/**
 * Set the active drop zone
 * @param {string} zone - The zone identifier
 */
const setActiveZone = (zone: 'top' | 'bottom' | 'left' | 'right' | 'center'): void => {
  // Clear any pending timeout
  if (hideActiveZoneTimeout) {
    clearTimeout(hideActiveZoneTimeout)
    hideActiveZoneTimeout = null
  }

  activeZone.value = zone
}

/**
 * Handle drag over event
 * @param {DragEvent} event - The drag event
 */
const onDragOver = (event: DragEvent): void => {
  event.preventDefault()
  if (event.dataTransfer) {
    event.dataTransfer.dropEffect = 'move'
  }
}

/**
 * Handle drag enter event
 * @param {DragEvent} event - The drag event
 */
const onDragEnter = (event: DragEvent): void => {
  event.preventDefault()
}

/**
 * Handle drag leave event
 * @param {DragEvent} event - The drag event
 */
const onDragLeave = (event: DragEvent): void => {
  // Only clear active zone if leaving the entire drop zones container
  const rect = (event.currentTarget as HTMLElement).getBoundingClientRect()
  const isLeavingContainer = (
    event.clientX < rect.left ||
    event.clientX > rect.right ||
    event.clientY < rect.top ||
    event.clientY > rect.bottom
  )

  if (isLeavingContainer) {
    // Add a small delay before clearing to prevent flickering
    hideActiveZoneTimeout = setTimeout(() => {
      activeZone.value = null
    }, 50)
  }
}

/**
 * Handle drop event
 * @param {DragEvent} event - The drop event
 */
const onDrop = (event: DragEvent): void => {
  event.preventDefault()

  if (event.dataTransfer) {
    const draggedTabData = event.dataTransfer.getData('application/json')
    if (draggedTabData) {
      try {
        const dragData = JSON.parse(draggedTabData)
        const draggedTab = dragData.tab as Tab
        const sourcePanelId = dragData.sourcePanelId as string

        if (activeZone.value && activeZone.value !== 'center') {
          // Split panel in the specified direction
          emit('splitPanel', activeZone.value, draggedTab, sourcePanelId)
        } else if (activeZone.value === 'center') {
          // Move tab to existing panel (only if from different panel)
          if (sourcePanelId !== props.panelId) {
            emit('moveTab', draggedTab, sourcePanelId)
          }
        }
      } catch (error) {
        console.error('Error parsing dragged tab data:', error)
      }
    }
  }

  // Reset active zone
  if (hideActiveZoneTimeout) {
    clearTimeout(hideActiveZoneTimeout)
    hideActiveZoneTimeout = null
  }
  activeZone.value = null
}
</script>

<style scoped>
.drop-zones-container {
  position: absolute;
  top: 32px; /* Start below TabBar */
  left: 0;
  right: 0;
  bottom: 0;
  z-index: 500;
  pointer-events: auto;
}

.drop-zone {
  position: absolute;
  background: rgba(59, 130, 246, 0.2);
  border: 2px dashed rgba(59, 130, 246, 0.6);
  border-radius: 8px;
  display: flex;
  align-items: center;
  justify-content: center;
  transition: all 0.2s ease;
  opacity: 0.9;
}

.drop-zone-active {
  background: rgba(59, 130, 246, 0.4);
  border-color: rgba(59, 130, 246, 1);
  opacity: 1;
  transform: scale(1.02);
}

.drop-zone-top {
  top: 0;
  left: 20%;
  right: 20%;
  height: 80px; /* Restored original height */
}

.drop-zone-bottom {
  bottom: 8px;
  left: 20%;
  right: 20%;
  height: 80px;
}

.drop-zone-left {
  left: 8px;
  top: 20%;
  bottom: 20%;
  width: 80px;
}

.drop-zone-right {
  right: 8px;
  top: 20%;
  bottom: 20%;
  width: 80px;
}

.drop-zone-center {
  top: 30%;
  bottom: 30%;
  left: 30%;
  right: 30%;
  min-width: 120px;
  min-height: 80px;
}

.drop-zone-indicator {
  display: flex;
  flex-direction: column;
  align-items: center;
  justify-content: center;
  color: rgba(255, 255, 255, 0.9);
  font-weight: 500;
  text-shadow: 0 1px 2px rgba(0, 0, 0, 0.5);
}

.drop-zone-icon {
  font-size: 24px;
  margin-bottom: 4px;
}

.drop-zone-text {
  font-size: 12px;
  text-transform: uppercase;
  letter-spacing: 0.5px;
  background: rgba(0, 0, 0, 0.5);
  padding: 2px 6px;
  border-radius: 4px;
}
</style>
