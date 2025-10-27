<template>
  <button
    :class="[
      'inline-flex items-center justify-center font-medium rounded-lg transition-all duration-200 focus:outline-none cursor-pointer touch-manipulation',
      'disabled:opacity-50 disabled:cursor-not-allowed',
      'active:scale-95 active:transform',
      sizeClasses,
      variantClasses,
      loading && 'cursor-not-allowed',
    ]"
    :title="title"
    :disabled="disabled || loading"
    @click="handleClick"
  >
    <!-- Loading Spinner -->
    <div
      v-if="loading"
      class="animate-spin rounded-full border-2 border-current border-t-transparent mr-2"
      :class="{
        'w-4 h-4': iconSize === 16,
        'w-3 h-3': iconSize === 14,
        'w-5 h-5': iconSize === 20,
      }"
    ></div>

    <!-- Icon (Left) -->
    <component
      :is="icon"
      v-if="icon && !iconRight && !loading"
      :size="iconSize"
      class="flex-shrink-0"
      :class="text || $slots.default ? 'mr-2' : ''"
    />

    <!-- Content -->
    <span v-if="$slots.default || text" class="truncate">
      <slot>{{ text }}</slot>
    </span>

    <!-- Icon (Right) -->
    <component
      :is="icon"
      v-if="icon && iconRight && !loading"
      :size="iconSize"
      class="flex-shrink-0"
      :class="text || $slots.default ? 'ml-2' : ''"
    />
  </button>
</template>

<script setup lang="ts">
import { computed } from "vue";
import type { Component } from "vue";

interface ButtonProps {
  variant?:
    | "primary"
    | "secondary"
    | "success"
    | "warning"
    | "danger"
    | "ghost"
    | "outline";
  size?: "sm" | "md" | "lg";
  icon?: Component;
  iconRight?: boolean;
  text?: string;
  loading?: boolean;
  title?: string;
  disabled?: boolean;
}

const props = withDefaults(defineProps<ButtonProps>(), {
  variant: "primary",
  size: "md",
  iconRight: false,
  loading: false,
  disabled: false,
});

const emit = defineEmits(["click"]);

const sizeClasses = computed(() => {
  const hasOnlyIcon = props.icon && !props.text;
  switch (props.size) {
    case "sm":
      return hasOnlyIcon ? "p-2 text-sm sm:p-1.5" : "px-3 py-2 text-sm sm:px-1.5 sm:py-0.75";
    case "lg":
      return hasOnlyIcon ? "p-3 text-lg" : "px-4 py-2.5 text-lg sm:px-3 sm:py-1.5";
    default:
      return hasOnlyIcon ? "p-2.5 text-base sm:p-2" : "px-3 py-2 text-base sm:px-2 sm:py-1";
  }
});

const iconSize = computed(() => {
  switch (props.size) {
    case "sm":
      return 14;
    case "lg":
      return 20;
    default:
      return 16;
  }
});

const variantClasses = computed(() => {
  switch (props.variant) {
    case "primary":
      return "bg-blue-600 hover:bg-blue-700 focus:ring-blue-500 text-white shadow-sm";
    case "secondary":
      return "bg-gray-600 hover:bg-gray-700 focus:ring-gray-500 text-white shadow-sm";
    case "success":
      return "bg-green-600 hover:bg-green-700 focus:ring-green-500 text-white shadow-sm";
    case "warning":
      return "bg-yellow-600 hover:bg-yellow-700 focus:ring-yellow-500 text-white shadow-sm";
    case "danger":
      return "bg-red-600 hover:bg-red-700 text-white shadow-sm";
    case "ghost":
      return "text-gray-400 hover:text-white hover:bg-gray-700/50 focus:ring-gray-500";
    case "outline":
      return "border border-gray-600 text-gray-300 hover:bg-gray-700/50 hover:text-white focus:ring-gray-500";
    default:
      return "bg-blue-600 hover:bg-blue-700 focus:ring-blue-500 text-white shadow-sm";
  }
});

const handleClick = (event: MouseEvent): void => {
  if (!props.loading && !props.disabled) {
    emit("click", event);
  }
};
</script>
