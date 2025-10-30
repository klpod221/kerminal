<template>
  <div class="space-y-2">
    <!-- Label and Value -->
    <div v-if="label || showValue" class="flex items-center justify-between">
      <label
        v-if="label"
        :for="sliderId"
        class="text-sm font-medium text-gray-300 flex items-center gap-2"
      >
        <component v-if="icon" :is="icon" class="w-4 h-4 text-blue-400" />
        {{ label }}
      </label>
      <span v-if="showValue" class="text-sm font-semibold text-blue-400">
        {{ modelValue }}{{ unit }}
      </span>
    </div>

    <!-- Slider Container -->
    <div class="space-y-2">
      <!-- Slider with Input -->
      <div class="flex items-center gap-4">
        <!-- Slider -->
        <div class="flex-1 relative">
          <input
            :id="sliderId"
            v-model.number="internalValue"
            type="range"
            :min="min"
            :max="max"
            :step="step"
            :disabled="disabled"
            class="w-full h-2 rounded-lg appearance-none cursor-pointer transition-opacity slider"
            :class="{
              'opacity-50 cursor-not-allowed': disabled,
              'bg-gray-700': !disabled,
              'bg-gray-800': disabled,
            }"
            @input="handleInput"
            @change="handleChange"
          />

          <!-- Scale marks -->
          <div v-if="showMarks" class="flex justify-between text-[10px] text-gray-500 mt-1 px-0.5">
            <span>{{ min }}</span>
            <span v-if="marks.length > 0">
              <span v-for="(mark, index) in marks" :key="index" class="mx-2">
                {{ mark }}
              </span>
            </span>
            <span>{{ max }}</span>
          </div>
        </div>

        <!-- Number Input (optional) -->
        <input
          v-if="showInput"
          v-model.number="internalValue"
          type="number"
          :min="min"
          :max="max"
          :step="step"
          :disabled="disabled"
          class="w-20 px-3 py-2 bg-gray-800 border border-gray-600 rounded-lg text-white text-sm focus:outline-none focus:border-blue-500 transition-colors"
          :class="{
            'opacity-50 cursor-not-allowed': disabled,
          }"
          @change="handleChange"
        />
      </div>

      <!-- Helper text or error -->
      <div v-if="helperText || error" class="min-h-4">
        <p v-if="error" class="text-xs text-red-400">
          {{ error }}
        </p>
        <p v-else-if="helperText" class="text-xs text-gray-400">
          {{ helperText }}
        </p>
      </div>
    </div>
  </div>
</template>

<script setup lang="ts">
import { ref, watch } from "vue";
import type { Component } from "vue";

interface SliderProps {
  modelValue: number;
  label?: string;
  icon?: Component;
  min?: number;
  max?: number;
  step?: number;
  unit?: string;
  showValue?: boolean;
  showInput?: boolean;
  showMarks?: boolean;
  marks?: number[];
  disabled?: boolean;
  helperText?: string;
  error?: string;
}

const props = withDefaults(defineProps<SliderProps>(), {
  min: 0,
  max: 100,
  step: 1,
  unit: "",
  showValue: true,
  showInput: true,
  showMarks: false,
  marks: () => [],
  disabled: false,
});

const emit = defineEmits<{
  "update:modelValue": [value: number];
  change: [value: number];
  input: [value: number];
}>();

const internalValue = ref(props.modelValue);
const sliderId = `slider-${Math.random().toString(36).substr(2, 9)}`;

// Watch for external changes
watch(
  () => props.modelValue,
  (newValue) => {
    internalValue.value = newValue;
  }
);

// Handle input (realtime)
const handleInput = () => {
  emit("update:modelValue", internalValue.value);
  emit("input", internalValue.value);
};

// Handle change (on release)
const handleChange = () => {
  emit("update:modelValue", internalValue.value);
  emit("change", internalValue.value);
};
</script>

<style scoped>
/* Range slider styling */
.slider::-webkit-slider-thumb {
  appearance: none;
  width: 18px;
  height: 18px;
  border-radius: 50%;
  background: #3b82f6;
  cursor: pointer;
  transition: all 0.2s;
  box-shadow: 0 2px 4px rgba(0, 0, 0, 0.2);
}

.slider::-webkit-slider-thumb:hover {
  background: #60a5fa;
  transform: scale(1.15);
  box-shadow: 0 2px 8px rgba(59, 130, 246, 0.4);
}

.slider::-webkit-slider-thumb:active {
  transform: scale(1.05);
}

.slider:disabled::-webkit-slider-thumb {
  background: #6b7280;
  cursor: not-allowed;
}

.slider::-moz-range-thumb {
  width: 18px;
  height: 18px;
  border-radius: 50%;
  background: #3b82f6;
  cursor: pointer;
  border: none;
  transition: all 0.2s;
  box-shadow: 0 2px 4px rgba(0, 0, 0, 0.2);
}

.slider::-moz-range-thumb:hover {
  background: #60a5fa;
  transform: scale(1.15);
  box-shadow: 0 2px 8px rgba(59, 130, 246, 0.4);
}

.slider::-moz-range-thumb:active {
  transform: scale(1.05);
}

.slider:disabled::-moz-range-thumb {
  background: #6b7280;
  cursor: not-allowed;
}

/* Track styling */
.slider::-webkit-slider-track {
  height: 8px;
  border-radius: 4px;
  background: linear-gradient(
    to right,
    #3b82f6 0%,
    #3b82f6 var(--value-percent, 50%),
    #374151 var(--value-percent, 50%),
    #374151 100%
  );
}

.slider::-moz-range-track {
  height: 8px;
  border-radius: 4px;
  background: #374151;
}

.slider::-moz-range-progress {
  height: 8px;
  border-radius: 4px;
  background: #3b82f6;
}
</style>

