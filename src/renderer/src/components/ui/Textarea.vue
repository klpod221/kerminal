<template>
  <div class="space-y-0.5">
    <label v-if="label" :for="inputId" class="block text-sm font-medium text-gray-300">
      {{ label }}
      <span v-if="required" class="text-red-400 ml-1">*</span>
    </label>
    <textarea
      :id="inputId"
      :value="modelValue"
      :class="textareaClasses"
      :placeholder="placeholder"
      :disabled="disabled"
      :required="required"
      :rows="rows"
      v-bind="$attrs"
      @input="$emit('update:modelValue', ($event.target as HTMLTextAreaElement).value)"
    />
    <div class="h-2">
      <span v-if="error" class="text-xs text-red-400">{{ error }}</span>
    </div>
  </div>
</template>

<script setup lang="ts">
import { computed, useAttrs } from 'vue'

interface Props {
  modelValue: string
  label?: string
  placeholder?: string
  error?: string
  disabled?: boolean
  required?: boolean
  rows?: number
  variant?: 'default' | 'error'
}

const props = withDefaults(defineProps<Props>(), {
  variant: 'default',
  rows: 3
})

defineEmits(['update:modelValue'])

const attrs = useAttrs()

const inputId = computed(() => {
  return (attrs.id as string) || `textarea-${Math.random().toString(36).substr(2, 9)}`
})

const textareaClasses = computed(() => {
  const baseClasses = [
    'block w-full rounded-lg border transition-all duration-200',
    'focus:outline-none focus:ring-2 focus:ring-offset-2 focus:ring-offset-gray-800',
    'disabled:opacity-50 disabled:cursor-not-allowed',
    'px-3 py-2 text-base',
    'resize-vertical'
  ]

  if (props.variant === 'error' || props.error) {
    baseClasses.push(
      'border-red-500 bg-red-500/5 text-white focus:border-red-400 focus:ring-red-500'
    )
  } else if (props.disabled) {
    baseClasses.push('border-gray-600 bg-gray-800 text-gray-400')
  } else {
    baseClasses.push(
      'border-gray-600 bg-gray-800 text-white placeholder-gray-400 hover:border-gray-500 focus:border-blue-500 focus:ring-blue-500'
    )
  }

  return baseClasses.join(' ')
})
</script>
