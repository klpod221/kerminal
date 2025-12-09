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
      <!-- Left icon -->
      <div
        v-if="leftIcon"
        class="absolute inset-y-0 left-0 pl-3 flex items-center pointer-events-none"
      >
        <component :is="leftIcon" :size="iconSize" class="text-gray-400" />
      </div>

      <select
        :id="inputId"
        ref="inputRef"
        v-model="inputValue"
        :placeholder="placeholder"
        :disabled="disabled"
        :readonly="readonly"
        :autocomplete="autocomplete"
        :class="[
          'block w-full rounded-lg border transition-all duration-200',
          'focus:outline-none appearance-none',
          'disabled:opacity-50 disabled:cursor-not-allowed',
          'readonly:bg-gray-700 readonly:cursor-default',
          sizeClasses,
          stateClasses,
          leftIcon ? 'pl-10' : 'pl-3',
          rightIcon ? 'pr-10' : 'pr-8',
        ]"
        :style="selectStyle"
        @blur="handleBlur"
        @focus="handleFocus"
        @change="handleChange"
        @keydown="handleKeydown"
      >
        <!-- Placeholder -->
        <option v-if="!inputValue" disabled selected value>
          {{ placeholder || "Select an option" }}
        </option>

        <!-- Options from prop -->
        <option
          v-for="option in options"
          :key="option.value"
          :value="option.value"
        >
          {{ option.label }}
        </option>

        <!-- Slot for custom options -->
        <slot></slot>
      </select>

      <!-- Right icon or password toggle -->
      <div
        v-if="rightIcon"
        class="absolute inset-y-0 right-0 pr-3 flex items-center"
      >
        <!-- Right icon -->
        <Button
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
import { ref, computed, type Component } from "vue";
import { TriangleAlert } from "lucide-vue-next";
import Button from "./Button.vue";
import { useFormField } from "../../composables/useFormField";
import { useFormStyles } from "../../composables/useFormStyles";

interface SelectProps {
  id: string;
  modelValue?: string | number;
  label?: string;
  placeholder?: string;
  options?: Array<{ value: string | number; label: string }>;
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
  space?: boolean;
}

const props = withDefaults(defineProps<SelectProps>(), {
  size: "md",
  disabled: false,
  readonly: false,
  helper: true,
  space: true,
  options: () => [],
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

const inputValue = computed({
  get: () => props.modelValue?.toString() ?? "",
  set: (value: string) => emit("update:modelValue", value),
});

const selectStyle = computed(() => ({
  backgroundImage: `url("data:image/svg+xml,%3csvg xmlns='http://www.w3.org/2000/svg' fill='none' viewBox='0 0 20 20'%3e%3cpath stroke='%236b7280' stroke-linecap='round' stroke-linejoin='round' stroke-width='1.5' d='m6 8 4 4 4-4'/%3e%3c/svg%3e")`,
  backgroundRepeat: "no-repeat",
  backgroundPosition: "right 0.5rem center",
  backgroundSize: "1rem",
}));

const handleChange = (event: Event): void => {
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

<style scoped>
select {
  -webkit-appearance: none;
  -moz-appearance: none;
  appearance: none;
  /* Ensure background color is applied */
  background-color: inherit !important;
}

select::-ms-expand {
  display: none;
}

/* Ensure background color is applied */
</style>
