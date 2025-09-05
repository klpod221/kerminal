<template>
  <div
    :class="[
      'bg-gradient-to-br from-[#2a2a2a] to-[#1f1f1f] rounded-xl border border-gray-600',
      'transition-all duration-300',
      {
        'hover:border-gray-500 hover:shadow-lg cursor-pointer': hover,
        'hover:scale-105 transform': hover && scale,
        'p-4': !noPadding,
        'p-6': size === 'sm' && !noPadding,
        'p-8': size === 'lg' && !noPadding
      },
      customClass
    ]"
    @click="handleClick"
  >
    <!-- Header Section -->
    <div
      v-if="$slots.header || title || icon"
      :class="['mb-6', center ? 'flex flex-col items-center text-center' : 'flex items-center']"
    >
      <!-- Icon -->
      <div
        v-if="icon || $slots.icon"
        :class="[
          'rounded-lg p-3 transition-all duration-300',
          center ? 'w-fit mx-auto mb-3' : 'mr-4',
          iconBackground || 'bg-blue-500/20'
        ]"
        :style="hover && center ? 'transition: background-color 0.3s ease;' : ''"
      >
        <slot name="icon">
          <component :is="icon" :class="['w-8 h-8', iconColor || 'text-blue-400']" />
        </slot>
      </div>

      <!-- Title -->
      <div :class="center ? '' : 'flex-1'">
        <slot name="header">
          <h3
            v-if="title"
            :class="[
              'font-semibold',
              size === 'sm' ? 'text-lg' : size === 'lg' ? 'text-2xl' : 'text-xl'
            ]"
          >
            {{ title }}
          </h3>
        </slot>
      </div>

      <!-- Action Slot -->
      <div v-if="$slots.action && !center" class="ml-4">
        <slot name="action" />
      </div>
    </div>

    <!-- Content Section -->
    <div :class="{ 'space-y-3': spacing }">
      <slot />
    </div>

    <!-- Footer Section -->
    <div v-if="$slots.footer" class="mt-6">
      <slot name="footer" />
    </div>
  </div>
</template>

<script setup lang="ts">
import type { CardProps, CardEmits } from '../../types/ui'

// Define props with defaults
withDefaults(defineProps<CardProps>(), {
  size: 'md',
  hover: false,
  scale: false,
  noPadding: false,
  spacing: true,
  center: false
})

// Define emits
const emit = defineEmits<CardEmits>()

/**
 * Handle click event
 */
const handleClick = (event: MouseEvent): void => {
  emit('click', event)
}
</script>

<style scoped>
/* Card entrance animation */
.card-enter-active {
  transition: all 0.4s cubic-bezier(0.25, 0.8, 0.25, 1);
}

.card-enter-from {
  opacity: 0;
  transform: translateY(20px) scale(0.95);
}

/* Enhanced hover effects */
.cursor-pointer:hover {
  transform: translateY(-2px) scale(1.02);
  box-shadow: 0 8px 25px rgba(0, 0, 0, 0.15);
}

/* Icon animation on hover */
.rounded-lg.p-3:hover {
  transform: scale(1.05);
  background-color: rgba(59, 130, 246, 0.3);
}

/* Ripple effect on click */
.cursor-pointer {
  position: relative;
  overflow: hidden;
}

.cursor-pointer::before {
  content: '';
  position: absolute;
  top: 50%;
  left: 50%;
  width: 0;
  height: 0;
  border-radius: 50%;
  background: radial-gradient(circle, rgba(59, 130, 246, 0.2) 0%, transparent 70%);
  transform: translate(-50%, -50%);
  transition:
    width 0.3s,
    height 0.3s;
  pointer-events: none;
  z-index: 1;
}

.cursor-pointer:active::before {
  width: 200px;
  height: 200px;
}

/* Content z-index to stay above ripple */
.cursor-pointer > * {
  position: relative;
  z-index: 2;
}
</style>
