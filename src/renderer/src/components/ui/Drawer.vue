<template>
  <Teleport to="body">
    <!-- Overlay -->
    <Transition
      enter-active-class="transition-opacity duration-300"
      enter-from-class="opacity-0"
      enter-to-class="opacity-100"
      leave-active-class="transition-opacity duration-300"
      leave-from-class="opacity-100"
      leave-to-class="opacity-0"
    >
      <div v-if="visible" class="fixed inset-0 bg-black/50 z-40" @click="handleOverlayClick"></div>
    </Transition>

    <!-- Drawer -->
    <Transition
      :enter-active-class="`transition-transform duration-300 ease-out`"
      :enter-from-class="position === 'left' ? '-translate-x-full' : 'translate-x-full'"
      :enter-to-class="'translate-x-0'"
      :leave-active-class="`transition-transform duration-300 ease-in`"
      :leave-from-class="'translate-x-0'"
      :leave-to-class="position === 'left' ? '-translate-x-full' : 'translate-x-full'"
    >
      <div
        v-if="visible"
        class="fixed top-0 bottom-0 z-50 bg-[#1a1a1a] border-gray-700 flex flex-col"
        :class="[position === 'left' ? 'left-0 border-r' : 'right-0 border-l', widthClass]"
      >
        <!-- Header -->
        <div class="flex items-center justify-between p-4 border-b border-gray-700 flex-shrink-0">
          <div class="flex items-center space-x-3">
            <div
              v-if="icon"
              class="flex items-center justify-center w-8 h-8 rounded-lg"
              :class="iconBackground"
            >
              <component :is="icon" :size="20" :class="iconColor" />
            </div>
            <h2 class="text-lg font-semibold text-white">{{ title }}</h2>
          </div>
          <Button variant="ghost" :icon="X" @click="close" />
        </div>

        <!-- Content -->
        <div class="flex-1 overflow-hidden">
          <slot />
        </div>

        <!-- Footer -->
        <div v-if="$slots.footer" class="border-t border-gray-700 p-4 flex-shrink-0">
          <slot name="footer" />
        </div>
      </div>
    </Transition>
  </Teleport>
</template>

<script setup lang="ts">
import { computed, watch } from 'vue'
import { X } from 'lucide-vue-next'
import Button from './Button.vue'
import type { Component } from 'vue'

interface Props {
  visible?: boolean
  title?: string
  position?: 'left' | 'right'
  width?: 'sm' | 'md' | 'lg' | 'xl' | '2xl'
  icon?: Component
  iconBackground?: string
  iconColor?: string
  closeOnOverlay?: boolean
}

const props = withDefaults(defineProps<Props>(), {
  visible: false,
  title: '',
  position: 'left',
  width: 'md',
  iconBackground: 'bg-gray-700',
  iconColor: 'text-gray-300',
  closeOnOverlay: true
})

const emit = defineEmits<{
  'update:visible': [visible: boolean]
  close: []
}>()

const widthClass = computed(() => {
  const widthMap = {
    sm: 'w-80',
    md: 'w-96',
    lg: 'w-[32rem]',
    xl: 'w-[36rem]',
    '2xl': 'w-[42rem]'
  }
  return widthMap[props.width]
})

const close = (): void => {
  emit('update:visible', false)
  emit('close')
}

const handleOverlayClick = (): void => {
  if (props.closeOnOverlay) {
    close()
  }
}

// Handle ESC key
watch(
  () => props.visible,
  (isVisible) => {
    if (isVisible) {
      document.addEventListener('keydown', handleKeydown)
    } else {
      document.removeEventListener('keydown', handleKeydown)
    }
  }
)

const handleKeydown = (event: KeyboardEvent): void => {
  if (event.key === 'Escape') {
    close()
  }
}
</script>
