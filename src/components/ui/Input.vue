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
          'block w-full rounded-lg border transition-all duration-200 touch-manipulation',
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

    <div v-if="helper" :class="space && 'min-h-5'">
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
import { ref, computed, nextTick, type Component } from "vue";
import { Eye, EyeOff, TriangleAlert } from "lucide-vue-next";
import Button from "./Button.vue";
import { useFormField } from "../../composables/useFormField";
import { useFormStyles } from "../../composables/useFormStyles";

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
  space?: boolean;
}

const props = withDefaults(defineProps<InputProps>(), {
  type: "text",
  size: "md",
  disabled: false,
  readonly: false,
  helper: true,
  space: true,
});

const emit = defineEmits([
  "update:modelValue",
  "blur",
  "focus",
  "keydown",
  "right-icon-click",
]);

// Use composables for shared logic
const {
  errorMessage,
  touched,
  inputId,
  validate,
  handleBlur,
  handleFocus,
  handleKeydown,
} = useFormField(props, emit);

const { sizeClasses, stateClasses, iconSize } = useFormStyles(props);

// Component-specific state
const inputRef = ref<HTMLInputElement>();
const isPasswordVisible = ref(false);

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

const togglePasswordVisibility = (): void => {
  isPasswordVisible.value = !isPasswordVisible.value;
  nextTick(() => {
    inputRef.value?.focus();
  });
};

const handleInput = (event: Event): void => {
  const target = event.target as HTMLInputElement;
  emit("update:modelValue", target.value);

  if (touched.value) {
    validate();
  }
};

defineExpose({
  focus: () => inputRef.value?.focus(),
  blur: () => inputRef.value?.blur(),
  select: () => inputRef.value?.select(),
});
</script>
