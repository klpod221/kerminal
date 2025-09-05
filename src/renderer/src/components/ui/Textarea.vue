<!-- Textarea component with consistent styling -->
<template>
  <div class="space-y-0.5">
    <!-- Label -->
    <label v-if="label" :for="textareaId" class="block text-sm font-medium text-gray-300">
      {{ label }}
      <span v-if="rules?.some((rule) => rule === 'required')" class="text-red-400">*</span>
    </label>

    <!-- Textarea element -->
    <textarea
      :id="textareaId"
      ref="textareaRef"
      v-model="textareaValue"
      :placeholder="placeholder"
      :disabled="disabled"
      :readonly="readonly"
      :rows="rows"
      :class="[
        'block w-full rounded-lg border transition-all duration-200',
        'focus:outline-none focus:ring-2 focus:ring-offset-2 focus:ring-offset-gray-800',
        'disabled:opacity-50 disabled:cursor-not-allowed',
        'readonly:bg-gray-700 readonly:cursor-default',
        'resize-vertical',
        sizeClasses,
        stateClasses
      ]"
      @blur="handleBlur"
      @focus="handleFocus"
      @input="handleInput"
      @keydown="handleKeydown"
    />

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
import type { TextareaProps } from '../../types/ui'

const props = withDefaults(defineProps<TextareaProps>(), {
  size: 'md',
  disabled: false,
  readonly: false,
  helper: true,
  rows: 3
})

const emit = defineEmits(['update:modelValue', 'blur', 'focus', 'keydown'])

// Refs
const textareaRef = ref<HTMLTextAreaElement>()

// Computed
const textareaId = computed(() => props.id || `textarea-${Math.random().toString(36).substr(2, 9)}`)

const textareaValue = computed({
  get: () => props.modelValue?.toString() ?? '',
  set: (value: string) => emit('update:modelValue', value)
})

const sizeClasses = computed(() => {
  switch (props.size) {
    case 'sm':
      return 'text-sm px-2 py-1.5'
    case 'lg':
      return 'text-lg px-4 py-3'
    default:
      return 'text-base px-3 py-2'
  }
})

const stateClasses = computed(() => {
  if (props.errorMessage) {
    return 'border-red-500 bg-red-500/5 text-white focus:border-red-400 focus:ring-red-500'
  }

  if (props.disabled) {
    return 'border-gray-600 bg-gray-800 text-gray-400'
  }

  if (props.readonly) {
    return 'border-gray-600 bg-gray-700 text-gray-300'
  }

  return 'border-gray-600 bg-gray-800 text-white placeholder-gray-400 hover:border-gray-500 focus:border-blue-500 focus:ring-blue-500'
})

// Methods
const handleBlur = (event: FocusEvent): void => {
  emit('blur', event)
}

const handleFocus = (event: FocusEvent): void => {
  emit('focus', event)
}

const handleInput = (event: Event): void => {
  const target = event.target as HTMLTextAreaElement
  emit('update:modelValue', target.value)
}

const handleKeydown = (event: KeyboardEvent): void => {
  emit('keydown', event)
}

// Expose methods for parent components
defineExpose({
  focus: () => textareaRef.value?.focus(),
  blur: () => textareaRef.value?.blur(),
  select: () => textareaRef.value?.select()
})
</script>
