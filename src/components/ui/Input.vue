<template>
  <div class="space-y-0.5">
    <!-- Label -->
    <label
      v-if="label"
      :for="inputId"
      class="block text-sm font-medium text-gray-300"
    >
      {{ label }}
      <span
        v-if="props.rules && props.rules.includes('required')"
        class="text-red-400"
        >*</span
      >
    </label>

    <!-- Input wrapper -->
    <div class="relative">
      <!-- Left icon -->
      <div
        v-if="leftIcon"
        class="absolute inset-y-0 left-0 pl-3 flex items-center pointer-events-none"
      >
        <component :is="leftIcon" :size="iconSize" class="text-gray-400" />
      </div>

      <!-- Input element -->
      <input
        :id="inputId"
        ref="inputRef"
        v-model="inputValue"
        :type="inputType"
        :placeholder="placeholder"
        :disabled="disabled"
        :readonly="readonly"
        :autocomplete="autocomplete"
        :autofocus="autofocus"
        :class="[
          'block w-full rounded-lg border transition-all duration-200',
          'focus:outline-none',
          'disabled:opacity-50 disabled:cursor-not-allowed',
          'readonly:bg-gray-700 readonly:cursor-default',
          sizeClasses,
          stateClasses,
          leftIcon ? 'pl-10' : 'pl-3',
          rightIcon || showPasswordToggle ? 'pr-10' : 'pr-3',
        ]"
        @blur="handleBlur"
        @focus="handleFocus"
        @input="handleInput"
        @keydown="handleKeydown"
      />

      <!-- Right icon or password toggle -->
      <div
        v-if="rightIcon || showPasswordToggle"
        class="absolute inset-y-0 right-0 pr-3 flex items-center"
      >
        <!-- Password toggle -->
        <Button
          v-if="showPasswordToggle"
          type="button"
          variant="ghost"
          class="text-gray-400 hover:text-gray-300 transition-colors cursor-pointer"
          tabindex="-1"
          @click="togglePasswordVisibility"
        >
          <component :is="isPasswordVisible ? EyeOff : Eye" :size="iconSize" />
        </Button>

        <!-- Right icon -->
        <Button
          v-else-if="rightIcon"
          type="button"
          variant="ghost"
          tabindex="-1"
          class="text-gray-400 hover:text-gray-300 transition-colors cursor-pointer"
          @click="emit('right-icon-click')"
        >
          <component :is="rightIcon" :size="iconSize" />
        </Button>
      </div>
    </div>

    <div v-if="helper" class="min-h-[1.25rem]">
      <!-- Helper text (only show if no error) -->
      <p v-if="helperText && !errorMessage" class="text-xs text-gray-400">
        {{ helperText }}
      </p>

      <!-- Error message -->
      <p v-if="errorMessage" class="text-xs text-red-400 flex items-center">
        <TriangleAlert class="mr-1" :size="12" />
        {{ errorMessage }}
      </p>
    </div>
  </div>
</template>

<script setup lang="ts">
import {
  ref,
  computed,
  nextTick,
  inject,
  onMounted,
  onUnmounted,
  toRef,
} from "vue";
import { Eye, EyeOff, TriangleAlert } from "lucide-vue-next";
import { validate as validateFn } from "../../utils/validators";
import Button from "./Button.vue";
import type { FormContext } from "../../types/form";
import type { Component } from "vue";

interface InputProps {
  id: string;
  modelValue?: string | number;
  type?: "text" | "password" | "email" | "number" | "tel" | "url" | "search";
  label?: string;
  placeholder?: string;
  rules?: string;
  helperText?: string;
  errorMessage?: string;
  size?: "sm" | "md" | "lg";
  leftIcon?: Component;
  rightIcon?: Component;
  disabled?: boolean;
  readonly?: boolean;
  autocomplete?: string;
  helper?: boolean;
  autofocus?: boolean;
}

const props = withDefaults(defineProps<InputProps>(), {
  type: "text",
  size: "md",
  disabled: false,
  readonly: false,
  helper: true,
});

const emit = defineEmits([
  "update:modelValue",
  "blur",
  "focus",
  "keydown",
  "right-icon-click",
]);

// Refs
const errorMessage = ref(props.errorMessage || "");
const touched = ref(false);
const inputRef = ref<HTMLInputElement>();
const isPasswordVisible = ref(false);

// Injected from parent Form
const formContext = inject<FormContext>("form-context");

// Computed
const inputId = computed(
  () => props.id || `input-${Math.random().toString(36).substr(2, 9)}`,
);

const inputValue = computed({
  get: () => props.modelValue?.toString() ?? "",
  set: (value: string) => emit("update:modelValue", value),
});

const inputType = computed(() => {
  if (props.type === "password") {
    return isPasswordVisible.value ? "text" : "password";
  }
  return props.type;
});

const showPasswordToggle = computed(() => props.type === "password");

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

  const allFormValues = formContext?.getAllFieldValues() || {};

  const error = validateFn(props.modelValue, props.rules, allFormValues);
  errorMessage.value = error;
  return error;
};

const togglePasswordVisibility = (): void => {
  isPasswordVisible.value = !isPasswordVisible.value;
  // Keep focus on input after toggling
  nextTick(() => {
    inputRef.value?.focus();
  });
};

const handleBlur = (event: FocusEvent): void => {
  emit("blur", event);
  touched.value = true;
  validate();
};

const handleFocus = (event: FocusEvent): void => {
  emit("focus", event);
};

const handleInput = (event: Event): void => {
  const target = event.target as HTMLInputElement;
  emit("update:modelValue", target.value);

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
