<template>
  <Teleport to="body">
    <Transition
      name="modal"
      enter-active-class="transition-all duration-300 ease-out"
      enter-from-class="opacity-0"
      enter-to-class="opacity-100"
      leave-active-class="transition-all duration-200 ease-in"
      leave-from-class="opacity-100"
      leave-to-class="opacity-0"
    >
      <div
        v-if="visible"
        class="fixed top-[30px] left-0 right-0 bottom-0 z-50 flex items-center justify-center bg-black/50 backdrop-blur-sm"
        @click="handleBackdropClick"
      >
        <Transition
          name="modal-content"
          enter-active-class="transition-all duration-300 ease-out"
          enter-from-class="opacity-0 scale-95 translate-y-4"
          enter-to-class="opacity-100 scale-100 translate-y-0"
          leave-active-class="transition-all duration-200 ease-in"
          leave-from-class="opacity-100 scale-100 translate-y-0"
          leave-to-class="opacity-0 scale-95 translate-y-4"
        >
          <div
            v-if="visible"
            class="relative bg-[#1a1a1a] border border-gray-700 rounded-lg shadow-2xl max-w-md w-full mx-4 max-h-[90vh] overflow-hidden"
            @click.stop
          >
            <!-- Header -->
            <div
              v-if="title || $slots.header || showCloseButton"
              class="flex items-center justify-between p-4 border-b border-gray-700"
            >
              <div class="flex items-center space-x-3">
                <div v-if="icon" class="rounded-lg p-2" :class="iconBackground || 'bg-blue-500/20'">
                  <component :is="icon" class="w-6 h-6" :class="iconColor || 'text-blue-400'" />
                </div>
                <div>
                  <h3 v-if="title" class="text-lg font-semibold text-white">{{ title }}</h3>
                  <slot name="header" />
                </div>
              </div>
              <Button
                v-if="showCloseButton"
                title="Close modal"
                variant="ghost"
                size="sm"
                :icon="X"
                @click="$emit('close')"
              />
            </div>

            <!-- Content -->
            <div class="p-4 overflow-y-auto max-h-[60vh]">
              <slot />
            </div>

            <!-- Footer -->
            <div
              v-if="$slots.footer"
              class="flex justify-end space-x-3 p-4 border-t border-gray-700 bg-[#171717]"
            >
              <slot name="footer" />
            </div>
          </div>
        </Transition>
      </div>
    </Transition>
  </Teleport>
</template>

<script setup lang="ts">
import { watch, onUnmounted } from 'vue'
import { X } from 'lucide-vue-next'
import Button from './Button.vue'
import type { Component } from 'vue'

interface Props {
  visible?: boolean
  title?: string
  icon?: Component
  iconBackground?: string
  iconColor?: string
  showCloseButton?: boolean
  closeOnBackdrop?: boolean
}

const props = withDefaults(defineProps<Props>(), {
  visible: false,
  showCloseButton: true,
  closeOnBackdrop: true
})

const emit = defineEmits(['close', 'update:visible'])

/**
 * Handle backdrop click to close modal
 */
function handleBackdropClick(): void {
  if (props.closeOnBackdrop) {
    emit('close')
    emit('update:visible', false)
  }
}

// Watch for visible prop changes
watch(
  () => props.visible,
  (isVisible) => {
    if (isVisible) {
      // Prevent body scroll when modal is open
      document.body.style.overflow = 'hidden'
      document.addEventListener('keydown', handleKeydown)
    } else {
      // Restore body scroll when modal is closed
      document.body.style.overflow = ''
      document.removeEventListener('keydown', handleKeydown)
    }
  }
)

const handleKeydown = (event: KeyboardEvent): void => {
  if (event.key === 'Escape') {
    emit('close')
    emit('update:visible', false)
  }
}

// Cleanup on unmount
onUnmounted(() => {
  document.body.style.overflow = ''
})
</script>

<style scoped>
.modal-enter-active,
.modal-leave-active {
  transition: opacity 0.3s ease;
}

.modal-enter-from,
.modal-leave-to {
  opacity: 0;
}

.modal-content-enter-active,
.modal-content-leave-active {
  transition: all 0.3s ease;
}

.modal-content-enter-from,
.modal-content-leave-to {
  opacity: 0;
  transform: scale(0.95) translateY(1rem);
}
</style>
