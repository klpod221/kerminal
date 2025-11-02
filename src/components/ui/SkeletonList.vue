<template>
  <div class="space-y-3">
    <div
      v-for="i in items"
      :key="i"
      class="flex items-center gap-3 p-3 bg-gray-900/50 rounded-lg"
    >
      <SkeletonLoader
        v-if="showAvatar"
        variant="circular"
        size="md"
        :show-shimmer="showShimmer"
      />
      <div class="flex-1 space-y-2">
        <SkeletonLoader
          variant="text"
          size="md"
          :width="i === items ? '70%' : '100%'"
          :show-shimmer="showShimmer"
        />
        <SkeletonLoader
          variant="text"
          size="sm"
          :width="i === items ? '50%' : '80%'"
          :show-shimmer="showShimmer"
        />
      </div>
      <SkeletonLoader
        v-if="showActions"
        variant="rectangular"
        size="custom"
        width="80px"
        height="32px"
        :show-shimmer="showShimmer"
      />
    </div>
  </div>
</template>

<script setup lang="ts">
import SkeletonLoader from "./SkeletonLoader.vue";

interface SkeletonListProps {
  /** Number of list items */
  items?: number;
  /** Show avatar skeleton */
  showAvatar?: boolean;
  /** Show action buttons skeleton */
  showActions?: boolean;
  /** Show shimmer effect */
  showShimmer?: boolean;
}

withDefaults(defineProps<SkeletonListProps>(), {
  items: 5,
  showAvatar: true,
  showActions: true,
  showShimmer: true,
});
</script>
