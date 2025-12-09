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
          'focus:outline-none',
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
import { ref } from "vue";
import { TriangleAlert } from "lucide-vue-next";
import { useFormField } from "../../composables/useFormField";
import { useFormStyles } from "../../composables/useFormStyles";

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

const handleChange = (event: Event): void => {
  const target = event.target as HTMLInputElement;
  emit("update:modelValue", target.checked);

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
