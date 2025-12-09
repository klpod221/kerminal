import { ref, computed, inject, onMounted, onUnmounted, toRef, type Ref, type ComputedRef } from "vue";
import { validate as validateFn } from "../utils/validators";
import type { FormContext } from "../types/form";

/**
 * Props interface for form field composable
 */
export interface FormFieldProps {
  id: string;
  modelValue?: string | number | boolean | string[] | null;
  rules?: string;
  errorMessage?: string;
}

/**
 * Emit type for form field composable - accepts any emit function
 */
export type FormFieldEmits = (...args: any[]) => void;

/**
 * Return type for useFormField composable
 */
export interface UseFormFieldReturn {
  errorMessage: Ref<string>;
  touched: Ref<boolean>;
  inputId: ComputedRef<string>;
  validate: () => string;
  handleBlur: (event: FocusEvent) => void;
  handleFocus: (event: FocusEvent) => void;
  handleKeydown: (event: KeyboardEvent) => void;
}

/**
 * Composable for shared form field logic
 * Handles validation, form context integration, and event handlers
 * 
 * @param props - Component props containing form field configuration
 * @param emit - Component emit function for events
 * @returns Object containing form field state and handlers
 * 
 * @example
 * ```vue
 * <script setup lang="ts">
 * const props = defineProps<FormFieldProps>();
 * const emit = defineEmits<FormFieldEmits>();
 * 
 * const { errorMessage, validate, handleBlur, handleFocus } = useFormField(props, emit);
 * </script>
 * ```
 */
export function useFormField(
  props: FormFieldProps,
  emit: FormFieldEmits
): UseFormFieldReturn {
  // State
  const errorMessage = ref(props.errorMessage || "");
  const touched = ref(false);
  
  // Form context
  const formContext = inject<FormContext>("form-context");

  /**
   * Generate unique input ID
   */
  const inputId = computed(
    () =>
      props.id ||
      `input-${crypto.getRandomValues(new Uint32Array(1))[0].toString(36)}`,
  );

  /**
   * Validate field value against rules
   * @returns Error message if validation fails, empty string otherwise
   */
  const validate = (): string => {
    if (!props.rules || props.rules.length === 0) {
      return "";
    }

    const allFormValues = formContext?.getAllFieldValues() || {};

    const error = validateFn(props.modelValue, props.rules, allFormValues);
    errorMessage.value = error;
    return error;
  };

  /**
   * Handle blur event - triggers validation
   */
  const handleBlur = (event: FocusEvent): void => {
    emit("blur", event);
    touched.value = true;
    validate();
  };

  /**
   * Handle focus event
   */
  const handleFocus = (event: FocusEvent): void => {
    emit("focus", event);
  };

  /**
   * Handle keydown event
   */
  const handleKeydown = (event: KeyboardEvent): void => {
    emit("keydown", event);
  };

  // Register with form context on mount
  onMounted(() => {
    if (formContext) {
      formContext.register({
        id: inputId.value,
        value: toRef(props, "modelValue"),
        validate,
      });
    }
  });

  // Unregister from form context on unmount
  onUnmounted(() => {
    if (formContext) {
      formContext.unregister(inputId.value);
    }
  });

  return {
    errorMessage,
    touched,
    inputId,
    validate,
    handleBlur,
    handleFocus,
    handleKeydown,
  };
}
