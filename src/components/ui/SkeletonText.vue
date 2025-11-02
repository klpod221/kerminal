<template>
  <div class="space-y-2">
    <SkeletonLoader
      v-for="i in lines"
      :key="i"
      variant="text"
      :size="i === lines && lastLineWidth !== 'full' ? 'custom' : 'md'"
      :width="i === lines ? lastLineWidth : '100%'"
      :show-shimmer="showShimmer"
      :additional-classes="additionalClasses"
    />
  </div>
</template>

<script setup lang="ts">
import SkeletonLoader from "./SkeletonLoader.vue";

interface SkeletonTextProps {
  /** Number of text lines */
  lines?: number;
  /** Width of last line (e.g., "60%", "200px") */
  lastLineWidth?: string;
  /** Show shimmer effect */
  showShimmer?: boolean;
  /** Additional CSS classes */
  additionalClasses?: string;
}

withDefaults(defineProps<SkeletonTextProps>(), {
  lines: 3,
  lastLineWidth: "60%",
  showShimmer: true,
  additionalClasses: "",
});
</script>
