<template>
  <div
    class="flex items-center h-[30px] min-h-[30px] max-h-[30px] border-b border-gray-800 relative bg-[#0D0D0D]"
  >
    <!-- Active panel TabBar overlay -->
    <div
      class="absolute inset-0 transition-opacity duration-200 pointer-events-none"
      :class="{
        'opacity-100 bg-gradient-to-r from-blue-900/20 to-transparent': isActive,
        'opacity-0': !isActive
      }"
    ></div>
    <!-- Tabs Container -->
    <div class="flex items-center flex-1 h-full max-h-[30px] min-w-0 relative z-10">
      <div class="flex items-center h-full max-h-[30px] overflow-hidden">
        <transition-group name="tab" tag="div" class="flex items-center h-full max-h-[30px]" appear>
          <Tab
            v-for="tab in panel.tabs"
            :key="tab.id"
            :tab="tab"
            :is-active="tab.id === panel.activeTabId"
            :min-width="tabMinWidth"
            :max-width="tabMaxWidth"
            @select="selectTab(tab.id)"
            @close="closeTab(tab.id)"
            @drag-start="onTabDragStart"
            @drop="onTabDrop"
          />
        </transition-group>
      </div>

      <!-- Add Tab Button -->
      <Button
        title="Add new tab"
        variant="ghost"
        size="sm"
        :icon="Plus"
        class="add-tab-btn"
        @click="addTab"
      />

      <!-- Spacer -->
      <div class="flex-1 h-full"></div>
    </div>

    <!-- Panel Controls -->
    <div class="flex items-center h-full max-h-[30px] flex-shrink-0 relative z-10">
      <!-- Split Horizontal Button -->
      <Button
        title="Split horizontal"
        variant="ghost"
        size="sm"
        :icon="SplitSquareHorizontal"
        @click="splitHorizontal"
      />

      <!-- Split Vertical Button -->
      <Button
        title="Split vertical"
        variant="ghost"
        size="sm"
        :icon="SplitSquareVertical"
        @click="splitVertical"
      />

      <!-- Close Panel Button -->
      <Button
        title="Close panel"
        variant="ghost"
        size="sm"
        :icon="X"
        class="hover:bg-red-600/20 hover:text-red-400"
        @click="closePanel"
      />
    </div>
  </div>
</template>

<script setup lang="ts">
import { computed } from 'vue'
import { Plus, SplitSquareHorizontal, SplitSquareVertical, X } from 'lucide-vue-next'
import Tab from './Tab.vue'
import Button from './Button.vue'
import type { Panel, Tab as TabType } from '../../types/panel'

interface Props {
  panel: Panel
  windowWidth: number
  isActive: boolean
}

const props = defineProps<Props>()

const emit = defineEmits<{
  selectTab: [panelId: string, tabId: string]
  closeTab: [panelId: string, tabId: string]
  addTab: [panelId: string]
  splitHorizontal: [panelId: string]
  splitVertical: [panelId: string]
  closePanel: [panelId: string]
  moveTab: [fromPanelId: string, toPanelId: string, tabId: string, targetTabId?: string]
}>()

// Computed properties for responsive tab sizing
const tabMinWidth = computed(() => {
  const tabCount = props.panel.tabs.length
  const addButtonWidth = 32 // Add button width
  const panelControlsWidth = 128 // Split + close buttons (updated for new button)
  const padding = 16
  const availableWidth = props.windowWidth - addButtonWidth - panelControlsWidth - padding

  // Calculate ideal width per tab
  const idealTabWidth = Math.floor(availableWidth / Math.max(tabCount, 1))

  // Apply breakpoints based on available space and tab count
  if (tabCount <= 4 && idealTabWidth >= 180) return 180
  if (tabCount <= 6 && idealTabWidth >= 150) return 150
  if (tabCount <= 8 && idealTabWidth >= 120) return 120
  if (tabCount <= 10 && idealTabWidth >= 100) return 100

  return Math.max(idealTabWidth, 20) // Minimum readable space
})

const tabMaxWidth = computed(() => {
  const tabCount = props.panel.tabs.length
  if (tabCount <= 4) return 180
  return 200
})

const selectTab = (tabId: string): void => {
  emit('selectTab', props.panel.id, tabId)
}

const closeTab = (tabId: string): void => {
  emit('closeTab', props.panel.id, tabId)
}

const addTab = (): void => {
  emit('addTab', props.panel.id)
}

const splitHorizontal = (): void => {
  emit('splitHorizontal', props.panel.id)
}

const splitVertical = (): void => {
  emit('splitVertical', props.panel.id)
}

const closePanel = (): void => {
  emit('closePanel', props.panel.id)
}

const onTabDragStart = (tab: TabType): void => {
  // Store the source panel info for drag operations
  console.log('Tab drag started:', tab, 'from panel:', props.panel.id)
}

const onTabDrop = (draggedTab: TabType, targetTab: TabType): void => {
  // Handle tab reordering within the same panel or moving between panels
  emit('moveTab', props.panel.id, props.panel.id, draggedTab.id, targetTab.id)
}
</script>

<style scoped>
/* Tab transition animations */
.tab-enter-active {
  transition: all 0.4s cubic-bezier(0.25, 0.8, 0.25, 1);
}

.tab-leave-active {
  transition: all 0.3s cubic-bezier(0.4, 0, 0.6, 1);
}

.tab-enter-from {
  opacity: 0;
  transform: scaleX(0.3) translateX(-30px);
  transform-origin: left center;
  max-width: 0;
}

.tab-enter-to {
  opacity: 1;
  transform: scaleX(1) translateX(0);
  transform-origin: left center;
}

.tab-leave-from {
  opacity: 1;
  transform: scaleX(1) translateX(0);
  transform-origin: right center;
}

.tab-leave-to {
  opacity: 0;
  transform: scaleX(0.3) translateX(30px);
  transform-origin: right center;
  max-width: 0;
}

.tab-move {
  transition: transform 0.4s cubic-bezier(0.25, 0.8, 0.25, 1);
}

/* Stagger effect for multiple tabs */
.tab-enter-active:nth-child(2) {
  transition-delay: 0.05s;
}

.tab-enter-active:nth-child(3) {
  transition-delay: 0.1s;
}

.tab-enter-active:nth-child(4) {
  transition-delay: 0.15s;
}

.tab-enter-active:nth-child(n + 5) {
  transition-delay: 0.2s;
}

/* Add Tab Button Animation */
:deep(.add-tab-btn) {
  position: relative;
  overflow: hidden;
}

:deep(.add-tab-btn):hover {
  transform: scale(1.1);
}

:deep(.add-tab-btn):active {
  transform: scale(0.95);
}

:deep(.add-tab-btn):hover::before {
  content: '';
  position: absolute;
  top: 50%;
  left: 50%;
  width: 0;
  height: 0;
  background: radial-gradient(circle, rgba(59, 130, 246, 0.3) 0%, transparent 70%);
  border-radius: 50%;
  transform: translate(-50%, -50%);
  animation: ripple 0.6s ease-out;
}

@keyframes ripple {
  to {
    width: 40px;
    height: 40px;
  }
}
</style>
