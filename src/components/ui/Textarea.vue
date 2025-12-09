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

    <div class="relative">
      <textarea
        :id="inputId"
        ref="inputRef"
        v-model="inputValue"
        :rows="rows"
        :placeholder="placeholder"
        :disabled="disabled"
        :readonly="readonly"
        :autocomplete="autocomplete"
        :class="[
          'block px-3 w-full rounded-lg border transition-all duration-200 resize-y',
          'focus:outline-none',
          'disabled:opacity-50 disabled:cursor-not-allowed',
          'readonly:bg-gray-700 readonly:cursor-default',
          sizeClasses,
          stateClasses,
        ]"
        @blur="handleBlur"
        @focus="handleFocus"
        @input="handleInput"
        @keydown="handleKeydown"
      />
    </div>

    <div v-if="helper" class="min-h-5">
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
import { ref, computed } from "vue";
import { TriangleAlert } from "lucide-vue-next";
import { useFormField } from "../../composables/useFormField";
import { useFormStyles } from "../../composables/useFormStyles";

interface TextareaProps {
  id: string;
  modelValue?: string | number;
  rows?: number;
  label?: string;
  placeholder?: string;
  rules?: string;
  helperText?: string;
  errorMessage?: string;
  size?: "sm" | "md" | "lg";
  disabled?: boolean;
  readonly?: boolean;
  autocomplete?: string;
  helper?: boolean;
}

const props = withDefaults(defineProps<TextareaProps>(), {
  rows: 4,
  size: "md",
  disabled: false,
  readonly: false,
  helper: true,
});

const emit = defineEmits(["update:modelValue", "blur", "focus", "keydown"]);

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

const { sizeClasses, stateClasses } = useFormStyles(props);

// Component-specific state
const inputRef = ref<HTMLInputElement>();

const inputValue = computed({
  get: () => props.modelValue?.toString() ?? "",
  set: (value: string) => emit("update:modelValue", value),
});

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
