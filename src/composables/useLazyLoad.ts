import { ref, computed, watch, onMounted, onUnmounted, type Ref } from "vue";

interface UseLazyLoadOptions {
  /** Number of items to load initially */
  initialLoad?: number;
  /** Number of items to load per batch */
  loadMoreCount?: number;
  /** Threshold for intersection observer (0-1) */
  threshold?: number;
  /** Root margin for intersection observer */
  rootMargin?: string;
}

/**
 * Composable for lazy loading list items
 * @param items - Full list of items
 * @param options - Lazy load options
 * @returns Object with visible items and load more function
 */
export function useLazyLoad<T>(
  items: Ref<T[]>,
  options: UseLazyLoadOptions = {},
) {
  const {
    initialLoad = 20,
    loadMoreCount = 20,
    threshold = 0.1,
    rootMargin = "100px",
  } = options;

  const visibleCount = ref(initialLoad);
  const sentinelRef = ref<HTMLElement | null>(null);
  const observer = ref<IntersectionObserver | null>(null);
  const isLoadingMore = ref(false);

  const loadMore = () => {
    if (isLoadingMore.value) return;
    if (visibleCount.value >= items.value.length) return;

    isLoadingMore.value = true;
    // Simulate small delay for smooth loading
    setTimeout(() => {
      visibleCount.value = Math.min(
        visibleCount.value + loadMoreCount,
        items.value.length,
      );
      isLoadingMore.value = false;
    }, 50);
  };

  const visibleItems = computed(() => {
    return items.value.slice(0, visibleCount.value);
  });

  const hasMore = computed(() => {
    return visibleCount.value < items.value.length;
  });

  const reset = () => {
    visibleCount.value = initialLoad;
  };

  // Create observer once
  const createObserver = () => {
    if (observer.value) {
      observer.value.disconnect();
    }

    observer.value = new IntersectionObserver(
      (entries) => {
        const entry = entries[0];
        if (entry.isIntersecting && hasMore.value && !isLoadingMore.value) {
          loadMore();
        }
      },
      {
        threshold,
        rootMargin,
      },
    );

    // Observe sentinel if it exists
    if (sentinelRef.value) {
      observer.value.observe(sentinelRef.value);
    }
  };

  onMounted(() => {
    createObserver();
  });

  onUnmounted(() => {
    if (observer.value) {
      if (sentinelRef.value) {
        observer.value.unobserve(sentinelRef.value);
      }
      observer.value.disconnect();
    }
  });

  // Watch for sentinel element changes
  watch(
    sentinelRef,
    (newSentinel, oldSentinel) => {
      if (!observer.value) {
        createObserver();
        return;
      }

      if (oldSentinel) {
        observer.value.unobserve(oldSentinel);
      }

      if (newSentinel) {
        observer.value.observe(newSentinel);
      }
    },
    { immediate: true },
  );

  // Note: We don't auto-reset on items change to avoid breaking user's scroll position
  // The parent component should call reset() when needed (e.g., on search query change)

  return {
    visibleItems,
    sentinelRef,
    hasMore,
    isLoadingMore,
    loadMore,
    reset,
  };
}
