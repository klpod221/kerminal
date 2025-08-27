<!-- Toast notification component -->
<template>
  <Teleport to="body">
    <div
      v-if="visible"
      :class="[
        'fixed z-50 flex items-center w-full max-w-xs p-4 mb-4 text-gray-100',
        'bg-gray-800 rounded-lg shadow-lg border border-gray-700',
        'transition-all duration-300 ease-out',
        positionClasses,
        visible ? 'translate-x-0 opacity-100' : 'translate-x-full opacity-0'
      ]"
      role="alert"
    >
      <!-- Icon -->
      <div
        :class="[
          'inline-flex items-center justify-center flex-shrink-0 w-8 h-8 rounded-lg mr-3',
          iconBackgroundClasses
        ]"
      >
        <component :is="iconComponent" :size="16" :class="iconColorClasses" />
      </div>

      <!-- Content -->
      <div class="text-sm font-normal flex-1 min-w-0">
        <div v-if="title" class="font-semibold mb-1 truncate">{{ title }}</div>
        <div class="text-gray-300 truncate">{{ message }}</div>
      </div>

      <!-- Close button -->
      <button
        v-if="closable"
        type="button"
        class="ml-3 -mx-1.5 -my-1.5 bg-gray-800 text-gray-400 hover:text-gray-200 rounded-lg focus:ring-2 focus:ring-gray-600 p-1.5 hover:bg-gray-700 inline-flex h-8 w-8 transition-colors"
        @click="handleClose"
      >
        <X :size="16" />
        <span class="sr-only">Close</span>
      </button>
    </div>
  </Teleport>
</template>

<script setup lang="ts">
import { computed, onMounted, ref } from 'vue'
import { CheckCircle, AlertCircle, XCircle, Info, X } from 'lucide-vue-next'

interface Props {
  visible?: boolean
  type?: 'success' | 'warning' | 'error' | 'info'
  title?: string
  message: string
  position?: 'top-right' | 'top-left' | 'bottom-right' | 'bottom-left'
  duration?: number
  closable?: boolean
}

interface Emits {
  close: []
}

const props = withDefaults(defineProps<Props>(), {
  visible: false,
  type: 'info',
  position: 'top-right',
  duration: 4000,
  closable: true
})

const emit = defineEmits<Emits>()

const timeoutId = ref<number | null>(null)

const iconComponent = computed(() => {
  switch (props.type) {
    case 'success':
      return CheckCircle
    case 'warning':
      return AlertCircle
    case 'error':
      return XCircle
    default:
      return Info
  }
})

const iconBackgroundClasses = computed(() => {
  switch (props.type) {
    case 'success':
      return 'bg-green-800'
    case 'warning':
      return 'bg-yellow-800'
    case 'error':
      return 'bg-red-800'
    default:
      return 'bg-blue-800'
  }
})

const iconColorClasses = computed(() => {
  switch (props.type) {
    case 'success':
      return 'text-green-300'
    case 'warning':
      return 'text-yellow-300'
    case 'error':
      return 'text-red-300'
    default:
      return 'text-blue-300'
  }
})

const positionClasses = computed(() => {
  switch (props.position) {
    case 'top-left':
      return 'top-5 left-5'
    case 'bottom-right':
      return 'bottom-5 right-5'
    case 'bottom-left':
      return 'bottom-5 left-5'
    default:
      return 'top-5 right-5'
  }
})

const handleClose = (): void => {
  if (timeoutId.value) {
    clearTimeout(timeoutId.value)
    timeoutId.value = null
  }
  emit('close')
}

onMounted(() => {
  if (props.visible && props.duration > 0) {
    timeoutId.value = window.setTimeout(() => {
      handleClose()
    }, props.duration)
  }
})
</script>
