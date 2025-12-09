import { computed, type ComputedRef } from "vue";

/**
 * Props interface for form styles composable
 */
export interface FormStylesProps {
  size?: "sm" | "md" | "lg";
  errorMessage?: string;
  disabled?: boolean;
  readonly?: boolean;
}

/**
 * Return type for useFormStyles composable
 */
export interface UseFormStylesReturn {
  sizeClasses: ComputedRef<string>;
  stateClasses: ComputedRef<string>;
  iconSize: ComputedRef<number>;
}

/**
 * Composable for shared form field styling logic
 * Handles size classes, state classes, and icon sizing
 * 
 * @param props - Component props containing styling configuration
 * @returns Object containing computed style classes
 * 
 * @example
 * ```vue
 * <script setup lang="ts">
 * const props = defineProps<FormStylesProps>();
 * const { sizeClasses, stateClasses, iconSize } = useFormStyles(props);
 * </script>
 * ```
 */
export function useFormStyles(props: FormStylesProps): UseFormStylesReturn {
  /**
   * Size-based CSS classes
   */
  const sizeClasses = computed(() => {
    switch (props.size) {
      case "sm":
        return "text-sm py-1.5";
      case "lg":
        return "text-lg py-3";
      default:
        return "text-base py-2";
    }
  });

  /**
   * State-based CSS classes (error, disabled, readonly, default)
   */
  const stateClasses = computed(() => {
    if (props.errorMessage) {
      return "border-red-500 bg-red-500/5 text-white focus:border-red-400";
    }

    if (props.disabled) {
      return "border-gray-600 bg-gray-800 text-gray-400";
    }

    if (props.readonly) {
      return "border-gray-600 bg-gray-700 text-gray-300";
    }

    return "border-gray-600 bg-gray-800 text-white placeholder-gray-400 hover:border-gray-500 focus:border-blue-500 focus:ring-blue-500";
  });

  /**
   * Icon size based on component size
   */
  const iconSize = computed(() => {
    switch (props.size) {
      case "sm":
        return 16;
      case "lg":
        return 20;
      default:
        return 18;
    }
  });

  return {
    sizeClasses,
    stateClasses,
    iconSize,
  };
}
