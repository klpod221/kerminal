<template>
  <nav class="flex space-x-1 bg-gray-800/50 rounded-lg p-1">
    <button
      v-for="tab in tabs"
      :key="tab.id"
      type="button"
      class="flex-1 px-4 py-2 text-sm font-medium rounded-md transition-colors"
      :class="{
        'bg-blue-600 text-white': modelValue === tab.id,
        'text-gray-300 hover:text-white hover:bg-gray-700': modelValue !== tab.id
      }"
      @click="$emit('update:modelValue', tab.id)"
    >
      <component :is="tab.icon" class="w-4 h-4 inline-block mr-2" />
      {{ tab.label }}
    </button>
  </nav>
</template>

<script setup lang="ts">
import type { Component } from 'vue'
import { defineProps, defineEmits } from 'vue'

interface TabItem {
  id: string
  label: string
  icon: Component
}

defineProps<{
  tabs: TabItem[]
  modelValue: string
}>()

defineEmits<{
  (e: 'update:modelValue', value: string): void
}>()
</script>
