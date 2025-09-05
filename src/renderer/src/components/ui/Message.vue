<template>
  <Teleport to="body">
    <div v-if="visible" class="fixed top-4 left-1/2 transform -translate-x-1/2 z-50">
      <Transition
        enter-active-class="transition-all duration-300 ease-out"
        enter-from-class="opacity-0 transform -translate-y-2 scale-95"
        enter-to-class="opacity-100 transform translate-y-0 scale-100"
        leave-active-class="transition-all duration-200 ease-in"
        leave-from-class="opacity-100 transform translate-y-0 scale-100"
        leave-to-class="opacity-0 transform -translate-y-2 scale-95"
      >
        <div
          v-if="visible"
          :class="[
            'flex items-center gap-2 px-4 py-3 rounded-lg shadow-lg border backdrop-blur-xs',
            messageClasses
          ]"
        >
          <!-- Icon -->
          <div :class="iconClasses">
            <component :is="iconComponent" class="w-4 h-4" />
          </div>

          <!-- Content -->
          <div class="flex-1">
            <div v-if="title" class="font-medium text-sm">{{ title }}</div>
            <div class="text-sm" :class="{ 'mt-1': title }">{{ content }}</div>
          </div>

          <!-- Close button -->
          <button
            v-if="closable"
            class="ml-2 text-gray-400 hover:text-gray-600 transition-colors"
            @click="close"
          >
            <svg class="w-4 h-4" fill="none" stroke="currentColor" viewBox="0 0 24 24">
              <path
                stroke-linecap="round"
                stroke-linejoin="round"
                stroke-width="2"
                d="M6 18L18 6M6 6l12 12"
              />
            </svg>
          </button>
        </div>
      </Transition>
    </div>
  </Teleport>
</template>

<script setup lang="ts">
import { computed, onMounted, ref } from 'vue'
import type { MessageProps } from '../../types/ui'

const props = withDefaults(defineProps<MessageProps>(), {
  type: 'info',
  duration: 3000,
  closable: true
})

const visible = ref(false)
let timer: number | null = null

const messageClasses = computed(() => {
  switch (props.type) {
    case 'success':
      return 'bg-green-50/90 border-green-200 text-green-800'
    case 'error':
      return 'bg-red-50/90 border-red-200 text-red-800'
    case 'warning':
      return 'bg-yellow-50/90 border-yellow-200 text-yellow-800'
    case 'loading':
      return 'bg-blue-50/90 border-blue-200 text-blue-800'
    default:
      return 'bg-blue-50/90 border-blue-200 text-blue-800'
  }
})

const iconClasses = computed(() => {
  switch (props.type) {
    case 'success':
      return 'text-green-500'
    case 'error':
      return 'text-red-500'
    case 'warning':
      return 'text-yellow-500'
    case 'loading':
      return 'text-blue-500 animate-spin'
    default:
      return 'text-blue-500'
  }
})

const iconComponent = computed(() => {
  switch (props.type) {
    case 'success':
      return {
        template: `
          <svg fill="none" stroke="currentColor" viewBox="0 0 24 24">
            <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M9 12l2 2 4-4m6 2a9 9 0 11-18 0 9 9 0 0118 0z" />
          </svg>
        `
      }
    case 'error':
      return {
        template: `
          <svg fill="none" stroke="currentColor" viewBox="0 0 24 24">
            <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M10 14l2-2m0 0l2-2m-2 2l-2-2m2 2l2 2m7-2a9 9 0 11-18 0 9 9 0 0118 0z" />
          </svg>
        `
      }
    case 'warning':
      return {
        template: `
          <svg fill="none" stroke="currentColor" viewBox="0 0 24 24">
            <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M12 9v2m0 4h.01m-6.938 4h13.856c1.54 0 2.502-1.667 1.732-2.5L13.732 4c-.77-.833-1.964-.833-2.732 0L3.732 16.5c-.77.833.192 2.5 1.732 2.5z" />
          </svg>
        `
      }
    case 'loading':
      return {
        template: `
          <svg fill="none" stroke="currentColor" viewBox="0 0 24 24">
            <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M16.023 9.348h4.992v-.001M2.985 19.644v-4.992m0 0h4.992m-4.993 0l3.181 3.183a8.25 8.25 0 0013.803-3.7M4.031 9.865a8.25 8.25 0 0113.803-3.7l3.181 3.182m0-4.991v4.99" />
          </svg>
        `
      }
    default:
      return {
        template: `
          <svg fill="none" stroke="currentColor" viewBox="0 0 24 24">
            <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M13 16h-1v-4h-1m1-4h.01M21 12a9 9 0 11-18 0 9 9 0 0118 0z" />
          </svg>
        `
      }
  }
})

const close = (): void => {
  visible.value = false
  if (timer) {
    clearTimeout(timer)
    timer = null
  }
  setTimeout(() => {
    props.onClose?.()
  }, 200)
}

onMounted(() => {
  visible.value = true

  if (props.duration > 0 && props.type !== 'loading') {
    timer = window.setTimeout(() => {
      close()
    }, props.duration)
  }
})

defineExpose({
  close
})
</script>
