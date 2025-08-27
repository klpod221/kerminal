<template>
  <label class="flex items-center space-x-2 cursor-pointer">
    <input
      :id="inputId"
      v-model="checked"
      type="checkbox"
      :disabled="disabled"
      :class="checkboxClasses"
      @change="$emit('update:modelValue', ($event.target as HTMLInputElement).checked)"
    />
    <span :class="labelClasses">{{ label }}</span>
  </label>
</template>

<script setup lang="ts">
import { computed, useAttrs } from 'vue'

interface Props {
  modelValue?: boolean
  label?: string
  disabled?: boolean
  size?: 'sm' | 'md' | 'lg'
}

const props = withDefaults(defineProps<Props>(), {
  modelValue: false,
  disabled: false,
  size: 'md'
})

defineEmits<{
  'update:modelValue': [value: boolean]
}>()

const attrs = useAttrs()

const inputId = computed(() => {
  return (attrs.id as string) || `checkbox-${Math.random().toString(36).substr(2, 9)}`
})

const checked = computed({
  get: () => props.modelValue,
  set: () => {
    // This setter won't be called directly since we handle it in the template
  }
})

const checkboxClasses = computed(() => {
  const baseClasses = [
    'rounded border transition-all duration-200',
    'focus:outline-none focus:ring-2 focus:ring-offset-2 focus:ring-offset-gray-800',
    'disabled:opacity-50 disabled:cursor-not-allowed'
  ]

  // Size classes
  switch (props.size) {
    case 'sm':
      baseClasses.push('w-3 h-3')
      break
    case 'lg':
      baseClasses.push('w-5 h-5')
      break
    default:
      baseClasses.push('w-4 h-4')
  }

  // Color classes
  if (props.disabled) {
    baseClasses.push('border-gray-600 bg-gray-800 text-gray-400')
  } else {
    baseClasses.push(
      'border-gray-600 bg-gray-800 text-orange-500',
      'hover:border-gray-500 focus:border-orange-500 focus:ring-orange-500'
    )
  }

  return baseClasses.join(' ')
})

const labelClasses = computed(() => {
  const baseClasses = ['select-none transition-colors']

  // Size classes
  switch (props.size) {
    case 'sm':
      baseClasses.push('text-xs')
      break
    case 'lg':
      baseClasses.push('text-base')
      break
    default:
      baseClasses.push('text-sm')
  }

  // Color classes
  if (props.disabled) {
    baseClasses.push('text-gray-500')
  } else {
    baseClasses.push('text-gray-300')
  }

  return baseClasses.join(' ')
})
</script>
