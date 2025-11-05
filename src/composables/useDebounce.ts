import { ref, watch, onUnmounted, type Ref, type WatchStopHandle } from "vue";

interface UseDebounceOptions {
  /** Delay in milliseconds (default: 300) */
  delay?: number;
  /** Callback to execute on immediate (first) change */
  immediate?: boolean;
}

/**
 * Composable for debouncing reactive values
 * Useful for search inputs, resize handlers, etc.
 *
 * @example
 * ```ts
 * const searchQuery = ref("");
 * const debouncedQuery = useDebounce(searchQuery, { delay: 500 });
 *
 * watch(debouncedQuery, (value) => {
 *   // This will only fire 500ms after user stops typing
 *   performSearch(value);
 * });
 * ```
 */
export function useDebounce<T>(
  source: Ref<T>,
  options: UseDebounceOptions = {},
): Ref<T> {
  const { delay = 300, immediate = false } = options;

  const debouncedValue = ref(source.value) as Ref<T>;
  let timeoutId: ReturnType<typeof setTimeout> | null = null;
  let watchStopHandle: WatchStopHandle | null = null;

  watchStopHandle = watch(
    source,
    (newValue) => {
      // Clear previous timeout
      if (timeoutId !== null) {
        clearTimeout(timeoutId);
      }

      // If immediate, update immediately on first change
      if (immediate && timeoutId === null) {
        debouncedValue.value = newValue;
      }

      // Set new timeout
      timeoutId = setTimeout(() => {
        debouncedValue.value = newValue;
        timeoutId = null;
      }, delay);
    },
    { immediate: true },
  );

  // Cleanup on unmount
  onUnmounted(() => {
    if (timeoutId !== null) {
      clearTimeout(timeoutId);
    }
    if (watchStopHandle) {
      watchStopHandle();
    }
  });

  return debouncedValue;
}

/**
 * Composable for debouncing function calls
 * Returns a debounced version of the function
 *
 * @example
 * ```ts
 * const handleSearch = useDebounceFn((query: string) => {
 *   performSearch(query);
 * }, 500);
 *
 * // Call it - will only execute 500ms after last call
 * handleSearch("test");
 * ```
 */
export function useDebounceFn<T extends (...args: any[]) => any>(
  fn: T,
  delay: number = 300,
): (...args: Parameters<T>) => void {
  let timeoutId: ReturnType<typeof setTimeout> | null = null;

  return (...args: Parameters<T>) => {
    if (timeoutId !== null) {
      clearTimeout(timeoutId);
    }

    timeoutId = setTimeout(() => {
      fn(...args);
      timeoutId = null;
    }, delay);
  };
}
