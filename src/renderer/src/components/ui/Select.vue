<template>
  <div class="space-y-1">
    <label v-if="label" :for="inputId" class="block text-sm font-medium text-gray-300">
      {{ label }}
      <span v-if="required" class="text-red-400 ml-1">*</span>
    </label>
    <select
      :id="inputId"
      :value="modelValue"
      :class="selectClasses"
      :disabled="disabled"
      :required="required"
      v-bind="$attrs"
      @change="$emit('update:modelValue', ($event.target as HTMLSelectElement).value)"
    >
      <option v-if="placeholder" value="" disabled>{{ placeholder }}</option>
      <slot></slot>
    </select>
    <span v-if="error" class="text-xs text-red-400">{{ error }}</span>
  </div>
</template>

<script setup lang="ts">
import { computed, useAttrs } from 'vue'

interface Props {
  modelValue: string | number
  label?: string
  placeholder?: string
  error?: string
  disabled?: boolean
  required?: boolean
  variant?: 'default' | 'error'
}

const props = withDefaults(defineProps<Props>(), {
  variant: 'default'
})

defineEmits<{
  'update:modelValue': [value: string | number]
}>()

const attrs = useAttrs()

const inputId = computed(() => {
  return (attrs.id as string) || `select-${Math.random().toString(36).substr(2, 9)}`
})

const selectClasses = computed(() => {
  const baseClasses = [
    'block w-full rounded-lg border transition-all duration-200',
    'focus:outline-none focus:ring-2 focus:ring-offset-2 focus:ring-offset-gray-800',
    'disabled:opacity-50 disabled:cursor-not-allowed',
    'px-3 py-2 text-base'
  ]

  if (props.variant === 'error' || props.error) {
    baseClasses.push(
      'border-red-500 bg-red-500/5 text-white focus:border-red-400 focus:ring-red-500'
    )
  } else if (props.disabled) {
    baseClasses.push('border-gray-600 bg-gray-800 text-gray-400')
  } else {
    baseClasses.push(
      'border-gray-600 bg-gray-800 text-white hover:border-gray-500 focus:border-blue-500 focus:ring-blue-500'
    )
  }

  return baseClasses.join(' ')
})
</script>
