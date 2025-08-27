<template>
  <div class="relative inline-block">
    <!-- Trigger Element -->
    <div
      ref="triggerRef"
      @click.stop="handleTriggerClick"
      @mouseenter="handleMouseEnter"
      @mouseleave="handleMouseLeave"
    >
      <slot />
    </div>

    <!-- PopConfirm Overlay -->
    <Teleport to="body">
      <div v-if="visible" ref="popconfirmRef" :style="popconfirmStyle" class="fixed z-50">
        <Transition
          enter-active-class="transition-all duration-200 ease-out"
          enter-from-class="opacity-0 transform scale-95"
          enter-to-class="opacity-100 transform scale-100"
          leave-active-class="transition-all duration-150 ease-in"
          leave-from-class="opacity-100 transform scale-100"
          leave-to-class="opacity-0 transform scale-95"
        >
          <div
            v-if="visible"
            class="bg-[#1f1f1f] border border-gray-600 rounded-lg shadow-lg p-4 min-w-[280px] max-w-[400px]"
          >
            <!-- Header with Icon and Title -->
            <div class="flex items-start space-x-3 mb-3">
              <div class="flex-shrink-0 mt-0.5">
                <component :is="iconComponent" :class="iconClass" class="w-5 h-5" />
              </div>
              <div class="flex-1 min-w-0">
                <div v-if="title" class="text-sm font-medium text-white mb-1">
                  {{ title }}
                </div>
                <div class="text-sm text-gray-300 leading-5">
                  {{ content }}
                </div>
              </div>
            </div>

            <!-- Action Buttons -->
            <div class="flex justify-end space-x-2">
              <button
                type="button"
                class="px-3 py-1.5 text-sm text-gray-400 hover:text-white transition-colors rounded border border-gray-600 hover:border-gray-500"
                @click="handleCancel"
              >
                {{ cancelText }}
              </button>
              <button
                type="button"
                :class="confirmButtonClass"
                class="px-3 py-1.5 text-sm font-medium rounded transition-colors"
                @click="handleConfirm"
              >
                {{ okText }}
              </button>
            </div>
          </div>
        </Transition>
      </div>
    </Teleport>

    <!-- Backdrop -->
    <Teleport to="body">
      <div v-if="visible" class="fixed inset-0 z-40" @click="handleBackdropClick" />
    </Teleport>
  </div>
</template>

<script setup lang="ts">
import { ref, computed, nextTick, onMounted, onUnmounted } from 'vue'

interface Props {
  title?: string
  content: string
  okText?: string
  cancelText?: string
  okType?: 'primary' | 'danger'
  trigger?: 'click' | 'hover'
  placement?:
    | 'top'
    | 'bottom'
    | 'left'
    | 'right'
    | 'topLeft'
    | 'topRight'
    | 'bottomLeft'
    | 'bottomRight'
  disabled?: boolean
}

const props = withDefaults(defineProps<Props>(), {
  okText: 'OK',
  cancelText: 'Cancel',
  okType: 'primary',
  trigger: 'click',
  placement: 'top',
  disabled: false
})

const emit = defineEmits<{
  confirm: []
  cancel: []
  visibleChange: [visible: boolean]
}>()

// Refs
const triggerRef = ref<HTMLElement>()
const popconfirmRef = ref<HTMLElement>()
const visible = ref(false)
const popconfirmStyle = ref({})

// Computed
const iconComponent = computed(() => {
  return {
    template: `
      <svg fill="none" stroke="currentColor" viewBox="0 0 24 24">
        <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M12 9v2m0 4h.01m-6.938 4h13.856c1.54 0 2.502-1.667 1.732-2.5L13.732 4c-.77-.833-1.964-.833-2.732 0L3.732 16.5c-.77.833.192 2.5 1.732 2.5z" />
      </svg>
    `
  }
})

const iconClass = computed(() => {
  if (props.okType === 'danger') {
    return 'text-red-400'
  } else {
    return 'text-yellow-400'
  }
})

const confirmButtonClass = computed(() => {
  if (props.okType === 'danger') {
    return 'bg-red-600 hover:bg-red-700 text-white'
  } else {
    return 'bg-blue-600 hover:bg-blue-700 text-white'
  }
})

// Methods
const calculatePosition = async (): Promise<void> => {
  if (!triggerRef.value || !popconfirmRef.value) return

  await nextTick()

  const triggerRect = triggerRef.value.getBoundingClientRect()
  const popconfirmRect = popconfirmRef.value.getBoundingClientRect()
  const viewport = {
    width: window.innerWidth,
    height: window.innerHeight
  }

  let top = 0
  let left = 0

  switch (props.placement) {
    case 'top':
      top = triggerRect.top - popconfirmRect.height - 8
      left = triggerRect.left + (triggerRect.width - popconfirmRect.width) / 2
      break
    case 'bottom':
      top = triggerRect.bottom + 8
      left = triggerRect.left + (triggerRect.width - popconfirmRect.width) / 2
      break
    case 'left':
      top = triggerRect.top + (triggerRect.height - popconfirmRect.height) / 2
      left = triggerRect.left - popconfirmRect.width - 8
      break
    case 'right':
      top = triggerRect.top + (triggerRect.height - popconfirmRect.height) / 2
      left = triggerRect.right + 8
      break
    case 'topLeft':
      top = triggerRect.top - popconfirmRect.height - 8
      left = triggerRect.left
      break
    case 'topRight':
      top = triggerRect.top - popconfirmRect.height - 8
      left = triggerRect.right - popconfirmRect.width
      break
    case 'bottomLeft':
      top = triggerRect.bottom + 8
      left = triggerRect.left
      break
    case 'bottomRight':
      top = triggerRect.bottom + 8
      left = triggerRect.right - popconfirmRect.width
      break
  }

  // Adjust position to stay within viewport
  if (left < 8) left = 8
  if (left + popconfirmRect.width > viewport.width - 8) {
    left = viewport.width - popconfirmRect.width - 8
  }
  if (top < 8) top = 8
  if (top + popconfirmRect.height > viewport.height - 8) {
    top = viewport.height - popconfirmRect.height - 8
  }

  popconfirmStyle.value = {
    top: `${top}px`,
    left: `${left}px`
  }
}

const show = async (): Promise<void> => {
  if (props.disabled) return

  visible.value = true
  emit('visibleChange', true)
  await nextTick()
  await calculatePosition()
}

const hide = (): void => {
  visible.value = false
  emit('visibleChange', false)
}

const handleTriggerClick = (): void => {
  if (props.trigger === 'click') {
    if (visible.value) {
      hide()
    } else {
      show()
    }
  }
}

const handleMouseEnter = (): void => {
  if (props.trigger === 'hover') {
    show()
  }
}

const handleMouseLeave = (): void => {
  if (props.trigger === 'hover') {
    // Add small delay for hover trigger
    setTimeout(() => {
      if (props.trigger === 'hover') {
        hide()
      }
    }, 100)
  }
}

const handleConfirm = (): void => {
  emit('confirm')
  hide()
}

const handleCancel = (): void => {
  emit('cancel')
  hide()
}

const handleBackdropClick = (): void => {
  hide()
}

const handleKeydown = (event: KeyboardEvent): void => {
  if (visible.value && event.key === 'Escape') {
    handleCancel()
  }
}

// Lifecycle
onMounted(() => {
  document.addEventListener('keydown', handleKeydown)
  window.addEventListener('resize', calculatePosition)
})

onUnmounted(() => {
  document.removeEventListener('keydown', handleKeydown)
  window.removeEventListener('resize', calculatePosition)
})

// Expose methods
defineExpose({
  show,
  hide
})
</script>
