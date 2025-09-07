<!-- Checkbox component with consistent styling -->
<template>
  <div class="space-y-0.5">
    <!-- Label (optional top label) -->
    <label
      v-if="label && labelPosition === 'top'"
      :for="checkboxId"
      class="block text-sm font-medium text-gray-300"
    >
      {{ label }}
      <span v-if="rules?.some((rule) => rule === 'required')" class="text-red-400">*</span>
    </label>

    <!-- Checkbox wrapper -->
    <label :for="checkboxId" class="flex items-center cursor-pointer">
      <input
        :id="checkboxId"
        ref="checkboxRef"
        v-model="checkboxValue"
        type="checkbox"
        :disabled="disabled"
        :class="[
          'rounded border transition-all duration-200',
          'focus:outline-none focus:ring-2 focus:ring-offset-2 focus:ring-offset-gray-800',
          'disabled:opacity-50 disabled:cursor-not-allowed',
          sizeClasses,
          stateClasses
        ]"
        @blur="handleBlur"
        @focus="handleFocus"
        @change="handleChange"
      />

      <!-- Inline label -->
      <span
        v-if="label && labelPosition === 'right'"
        :class="['select-none transition-colors ml-2', labelSizeClasses, labelStateClasses]"
      >
        {{ label }}
        <span v-if="rules?.some((rule) => rule === 'required')" class="text-red-400 ml-1">*</span>
      </span>
    </label>

    <div v-if="helper" class="min-h-[1.25rem]">
      <!-- Helper text (only show if no error) -->
      <p v-if="helperText && !errorMessage" class="text-xs text-gray-400">{{ helperText }}</p>

      <!-- Error message -->
      <p v-if="errorMessage" class="text-xs text-red-400 flex items-center">
        <span class="mr-1">⚠</span>
        {{ errorMessage }}
      </p>
    </div>
  </div>
</template>

<script setup lang="ts">
import { ref, computed } from 'vue'
import type { CheckboxProps } from '../../types/ui'

const props = withDefaults(defineProps<CheckboxProps>(), {
  modelValue: false,
  size: 'md',
  disabled: false,
  helper: true,
  labelPosition: 'right'
})

const emit = defineEmits(['update:modelValue', 'blur', 'focus', 'change'])

// Refs
const checkboxRef = ref<HTMLInputElement>()

// Computed
const checkboxId = computed(() => props.id || `checkbox-${Math.random().toString(36).substr(2, 9)}`)

const checkboxValue = computed({
  get: () => props.modelValue ?? false,
  set: (value: boolean) => emit('update:modelValue', value)
})

const sizeClasses = computed(() => {
  switch (props.size) {
    case 'sm':
      return 'w-3 h-3'
    case 'lg':
      return 'w-5 h-5'
    default:
      return 'w-4 h-4'
  }
})

const labelSizeClasses = computed(() => {
  switch (props.size) {
    case 'sm':
      return 'text-xs'
    case 'lg':
      return 'text-base'
    default:
      return 'text-sm'
  }
})

const stateClasses = computed(() => {
  if (props.errorMessage) {
    return 'border-red-500 text-red-500 focus:border-red-400 focus:ring-red-500'
  }

  if (props.disabled) {
    return 'border-gray-600 bg-gray-800 text-gray-400'
  }

  return 'border-gray-600 bg-gray-800 text-orange-500 hover:border-gray-500 focus:border-orange-500 focus:ring-orange-500'
})

const labelStateClasses = computed(() => {
  if (props.disabled) {
    return 'text-gray-500'
  }
  return 'text-gray-300'
})

// Methods
const handleBlur = (event: FocusEvent): void => {
  emit('blur', event)
}

const handleFocus = (event: FocusEvent): void => {
  emit('focus', event)
}

const handleChange = (event: Event): void => {
  const target = event.target as HTMLInputElement
  emit('update:modelValue', target.checked)
  emit('change', event)
}

// Expose methods for parent components
defineExpose({
  focus: () => checkboxRef.value?.focus(),
  blur: () => checkboxRef.value?.blur()
})
</script>
