<template>
  <div class="space-y-0.5">
    <!-- Label -->
    <label
      v-if="label"
      :for="pickerId"
      class="block text-sm font-medium text-gray-300"
    >
      {{ label }}
      <span
        v-if="props.rules && props.rules.includes('required')"
        class="text-red-400"
      >
        *
      </span>
    </label>

    <!-- Color input wrapper -->
    <div class="relative flex items-center gap-2">
      <!-- Color input -->
      <input
        :id="pickerId"
        ref="pickerRef"
        v-model="colorValue"
        type="color"
        :disabled="disabled"
        :readonly="readonly"
        :class="[
          'block w-10 h-10 overflow-hidden border rounded-lg cursor-pointer transition-all duration-200',
          'focus:outline-none',
          'disabled:opacity-50 disabled:cursor-not-allowed',
          'readonly:bg-gray-700 readonly:cursor-default',
          stateClasses,
        ]"
        @blur="handleBlur"
        @focus="handleFocus"
        @input="handleColorInput"
      />

      <!-- Hex input -->
      <input
        v-model="hexInput"
        type="text"
        :disabled="disabled"
        :readonly="readonly"
        placeholder="#RRGGBB"
        maxlength="7"
        :class="[
          'block flex-1 rounded-lg border transition-all duration-200',
          'focus:outline-none',
          'disabled:opacity-50 disabled:cursor-not-allowed',
          'readonly:bg-gray-700 readonly:cursor-default',
          sizeClasses,
          hexStateClasses,
        ]"
        @input="handleHexInput"
      />
    </div>

    <div v-if="helper" class="min-h-5">
      <!-- Helper text (only show if no error) -->
      <p
        v-if="helperText && !errorMessage && !localError"
        class="text-xs text-gray-400"
      >
        {{ helperText }}
      </p>

      <!-- Error message -->
      <p
        v-if="errorMessage || localError"
        class="text-xs text-red-400 flex items-center"
      >
        <TriangleAlert class="mr-1" :size="12" />
        {{ errorMessage || localError }}
      </p>
    </div>
  </div>
</template>

<script setup lang="ts">
import { ref, computed, watch } from "vue";
import { TriangleAlert } from "lucide-vue-next";
import { useFormField } from "../../composables/useFormField";
import { useFormStyles } from "../../composables/useFormStyles";

/**
 * Props interface for Color Picker component
 */
interface ColorPickerProps {
  id: string;
  modelValue?: string;
  label?: string;
  rules?: string;
  helperText?: string;
  errorMessage?: string;
  size?: "sm" | "md" | "lg";
  disabled?: boolean;
  readonly?: boolean;
  helper?: boolean;
}

const props = withDefaults(defineProps<ColorPickerProps>(), {
  modelValue: "#000000",
  size: "md",
  disabled: false,
  readonly: false,
  helper: true,
});

const emit = defineEmits(["update:modelValue", "blur", "focus"]);

// Use composables for shared logic
const {
  errorMessage,
  inputId: pickerId,
  handleBlur,
  handleFocus,
} = useFormField(props, emit);

const { sizeClasses, stateClasses } = useFormStyles(props);

// Component-specific state
const pickerRef = ref<HTMLInputElement>();
const hexInput = ref("");
const localError = ref("");

const colorValue = computed({
  get: () =>
    isValidHex(props.modelValue ?? "") ? props.modelValue! : "#000000",
  set: (value: string) => emit("update:modelValue", value),
});

const hexStateClasses = computed(() => {
  if (props.errorMessage || localError.value) {
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

const handleColorInput = (event: Event): void => {
  const target = event.target as HTMLInputElement;
  hexInput.value = target.value;
  emit("update:modelValue", target.value);
  localError.value = "";
};

const handleHexInput = (event: Event): void => {
  const value = (event.target as HTMLInputElement).value;
  hexInput.value = value;
  if (isValidHex(value)) {
    emit("update:modelValue", value);
    localError.value = "";
  } else if (value && value !== "#") {
    localError.value = "Invalid hex color code.";
  } else {
    localError.value = "";
  }
};

function isValidHex(hex: string): boolean {
  return /^#([A-Fa-f0-9]{6}|[A-Fa-f0-9]{3})$/.test(hex);
}

watch(
  () => props.modelValue,
  (val) => {
    if (isValidHex(val ?? "")) {
      hexInput.value = val!;
      localError.value = "";
    }
  },
  { immediate: true },
);

defineExpose({
  focus: () => pickerRef.value?.focus(),
  blur: () => pickerRef.value?.blur(),
  select: () => pickerRef.value?.select(),
});
</script>
