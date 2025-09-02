<!-- ColorPicker component with consistent styling -->
<template>
  <div class="space-y-0.5">
    <!-- Label -->
    <label v-if="label" :for="pickerId" class="block text-sm font-medium text-gray-300">
      {{ label }}
      <span v-if="rules?.some((rule) => rule === 'required')" class="text-red-400">*</span>
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
          'block w-10 h-10 p-0 border rounded-lg cursor-pointer transition-all duration-200',
          'focus:outline-none focus:ring-2 focus:ring-offset-2 focus:ring-offset-gray-800',
          'disabled:opacity-50 disabled:cursor-not-allowed',
          'readonly:bg-gray-700 readonly:cursor-default',
          stateClasses
        ]"
        @blur="handleBlur"
        @focus="handleFocus"
        @input="handleColorInput"
      />
      <!-- Hex input sử dụng Input component -->
      <input
        v-model="hexInput"
        type="text"
        :disabled="disabled"
        :readonly="readonly"
        :placeholder="'#RRGGBB'"
        maxlength="7"
        :class="localError ? 'border-red-500 bg-red-500/5' : ''"
        class="block w-full rounded-lg border transition-all duration-200 text-sm px-3 py-2 bg-gray-800 text-white placeholder-gray-400 focus:outline-none focus:ring-2 focus:ring-offset-2 focus:ring-offset-gray-800 focus:border-blue-500 focus:ring-blue-500 disabled:opacity-50 disabled:cursor-not-allowed readonly:bg-gray-700 readonly:cursor-default border-gray-600 hover:border-gray-500"
        @input="handleHexInput"
      />
    </div>

    <div class="h-2">
      <!-- Helper text -->
      <p v-if="!helperText" class="text-xs text-gray-400">{{ helperText }}</p>

      <!-- Error message -->
      <p v-if="errorMessage" class="text-xs text-red-400">{{ errorMessage }}</p>
    </div>
  </div>
</template>

<script setup lang="ts">
import { ref, computed, watch } from 'vue'

interface Props {
  modelValue?: string
  label?: string
  helperText?: string
  errorMessage?: string
  rules?: Array<string | ((value: string) => boolean)>
  disabled?: boolean
  readonly?: boolean
  id?: string
}

const props = withDefaults(defineProps<Props>(), {
  modelValue: '#000000',
  disabled: false,
  readonly: false,
  required: false
})

const emit = defineEmits(['update:modelValue', 'blur', 'focus'])

// Refs
const pickerRef = ref<HTMLInputElement>()

// Computed
const pickerId = computed(
  () => props.id || `color-picker-${Math.random().toString(36).substr(2, 9)}`
)

const colorValue = computed({
  get: () => (isValidHex(props.modelValue ?? '') ? props.modelValue! : '#000000'),
  set: (value: string) => emit('update:modelValue', value)
})

const hexInput = ref(colorValue.value)
const localError = ref<string>('')

const stateClasses = computed(() => {
  if (props.errorMessage || localError.value) {
    return 'border-red-500 bg-red-500/5 focus:border-red-400 focus:ring-red-500'
  }
  if (props.disabled) {
    return 'border-gray-600 bg-gray-800'
  }
  if (props.readonly) {
    return 'border-gray-600 bg-gray-700'
  }
  return 'border-gray-600 bg-gray-800 hover:border-gray-500 focus:border-blue-500 focus:ring-blue-500'
})

// Methods
const handleBlur = (event: FocusEvent): void => {
  emit('blur', event)
}

const handleFocus = (event: FocusEvent): void => {
  emit('focus', event)
}

const handleColorInput = (event: Event): void => {
  const target = event.target as HTMLInputElement
  hexInput.value = target.value
  emit('update:modelValue', target.value)
  localError.value = ''
}

const handleHexInput = (event: Event): void => {
  const value = (event.target as HTMLInputElement).value
  hexInput.value = value
  if (isValidHex(value)) {
    emit('update:modelValue', value)
    localError.value = ''
  } else {
    localError.value = 'Invalid hex color code.'
  }
}

function isValidHex(hex: string): boolean {
  return /^#([A-Fa-f0-9]{6}|[A-Fa-f0-9]{3})$/.test(hex)
}

// Sync hex input when modelValue changes externally
watch(
  () => props.modelValue,
  (val) => {
    if (isValidHex(val ?? '')) {
      hexInput.value = val!
      localError.value = ''
    }
  }
)

// Expose methods for parent components
defineExpose({
  focus: () => pickerRef.value?.focus(),
  blur: () => pickerRef.value?.blur(),
  select: () => pickerRef.value?.select()
})
</script>
