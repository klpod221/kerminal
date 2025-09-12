<template>
  <div class="space-y-0.5">
    <div class="relative flex items-center">
      <input
        :id="inputId"
        ref="inputRef"
        :checked="modelValue"
        type="checkbox"
        :disabled="disabled"
        :readonly="readonly"
        :class="[
          'mr-2 block rounded-lg border transition-all duration-200',
          'focus:outline-none focus:ring-2 focus:ring-offset-2 focus:ring-offset-gray-800',
          'disabled:opacity-50 disabled:cursor-not-allowed',
          'readonly:bg-gray-700 readonly:cursor-default',
          sizeClasses,
          stateClasses,
        ]"
        @blur="handleBlur"
        @focus="handleFocus"
        @change="handleChange"
        @keydown="handleKeydown"
      />

      <label
        v-if="label"
        :for="inputId"
        class="block text-sm font-medium text-gray-300 cursor-pointer"
      >
        {{ label }}
        <span
          v-if="props.rules && props.rules.includes('required')"
          class="text-red-400"
          >*</span
        >
      </label>
    </div>

    <div v-if="helper" class="min-h-[1.25rem]">
      <!-- Helper text (only show if no error) -->
      <p v-if="helperText && !errorMessage" class="text-xs text-gray-400">
        {{ helperText }}
      </p>

      <!-- Error message -->
      <p v-if="errorMessage" class="text-xs text-red-400 flex items-center">
        <span class="mr-1">⚠</span>
        {{ errorMessage }}
      </p>
    </div>
  </div>
</template>

<script setup lang="ts">
import { ref, computed, inject, onMounted, onUnmounted, toRef } from "vue";
import { validate as validateFn } from "../../utils/validators";
import type { FormContext } from "../../types/form";

interface CheckboxProps {
  id: string;
  modelValue?: boolean;
  label?: string;
  rules?: string;
  helperText?: string;
  errorMessage?: string;
  size?: "sm" | "md" | "lg";
  disabled?: boolean;
  readonly?: boolean;
  helper?: boolean;
}

const props = withDefaults(defineProps<CheckboxProps>(), {
  size: "md",
  disabled: false,
  readonly: false,
  helper: true,
});

const emit = defineEmits(["update:modelValue", "blur", "focus", "keydown"]);

// Refs
const errorMessage = ref(props.errorMessage || "");
const touched = ref(false);
const inputRef = ref<HTMLInputElement>();

// Injected from parent Form
const formContext = inject<FormContext>("form-context");

// Computed
const inputId = computed(
  () => props.id || `input-${Math.random().toString(36).substr(2, 9)}`
);

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

const stateClasses = computed(() => {
  if (props.errorMessage) {
    return "border-red-500 bg-red-500/5 text-white focus:border-red-400 focus:ring-red-500";
  }

  if (props.disabled) {
    return "border-gray-600 bg-gray-800 text-gray-400";
  }

  if (props.readonly) {
    return "border-gray-600 bg-gray-700 text-gray-300";
  }

  return "border-gray-600 bg-gray-800 text-white placeholder-gray-400 hover:border-gray-500 focus:border-blue-500 focus:ring-blue-500";
});

// Methods
const validate = (): string => {
  if (!props.rules || props.rules.length === 0) {
    return "";
  }

  const allFormValues = formContext
    ? Object.fromEntries(
        Array.from(
          (formContext as FormContext).getFieldValue ? new Map() : new Map()
        )
      )
    : {};

  const error = validateFn(props.modelValue, props.rules, allFormValues);
  errorMessage.value = error;
  return error;
};

const handleBlur = (event: FocusEvent): void => {
  emit("blur", event);
  touched.value = true;
  validate();
};

const handleFocus = (event: FocusEvent): void => {
  emit("focus", event);
};

const handleChange = (event: Event): void => {
  const target = event.target as HTMLInputElement;
  emit("update:modelValue", target.checked);

  if (touched.value) {
    validate();
  }
};

const handleKeydown = (event: KeyboardEvent): void => {
  emit("keydown", event);
};

// Lifecycle hooks
onMounted(() => {
  if (formContext) {
    formContext.register({
      id: inputId.value,
      value: toRef(props, "modelValue"),
      validate,
    });
  }
});

onUnmounted(() => {
  if (formContext) {
    formContext.unregister(inputId.value);
  }
});

// Expose methods for parent components
defineExpose({
  focus: () => inputRef.value?.focus(),
  blur: () => inputRef.value?.blur(),
  select: () => inputRef.value?.select(),
});
</script>
