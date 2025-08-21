<template>
  <div
    :class="[
      'bg-gradient-to-br from-[#2a2a2a] to-[#1f1f1f] rounded-xl border border-gray-600',
      'transition-all duration-300',
      {
        'hover:border-gray-500 hover:shadow-lg cursor-pointer': hover,
        'hover:scale-105 transform': hover && scale,
        'p-6': !noPadding,
        'p-4': size === 'sm' && !noPadding,
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
import type { Component } from 'vue'

// Props interface
interface Props {
  /**
   * Card title displayed in header
   */
  title?: string
  /**
   * Icon component to display in header
   */
  icon?: Component
  /**
   * Background color class for icon container
   */
  iconBackground?: string
  /**
   * Color class for icon
   */
  iconColor?: string
  /**
   * Card size variant
   */
  size?: 'sm' | 'md' | 'lg'
  /**
   * Enable hover effects
   */
  hover?: boolean
  /**
   * Enable scale transform on hover
   */
  scale?: boolean
  /**
   * Remove default padding
   */
  noPadding?: boolean
  /**
   * Add default spacing between content elements
   */
  spacing?: boolean
  /**
   * Center align content (for action buttons)
   */
  center?: boolean
  /**
   * Custom CSS classes to apply
   */
  customClass?: string
}

// Emits interface
interface Emits {
  /**
   * Emitted when card is clicked
   */
  click: [event: MouseEvent]
}

// Define props with defaults
withDefaults(defineProps<Props>(), {
  size: 'md',
  hover: false,
  scale: false,
  noPadding: false,
  spacing: true,
  center: false
})

// Define emits
const emit = defineEmits<Emits>()

/**
 * Handle card click event
 * @param event - Mouse click event
 */
function handleClick(event: MouseEvent): void {
  emit('click', event)
}
</script>
