<template>
  <div
    :class="[
      'animate-pulse bg-gray-800 rounded',
      variantClasses,
      sizeClasses,
      additionalClasses,
    ]"
  >
    <div v-if="showShimmer" class="shimmer-overlay"></div>
  </div>
</template>

<script setup lang="ts">
import { computed } from "vue";

interface SkeletonLoaderProps {
  /** Variant of skeleton loader */
  variant?: "text" | "circular" | "rectangular" | "rounded";
  /** Size preset */
  size?: "sm" | "md" | "lg" | "xl" | "custom";
  /** Custom width (e.g., "100%", "200px") */
  width?: string;
  /** Custom height (e.g., "20px", "100%") */
  height?: string;
  /** Show shimmer effect */
  showShimmer?: boolean;
  /** Additional CSS classes */
  additionalClasses?: string;
}

const props = withDefaults(defineProps<SkeletonLoaderProps>(), {
  variant: "rectangular",
  size: "md",
  showShimmer: true,
  additionalClasses: "",
});

const variantClasses = computed(() => {
  switch (props.variant) {
    case "text":
      return "h-4";
    case "circular":
      return "rounded-full";
    case "rounded":
      return "rounded-lg";
    case "rectangular":
    default:
      return "rounded";
  }
});

const sizeClasses = computed(() => {
  if (props.size === "custom") {
    return "";
  }

  switch (props.variant) {
    case "text":
      switch (props.size) {
        case "sm":
          return "h-3";
        case "md":
          return "h-4";
        case "lg":
          return "h-5";
        case "xl":
          return "h-6";
        default:
          return "h-4";
      }
    case "circular":
      switch (props.size) {
        case "sm":
          return "w-8 h-8";
        case "md":
          return "w-12 h-12";
        case "lg":
          return "w-16 h-16";
        case "xl":
          return "w-24 h-24";
        default:
          return "w-12 h-12";
      }
    default:
      switch (props.size) {
        case "sm":
          return "h-12";
        case "md":
          return "h-20";
        case "lg":
          return "h-32";
        case "xl":
          return "h-48";
        default:
          return "h-20";
      }
  }
});
</script>

<style scoped>
.shimmer-overlay {
  position: absolute;
  inset: 0;
  background: linear-gradient(
    90deg,
    transparent,
    rgba(255, 255, 255, 0.1),
    transparent
  );
  background-size: 200% 100%;
  animation: shimmer 1.5s infinite;
  border-radius: inherit;
}

@keyframes shimmer {
  0% {
    background-position: -200% 0;
  }
  100% {
    background-position: 200% 0;
  }
}
</style>
