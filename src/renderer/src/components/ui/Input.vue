<!-- Input component with consistent styling -->
<template>
  <div class="space-y-0.5">
    <!-- Label -->
    <label v-if="label" :for="inputId" class="block text-sm font-medium text-gray-300">
      {{ label }}
      <span v-if="rules?.some((rule) => rule === 'required')" class="text-red-400">*</span>
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
        :class="[
          'block w-full rounded-lg border transition-all duration-200',
          'focus:outline-none focus:ring-2 focus:ring-offset-2 focus:ring-offset-gray-800',
          'disabled:opacity-50 disabled:cursor-not-allowed',
          'readonly:bg-gray-700 readonly:cursor-default',
          sizeClasses,
          stateClasses,
          leftIcon ? 'pl-10' : 'pl-3',
          rightIcon || showPasswordToggle ? 'pr-10' : 'pr-3'
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
        <button
          v-if="showPasswordToggle"
          type="button"
          class="text-gray-400 hover:text-gray-300 transition-colors cursor-pointer"
          @click="togglePasswordVisibility"
        >
          <component :is="isPasswordVisible ? EyeOff : Eye" :size="iconSize" />
        </button>

        <!-- Right icon -->
        <button
          v-else-if="rightIcon"
          type="button"
          class="text-gray-400 hover:text-gray-300 transition-colors cursor-pointer"
          @click="emit('right-icon-click')"
        >
          <component :is="rightIcon" :size="iconSize" />
        </button>
      </div>
    </div>

    <div class="min-h-[1.25rem]">
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
import { ref, computed, nextTick } from 'vue'
import { Eye, EyeOff } from 'lucide-vue-next'
import type { Component } from 'vue'

interface Props {
  modelValue?: string | number
  type?: 'text' | 'password' | 'email' | 'number' | 'tel' | 'url' | 'search'
  label?: string
  placeholder?: string
  rules?: Array<string | ((value: string) => boolean)>
  helperText?: string
  errorMessage?: string
  size?: 'sm' | 'md' | 'lg'
  leftIcon?: Component
  rightIcon?: Component
  disabled?: boolean
  readonly?: boolean
  autocomplete?: string
  id?: string
}

interface Emits {
  'update:modelValue': [value: string]
  blur: [event: FocusEvent]
  focus: [event: FocusEvent]
  keydown: [event: KeyboardEvent]
  'right-icon-click': []
}

const props = withDefaults(defineProps<Props>(), {
  type: 'text',
  size: 'md',
  disabled: false,
  readonly: false
})

const emit = defineEmits<Emits>()

// Refs
const inputRef = ref<HTMLInputElement>()
const isPasswordVisible = ref(false)

// Computed
const inputId = computed(() => props.id || `input-${Math.random().toString(36).substr(2, 9)}`)

const inputValue = computed({
  get: () => props.modelValue?.toString() ?? '',
  set: (value: string) => emit('update:modelValue', value)
})

const inputType = computed(() => {
  if (props.type === 'password') {
    return isPasswordVisible.value ? 'text' : 'password'
  }
  return props.type
})

const showPasswordToggle = computed(() => props.type === 'password')

const iconSize = computed(() => {
  switch (props.size) {
    case 'sm':
      return 16
    case 'lg':
      return 20
    default:
      return 18
  }
})

const sizeClasses = computed(() => {
  switch (props.size) {
    case 'sm':
      return 'text-sm py-1.5'
    case 'lg':
      return 'text-lg py-3'
    default:
      return 'text-base py-2'
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
const togglePasswordVisibility = (): void => {
  isPasswordVisible.value = !isPasswordVisible.value
  // Keep focus on input after toggling
  nextTick(() => {
    inputRef.value?.focus()
  })
}

const handleBlur = (event: FocusEvent): void => {
  emit('blur', event)
}

const handleFocus = (event: FocusEvent): void => {
  emit('focus', event)
}

const handleInput = (event: Event): void => {
  const target = event.target as HTMLInputElement
  emit('update:modelValue', target.value)
}

const handleKeydown = (event: KeyboardEvent): void => {
  emit('keydown', event)
}

// Expose methods for parent components
defineExpose({
  focus: () => inputRef.value?.focus(),
  blur: () => inputRef.value?.blur(),
  select: () => inputRef.value?.select()
})
</script>
