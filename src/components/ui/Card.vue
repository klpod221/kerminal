<template>
  <div
    :class="[
      'bg-gradient-to-br from-[#2a2a2a] to-[#1f1f1f] rounded-xl border border-gray-600',
      'transition-all duration-300 relative overflow-hidden',
      {
        'hover:border-gray-500 hover:shadow-lg cursor-pointer hover:-translate-y-0.5 hover:scale-[1.02]':
          hover,
        'card-ripple': hover,
        'p-4': !noPadding,
        'p-6': size === 'sm' && !noPadding,
        'p-8': size === 'lg' && !noPadding,
      },
      customClass,
    ]"
    @click="handleClick"
  >
    <!-- Header Section -->
    <div
      v-if="$slots.header || title || icon"
      :class="[
        'mb-6 relative z-[2]',
        center ? 'flex flex-col items-center text-center' : 'flex items-center',
      ]"
    >
      <!-- Icon -->
      <div
        v-if="icon || $slots.icon"
        :class="[
          'rounded-lg p-3 transition-all duration-300',
          center ? 'w-fit mx-auto mb-3' : 'mr-4',
          'hover:scale-105',
          iconBackground || 'bg-blue-500/20',
        ]"
      >
        <slot name="icon">
          <component
            :is="icon"
            :size="32"
            :class="iconColor || 'text-blue-400'"
          />
        </slot>
      </div>

      <!-- Title -->
      <div :class="center ? '' : 'flex-1'">
        <slot name="header">
          <h3
            v-if="title"
            :class="[
              'font-semibold',
              size === 'sm'
                ? 'text-lg'
                : size === 'lg'
                ? 'text-2xl'
                : 'text-xl',
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
    <div :class="[{ 'space-y-3': spacing }, 'relative z-[2]']">
      <slot />
    </div>

    <!-- Footer Section -->
    <div v-if="$slots.footer" class="mt-6 relative z-[2]">
      <slot name="footer" />
    </div>
  </div>
</template>

<script setup lang="ts">
import type { Component } from "vue";

interface CardProps {
  title?: string;
  icon?: Component;
  iconBackground?: string;
  iconColor?: string;
  size?: "sm" | "md" | "lg";
  hover?: boolean;
  scale?: boolean;
  noPadding?: boolean;
  spacing?: boolean;
  center?: boolean;
  customClass?: string;
}

// Define props with defaults
withDefaults(defineProps<CardProps>(), {
  size: "md",
  hover: false,
  scale: false,
  noPadding: false,
  spacing: true,
  center: false,
});

// Define emits
const emit = defineEmits<{
  (e: "click", event: MouseEvent): void;
}>();

/**
 * Handle click event
 */
const handleClick = (event: MouseEvent): void => {
  emit("click", event);
};
</script>

<style scoped>
/* Ripple effect on click - cannot be achieved with Tailwind alone */
.card-ripple::before {
  content: "";
  position: absolute;
  top: 50%;
  left: 50%;
  width: 0;
  height: 0;
  border-radius: 50%;
  background: radial-gradient(
    circle,
    rgba(59, 130, 246, 0.2) 0%,
    transparent 70%
  );
  transform: translate(-50%, -50%);
  transition: width 0.3s, height 0.3s;
  pointer-events: none;
  z-index: 1;
}

.card-ripple:active::before {
  width: 200px;
  height: 200px;
}
</style>
